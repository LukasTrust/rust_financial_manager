use std::time::Instant;

use chrono::NaiveDate;
use log::{error, info, warn};
use rocket::{http::CookieJar, response::Redirect, serde::json::Json, State};
use rocket_db_pools::Connection;

use crate::{
    database::db_connector::DbConn, routes::error_page::show_error_page,
    utils::loading_utils::load_transaction_by_id,
};

use super::{
    appstate::AppState,
    display_utils::{generate_graph_data, generate_performance_value},
    loading_utils::{
        load_contract_history, load_contracts_of_bank, load_last_transaction_data_of_contract,
        load_transactions_of_bank, load_transactions_of_contract,
    },
    structs::{
        Bank, ContractWithHistory, PerformanceData, ResponseData, Transaction,
        TransactionWithContract,
    },
};

/// Extract the user ID from the user ID cookie.
/// If the user ID cookie is not found or cannot be parsed, an error page is displayed.
/// The user ID is returned if the user ID cookie is found and parsed successfully.
pub fn get_user_id(cookies: &CookieJar<'_>) -> Result<i32, Box<Redirect>> {
    if let Some(cookie_user_id) = cookies.get_private("user_id") {
        info!("User ID cookie found: {:?}", cookie_user_id.value());

        cookie_user_id.value().parse::<i32>().map_err(|_| {
            error!("Error parsing user ID cookie.");
            show_error_page(
                "Error validating the login!".to_string(),
                "Please login again.".to_string(),
            )
        })
    } else {
        error!("No user ID cookie found.");
        Err(show_error_page(
            "Error validating the login!".to_string(),
            "Please login again.".to_string(),
        ))
    }
}

pub fn get_first_date_and_last_date_from_bank(
    transactions: Option<&Vec<Transaction>>,
) -> (NaiveDate, NaiveDate) {
    let mut first_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let mut last_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

    if transactions.is_some() {
        let transactions = transactions.unwrap();

        first_date = transactions
            .iter()
            .min_by_key(|t| t.date)
            .map(|t| t.date)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

        last_date = transactions
            .iter()
            .max_by_key(|t| t.date)
            .map(|t| t.date)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }

    (first_date, last_date)
}

pub async fn get_performance_value_and_graph_data(
    banks: &Vec<Bank>,
    input_first_date: Option<NaiveDate>,
    input_last_date: Option<NaiveDate>,
    mut db: Connection<DbConn>,
) -> Result<(PerformanceData, String), String> {
    let mut all_transactions = Vec::new();
    let mut all_contracts = Vec::new();

    for bank in banks {
        let transactions = load_transactions_of_bank(bank.id, &mut db).await?;

        all_transactions.extend(transactions);

        let contracts = load_contracts_of_bank(bank.id, &mut db).await?;

        all_contracts.extend(contracts);
    }

    let (first_date, last_date);

    if input_first_date.is_none() || input_last_date.is_none() {
        (first_date, last_date) = get_first_date_and_last_date_from_bank(Some(&all_transactions));
    } else {
        first_date = input_first_date.unwrap();
        last_date = input_last_date.unwrap();
    }

    let performance_value = generate_performance_value(
        banks,
        &all_transactions,
        &all_contracts,
        &first_date,
        &last_date,
    );

    let graph_data = generate_graph_data(
        banks,
        &all_transactions,
        &performance_value.1,
        &first_date,
        &last_date,
    )
    .await;

    Ok((performance_value.0, graph_data))
}

pub async fn get_total_amount_paid_of_contract(
    contract_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<f64, String> {
    let transactions = load_transactions_of_contract(contract_id, db).await?;

    Ok(transactions.iter().map(|t| t.amount).sum())
}

pub async fn get_contracts_with_history(
    bank_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let mut contracts_with_history: Vec<ContractWithHistory> = Vec::new();

    let contracts = load_contracts_of_bank(bank_id, db).await;

    match contracts {
        Ok(contracts) => {
            for contract in contracts.iter() {
                let contract_history = load_contract_history(contract.id, db).await;

                if contract_history.is_err() {
                    return Err("Error loading contract history.".to_string());
                }

                let total_amount_paid = get_total_amount_paid_of_contract(contract.id, db).await?;

                let last_payment_date =
                    load_last_transaction_data_of_contract(contract.id, db).await?;

                if last_payment_date.is_none() {
                    return Err("Error loading last payment date.".to_string());
                }

                let last_payment_date = last_payment_date.unwrap().date;

                let contract_with_history = ContractWithHistory {
                    contract: contract.clone(),
                    contract_history: contract_history.unwrap(),
                    total_amount_paid,
                    last_payment_date,
                };

                contracts_with_history.push(contract_with_history);
            }
        }
        Err(err) => return Err(err),
    }

    Ok(serde_json::to_string(&contracts_with_history).unwrap())
}

pub async fn get_transactions_with_contract(
    bank_id: i32,
    mut db: Connection<DbConn>,
) -> Result<String, String> {
    let transactions = load_transactions_of_bank(bank_id, &mut db).await?;

    let mut transactions_with_contract = Vec::new();
    let contracts = load_contracts_of_bank(bank_id, &mut db).await?;

    for transaction in transactions.iter() {
        let contract = if transaction.contract_id.is_some() {
            contracts
                .iter()
                .find(|c| c.id == transaction.contract_id.unwrap())
        } else {
            None
        };

        let transaction_with_contract = TransactionWithContract {
            transaction: transaction.clone(),
            contract: contract.cloned(),
        };

        transactions_with_contract.push(transaction_with_contract);
    }

    Ok(serde_json::to_string(&transactions_with_contract).unwrap())
}

pub async fn get_transaction(
    transaction_id: i32,
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Result<Transaction, Json<ResponseData>> {
    let start_time = Instant::now();

    let transaction = load_transaction_by_id(transaction_id, db).await;

    if let Err(error) = transaction {
        error!("Error loading transaction {}: {}", transaction_id, error);
        return Err(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_loading_transactions")
                .await,
        )));
    }

    let transaction = transaction.unwrap();

    if transaction.is_none() {
        info!("Transaction {} not found", transaction_id);
        return Err(Json(ResponseData::new_error(
            state
                .localize_message(cookie_user_id, "transaction_not_found")
                .await,
            state
                .localize_message(cookie_user_id, "transaction_not_found_details")
                .await,
        )));
    }

    warn!("Transaction loaded in {:?}", start_time.elapsed());
    Ok(transaction.unwrap())
}
