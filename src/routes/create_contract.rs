use chrono::{Duration, NaiveDate};
use diesel::{ExpressionMethods, QueryDsl};
use log::{error, info};
use rocket::State;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::Connection;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, NewContract, NewContractHistory};
use crate::schema::{contract_history, contracts};
use crate::utils::appstate::AppState;
use crate::utils::structs::Transaction;

pub async fn create_contract_from_transactions(
    state: State<AppState>,
    mut db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let mut new_contracts: HashMap<i32, Vec<NewContract>> = HashMap::new();
    let transactions = state.transactions.read().await.clone();
    let existing_contracts = state.contracts.read().await.clone();

    info!("Starting to create contracts from transactions.");

    for (bank_id, bank_transactions) in transactions {
        info!("Processing bank ID: {}", bank_id);
        let transactions_grouped_by_counterparty =
            group_transactions_by_counterparty(bank_transactions);

        for (counterparty, transactions) in transactions_grouped_by_counterparty {
            info!("Processing counterparty: {}", counterparty);
            let transactions_grouped_by_amount = group_transactions_by_amount(transactions);

            for (_, transactions) in transactions_grouped_by_amount {
                if transactions.len() <= 2 {
                    info!(
                        "Skipping transactions for counterparty {} with insufficient transactions.",
                        counterparty
                    );
                    continue;
                }

                if let Some(months_between) = check_time_frame(&transactions) {
                    let new_contract = NewContract {
                        bank_id,
                        name: counterparty.clone(),
                        current_amount: transactions[0].0,
                        months_between_payment: months_between,
                    };

                    if process_new_contract(&new_contract, &existing_contracts, &mut db).await? {
                        let contract_list = new_contracts.entry(bank_id).or_insert_with(Vec::new);
                        contract_list.push(new_contract);
                        info!(
                            "New contract created for counterparty {} with bank ID {}",
                            counterparty, bank_id
                        );
                    }
                }
            }
        }
    }

    // Insert new contracts into the database
    let inserted_contracts = insert_new_contracts(&new_contracts, &mut db).await?;

    // Update the state with newly inserted contracts
    state.update_contracts(inserted_contracts).await;

    let contract_count = new_contracts.values().flatten().count();
    info!("Found {} new contracts!", contract_count);

    Ok(format!("Found {} new contracts!", contract_count))
}

async fn process_new_contract(
    new_contract: &NewContract,
    existing_contracts: &HashMap<i32, Vec<Contract>>,
    db: &mut Connection<DbConn>,
) -> Result<bool, String> {
    if let Some(existing_contract_list) = existing_contracts.get(&new_contract.bank_id) {
        for existing_contract in existing_contract_list {
            if existing_contract.bank_id == new_contract.bank_id
                && existing_contract.name == new_contract.name
                && existing_contract.months_between_payment == new_contract.months_between_payment
            {
                if existing_contract.current_amount == new_contract.current_amount {
                    info!(
                        "Duplicate contract found for {} at bank ID {}. Skipping insertion.",
                        new_contract.name, new_contract.bank_id
                    );
                    return Ok(false);
                } else if is_contract_with_reasonable_change(
                    existing_contract,
                    new_contract,
                    10.0, // max percentage change allowed
                    50.0, // max absolute change allowed
                ) {
                    info!("Reasonable update found for contract {} at bank ID {}. Updating existing contract.", new_contract.name, new_contract.bank_id);
                    update_existing_contract(existing_contract, new_contract, db).await?;
                    return Ok(false);
                }
            }
        }
    }
    info!(
        "No duplicate found. Proceeding with insertion for contract {} at bank ID {}.",
        new_contract.name, new_contract.bank_id
    );
    Ok(true)
}

async fn update_existing_contract(
    existing_contract: &Contract,
    new_contract: &NewContract,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    // Update existing contract's current_amount
    diesel::update(contracts::table.find(existing_contract.id))
        .set(contracts::current_amount.eq(new_contract.current_amount))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating contract: {}", e);
            format!("Error updating contract: {}", e)
        })?;

    // Create a new contract history entry
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
        .map_err(|e| {
            error!("Error creating contract history: {}", e);
            format!("Error creating contract history: {}", e)
        })?;

    info!(
        "Updated contract ID {} with new amount {}.",
        existing_contract.id, new_contract.current_amount
    );
    Ok(())
}

async fn insert_new_contracts(
    new_contracts: &HashMap<i32, Vec<NewContract>>,
    db: &mut Connection<DbConn>,
) -> Result<HashMap<i32, Vec<Contract>>, String> {
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
            .map_err(|e| {
                error!("Error creating contract: {}", e);
                format!("Error creating contract: {}", e)
            })?;

        inserted_contracts.insert(*bank_id, result);
    }

    Ok(inserted_contracts)
}

fn group_transactions_by_counterparty(
    transactions: Vec<Transaction>,
) -> HashMap<String, Vec<(f64, NaiveDate)>> {
    transactions
        .into_iter()
        .fold(HashMap::new(), |mut acc, transaction| {
            let counterparty = transaction.counterparty;
            let amount = transaction.amount;
            let date = transaction.date;
            acc.entry(counterparty)
                .or_insert_with(Vec::new)
                .push((amount, date));
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
        let (_, date1) = window[0];
        let (_, date2) = window[1];
        let (_, date3) = window[2];

        let duration1 = date2.signed_duration_since(date1);
        let duration2 = date3.signed_duration_since(date2);

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

fn is_within_time_frame(duration: Duration, target_days: i64) -> bool {
    (duration.num_days() - target_days).abs() <= 5
}

fn is_contract_with_reasonable_change(
    existing_contract: &Contract,
    new_contract: &NewContract,
    max_percentage_change: f64,
    max_absolute_change: f64,
) -> bool {
    if existing_contract.bank_id == new_contract.bank_id
        && existing_contract.name == new_contract.name
        && existing_contract.months_between_payment == new_contract.months_between_payment
    {
        let old_amount = existing_contract.current_amount as f64;
        let new_amount = new_contract.current_amount as f64;

        // Calculate percentage change
        let percentage_change = ((new_amount - old_amount) / old_amount).abs() * 100.0;

        // Calculate absolute change
        let absolute_change =
            (new_contract.current_amount - existing_contract.current_amount).abs();

        // Check if the change is within reasonable limits
        if percentage_change <= max_percentage_change || absolute_change <= max_absolute_change {
            return true;
        }
    }

    false
}
