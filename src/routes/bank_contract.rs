use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_contracts_with_history, get_user_id};
use crate::utils::structs::ResponseData;

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
            "bank_contract",
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

    let result = get_contracts_with_history(current_bank.id, db).await;

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

    let mut result = json!(ResponseData {
        success: None,
        error: if error.is_none() {
            None
        } else {
            Some(format!(
                "There was an internal error trying to load the contracts of '{}'.",
                current_bank.name
            ))
        },
        header: error,
    });

    result["contracts"] = json!(contract_string);

    Ok(Template::render("bank_contract", json!(result)))
}
