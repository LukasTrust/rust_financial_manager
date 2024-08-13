use diesel::prelude::*;
use rocket::form::{Form, FromForm};
use rocket::serde::json::Json;
use rocket::{http::CookieJar, response::Redirect};
use rocket::{post, State};
use rocket_db_pools::diesel::AsyncPgConnection;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use std::collections::HashMap;

use super::error_page::show_error_page;
use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewCSVConverter};
use crate::schema::csv_converters;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_current_bank, get_user_id};
use crate::utils::structs::ResponseData;

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
) -> Result<Json<ResponseData>, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    find_bank_and_update(cookie_user_id, state, &mut *db, |converter| {
        if let Some(amount_column) = form.amount_column {
            converter.amount_column = Some(amount_column);
        }
        if let Some(counterparty_column) = form.counterparty_column {
            converter.counterparty_column = Some(counterparty_column);
        }
        if let Some(bank_balance_after_column) = form.bank_balance_after_column {
            converter.bank_balance_after_column = Some(bank_balance_after_column);
        }
        if let Some(date_column) = form.date_column {
            converter.date_column = Some(date_column);
        }
    })
    .await
}

async fn find_bank_and_update<F>(
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut AsyncPgConnection,
    update_field: F,
) -> Result<Json<ResponseData>, Redirect>
where
    F: Fn(&mut CSVConverter),
{
    match get_current_bank(cookie_user_id, state).await {
        Ok(current_bank) => {
            let current_bank_id = current_bank.id;
            match save_update_csv(state, db, update_field, current_bank_id).await {
                Ok(success) => Ok(Json(ResponseData {
                    error: None,
                    success: Some(success),
                })),
                Err(error) => Ok(Json(ResponseData {
                    error: Some(error),
                    success: None,
                })),
            }
        }
        Err(error) => Err(show_error_page(
            "Error updating CSV converter".to_string(),
            error,
        )),
    }
}

pub async fn save_update_csv<F>(
    state: &State<AppState>,
    db: &mut AsyncPgConnection,
    update_field: F,
    current_bank_id: i32,
) -> Result<String, String>
where
    F: Fn(&mut CSVConverter),
{
    // Obtain a lock on the state
    let mut csv_converters_lock = state.csv_convert.write().await;
    let current_csv_converter = csv_converters_lock.get_mut(&current_bank_id).cloned();
    drop(csv_converters_lock); // Release the lock early

    let result = if let Some(mut converter) = current_csv_converter {
        // Update the fields and save the changes
        update_field(&mut converter);
        let update_result = diesel::update(csv_converters::table.find(converter.id))
            .set(&converter)
            .execute(db)
            .await;

        match update_result {
            Ok(_) => {
                // Update the state with the new data
                state
                    .update_csv_converters(HashMap::from([(current_bank_id, converter)]))
                    .await;
                Ok("CSV converter updated successfully".to_string())
            }
            Err(err) => Err(format!("Internal server error: {}", err)),
        }
    } else {
        // Create a new converter if it does not exist
        let mut new_converter = CSVConverter {
            id: 0,
            bank_id: current_bank_id,
            date_column: None,
            counterparty_column: None,
            amount_column: None,
            bank_balance_after_column: None,
        };

        update_field(&mut new_converter);

        let insert_result = diesel::insert_into(csv_converters::table)
            .values(&NewCSVConverter {
                bank_id: new_converter.bank_id,
                date_column: new_converter.date_column,
                counterparty_column: new_converter.counterparty_column,
                amount_column: new_converter.amount_column,
                bank_balance_after_column: new_converter.bank_balance_after_column,
            })
            .get_result::<CSVConverter>(db)
            .await;

        match insert_result {
            Ok(converter) => {
                // Update the state with the new data
                state
                    .update_csv_converters(HashMap::from([(current_bank_id, converter)]))
                    .await;
                Ok("CSV converter created successfully".to_string())
            }
            Err(err) => Err(format!("Internal server error: {}", err)),
        }
    };

    result
}
