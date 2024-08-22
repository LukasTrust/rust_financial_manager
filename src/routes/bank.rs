use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_performance_value_and_graph_data, get_user_id};
use crate::utils::loading_utils::{load_bank_of_user, load_transactions_of_bank};
use crate::utils::structs::ResponseData;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = load_bank_of_user(cookie_user_id, bank_id, &mut db).await;

    if let Err(error) = current_bank {
        return Ok(Template::render(
            "bank",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into()
                ),
                header: Some(error),
            }),
        ));
    }

    let current_bank = current_bank.unwrap();

    if current_bank.is_none() {
        return Ok(Template::render(
            "bank",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into()
                ),
                header: Some("No bank selected".into()),
            }),
        ));
    }

    let current_bank = current_bank.unwrap();

    state
        .set_current_bank(cookie_user_id, Some(current_bank.clone()))
        .await;

    let transactions = load_transactions_of_bank(current_bank.id, &mut db).await;

    let result =
        get_performance_value_and_graph_data(&vec![current_bank.clone()], None, None, db).await;

    if let Err(error) = result {
        return Ok(Template::render(
            "bank",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into()
                ),
                header: Some(error),
            }),
        ));
    }

    let (performance_value, graph_data) = result.unwrap();

    return Ok(Template::render(
        "bank",
        json!({
            "bank": current_bank,
            "transactions": transactions,
            "graph_data": graph_data,
            "performance_value": performance_value,
        }),
    ));
}
