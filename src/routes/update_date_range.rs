use chrono::NaiveDate;
use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::{json, Value};

use crate::utils::appstate::AppState;
use crate::utils::display_utils::generate_performance_value;
use crate::utils::get_utils::{get_banks_of_user, get_current_bank, get_user_id};

#[get("/update_date_range/<start_date>/<end_date>")]
pub async fn update_date_range(
    cookies: &CookieJar<'_>,
    start_date: &str,
    end_date: &str,
    state: &State<AppState>,
) -> Result<Json<Value>, Redirect> {
    info!("Updating date range to {} - {}", start_date, end_date);

    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = get_current_bank(cookie_user_id, state).await;

    let first_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let last_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let result;

    let transactions_map = state.transactions.read().await;
    match current_bank {
        Ok(bank) => {
            let banks = vec![bank];

            let performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);
            result = Some(performance_value);
        }
        Err(_) => {
            let banks = get_banks_of_user(cookie_user_id, state).await;

            let performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            result = Some(performance_value);
        }
    }

    let performance_value = result.unwrap();

    Ok(Json(json!({
        "performance_value": performance_value.0,
    })))
}
