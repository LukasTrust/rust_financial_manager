use rocket::form::Form;
use rocket::http::CookieJar;
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
use crate::utils::appstate::{AppState, LOCALIZATION};
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::{insert_bank, insert_csv_converter};
use crate::utils::loading_utils::load_banks_of_user;
use crate::utils::structs::{FormBank, ResponseData};

#[get("/add-bank")]
pub async fn add_bank() -> Template {
    Template::render("add_bank", json!({}))
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;
    let user_language = state.get_user_language(cookie_user_id).await;

    // Create a new bank instance
    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    let bank = match state.use_mocking {
        true => insert_bank_mocking(new_bank.clone())?,
        false => {
            insert_bank(
                new_bank.clone(),
                state.get_user_language(cookie_user_id).await,
                &mut db,
            )
            .await?
        }
    };

    let new_csv_converter = NewCSVConverter {
        bank_id: bank.id,
        counterparty_column: bank_form.counterparty_column,
        amount_column: bank_form.amount_column,
        bank_balance_after_column: bank_form.bank_balance_after_column,
        date_column: bank_form.date_column,
    };

    match state.use_mocking {
        true => insert_csv_converter_mocking(new_csv_converter)?,
        false => insert_csv_converter(new_csv_converter, user_language, &mut db).await?,
    };

    let banks = match state.use_mocking {
        true => load_banks_of_user_mocking(cookie_user_id, Some(new_bank.clone()))?,
        false => load_banks_of_user(cookie_user_id, user_language, &mut db).await?,
    };

    let message = LOCALIZATION.get_localized_string(user_language, "bank_added_details");
    let formatted_message = message.replace("{}", &new_bank.name);

    let mut result = json!(ResponseData::new_success(
        LOCALIZATION.get_localized_string(user_language, "bank_added"),
        formatted_message
    ));
    result["banks"] = json!(banks);

    Ok(Json(result))
}
