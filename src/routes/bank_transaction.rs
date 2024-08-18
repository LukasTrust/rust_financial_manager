use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_transactions_with_contract, get_user_id};

#[get("/bank/transaction")]
pub async fn bank_trasaction(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Template::render(
            "bank_trasaction",
            json!({ "error": "No bank selected" }),
        ));
    }

    let current_bank = current_bank.unwrap();

    let result = get_transactions_with_contract(current_bank.id, db).await;

    let error = if result.is_err() {
        Some(result.clone().err().unwrap())
    } else {
        None
    };

    let transaction_string = if result.is_ok() {
        result.unwrap()
    } else {
        String::new()
    };

    Ok(Template::render(
        "bank_transaction",
        json!({"transactions": transaction_string, "error": error}),
    ))
}
