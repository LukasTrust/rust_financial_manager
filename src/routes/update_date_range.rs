use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, State};
use rocket_dyn_templates::Template;

use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_performance_value, show_base_or_subview_with_data};
use crate::utils::get_utils::{get_banks_of_user, get_current_bank, get_user_id};

#[get("/update_date_range/<start_date>/<end_date>")]
pub async fn update_date_range(
    cookies: &CookieJar<'_>,
    start_date: String,
    end_date: String,
    state: &State<AppState>,
) -> Result<Template, Redirect> {
    info!("Updating date range to {} - {}", start_date, end_date);

    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = get_current_bank(cookie_user_id, state).await;

    let mut performance_data = None;
    let mut view = "base".to_string();

    let transactions = state.transactions.read().await.clone();
    match current_bank {
        Ok(current_bank) => {
            view = "bank".to_string();
            performance_data = Some(generate_performance_value(
                &vec![current_bank],
                &transactions,
                start_date,
                end_date,
            ));
        }
        Err(_) => {
            let banks = get_banks_of_user(cookie_user_id, state).await;

            performance_data = Some(generate_performance_value(
                &banks,
                &transactions,
                start_date,
                end_date,
            ));
        }
    }

    let data = performance_data.unwrap();

    match data {
        Ok(performance_data) => Ok(show_base_or_subview_with_data(
            cookie_user_id,
            state,
            view,
            true,
            true,
            None,
            None,
            Some(performance_data),
        )
        .await),
        Err(error) => Ok(show_base_or_subview_with_data(
            cookie_user_id,
            state,
            view,
            true,
            true,
            None,
            Some(error.to_string()),
            None,
        )
        .await),
    }
}
