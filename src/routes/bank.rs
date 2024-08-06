use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use log::{error, info};
use rocket::data::{Data, ToByteUnit};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::io::Cursor;

use super::error_page::show_error_page;
use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewTransactions};
use crate::schema::transactions;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::show_home_or_subview_with_data;
use crate::utils::get_utils::{get_banks_of_user, get_current_bank, get_user_id};
use crate::utils::loading_utils::load_transactions;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match get_user_id(cookies) {
        Ok(cookie_user_id) => {
            let banks = get_banks_of_user(cookie_user_id, state).await;
            let bank = banks.iter().find(|&b| b.id == bank_id);

            match bank {
                Some(new_current_bank) => {
                    state
                        .update_current_bank(cookie_user_id, new_current_bank.clone())
                        .await;

                    return Ok(show_home_or_subview_with_data(
                        cookie_user_id,
                        state,
                        "bank".to_string(),
                        true,
                        true,
                        None,
                        None,
                    )
                    .await);
                }
                None => {
                    return Err(Box::new(show_error_page(
                        "Bank not found".to_string(),
                        "The bank you are looking for does not exist.".to_string(),
                    )))
                }
            }
        }
        Err(err) => return Err(Box::new(err)),
    }
}

#[post("/upload_csv", data = "<data>")]
pub async fn upload_csv(
    data: Data<'_>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Box<Redirect>> {
    match get_user_id(cookies) {
        Ok(cookie_user_id) => {
            let current_bank_id = get_current_bank(cookie_user_id, state).await?.id;

            let mut csv_converters_lock = state.csv_convert.write().await;
            let csv_converter = validate_csv_converters(&mut csv_converters_lock, current_bank_id)?;

            let headers_to_extract = get_headers_to_extract(csv_converter);

            // Read the CSV file
            let data_stream = match data.open(512.kibibytes()).into_bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    error!("Failed to read CSV file");
                    return Err(Box::new(show_error_page(
                        "Failed to read CSV file".to_string(),
                        "Please try again.".to_string(),
                    )));
                }
            };

            let cursor = Cursor::new(data_stream.to_vec());
            let mut rdr = ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(cursor);

            let all_transactions = load_transactions(current_bank_id, &mut db).await;

            match extract_and_process_records(&mut rdr, &headers_to_extract, current_bank_id, db)
                .await
            {
                Ok(inserts) => Ok(render_template_with_success(
                    state,
                    format!(
                        "Succesfully insertet {} and {} were duplicates",
                        inserts.0, inserts.1
                    ),
                    all_transactions,
                )
                .await),
                Err(err) => Ok(render_template_with_error(state, Some(&err)).await),
            }
        }
        Err(err) => Err(Box::new(err)),
    }
}

async fn extract_and_process_records<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    headers_to_extract: &[String],
    current_bank_id: i32,
    mut db: Connection<DbConn>,
) -> Result<(i32, i32), Box<Redirect>> {
    let headers_map = match find_header_indices(rdr, headers_to_extract) {
        Ok(map) => map,
        Err(e) => return Err(e),
    };

    let header_row = headers_map.1 as usize;
    let headers_map = headers_map.0;

    let mut succesful_inserts = 0;
    let mut failed_inserts = 0;

    for (i, result) in rdr.records().enumerate() {
        if i < header_row {
            continue;
        }

        let record = match result {
            Ok(rec) => rec,
            Err(_) => {
                error!("Failed to read CSV file");
                return Err(Box::new(show_error_page(
                    "Failed to read CSV file".to_string(),
                    "Please try again.".to_string(),
                )));
            }
        };

        let mut date_from_csv = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let mut counterparty_from_csv = "";
        let mut amount_from_csv = 0.0;
        let mut bank_current_balance_after = 0.0;

        let date_index = headers_map.get("Date");
        let counterparty_index = headers_map.get("Counterparty");
        let amount_index = headers_map.get("Amount");
        let bank_current_balance_after_index = headers_map.get("Bank current balance after");

        if date_index.is_none()
            || counterparty_index.is_none()
            || amount_index.is_none()
            || bank_current_balance_after_index.is_none()
        {
            error!("Failed to find headers");
            return Err(Box::new(show_error_page(
                "Failed to find headers".to_string(),
                "Please try again.".to_string(),
            )));
        }

        let date_index = *date_index.unwrap();
        let counterparty_index = *counterparty_index.unwrap();
        let amount_index = *amount_index.unwrap();
        let bank_current_balance_after_index = *bank_current_balance_after_index.unwrap();

        for (j, value) in record.as_slice().split(';').enumerate() {
            match j {
                idx if idx == date_index => {
                    date_from_csv = NaiveDate::parse_from_str(value, "%d.%m.%Y").map_err(|e| {
                        show_error_page("Failed to pase date".to_string(), format!("Error: {}", e))
                    })?;
                }
                idx if idx == counterparty_index => {
                    counterparty_from_csv = value;
                }
                idx if idx == amount_index => {
                    // Determine and handle the decimal separator
                    let processed_value = if value.contains(',') {
                        // If value contains a comma, use it as is
                        value.to_string()
                    } else if value.contains('.') {
                        // If value contains a dot, replace it with a comma for consistency
                        value.replace('.', ",")
                    } else {
                        // Insert a comma before the last two digits (assuming no decimal point is present)
                        let len = value.len();
                        if len > 2 {
                            format!("{}.{:02}", &value[..len - 2], &value[len - 2..])
                        } else {
                            format!("0.{}", value)
                        }
                    };

                    amount_from_csv = processed_value
                        .replace(',', ".") // Convert comma to dot for parsing
                        .parse::<f64>()
                        .map_err(|e| {
                            show_error_page(
                                "Failed to parse amount".to_string(),
                                format!("Error: {}", e),
                            )
                        })?;
                }
                idx if idx == bank_current_balance_after_index => {
                    // Determine and handle the decimal separator
                    let processed_value = if value.contains(',') {
                        // If value contains a comma, use it as is
                        value.to_string()
                    } else if value.contains('.') {
                        // If value contains a dot, replace it with a comma for consistency
                        value.replace('.', ",")
                    } else {
                        // Insert a comma before the last two digits (assuming no decimal point is present)
                        let len = value.len();
                        if len > 2 {
                            format!("{}.{:02}", &value[..len - 2], &value[len - 2..])
                        } else {
                            format!("0.{}", value)
                        }
                    };

                    bank_current_balance_after = processed_value
                        .replace(',', ".") // Convert comma to dot for parsing
                        .parse::<f64>()
                        .map_err(|e| {
                            show_error_page(
                                "Failed to parse bank current balance after".to_string(),
                                format!("Error: {}", e),
                            )
                        })?;
                }
                _ => (),
            }
        }

        if date_from_csv.year() == 1 || counterparty_from_csv == "" || amount_from_csv == 0.0 {
            continue;
        }

        let new_transaction = NewTransactions {
            bank_id: current_bank_id,
            date: date_from_csv,
            counterparty: counterparty_from_csv.to_string(),
            amount: amount_from_csv,
            bank_current_balance_after: bank_current_balance_after,
        };

        let result = diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .execute(&mut db)
            .await;

        match result {
            Ok(_) => succesful_inserts += 1,
            Err(_) => failed_inserts += 1,
        }
    }

    Ok((succesful_inserts, failed_inserts))
}

