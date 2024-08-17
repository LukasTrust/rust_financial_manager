use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_contracts_with_history, get_user_id};

#[get("/bank/contract")]
pub async fn bank_contract(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Template::render(
            "contract",
            json!({ "error": "No bank selected" }),
        ));
    }

    let current_bank = current_bank.unwrap();

    let result = get_contracts_with_history(vec![current_bank], db).await;

    let error = if result.is_err() {
        Some(result.clone().err().unwrap())
    } else {
        None
    };

    let contract_string = if result.is_ok() {
        result.unwrap()
    } else {
        String::new()
    };

    Ok(Template::render(
        "bank_contracts",
        json!({"contracts": contract_string,
                       "error": error}),
    ))
}
