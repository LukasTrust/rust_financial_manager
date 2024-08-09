use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use serde::de::IntoDeserializer;

use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_performance_value, show_base_or_subview_with_data};
use crate::utils::get_utils::{get_banks_of_user, get_current_bank, get_user_id};

#[get("/update_date_range/<start_date>/<end_date>")]
pub async fn update_date_range(
    cookies: &CookieJar<'_>,
    start_date: String,
    end_date: String,
    state: &State<AppState>,
) {
    info!("Updating date range to {} - {}", start_date, end_date);

    let cookie_user_id = get_user_id(cookies).unwrap();

    let current_bank = get_current_bank(cookie_user_id, state).await;

    let mut view = "base".to_string();
}
