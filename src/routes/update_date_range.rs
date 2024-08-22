use chrono::NaiveDate;
use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use serde_json::{json, Value};

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_performance_value_and_graph_data, get_user_id};
use crate::utils::loading_utils::load_banks;
use crate::utils::structs::ResponseData;

#[get("/update_date_range/<start_date>/<end_date>")]
pub async fn update_date_range(
    cookies: &CookieJar<'_>,
    start_date: &str,
    end_date: &str,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Redirect> {
    info!("Updating date range to {} - {}", start_date, end_date);

    let cookie_user_id = get_user_id(cookies)?;

    let first_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let last_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let current_bank = state.get_current_bank(cookie_user_id).await;

    info!("Current bank: {:?}", current_bank);

    match current_bank {
        Some(current_bank) => {
            let result = get_performance_value_and_graph_data(
                &vec![current_bank],
                Some(first_date),
                Some(last_date),
                db,
            )
            .await;

            if let Err(error) = result {
                return Ok(Json(json!({
                    "response":
                    ResponseData {
                        success: None,
                        error: Some("There was an internal error while loading the bank. Please try again.".into()),
                        header: Some(error),
                    },
                })));
            }

            let (performance_value, graph_data) = result.unwrap();

            Ok(Json(json!({
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
        None => {
            let banks = load_banks(cookie_user_id, &mut db).await;

            if let Err(error) = banks {
                return Ok(Json(json!({ "response":
                ResponseData {
                    success: None,
                    error: Some("There was an internal error trying to load the banks of the profile".into()),
                    header: Some(error),
                }})));
            }

            let banks = banks.unwrap();

            let result =
                get_performance_value_and_graph_data(&banks, Some(first_date), Some(last_date), db)
                    .await;

            if let Err(error) = result {
                return Ok(Json(json!({ "response":
                ResponseData {
                    success: None,
                    error: Some("There was an internal error while loading the bank. Please try again.".into()),
                    header: Some(error),
                }})));
            }

            let (performance_value, graph_data) = result.unwrap();

            Ok(Json(json!({
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
    }
}
