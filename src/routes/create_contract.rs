use std::collections::{HashMap, HashSet};

use chrono::{Datelike, NaiveDate};
use log::info;
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, NewContract, NewContractHistory};
use crate::utils::insert_utiles::{insert_contract, insert_contract_history};
use crate::utils::loading_utils::{
    load_contracts_of_bank_without_end_date, load_transactions_of_bank_without_contract,
};
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{
    update_contract_with_new_amount, update_transaction_with_contract_id,
};

type Result<T> = std::result::Result<T, String>;

pub async fn create_contract_from_transactions(
    bank_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<String> {
    let existing_contracts = load_contracts_of_bank_without_end_date(bank_id, db).await?;

    info!("Contracts loaded: {}", existing_contracts.len());

    let mut transactions = load_transactions_of_bank_without_contract(bank_id, db).await?;

    info!("Transactions loaded: {}", transactions.len());

    let transactions_matching_a_contract = filter_transactions_matching_to_existing_contract(
        transactions.clone(),
        existing_contracts.clone(),
    );

    info!(
        "Transactions matching a contract: {}",
        transactions_matching_a_contract.len()
    );

    update_transactions_with_contract_id(transactions_matching_a_contract.clone(), db).await?;

    transactions.retain(|transaction| {
        !transactions_matching_a_contract
            .values()
            .flatten()
            .any(|t| t.id == transaction.id)
    });

    info!("Transactions left after matching: {}", transactions.len());

    let transactions_matching_changed_contracts = filter_transactions_matching_changed_contract(
        transactions.clone(),
        existing_contracts.clone(),
    );

    info!(
        "Transactions matching changed contracts: {}",
        transactions_matching_changed_contracts.len()
    );

    create_contract_history(
        transactions_matching_changed_contracts.clone(),
        existing_contracts,
        db,
    )
    .await?;

    transactions.retain(|transaction| {
        !transactions_matching_changed_contracts
            .values()
            .flatten()
            .any(|t| t.id == transaction.id)
    });

    info!("Transactions left after matching: {}", transactions.len());

    let grouped_transactions = group_transactions_by_counterparty_and_amount(transactions);

    info!(
        "Transactions grouped by counterparty: {}",
        grouped_transactions.len()
    );

    let count_new_contracts =
        create_contracts_from_transactions(bank_id, grouped_transactions, db).await?;

    Ok(format!("Found {} new contracts!", count_new_contracts))
}

fn filter_transactions_matching_to_existing_contract(
    transactions: Vec<Transaction>,
    existing_contracts: Vec<Contract>,
) -> HashMap<i32, Vec<Transaction>> {
    let mut contract_transactions: HashMap<i32, Vec<Transaction>> = HashMap::new();

    for transaction in transactions.into_iter() {
        if let Some(contract) = existing_contracts.iter().find(|contract| {
            transaction.amount == contract.current_amount
                && transaction.counterparty == contract.name
        }) {
            contract_transactions
                .entry(contract.id)
                .or_insert_with(Vec::new)
                .push(transaction);
        }
    }

    contract_transactions
}

fn filter_transactions_matching_changed_contract(
    transactions: Vec<Transaction>,
    existing_contracts: Vec<Contract>,
) -> HashMap<i32, Vec<Transaction>> {
    let mut contract_transactions: HashMap<i32, Vec<Transaction>> = HashMap::new();

    for transaction in transactions.into_iter() {
        if let Some(contract) = existing_contracts.iter().find(|contract| {
            if transaction.counterparty == contract.name {
                let threshold_percentage = 0.1;
                let diff = (transaction.amount as f64 - contract.current_amount as f64).abs();
                let allowed_change = contract.current_amount as f64 * threshold_percentage;
                diff <= allowed_change
            } else {
                false
            }
        }) {
            contract_transactions
                .entry(contract.id)
                .or_insert_with(Vec::new)
                .push(transaction);
        }
    }

    contract_transactions
}

async fn update_transactions_with_contract_id(
    contracts_with_transactions: HashMap<i32, Vec<Transaction>>,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    for (contract_id, transactions) in contracts_with_transactions {
        for transaction in transactions {
            update_transaction_with_contract_id(transaction.id, contract_id, db).await?;
        }
    }

    Ok(())
}

fn round_to_i64(amount: f64, precision: u32) -> i64 {
    let scale = 10_f64.powi(precision as i32);
    (amount * scale).round() as i64
}

async fn create_contract_history(
    contracts_with_transactions: HashMap<i32, Vec<Transaction>>,
    existing_contracts: Vec<Contract>,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    for (contract_id, mut transactions) in contracts_with_transactions.clone() {
        // Find the corresponding contract
        let contract = match existing_contracts
            .iter()
            .find(|contract| contract.id == contract_id)
        {
            Some(contract) => contract,
            None => {
                // Return an error if the contract is not found
                return Err(format!("Contract with ID {} not found", contract_id).into());
            }
        };

        // Sort transactions by date to process them in chronological order
        transactions.sort_by_key(|transaction| transaction.date);

        // Use a set to track processed (amount, date) pairs
        let mut processed_pairs = HashSet::new();

        for transaction in transactions.iter() {
            // Round amount to 2 decimal places (or other precision)
            let rounded_amount = round_to_i64(transaction.amount, 2);
            let pair = (rounded_amount, transaction.date);

            if processed_pairs.contains(&pair) {
                continue; // Skip if we've already processed this pair
            }

            processed_pairs.insert(pair);

            let contract_history = NewContractHistory {
                contract_id: contract_id,
                old_amount: contract.current_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            // Insert the new contract history
            insert_contract_history(contract_history, db).await?;

            // Update the contract with the new amount
            update_contract_with_new_amount(contract_id, transaction.amount, db).await?;
        }
    }

    update_transactions_with_contract_id(contracts_with_transactions, db).await?;

    Ok(())
}

fn group_transactions_by_counterparty_and_amount(
    transactions: Vec<Transaction>,
) -> HashMap<String, HashMap<i64, Vec<(f64, NaiveDate)>>> {
    let mut counterparty_map: HashMap<String, HashMap<i64, Vec<(f64, NaiveDate)>>> = HashMap::new();

    // Step 1: Group by counterparty and amount
    for transaction in transactions {
        let counterparty = transaction.counterparty.clone();
        let amount = transaction.amount;
        let date = transaction.date;
        let amount_key = (amount * 100.0) as i64;

        let inner_map = counterparty_map
            .entry(counterparty.clone())
            .or_insert_with(HashMap::new);

        inner_map
            .entry(amount_key)
            .or_insert_with(Vec::new)
            .push((amount, date));
    }

    // Step 2: Remove inner HashMaps with fewer than 2 values
    counterparty_map.retain(|_, inner_map| {
        inner_map.retain(|_, values| values.len() >= 2);
        !inner_map.is_empty()
    });

    counterparty_map
}

async fn create_contracts_from_transactions(
    bank_id: i32,
    grouped_transactions: HashMap<String, HashMap<i64, Vec<(f64, NaiveDate)>>>,
    db: &mut Connection<DbConn>,
) -> Result<usize> {
    let mut contracts = Vec::new();

    for (counterparty, amount_groups) in grouped_transactions {
        for (amount_key, transactions) in amount_groups {
            // Sort transactions by date
            let mut sorted_transactions = transactions.clone();
            sorted_transactions.sort_by_key(|(_, date)| *date);

            let mut i = 0;
            while i < sorted_transactions.len() {
                let mut j = i + 1;
                while j < sorted_transactions.len() {
                    let date_i = sorted_transactions[i].1;
                    let date_j = sorted_transactions[j].1;

                    if let Some(months) = months_between(date_i, date_j) {
                        if [1, 3, 6].contains(&months) {
                            let contract = NewContract {
                                bank_id: bank_id,
                                name: counterparty.clone(),
                                current_amount: amount_key as f64 / 100.0,
                                months_between_payment: months,
                            };
                            contracts.push(contract);
                            break;
                        }
                    }
                    j += 1;
                }
                i += 1;
            }
        }
    }

    for contract in contracts.clone() {
        insert_contract(contract, db).await?;
    }

    Ok(contracts.len())
}

fn months_between(date1: NaiveDate, date2: NaiveDate) -> Option<i32> {
    let months_diff =
        (date2.year() - date1.year()) * 12 + date2.month() as i32 - date1.month() as i32;
    let diff_days = (date2 - date1).num_days() as i32;
    if diff_days.abs() <= 5 + (months_diff * 30) {
        Some(months_diff)
    } else {
        None
    }
}
