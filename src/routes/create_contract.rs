use chrono::{Duration, NaiveDate};
use diesel::{ExpressionMethods, QueryDsl};
use log::info;
use rocket::State;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, NewContract, NewContractHistory};
use crate::schema::{contract_history, contracts};
use crate::utils::appstate::AppState;
use crate::utils::structs::{Bank, Transaction};
use crate::utils::update_utils::update_transaction_with_contract_id;

type Result<T> = std::result::Result<T, String>;

pub async fn create_contract_from_transactions(
    banks: Vec<Bank>,
    state: &State<AppState>,
    mut db: &mut Connection<DbConn>,
) -> Result<String> {
    let mut new_contracts: HashMap<i32, Vec<NewContract>> = HashMap::new();
    let transactions = state.transactions.read().await;
    let existing_contracts = state.contracts.read().await;

    info!("Starting to create contracts from transactions.");

    for bank in banks.iter() {
        let bank_transactions = transactions.get(&bank.id).unwrap();

        info!("Processing bank ID: {}", bank.id);
        let transactions_grouped_by_counterparty =
            group_transactions_by_counterparty(bank_transactions);

        for (counterparty, transactions) in transactions_grouped_by_counterparty {
            info!("Processing counterparty: {}", counterparty);
            let transactions_grouped_by_amount = group_transactions_by_amount(transactions);

            for transactions in transactions_grouped_by_amount.values() {
                if transactions.len() <= 2 {
                    info!(
                        "Skipping counterparty {} due to insufficient transactions.",
                        counterparty
                    );
                    continue;
                }

                if let Some(months_between) = check_time_frame(transactions) {
                    let new_contract = NewContract {
                        bank_id: bank.id,
                        name: counterparty.clone(),
                        current_amount: transactions[0].0,
                        months_between_payment: months_between,
                    };

                    if process_new_contract(
                        &new_contract,
                        &existing_contracts,
                        &mut db,
                        &transactions,
                    )
                    .await?
                    {
                        new_contracts
                            .entry(bank.id)
                            .or_insert_with(Vec::new)
                            .push(new_contract);
                        info!(
                            "New contract created for counterparty {} with bank ID {}",
                            counterparty, bank.id
                        );
                    }
                }
            }
        }
    }

    let inserted_contracts = insert_new_contracts(&new_contracts, &mut db).await?;

    drop(existing_contracts);

    state.update_contracts(inserted_contracts).await;

    let contracts_write_guard = state.contracts.write().await;

    update_transactions_with_contract_ids(&transactions, &contracts_write_guard, &mut db).await?;

    let updated_transactions = transactions.clone();

    drop(transactions);

    // After updating contract IDs, integrate the update_transactions function to update the state
    state
        .update_transactions(updated_transactions) // Clone transactions to avoid borrow issues
        .await;

    let contract_count = new_contracts.values().flatten().count();
    info!("Found {} new contracts!", contract_count);

    Ok(format!("Found {} new contracts!", contract_count))
}

