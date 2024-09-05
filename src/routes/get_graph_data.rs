use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use serde_json::{json, Value};

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_performance_value_and_graph_data, get_user_id};
use crate::utils::loading_utils::load_banks_of_user;
use crate::utils::structs::ResponseData;

#[get("/get/graph/data")]
pub async fn get_graph_data(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    match current_bank {
        Some(current_bank) => {
            let result =
                get_performance_value_and_graph_data(&vec![current_bank.clone()], None, None, db)
                    .await;

            let (performance_value, graph_data) = result.unwrap();

            let graph_data = json!(graph_data);

            Ok(Json(json!({
                "bank": current_bank,
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
        None => {
            let banks = load_banks_of_user(cookie_user_id, &mut db).await;

            if let Err(error) = banks {
                return Ok(Json(json!(ResponseData::new_error(
                    error,
                    "There was an internal error trying to load the banks of the profile"
                ))));
            }

            let banks = banks.unwrap();

            let result = get_performance_value_and_graph_data(&banks, None, None, db).await;

            let (performance_value, graph_data) = result.unwrap();

            let graph_data = json!(graph_data);

            Ok(Json(json!({
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
    }
}