fn find_header_indices<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    headers_to_extract: &[String],
) -> Result<(HashMap<String, usize>, i32), Box<Redirect>> {
    let mut header_indices = HashMap::new();
    let mut header_row = -1;

    for (i, result) in rdr.records().enumerate() {
        header_row += 1;
        let record = match result {
            Ok(rec) => rec,
            Err(_) => {
                return Err(Box::new(show_error_page(
                    "Failed to read CSV file".to_string(),
                    "Please try again.".to_string(),
                )))
            }
        };

        let array = record.as_slice().split(';');
        for (j, value) in array.enumerate() {
            if headers_to_extract.get(0) == Some(&value.to_string()) {
                header_indices.insert("Date".to_string(), j);
            } else if headers_to_extract.get(1) == Some(&value.to_string()) {
                header_indices.insert("Counterparty".to_string(), j);
            } else if headers_to_extract.get(2) == Some(&value.to_string()) {
                header_indices.insert("Amount".to_string(), j);
            } else if headers_to_extract.get(3) == Some(&value.to_string()) {
                header_indices.insert("Bank current balance after".to_string(), j);
            }
        }

        if header_indices.len() == headers_to_extract.len() {
            break;
        }
    }

    if header_indices.len() != headers_to_extract.len() {
        return Err(Box::new(show_error_page(
            "Failed to find headers".to_string(),
            "Please try again.".to_string(),
        )));
    }

    Ok((header_indices, header_row - 2))
}

fn validate_csv_converters(
    csv_converters_lock: &mut HashMap<i32, CSVConverter>,
    current_bank_id: i32,
) -> Result<&mut CSVConverter, Redirect> {
    let csv_converter = &csv_converters_lock.get(&current_bank_id);

    match csv_converter {
        Some(csv_converter) => {
            if csv_converter.date_conv.is_none()
                || csv_converter.counterparty_conv.is_none()
                || csv_converter.amount_conv.is_none()
            {
                error!("CSV converter not set up");
                return Err(show_error_page(
                    "CSV converter not set up".to_string(),
                    "Please set up the CSV converter before uploading a CSV file".to_string(),
                ));
            }
            info!("CSV converter found");

            Ok(csv_converters_lock.get_mut(&current_bank_id).unwrap())
        }
        None => {
            error!("CSV converter not found");
            Err(show_error_page(
                "CSV converter not found".to_string(),
                "Please set up the CSV converter before uploading a CSV file".to_string(),
            ))
        }
    }
}

fn get_headers_to_extract(csv_converter: &CSVConverter) -> Vec<String> {
    vec![
        csv_converter.date_conv.clone().unwrap(),
        csv_converter.counterparty_conv.clone().unwrap(),
        csv_converter.amount_conv.clone().unwrap(),
        csv_converter
            .bank_current_balance_after_conv
            .clone()
            .unwrap(),
    ]
}
