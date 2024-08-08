use log::info;
use rocket::form::FromForm;
use rocket::{get, State};

use crate::utils::appstate::AppState;

#[derive(Debug, FromForm)]
pub struct DateRangeForm {
    pub start_date: String,
    pub end_date: String,
}

#[get("/update_date_range/<start_date>/<end_date>")]
pub fn update_date_range(start_date: String, end_date: String, state: &State<AppState>) {
    info!("Updating date range to: {} - {}", start_date, end_date);
}
