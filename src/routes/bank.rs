use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_dyn_templates::Template;

use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_balance_graph_data, generate_performance_value};
use crate::utils::get_utils::{
    get_banks_of_user, get_first_date_and_last_date_from_bank, get_user_id,
};

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

            let transactions_of_bank = transactions_map.get(&bank_id);

            let transactions = match transactions_of_bank {
                Some(transactions) => Some(transactions),
                None => None,
            };

            let (first_date, last_date) = get_first_date_and_last_date_from_bank(transactions);

            let performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            let graph_data = generate_balance_graph_data(
                &banks,
                &transactions_map,
                performance_value.1,
                None,
                None,
            )
            .await;

            Ok(Template::render(
                "bank",
                json!({
                    "bank": bank,
                    "graph_data": graph_data,
                    "performance_value": performance_value.0,
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
