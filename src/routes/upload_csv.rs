use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use log::{error, info};
use rocket::data::{Data, ToByteUnit};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::Cursor;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewTransactions};
use crate::schema::transactions;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_balance_graph_data, generate_performance_value};
use crate::utils::get_utils::{
    get_csv_converter, get_current_bank, get_first_date_and_last_date_from_bank, get_user_id,
};
use crate::utils::structs::Transaction;

use super::create_contract::create_contract_from_transactions;
use super::error_page::show_error_page;

#[post("/upload_csv", data = "<data>")]
pub async fn upload_csv(
    data: Data<'_>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;
    let current_bank = get_current_bank(cookie_user_id, state).await;

    if let Err(error) = current_bank {
        return Err(show_error_page("Error uploading csv".to_string(), error));
    }

    let current_bank_id = current_bank.clone().unwrap().id;

    // Read the CSV file
    let data_stream = match data.open(512.kibibytes()).into_bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            error!("Failed to read CSV file");
            return Ok(Json(json!({
                "error": "Failed to read CSV file",
            })));
        }
    };

    let cursor = Cursor::new(data_stream.to_vec());
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(cursor);

    let result = extract_and_process_records(&mut rdr, current_bank_id, state, &mut db).await;

    match result {
        Ok((succesful_inserts, failed_inserts)) => {
            info!(
                "Succesfully insertet {} and {} were duplicates",
                succesful_inserts, failed_inserts
            );

            let transactions_map = state.transactions.read().await;
            let transactions = transactions_map.get(&current_bank_id);
            let banks = vec![current_bank.unwrap()];

            let (first_date, last_date) = get_first_date_and_last_date_from_bank(transactions);

            let performance_value =
                generate_performance_value(&banks, &transactions_map, first_date, last_date);

            let graph_data = generate_balance_graph_data(
                &banks,
                &transactions_map,
                performance_value.1,
                None,
                None,
            )
            .await;

            Ok(Json(json!({
                "success": format!("Succesfully insertet {} and {} were duplicates", succesful_inserts, failed_inserts),
                "graph_data": graph_data,
                "performance_value": performance_value.0,
            })))
        }
        Err(e) => {
            error!("Failed to insert records: {}", e);
            return Ok(Json(json!({
                "error": e.to_string(),
                "success": false
            })));
        }
    }
}

async fn extract_and_process_records<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    current_bank_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Result<(i32, i32), String> {
    let mut succesful_inserts = 0;
    let mut failed_inserts = 0;

    let mut new_transactions: HashMap<i32, Vec<Transaction>> = HashMap::new();

    let csv_converter = get_csv_converter(current_bank_id, state).await?;

    validate_csv_converters(csv_converter)?;

    let date_index = csv_converter.date_column.unwrap() as usize;
    let counterparty_index = csv_converter.counterparty_column.unwrap() as usize;
    let amount_index = csv_converter.amount_column.unwrap() as usize;
    let bank_balance_after_index = csv_converter.bank_balance_after_column.unwrap() as usize;

    for (i, result) in rdr.records().enumerate() {
        if i < 3 {
            continue;
        }

        let record = match result {
            Ok(rec) => rec,
            Err(_) => {
                error!("Failed to read CSV file");
                return Err("Failed to read CSV file".to_string());
            }
        };

        let mut date_from_csv = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let mut counterparty_from_csv = "";
        let mut amount_from_csv = 0.0;
        let mut bank_balance_after = 0.0;

        for (j, value) in record.as_slice().split(';').enumerate() {
            match j {
                idx if idx == date_index => {
                    date_from_csv = NaiveDate::parse_from_str(value, "%d.%m.%Y")
                        .map_err(|e| format!("Failed to parse date: {}", e))?;
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
                        .map_err(|e| format!("Failed to parse amount: {}", e))?;
                }
                idx if idx == bank_balance_after_index => {
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

                    bank_balance_after = processed_value
                        .replace(',', ".") // Convert comma to dot for parsing
                        .parse::<f64>()
                        .map_err(|e| format!("Failed to parse bank balance after: {}", e))?;
                }
                _ => (),
            }
        }

        if date_from_csv.year() == 1 || amount_from_csv == 0.0 {
            continue;
        }

        let new_transaction = NewTransactions {
            bank_id: current_bank_id,
            date: date_from_csv,
            counterparty: counterparty_from_csv.to_string(),
            amount: amount_from_csv,
            bank_balance_after,
        };

        let result = diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .get_result::<Transaction>(db)
            .await;

        match result {
            Ok(transaction) => {
                new_transactions
                    .entry(current_bank_id)
                    .or_insert_with(Vec::new)
                    .push(transaction);

                succesful_inserts += 1
            }
            Err(_) => failed_inserts += 1,
        }
    }

    state.update_transactions(new_transactions).await;

    create_contract_from_transactions(state, db).await?;

    Ok((succesful_inserts, failed_inserts))
}

fn validate_csv_converters(csv_converter: CSVConverter) -> Result<(), String> {
    if csv_converter.date_column.is_none()
        || csv_converter.counterparty_column.is_none()
        || csv_converter.amount_column.is_none()
        || csv_converter.bank_balance_after_column.is_none()
    {
        error!("CSV converter not set up");
        return Err("CSV converter not set up".to_string());
    }
    info!("CSV converter found");

    Ok(())
}
