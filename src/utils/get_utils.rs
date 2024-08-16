use chrono::NaiveDate;
use log::{error, info};
use rocket::{http::CookieJar, response::Redirect};
use rocket_db_pools::Connection;

use crate::{database::db_connector::DbConn, routes::error_page::show_error_page};

use super::{
    display_utils::{generate_graph_data, generate_performance_value},
    loading_utils::{
        load_contracts_of_bank, load_transactions_of_bank, load_transactions_of_contract,
    },
    structs::{Bank, PerformanceData, Transaction},
};

/// Extract the user ID from the user ID cookie.
/// If the user ID cookie is not found or cannot be parsed, an error page is displayed.
/// The user ID is returned if the user ID cookie is found and parsed successfully.
pub fn get_user_id(cookies: &CookieJar<'_>) -> Result<i32, Redirect> {
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
        let transactions = load_transactions_of_bank(bank.id, &mut db).await;

        if let Err(e) = transactions {
            return Err(e);
        }

        let transactions = transactions.unwrap();

        all_transactions.extend(transactions);

        let contracts = load_contracts_of_bank(bank.id, &mut db).await;

        if let Err(e) = contracts {
            return Err(e);
        }

        let contracts = contracts.unwrap();

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
