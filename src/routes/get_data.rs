use rocket::http::CookieJar;
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
) -> Result<Json<Value>, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    match current_bank {
        Ok(current_bank) => {
            let (performance_value, graph_data) = get_performance_value_and_graph_data(
                &vec![current_bank.clone()],
                None,
                None,
                language,
                db,
            )
            .await?;

            let graph_data = json!(graph_data);

            Ok(Json(json!({
                "bank": current_bank,
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
        Err(_) => {
            let banks = load_banks_of_user(cookie_user_id, language, &mut db).await?;

            let (performance_value, graph_data) =
                get_performance_value_and_graph_data(&banks, None, None, language, db).await?;

            let graph_data = json!(graph_data);

            Ok(Json(json!({
                "graph_data": graph_data,
                "performance_value": performance_value,
            })))
        }
    }
}
