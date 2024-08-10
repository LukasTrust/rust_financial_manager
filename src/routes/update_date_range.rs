use log::info;
use rocket::http::CookieJar;
use rocket::{get, State};

use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;

#[get("/update_date_range/<start_date>/<end_date>")]
pub async fn update_date_range(
    cookies: &CookieJar<'_>,
    start_date: String,
    end_date: String,
    state: &State<AppState>,
) {
    info!("Updating date range to {} - {}", start_date, end_date);

    let cookie_user_id = get_user_id(cookies).unwrap();
}