async fn update_transactions_with_contract_ids(
    transactions: &HashMap<i32, Vec<Transaction>>,
    existing_contracts: &HashMap<i32, Vec<Contract>>,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    let transactions_without_contract: Vec<_> = transactions
        .values()
        .flat_map(|bank_transactions| bank_transactions.iter())
        .filter(|transaction| transaction.contract_id.is_none())
        .cloned()
        .collect();

    for transaction in transactions_without_contract {
        if let Some(contracts) = existing_contracts.get(&transaction.bank_id) {
            for contract in contracts {
                if transaction.counterparty.contains(&contract.name) {
                    // Round amounts to two decimal places
                    let rounded_transaction_amount = (transaction.amount * 100.0).round() / 100.0;
                    let rounded_contract_amount = (contract.current_amount * 100.0).round() / 100.0;

                    if rounded_transaction_amount == rounded_contract_amount {
                        update_transaction_with_contract_id(transaction.id, contract.id, db)
                            .await?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_new_contract(
    new_contract: &NewContract,
    existing_contracts: &HashMap<i32, Vec<Contract>>,
    db: &mut Connection<DbConn>,
    transactions: &[(f64, NaiveDate)],
) -> Result<bool> {
    if let Some(existing_contract_list) = existing_contracts.get(&new_contract.bank_id) {
        for existing_contract in existing_contract_list {
            if existing_contract.name == new_contract.name
                && existing_contract.months_between_payment == new_contract.months_between_payment
            {
                if existing_contract.current_amount == new_contract.current_amount {
                    if let Some(end_date) = calculate_end_date(transactions) {
                        if existing_contract.end_date.is_none()
                            || existing_contract.end_date.unwrap() != end_date
                        {
                            info!(
                                "Updating end date for contract {} at bank ID {}.",
                                new_contract.name, new_contract.bank_id
                            );
                            update_contract_end_date(existing_contract, end_date, db).await?;
                        }
                    }

                    info!(
                        "Duplicate contract for {} at bank ID {}. Skipping.",
                        new_contract.name, new_contract.bank_id
                    );
                    return Ok(false);
                } else if is_contract_with_reasonable_change(
                    existing_contract,
                    new_contract,
                    10.0,
                    50.0,
                ) {
                    info!(
                        "Updating contract {} at bank ID {}.",
                        new_contract.name, new_contract.bank_id
                    );
                    update_existing_contract(existing_contract, new_contract, db, transactions)
                        .await?;
                    return Ok(false);
                }
            }
        }
    }

    info!(
        "Inserting new contract for {} at bank ID {}.",
        new_contract.name, new_contract.bank_id
    );
    Ok(true)
}

async fn update_existing_contract(
    existing_contract: &Contract,
    new_contract: &NewContract,
    db: &mut Connection<DbConn>,
    transactions: &[(f64, NaiveDate)],
) -> Result<()> {
    diesel::update(contracts::table.find(existing_contract.id))
        .set(contracts::current_amount.eq(new_contract.current_amount))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating the contract: {}", e))?;

    let new_history = NewContractHistory {
        contract_id: existing_contract.id,
        old_amount: existing_contract.current_amount,
        new_amount: new_contract.current_amount,
        changed_at: Some(chrono::Utc::now().naive_utc()),
    };

    diesel::insert_into(contract_history::table)
        .values(&new_history)
        .execute(db)
        .await
        .map_err(|e| format!("Error inserting into contract history: {}", e))?;

    if let Some(end_date) = calculate_end_date(transactions) {
        update_contract_end_date(existing_contract, end_date, db).await?;
    }

    info!(
        "Updated contract ID {} with new amount {}.",
        existing_contract.id, new_contract.current_amount
    );
    Ok(())
}

async fn insert_new_contracts(
    new_contracts: &HashMap<i32, Vec<NewContract>>,
    db: &mut Connection<DbConn>,
) -> Result<HashMap<i32, Vec<Contract>>> {
    let mut inserted_contracts = HashMap::new();

    for (bank_id, contracts) in new_contracts.iter() {
        info!(
            "Inserting {} new contracts for bank ID {}.",
            contracts.len(),
            bank_id
        );
        let result = diesel::insert_into(contracts::table)
            .values(contracts)
            .get_results::<Contract>(db)
            .await
            .map_err(|e| format!("Error inserting contract: {}", e))?;

        inserted_contracts.insert(*bank_id, result);
    }

    Ok(inserted_contracts)
}

fn group_transactions_by_counterparty(
    transactions: &Vec<Transaction>,
) -> HashMap<String, Vec<(f64, NaiveDate)>> {
    transactions
        .iter()
        .fold(HashMap::new(), |mut acc, transaction| {
            acc.entry(transaction.counterparty.clone())
                .or_insert_with(Vec::new)
                .push((transaction.amount, transaction.date));
            acc
        })
}

fn group_transactions_by_amount(
    transactions: Vec<(f64, NaiveDate)>,
) -> HashMap<i64, Vec<(f64, NaiveDate)>> {
    transactions
        .into_iter()
        .fold(HashMap::new(), |mut acc, (amount, date)| {
            let key = (amount * 100.0) as i64;
            acc.entry(key).or_insert_with(Vec::new).push((amount, date));
            acc
        })
}

fn check_time_frame(transactions: &[(f64, NaiveDate)]) -> Option<i32> {
    let mut sorted_transactions = transactions.to_vec();
    sorted_transactions.sort_by_key(|&(_, date)| date);

    for window in sorted_transactions.windows(3) {
        let duration1 = window[1].1.signed_duration_since(window[0].1);
        let duration2 = window[2].1.signed_duration_since(window[1].1);

        if is_within_time_frame(duration1, 30) && is_within_time_frame(duration2, 30) {
            return Some(1);
        } else if is_within_time_frame(duration1, 90) && is_within_time_frame(duration2, 90) {
            return Some(3);
        } else if is_within_time_frame(duration1, 180) && is_within_time_frame(duration2, 180) {
            return Some(6);
        }
    }

    None
}

fn is_within_time_frame(duration: Duration, expected_days: i64) -> bool {
    let margin = 5;
    (duration.num_days() - expected_days).abs() <= margin
}

fn is_contract_with_reasonable_change(
    existing_contract: &Contract,
    new_contract: &NewContract,
    max_percentage_change: f64,
    max_absolute_change: f64,
) -> bool {
    let amount_change = (new_contract.current_amount - existing_contract.current_amount).abs();
    let percentage_change = (amount_change / existing_contract.current_amount) * 100.0;

    percentage_change <= max_percentage_change || amount_change <= max_absolute_change
}

async fn update_contract_end_date(
    existing_contract: &Contract,
    end_date: NaiveDate,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    diesel::update(contracts::table.find(existing_contract.id))
        .set(contracts::end_date.eq(Some(end_date)))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating the contract end date: {}", e))?;

    info!(
        "Set end date {} for contract ID {}.",
        end_date, existing_contract.id
    );
    Ok(())
}

fn calculate_end_date(transactions: &[(f64, NaiveDate)]) -> Option<NaiveDate> {
    transactions.last().map(|&(_, date)| date)
}
