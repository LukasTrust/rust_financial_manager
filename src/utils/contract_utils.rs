use chrono::{Local, NaiveDate};
use log::{error, info, warn};
use rocket_db_pools::Connection;
use std::time::Instant;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, ContractHistory, NewContractHistory};
use crate::utils::delete_utils::delete_contract_history_with_ids;
use crate::utils::insert_utiles::insert_contract_histories;
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{
    update_contract_history, update_contract_with_end_date, update_contract_with_new_amount,
    update_transactions_with_contract,
};

use super::loading_utils::{
    load_contract_history, load_contracts_from_ids, load_last_transaction_data_of_contract,
    load_transaction_by_id,
};

pub async fn handle_add_update(
    transaction: Transaction,
    contract: Contract,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let start_time = Instant::now();

    if transaction.amount == contract.current_amount {
        info!("Amount is the same, no need to update contract.");
        return Ok("Added transaction to contract".into());
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
            result
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
            result
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
    let new_contract_history = if contract_histories.is_empty() {
        NewContractHistory {
            contract_id: contract.id,
            old_amount: transaction.amount,
            new_amount: transaction.amount,
            changed_at: transaction.date,
        }
    } else {
        process_contract_histories(&contract_histories, contract, transaction, db).await
    };

    let result = insert_contract_histories(&vec![new_contract_history], db).await;
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
        let new_end_date = transaction.date + chrono::Duration::days(days_to_add as i64);
        let now = NaiveDate::from(Local::now().naive_local());

        if new_end_date > now {
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

async fn process_contract_histories(
    contract_histories: &[ContractHistory],
    contract: Contract,
    transaction: Transaction,
    db: &mut Connection<DbConn>,
) -> NewContractHistory {
    let closest_older_contract_history = contract_histories
        .iter()
        .filter(|history| {
            history.changed_at <= transaction.date && history.new_amount != transaction.amount
        })
        .min_by_key(|history| (transaction.date - history.changed_at).num_days());

    let closest_newer_contract_history = contract_histories
        .iter()
        .filter(|history| {
            history.changed_at >= transaction.date && history.new_amount != transaction.amount
        })
        .min_by_key(|history| (history.changed_at - transaction.date).num_days());

    match (
        closest_older_contract_history,
        closest_newer_contract_history,
    ) {
        (Some(older), Some(newer)) => {
            let mut older = older.clone();
            let mut newer = newer.clone();

            let new_history = NewContractHistory {
                contract_id: contract.id,
                old_amount: older.new_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            older.new_amount = transaction.amount;
            newer.old_amount = transaction.amount;

            update_contract_history(older, db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                })
                .unwrap();

            update_contract_history(newer, db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                })
                .unwrap();

            new_history
        }
        (None, Some(newer)) => {
            let mut newer = newer.clone();
            let new_history = NewContractHistory {
                contract_id: contract.id,
                old_amount: newer.old_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            newer.old_amount = transaction.amount;

            update_contract_history(newer, db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                })
                .unwrap();

            new_history
        }
        (Some(older), None) => NewContractHistory {
            contract_id: contract.id,
            old_amount: older.new_amount,
            new_amount: transaction.amount,
            changed_at: transaction.date,
        },
        (None, None) => NewContractHistory {
            contract_id: contract.id,
            old_amount: contract.current_amount,
            new_amount: transaction.amount,
            changed_at: transaction.date,
        },
    }
}

pub async fn handle_remove_contract(
    transaction_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    // Load transaction by ID
    let transaction = load_transaction_by_id(transaction_id, db)
        .await
        .map_err(|e| {
            error!("Failed to load transaction: {}", e);
            e
        })?
        .ok_or_else(|| {
            error!("Transaction not found");
            "Transaction not found".to_string()
        })?;

    // Check if transaction has a contract
    let contract_id = transaction.contract_id.ok_or_else(|| {
        error!("Transaction has no contract");
        "Transaction has no contract".to_string()
    })?;

    // Load contract
    let contract = load_contracts_from_ids(vec![contract_id], db)
        .await
        .map_err(|e| {
            error!("Failed to load contract: {}", e);
            e
        })?
        .into_iter()
        .next()
        .ok_or_else(|| {
            error!("Contract not found");
            "Contract not found".to_string()
        })?;

    // Load contract history
    let contract_histories = load_contract_history(contract.id, db).await.map_err(|e| {
        error!("Failed to load contract histories: {}", e);
        e
    })?;

    // Find contract history corresponding to the transaction amount
    if let Some(history) = contract_histories
        .iter()
        .find(|h| h.new_amount == transaction.amount)
    {
        let history_before = contract_histories
            .iter()
            .filter(|h| h.changed_at < history.changed_at)
            .max_by_key(|h| h.changed_at);

        let history_after = contract_histories
            .iter()
            .filter(|h| h.changed_at > history.changed_at)
            .min_by_key(|h| h.changed_at);

        delete_contract_history_with_ids(vec![history.id], db)
            .await
            .map_err(|e| {
                error!("Failed to delete contract history: {}", e);
                e
            })?;

        match (history_before, history_after) {
            (Some(before), Some(_)) => {
                // Case 1: Both before and after history entries exist
                let mut updated_before = before.clone();
                updated_before.new_amount = history.new_amount;

                update_contract_history(updated_before, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract history: {}", e);
                        e
                    })?;
            }
            (None, Some(after)) => {
                // Case 2: Only after history entry exists
                let mut updated_after = after.clone();

                updated_after.old_amount = history.old_amount;

                update_contract_history(updated_after, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract history: {}", e);
                        e
                    })?;
            }
            (Some(_), None) => {
                // Case 3: Only before history entry exists
                update_contract_with_new_amount(contract.id, history.old_amount, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract: {}", e);
                        e
                    })?;
            }
            (None, None) => {
                // Case 4: No history entries exist

                update_contract_with_new_amount(contract.id, history.old_amount, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract: {}", e);
                        e
                    })?;
            }
        }
    }

    // Update the transaction to remove the contract association
    update_transactions_with_contract(vec![transaction_id], None::<i32>, db)
        .await
        .map_err(|e| {
            error!(
                "Error removing contract from transaction {}: {}",
                transaction_id, e
            );
            e
        })?;

    Ok("Contract removed".into())
}
