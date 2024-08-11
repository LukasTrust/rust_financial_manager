use chrono::NaiveDate;
use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, State};
use serde_json::{json, Value};

use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_balance_graph_data, generate_performance_value};
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

    let performance_value;
    let graph_data;

    let transactions_map = state.transactions.read().await;
    match current_bank {
        Ok(bank) => {
            let banks = vec![bank];

            performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            graph_data = generate_balance_graph_data(
                &banks,
                &transactions_map,
                performance_value.1,
                Some(first_date),
                Some(last_date),
            )
            .await;
        }
        Err(_) => {
            let banks = get_banks_of_user(cookie_user_id, state).await;

            performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            graph_data = generate_balance_graph_data(
                &banks,
                &transactions_map,
                performance_value.1,
                Some(first_date),
                Some(last_date),
            )
            .await;
        }
    }

    Ok(Json(json!({
        "graph_data": graph_data,
        "performance_value": performance_value.0,
    })))
}
