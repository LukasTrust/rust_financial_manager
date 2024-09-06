use log::{error, info};
use rocket::form::{Form, FromForm};
use rocket::serde::json::Json;
use rocket::{http::CookieJar, response::Redirect};
use rocket::{post, State};
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::NewCSVConverter;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::insert_csv_converter;
use crate::utils::loading_utils::load_csv_converter_of_bank;
use crate::utils::structs::ResponseData;
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
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Json(ResponseData::new_error(
            state
                .localize_message(cookie_user_id, "no_bank_selected")
                .await,
            state
                .localize_message(cookie_user_id, "no_bank_selected_details")
                .await,
        )));
    }

    let current_bank = current_bank.unwrap();

    let csv_converter_of_bank = load_csv_converter_of_bank(current_bank.id, &mut db).await;

    if let Err(error) = csv_converter_of_bank {
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_loading_csv_converter")
                .await,
        )));
    }

    let csv_converter_of_bank = csv_converter_of_bank.unwrap();

    match csv_converter_of_bank {
        Some(mut csv_converter) => {
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

            let result = update_csv_converter(csv_converter, &mut db).await;

            match result {
                Ok(_) => {
                    info!("CSV converter updated");
                    Ok(Json(ResponseData::new_success(
                        state
                            .localize_message(cookie_user_id, "csv_converter_updated")
                            .await,
                        state
                            .localize_message(cookie_user_id, "csv_converter_updated_details")
                            .await,
                    )))
                }
                Err(error) => {
                    error!("Error updating CSV converter: {}", error);
                    Ok(Json(ResponseData::new_error(
                        error,
                        state
                            .localize_message(cookie_user_id, "error_updating_csv_converter")
                            .await,
                    )))
                }
            }
        }
        None => {
            let new_csv_converter = NewCSVConverter {
                bank_id: current_bank.id,
                counterparty_column: form.counterparty_column,
                amount_column: form.amount_column,
                bank_balance_after_column: form.bank_balance_after_column,
                date_column: form.date_column,
            };

            let result = insert_csv_converter(new_csv_converter, &mut db).await;

            match result {
                Ok(_) => {
                    info!("CSV converter updated");
                    Ok(Json(ResponseData::new_success(
                        state
                            .localize_message(cookie_user_id, "csv_converter_updated")
                            .await,
                        state
                            .localize_message(cookie_user_id, "csv_converter_updated_details")
                            .await,
                    )))
                }
                Err(error) => {
                    error!("Error adding CSV converter: {}", error);
                    Ok(Json(ResponseData::new_error(
                        error,
                        state
                            .localize_message(cookie_user_id, "error_updating_csv_converter")
                            .await,
                    )))
                }
            }
        }
    }
}
