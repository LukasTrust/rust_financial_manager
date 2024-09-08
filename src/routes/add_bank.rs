use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::serde::json::{json, Json};
use rocket::{get, post};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::database::models::{NewBank, NewCSVConverter};
use crate::utils::appstate::LOCALIZATION;
use crate::utils::get_utils::get_user_id_and_language;
use crate::utils::insert_utiles::{insert_bank, insert_csv_converter};
use crate::utils::loading_utils::load_banks_of_user;
use crate::utils::structs::{ErrorResponse, FormBank, SuccessResponse};

#[get("/add-bank")]
pub async fn add_bank() -> Template {
    Template::render("add_bank", json!({}))
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    // Create a new bank instance
    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    let bank = insert_bank(new_bank.clone(), cookie_user_language, &mut db).await?;

    let new_csv_converter = NewCSVConverter {
        bank_id: bank.id,
        counterparty_column: bank_form.counterparty_column,
        amount_column: bank_form.amount_column,
        bank_balance_after_column: bank_form.bank_balance_after_column,
        date_column: bank_form.date_column,
    };

    insert_csv_converter(new_csv_converter, cookie_user_language, &mut db).await?;

    let banks = load_banks_of_user(cookie_user_id, cookie_user_language, &mut db).await?;

    let message = LOCALIZATION.get_localized_string(cookie_user_language, "bank_added_details");
    let formatted_message = message.replace("{}", &new_bank.name);

    let mut result = json!(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "bank_added"),
        formatted_message
    ));
    result["banks"] = json!(banks);

    Ok(Json(result))
}
