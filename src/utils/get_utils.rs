use chrono::NaiveDate;
use log::{error, info};
use rocket::{http::CookieJar, response::Redirect, State};

use super::{
    appstate::AppState,
    structs::{Bank, Transaction},
};
use crate::{
    database::models::{CSVConverter, Contract},
    routes::error_page::show_error_page,
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

pub async fn get_current_bank(
    cookie_user_id: i32,
    state: &State<AppState>,
) -> Result<Bank, String> {
    let current_bank = state.current_bank.read().await;

    let current_bank = current_bank.get(&cookie_user_id);

    match current_bank {
        Some(current_bank) => {
            info!("Current bank found: {:?}", current_bank.id);
            Ok(current_bank.clone())
        }
        None => {
            error!("No current bank found.");
            Err("No current bank found.".to_string())
        }
    }
}

pub async fn get_banks_of_user(cookie_user_id: i32, state: &State<AppState>) -> Vec<Bank> {
    state
        .banks
        .read()
        .await
        .get(&cookie_user_id)
        .cloned()
        .unwrap_or_default()
}

pub async fn get_csv_converter(
    current_bank_id: i32,
    state: &State<AppState>,
) -> Result<CSVConverter, String> {
    let csv_converters = state.csv_convert.read().await;

    let csv_converter = csv_converters.get(&current_bank_id);

    match csv_converter {
        Some(csv_converter) => {
            info!("CSV converter found: {:?}", csv_converter.id);
            Ok(csv_converter.clone())
        }
        None => {
            error!("No CSV converter found.");
            Err("No CSV converter found.".to_string())
        }
    }
}

pub async fn get_contracts(
    current_bank_id: i32,
    state: &State<AppState>,
) -> Result<Vec<Contract>, String> {
    let contracts = state.contracts.read().await;

    let contracts = contracts.get(&current_bank_id);

    match contracts {
        Some(contracts) => {
            info!("Contracts found: {:?}", contracts.len());
            Ok(contracts.clone())
        }
        None => {
            error!("No contracts found.");
            Err("No contracts found.".to_string())
        }
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
