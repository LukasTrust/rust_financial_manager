use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, post, State};
use rocket_db_pools::diesel::AsyncPgConnection;
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::database::models::NewBank;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::insert_bank;
use crate::utils::structs::FormBank;

use super::update_csv::save_update_csv;

#[get("/add-bank")]
pub async fn add_bank(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    get_user_id(cookies)?; // Ensure user is authenticated
    Ok(Template::render("add_bank", json!({})))
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    // Create a new bank instance
    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    // Insert the new bank into the database
    let bank_id = match insert_bank(cookie_user_id, new_bank.clone(), state, &mut db).await {
        Ok(bank_id) => bank_id,
        Err(e) => return Ok(Json(json!({ "error": e }))),
    };

    // Update CSV fields if provided in the form
    if let Err(e) = update_csv_fields(bank_form, state, &mut db, bank_id).await {
        return Ok(Json(json!({ "error": e })));
    }

    // If successful, fetch the updated list of banks
    let banks_clone = state.banks.read().await;
    let banks = banks_clone.get(&cookie_user_id).unwrap();

    Ok(Json(json!({
        "banks": banks,
        "success": format!("Bank {} added", new_bank.name),
    })))
}

async fn update_csv_fields(
    bank_form: Form<FormBank>,
    state: &State<AppState>,
    db: &mut AsyncPgConnection,
    bank_id: i32,
) -> Result<String, String> {
    save_update_csv(
        state,
        db,
        |converter| {
            if let Some(counterparty_column) = bank_form.counterparty_column {
                converter.counterparty_column = Some(counterparty_column);
            }
            if let Some(amount_column) = bank_form.amount_column {
                converter.amount_column = Some(amount_column);
            }
            if let Some(bank_balance_after_column) = bank_form.bank_balance_after_column {
                converter.bank_balance_after_column = Some(bank_balance_after_column);
            }
            if let Some(date_column) = bank_form.date_column {
                converter.date_column = Some(date_column);
            }
        },
        bank_id,
    )
    .await
}
