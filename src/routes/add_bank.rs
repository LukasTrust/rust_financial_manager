use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::database::models::NewBank;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::insert_bank;
use crate::utils::structs::FormBank;

use super::update_csv::update_csv;

#[get("/add-bank")]
pub async fn add_bank(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    let _ = get_user_id(cookies)?;

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

    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    let result = insert_bank(cookie_user_id, new_bank.clone(), state, &mut db).await;

    let mut error = None;

    match result {
        Ok(bank_id) => {
            if error.is_none() && bank_form.counterparty_column.is_some() {
                let counterparty_result = update_csv(
                    state,
                    db.as_mut(),
                    |converter| {
                        converter.counterparty_column = bank_form.counterparty_column.clone();
                    },
                    bank_id,
                )
                .await;

                if counterparty_result.is_err() {
                    error = Some("Error updating counterparty".to_string());
                }
            }

            if error.is_none() && bank_form.amount_column.is_some() {
                let amount_result = update_csv(
                    state,
                    db.as_mut(),
                    |converter| {
                        converter.amount_column = bank_form.amount_column.clone();
                    },
                    bank_id,
                )
                .await;

                if amount_result.is_err() {
                    error = Some("Error updating amount".to_string());
                }
            }

            if error.is_none() && bank_form.bank_balance_after_column.is_some() {
                let bank_balance_after_result = update_csv(
                    state,
                    db.as_mut(),
                    |converter| {
                        converter.bank_balance_after_column =
                            bank_form.bank_balance_after_column.clone();
                    },
                    bank_id,
                )
                .await;

                if bank_balance_after_result.is_err() {
                    error = Some("Error updating bank balance after".to_string());
                }
            }

            if error.is_none() && bank_form.date_column.is_some() {
                let date_result = update_csv(
                    state,
                    db.as_mut(),
                    |converter| {
                        converter.date_column = bank_form.date_column.clone();
                    },
                    bank_id,
                )
                .await;

                if date_result.is_err() {
                    error = Some("Error updating date".to_string());
                }
            }
        }
        Err(e) => error = Some(e),
    };

    let mut success = None;
    if error.is_none() {
        success = Some(format!("Bank {} added", new_bank.name));
    }

    let banks_clone = state.banks.read().await;
    let banks = banks_clone.get(&cookie_user_id).unwrap();

    Ok(Json(json!({
        "banks": banks,
        "success": success,
        "error": error,
    })))
}
