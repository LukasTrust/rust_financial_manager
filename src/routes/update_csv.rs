use log::info;
use rocket::form::{Form, FromForm};
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{post, State};
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::NewCSVConverter;
use crate::utils::appstate::{AppState, LOCALIZATION};
use crate::utils::get_utils::get_user_id_and_language;
use crate::utils::insert_utiles::insert_csv_converter;
use crate::utils::loading_utils::load_csv_converter_of_bank;
use crate::utils::structs::{ErrorResponse, SuccessResponse};
use crate::utils::update_utils::update_csv_converter;

#[derive(FromForm)]
pub struct UpdateCSVForm {
    counterparty_column: Option<i32>,
    amount_column: Option<i32>,
    bank_balance_after_column: Option<i32>,
    date_column: Option<i32>,
}

#[post("/update_csv", data = "<form>")]
pub async fn update_csv(
    form: Form<UpdateCSVForm>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let csv_converter_of_bank =
        load_csv_converter_of_bank(current_bank.id, cookie_user_language, &mut db).await;

    match csv_converter_of_bank {
        Ok(mut csv_converter) => {
            if form.counterparty_column.is_some() {
                csv_converter.counterparty_column = form.counterparty_column;
            }

            if form.amount_column.is_some() {
                csv_converter.amount_column = form.amount_column;
            }

            if form.bank_balance_after_column.is_some() {
                csv_converter.bank_balance_after_column = form.bank_balance_after_column;
            }

            if form.date_column.is_some() {
                csv_converter.date_column = form.date_column;
            }

            update_csv_converter(csv_converter, cookie_user_language, &mut db).await?;

            info!("CSV converter updated");
            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(cookie_user_language, "csv_converter_updated"),
                LOCALIZATION
                    .get_localized_string(cookie_user_language, "csv_converter_updated_details"),
            )))
        }
        Err(_) => {
            let new_csv_converter = NewCSVConverter {
                bank_id: current_bank.id,
                counterparty_column: form.counterparty_column,
                amount_column: form.amount_column,
                bank_balance_after_column: form.bank_balance_after_column,
                date_column: form.date_column,
            };

            insert_csv_converter(new_csv_converter, cookie_user_language, &mut db).await?;

            info!("CSV converter updated");
            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(cookie_user_language, "csv_converter_updated"),
                LOCALIZATION
                    .get_localized_string(cookie_user_language, "csv_converter_updated_details"),
            )))
        }
    }
}
