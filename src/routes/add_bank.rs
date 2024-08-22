use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, post};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::database::models::{NewBank, NewCSVConverter};
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::{insert_bank, insert_csv_converter};
use crate::utils::loading_utils::load_banks;
use crate::utils::structs::{FormBank, ResponseData};

#[get("/add-bank")]
pub async fn add_bank(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    get_user_id(cookies)?; // Ensure user is authenticated
    Ok(Template::render("add_bank", json!({})))
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
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
    match insert_bank(new_bank.clone(), &mut db).await {
        Ok(bank) => {
            let new_csv_converter = NewCSVConverter {
                bank_id: bank.id,
                counterparty_column: bank_form.counterparty_column,
                amount_column: bank_form.amount_column,
                bank_balance_after_column: bank_form.bank_balance_after_column,
                date_column: bank_form.date_column,
            };

            match insert_csv_converter(new_csv_converter, &mut db).await {
                Ok(_) => {
                    let banks_result = load_banks(cookie_user_id, &mut db).await;

                    if let Err(e) = banks_result {
                        return Ok(Json(json!( ResponseData {
                            success: None,
                            error: Some("There was an internal error trying to load the banks. Please login again and retry.".into()),
                            header: Some(e.into()),
                        })));
                    }

                    let banks = banks_result.unwrap();

                    let mut result = json!(ResponseData {
                        success: Some(format!(
                            "The new bank '{}' has been added to your profile.",
                            new_bank.name
                        )),
                        error: None,
                        header: Some("New bank added".into()),
                    });
                    result["banks"] = json!(banks);

                    return Ok(Json(result));
                }
                Err(e) => {
                    return Ok(Json(json!(ResponseData {
                        success: None,
                        error: Some("There was an internal error trying to add the csv converter of the new bank. The bank was added but the csv converter was not.".into()),
                        header: Some(e.to_string()),
                    })));
                }
            }
        }
        Err(e) => {
            return Ok(Json(json!(ResponseData {
                success: None,
                error: Some(format!(
                    "The bank '{}' could not be added because it already exists in your profile.",
                    new_bank.name
                )),
                header: Some(e.to_string()),
            })));
        }
    };
}
