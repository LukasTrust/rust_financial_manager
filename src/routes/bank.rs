use chrono::NaiveDate;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_dyn_templates::Template;

use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_balance_graph_data, generate_performance_value};
use crate::utils::get_utils::{get_banks_of_user, get_user_id};

use super::error_page::show_error_page;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let transactions_map = state.transactions.read().await;
    let banks = get_banks_of_user(cookie_user_id, state).await;

    let current_bank = banks.iter().find(|b| b.id == bank_id);

    match current_bank {
        Some(bank) => {
            state
                .update_current_bank(cookie_user_id, Some(bank.clone()))
                .await;

            let banks = vec![bank.clone()];

            // Fetch the transactions for the bank_id
            let transactions = transactions_map.get(&bank_id);

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
                    .min_by_key(|t| t.date)
                    .map(|t| t.date)
                    .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
            }

            let graph_data = generate_balance_graph_data(&banks, &transactions_map).await;
            let performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            Ok(Template::render(
                "bank",
                json!({
                    "bank": bank,
                    "graph_data": graph_data,
                    "performance_value": performance_value,
                }),
            ))
        }
        None => {
            return Err(show_error_page(
                "Bank not found".to_string(),
                "Please try again later".to_string(),
            ))
        }
    }
}
