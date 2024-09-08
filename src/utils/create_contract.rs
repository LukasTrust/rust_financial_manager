use chrono::{Datelike, NaiveDate};
use log::info;
use rocket::serde::json::Json;
use rocket::tokio;
use rocket_db_pools::Connection;
use std::collections::{HashMap, HashSet};

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, NewContract, NewContractHistory};
use crate::utils::insert_utiles::{insert_contract_histories, insert_contracts};
use crate::utils::loading_utils::{
    load_contracts_of_bank_without_end_date, load_last_transaction_data_of_bank,
    load_last_transaction_of_contract,
    load_transactions_of_bank_without_contract_and_contract_allowed,
};
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{
    update_contract_with_end_date, update_contract_with_new_amount,
    update_transactions_with_contract,
};

use super::appstate::{Language, LOCALIZATION};
use super::structs::{CounterpartyMap, ErrorResponse};

type Result<T> = std::result::Result<T, Json<ErrorResponse>>;

pub async fn create_contract_from_transactions(
    bank_id: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<String> {
    let existing_contracts = load_contracts_of_bank_without_end_date(bank_id, language, db).await?;

    let mut transactions =
        load_transactions_of_bank_without_contract_and_contract_allowed(bank_id, language, db)
            .await?;

    if transactions.is_empty() {
        return Ok("No transactions without contract found!".to_string());
    }

    let transactions_matching_a_contract = filter_transactions_matching_to_existing_contract(
        transactions.clone(),
        existing_contracts.clone(),
    );

    info!(
        "Transactions matching a contract: {}",
        transactions_matching_a_contract.len()
    );

    update_transactions_with_contract_id_local(
        transactions_matching_a_contract.clone(),
        language,
        db,
    )
    .await?;

    transactions.retain(|transaction| {
        !transactions_matching_a_contract
            .values()
            .flatten()
            .any(|t| t.id == transaction.id)
    });

    info!("Transactions left after matching: {}", transactions.len());

    let transaction_clone = transactions.clone();
    let existing_contracts_clone = existing_contracts.clone();

    let transactions_matching_changed_contracts_task = tokio::spawn(async {
        filter_transactions_matching_changed_contract(transaction_clone, existing_contracts_clone)
    });

    let transactions_matching_changed_contracts =
        transactions_matching_changed_contracts_task.await.unwrap();

    info!(
        "Transactions matching changed contracts: {}",
        transactions_matching_changed_contracts.len()
    );

    create_contract_history(
        transactions_matching_changed_contracts.clone(),
        existing_contracts.clone(),
        language,
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

    let new_contracts_ids =
        create_contracts_from_transactions(bank_id, grouped_transactions, language, db).await?;

    let mut all_contracts = new_contracts_ids.clone();
    all_contracts.extend(existing_contracts);

    let last_transaction = load_last_transaction_data_of_bank(bank_id, language, db).await?;

    let last_transaction_date = last_transaction.date;

    let contracts_closed =
        check_if_contract_should_be_closed(all_contracts, last_transaction_date, language, db)
            .await?;

    info!("Contracts closed: {}", contracts_closed);

    let base_message = format!("Found {} new contracts!", new_contracts_ids.len());

    let return_string = if contracts_closed > 0 {
        format!("{} Closed {} contracts!", base_message, contracts_closed)
    } else {
        base_message
    };

    Ok(return_string)
}

fn filter_transactions_matching_to_existing_contract(
    transactions: Vec<Transaction>,
    existing_contracts: Vec<Contract>,
) -> HashMap<i32, Vec<Transaction>> {
    let mut contract_transactions: HashMap<i32, Vec<Transaction>> = HashMap::new();

    for transaction in transactions.into_iter() {
        if let Some(contract) = existing_contracts.iter().find(|contract| {
            transaction.amount == contract.current_amount
                && transaction.counterparty == contract.parse_name
        }) {
            contract_transactions
                .entry(contract.id)
                .or_default()
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
            if transaction.counterparty == contract.parse_name {
                let threshold_percentage = 0.15;
                let diff = (transaction.amount - contract.current_amount).abs();
                let allowed_change = contract.current_amount * threshold_percentage;
                diff <= allowed_change
            } else {
                false
            }
        }) {
            contract_transactions
                .entry(contract.id)
                .or_default()
                .push(transaction);
        }
    }

    contract_transactions
}

async fn update_transactions_with_contract_id_local(
    contracts_with_transactions: HashMap<i32, Vec<Transaction>>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    for (contract_id, transactions) in contracts_with_transactions {
        let ids = transactions
            .iter()
            .map(|transaction| transaction.id)
            .collect::<Vec<i32>>();

        update_transactions_with_contract(ids, Some(contract_id), language, db).await?;
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
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<()> {
    let mut new_contract_histories = Vec::new();

    for (contract_id, mut transactions) in contracts_with_transactions.clone() {
        let contract = match existing_contracts
            .iter()
            .find(|contract| contract.id == contract_id)
        {
            Some(contract) => contract,
            None => {
                return Err(Json(ErrorResponse::new(
                    LOCALIZATION.get_localized_string(language, "error_contract_not_found"),
                    LOCALIZATION.get_localized_string(language, "error_contract_not_found_details"),
                )))
            }
        };

        transactions.sort_by_key(|transaction| transaction.date);
        let mut processed_pairs = HashSet::new();

        for transaction in transactions.iter() {
            let rounded_amount = round_to_i64(transaction.amount, 2);
            let pair = (rounded_amount, transaction.date);

            if processed_pairs.contains(&pair) {
                continue;
            }

            processed_pairs.insert(pair);

            let contract_history = NewContractHistory {
                contract_id,
                old_amount: contract.current_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            new_contract_histories.push(contract_history);

            update_contract_with_new_amount(contract_id, transaction.amount, language, db).await?;
        }
    }

    insert_contract_histories(&new_contract_histories, language, db).await?;

    update_transactions_with_contract_id_local(contracts_with_transactions, language, db).await?;

    Ok(())
}

fn group_transactions_by_counterparty_and_amount(
    transactions: Vec<Transaction>,
) -> CounterpartyMap {
    let mut counterparty_map: CounterpartyMap = HashMap::new();

    for transaction in transactions {
        let counterparty = transaction.counterparty.clone();
        let amount = transaction.amount;
        let date = transaction.date;
        let amount_key = (amount * 100.0) as i64;

        let inner_map = counterparty_map.entry(counterparty.clone()).or_default();

        inner_map
            .entry(amount_key)
            .or_default()
            .push((amount, date, transaction.id));
    }

    counterparty_map.retain(|_, inner_map| {
        inner_map.retain(|_, values| values.len() > 2);
        !inner_map.is_empty()
    });

    counterparty_map
}

async fn create_contracts_from_transactions(
    bank_id: i32,
    grouped_transactions: CounterpartyMap,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>> {
    let mut created_contracts = HashSet::new();
    let mut contract_insertions = Vec::new();
    let mut contract_keys = Vec::new();
    let mut transaction_updates = Vec::new();
    let allowable_gap = 6;

    for (counterparty, amount_groups) in grouped_transactions {
        for (amount_key, transactions) in amount_groups {
            let mut sorted_transactions = transactions.clone();
            sorted_transactions.sort_by_key(|(_, date, _)| *date);

            let mut i = 0;
            while i < sorted_transactions.len() {
                let mut transaction_ids = vec![sorted_transactions[i].2];
                let mut months_pattern = None;
                let mut j = i + 1;
                let mut last_valid_index = i;

                while j < sorted_transactions.len() {
                    let date_i = sorted_transactions[last_valid_index].1;
                    let date_j = sorted_transactions[j].1;

                    if let Some(months) = months_between(date_i, date_j) {
                        if [1, 2, 3, 6, 12].contains(&months)
                            || (months != 0 && months <= allowable_gap)
                        {
                            if months_pattern.is_none() {
                                months_pattern = Some(months);
                            } else if months_pattern != Some(months) && months > allowable_gap {
                                break;
                            }
                            transaction_ids.push(sorted_transactions[j].2);
                            last_valid_index = j; // Only update the last valid index
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                    j += 1;
                }

                if let Some(months) = months_pattern {
                    let contract_key = (counterparty.clone(), amount_key, months);

                    if !created_contracts.contains(&contract_key) {
                        let contract = NewContract {
                            bank_id,
                            name: counterparty.clone(),
                            parse_name: counterparty.clone(),
                            current_amount: amount_key as f64 / 100.0,
                            months_between_payment: months,
                        };

                        contract_insertions.push(contract);
                        contract_keys.push(contract_key.clone());
                        transaction_updates.push((transaction_ids, contract_key.clone()));

                        created_contracts.insert(contract_key);
                    }
                }
                i = last_valid_index + 1;
            }
        }
    }

    // Batch insert contracts and retrieve the inserted contract IDs
    let inserted_contracts = insert_contracts(&contract_insertions, language, db).await?;

    // Ensure the number of inserted contracts matches the contract_keys
    assert_eq!(inserted_contracts.len(), contract_keys.len());

    // Update transactions with the corresponding contract IDs
    for (i, (transaction_ids, _contract_key)) in transaction_updates.iter().enumerate() {
        let contract_id = inserted_contracts[i].id;
        update_transactions_with_contract(transaction_ids.clone(), Some(contract_id), language, db)
            .await?;
    }

    Ok(inserted_contracts)
}

fn months_between(date1: NaiveDate, date2: NaiveDate) -> Option<i32> {
    let year_diff = date2.year() - date1.year();
    let month_diff = date2.month() as i32 - date1.month() as i32;

    // Calculate the total difference in months
    let total_months = year_diff * 12 + month_diff;

    // Calculate the difference in days
    let day_diff = (date2 - date1).num_days().abs();

    // Set a tolerance for days, e.g., 5 days
    let day_tolerance = 5;

    // Adjusted logic
    if total_months > 0 {
        Some(total_months)
    } else if total_months == 0 && day_diff <= day_tolerance {
        Some(0) // In the same month, within tolerance
    } else if total_months == 1 && date2.day() < date1.day() && day_diff <= day_tolerance {
        Some(1) // Within the next month and within day tolerance
    } else {
        None // Too far apart to be considered the same or sequential month(s)
    }
}

async fn check_if_contract_should_be_closed(
    contracts: Vec<Contract>,
    last_transaction_date: NaiveDate,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<i32> {
    let mut closed_contracts = 0;

    for contract in contracts {
        let last_transaction_of_contract =
            load_last_transaction_of_contract(contract.id, language, db).await?;

        let last_transaction_of_contract = last_transaction_of_contract.date;

        let months_between = months_between(last_transaction_of_contract, last_transaction_date);

        if let Some(months) = months_between {
            if months > contract.months_between_payment * 2 {
                update_contract_with_end_date(
                    contract.id,
                    Some(last_transaction_of_contract),
                    language,
                    db,
                )
                .await?;

                closed_contracts += 1;
            }
        }
    }

    Ok(closed_contracts)
}
