use chrono::{Local, NaiveDate};
use log::{error, info, warn};
use rocket_db_pools::Connection;
use std::time::Instant;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, ContractHistory, NewContractHistory};
use crate::utils::insert_utiles::insert_contract_histories;
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{update_contract_with_end_date, update_contract_with_new_amount};

use super::loading_utils::{load_contract_history, load_last_transaction_data_of_contract};

pub async fn handle_contract_update(
    transaction: Transaction,
    contract: Contract,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let start_time = Instant::now();

    if transaction.amount == contract.current_amount {
        info!("Amount is the same, no need to update contract.");
        return Ok("".into());
    }

    match contract.end_date {
        Some(end_date) => {
            info!("Contract has an end date, checking if transaction date is before end date.");
            if end_date < transaction.date {
                info!("Transaction date is after end date, handling as closed contract.");
                let result = handle_new_date_logic(contract, transaction, db, true).await;
                warn!("Handled closed contract in {:?}", start_time.elapsed());
                return result;
            }
            info!("Transaction date is before end date, handling as open contract.");
            let result = handle_old_date_logic(contract, transaction, db).await;
            warn!("Handled open contract in {:?}", start_time.elapsed());
            return result;
        }
        None => {
            info!(
                "Contract has no end date, checking if transaction date is after last transaction."
            );
            let last_transaction = load_last_transaction_data_of_contract(contract.id, db).await;

            if let Err(e) = last_transaction {
                error!("Failed to load last transaction data: {}", e);
                return Err(e);
            }

            let last_transaction = last_transaction.unwrap().unwrap();

            if last_transaction.date < transaction.date {
                info!("Transaction date is after last transaction, handling as open contract.");
                let result = handle_new_date_logic(contract, transaction, db, false).await;
                warn!("Handled open contract in {:?}", start_time.elapsed());
                return result;
            }

            info!("Transaction date is before last transaction, handling as closed contract.");
            let result = handle_old_date_logic(contract, transaction, db).await;
            warn!("Handled closed contract in {:?}", start_time.elapsed());
            return result;
        }
    }
}

async fn handle_old_date_logic(
    contract: Contract,
    transaction: Transaction,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let start_time = Instant::now();

    let existing_contract_histories = load_contract_history(contract.id, db).await;

    if let Err(e) = existing_contract_histories {
        error!("Failed to load contract history: {}", e);
        return Err(e);
    }

    let contract_histories = existing_contract_histories.unwrap();
    let mut new_contract_histories = Vec::new();

    if contract_histories.is_empty() {
        new_contract_histories.push(NewContractHistory {
            contract_id: contract.id,
            old_amount: transaction.amount,
            new_amount: transaction.amount,
            changed_at: transaction.date,
        });
    } else {
        process_contract_histories(
            &contract_histories,
            contract,
            transaction,
            &mut new_contract_histories,
        );
    }

    let result = insert_contract_histories(&new_contract_histories, db).await;
    if let Err(e) = result {
        error!("Failed to insert contract histories: {}", e);
        return Err(e);
    }

    warn!("Handled old date logic in {:?}", start_time.elapsed());
    Ok("Contract history added.".into())
}

async fn handle_new_date_logic(
    contract: Contract,
    transaction: Transaction,
    db: &mut Connection<DbConn>,
    is_closed: bool,
) -> Result<String, String> {
    let start_time = Instant::now();

    let contract_history = NewContractHistory {
        contract_id: contract.id,
        old_amount: contract.current_amount,
        new_amount: transaction.amount,
        changed_at: transaction.date,
    };

    let mut result = insert_contract_histories(&vec![contract_history], db).await;

    if let Err(e) = result {
        error!("Failed to insert contract histories: {}", e);
        return Err(e);
    }

    result = update_contract_with_new_amount(contract.id, transaction.amount, db).await;

    if let Err(e) = result {
        error!("Failed to update contract with new amount: {}", e);
        return Err(e);
    }

    if is_closed {
        let days_to_add = contract.months_between_payment * 30;
        let new_end_date = contract.end_date.unwrap() + chrono::Duration::days(days_to_add as i64);

        if new_end_date > NaiveDate::from(Local::now().naive_local()) {
            result = update_contract_with_end_date(contract.id, None, db).await;
            if let Err(e) = result {
                error!("Failed to update contract end date: {}", e);
                return Err(e);
            }
            warn!(
                "Handled closed contract reopening in {:?}",
                start_time.elapsed()
            );
            return Ok("Contract is open again".into());
        }

        result = update_contract_with_end_date(contract.id, Some(new_end_date), db).await;

        if let Err(e) = result {
            error!("Failed to update contract end date: {}", e);
            return Err(e);
        }

        warn!("Handled closed contract in {:?}", start_time.elapsed());
        return Ok(format!(
            "Contract updated, end date set to {}.",
            new_end_date
        ));
    }

    warn!("Handled new date logic in {:?}", start_time.elapsed());
    Ok(format!(
        "Contract updated, new amount set to {}.",
        transaction.amount
    ))
}

fn process_contract_histories(
    contract_histories: &Vec<ContractHistory>,
    contract: Contract,
    transaction: Transaction,
    new_contract_histories: &mut Vec<NewContractHistory>,
) {
    let start_time = Instant::now();

    let closest_older_contract_history = contract_histories
        .iter()
        .filter(|history| history.changed_at <= transaction.date)
        .min_by_key(|history| (transaction.date - history.changed_at).num_days());

    if let Some(closest_older) = closest_older_contract_history {
        new_contract_histories.push(NewContractHistory {
            contract_id: contract.id,
            old_amount: closest_older.new_amount,
            new_amount: transaction.amount,
            changed_at: transaction.date,
        });
    }

    let closest_newer_contract_history = contract_histories
        .iter()
        .filter(|history| history.changed_at >= transaction.date)
        .min_by_key(|history| (history.changed_at - transaction.date).num_days());

    if let Some(closest_newer) = closest_newer_contract_history {
        new_contract_histories.push(NewContractHistory {
            contract_id: contract.id,
            old_amount: transaction.amount,
            new_amount: closest_newer.old_amount,
            changed_at: transaction.date,
        });
    } else {
        new_contract_histories.push(NewContractHistory {
            contract_id: contract.id,
            old_amount: transaction.amount,
            new_amount: contract.current_amount,
            changed_at: transaction.date,
        });
    }

    warn!("Processed contract histories in {:?}", start_time.elapsed());
}
