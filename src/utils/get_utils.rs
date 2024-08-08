use log::{error, info};
use rocket::{http::CookieJar, response::Redirect, State};

use super::{appstate::AppState, structs::Bank};
use crate::routes::error_page::show_error_page;

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
) -> Result<Bank, Redirect> {
    let current_bank = state.current_bank.read().await;

    let current_bank = current_bank.get(&cookie_user_id);

    match current_bank {
        Some(current_bank) => {
            info!("Current bank found: {:?}", current_bank.id);
            Ok(current_bank.clone())
        }
        None => {
            error!("No current bank found.");
            Err(show_error_page(
                "Error validating the login!".to_string(),
                "Please login again.".to_string(),
            ))
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
) -> Result<crate::database::models::CSVConverter, String> {
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
