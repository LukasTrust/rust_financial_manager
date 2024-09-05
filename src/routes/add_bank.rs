use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::database::db_mocking::{
    insert_bank_mocking, insert_csv_converter_mocking, load_banks_of_user_mocking,
};
use crate::database::models::{NewBank, NewCSVConverter};
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::{insert_bank, insert_csv_converter};
use crate::utils::loading_utils::load_banks_of_user;
use crate::utils::structs::{FormBank, ResponseData};

#[get("/add-bank")]
pub async fn add_bank(cookies: &CookieJar<'_>) -> Result<Template, Box<Redirect>> {
    get_user_id(cookies)?;
    Ok(Template::render("add_bank", json!({})))
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    // Create a new bank instance
    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    let insert_bank_result = match state.use_mocking {
        true => insert_bank_mocking(new_bank.clone()),
        false => insert_bank(new_bank.clone(), &mut db).await,
    };

    match insert_bank_result {
        Ok(bank) => {
            let new_csv_converter = NewCSVConverter {
                bank_id: bank.id,
                counterparty_column: bank_form.counterparty_column,
                amount_column: bank_form.amount_column,
                bank_balance_after_column: bank_form.bank_balance_after_column,
                date_column: bank_form.date_column,
            };

            let insert_csv_converter_result = match state.use_mocking {
                true => insert_csv_converter_mocking(new_csv_converter),
                false => insert_csv_converter(new_csv_converter, &mut db).await,
            };

            match insert_csv_converter_result {
                Ok(_) => {
                    let banks_result = match state.use_mocking {
                        true => load_banks_of_user_mocking(cookie_user_id, Some(new_bank.clone())),
                        false => load_banks_of_user(cookie_user_id, &mut db).await,
                    };

                    if let Err(e) = banks_result {
                        return Ok(Json(json!( ResponseData::new_error(e, "There was an internal error trying to load the banks. Please login again and retry."))));
                    }

                    let banks = banks_result.unwrap();

                    let mut result = json!(ResponseData::new_success("New bank added".to_string(), &format!("The new bank '{}' has been added to your profile.", new_bank.name)));
                    result["banks"] = json!(banks);

                    Ok(Json(result))
                }
                Err(e) => {
                    Ok(Json(json!(ResponseData::new_error(e, "There was an internal error trying to add the csv converter of the new bank. The bank was added but the csv converter was not."))))
                }
            }
        }
        Err(e) => Ok(Json(json!(ResponseData::new_error(
            e,
            &format!(
                "The bank '{}' could not be added because it already exists in your profile.",
                new_bank.name
            )
        )))),
    }
}
