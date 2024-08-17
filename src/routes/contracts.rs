use rocket::get;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::get_utils::{get_contracts_with_history, get_user_id};
use crate::utils::loading_utils::load_banks;

#[get("/contract")]
pub async fn contract(
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let banks = load_banks(cookie_user_id, &mut db).await;

    if let Err(e) = banks {
        return Ok(Template::render("contract", json!({ "error": e })));
    }

    let banks = banks.unwrap();

    let result = get_contracts_with_history(banks, db).await;

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
        "contract",
        json!({"contracts": contract_string,
                       "error": error}),
    ))
}
