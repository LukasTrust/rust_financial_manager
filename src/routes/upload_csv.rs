use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use log::{error, info};
use rocket::data::{Data, ToByteUnit};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{post, State};
use rocket_db_pools::Connection;
use std::io::Cursor;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewTransaction};
use crate::utils::appstate::AppState;
use crate::utils::create_contract::create_contract_from_transactions;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::insert_transactions;
use crate::utils::loading_utils::{load_csv_converter_of_bank, load_transactions_of_bank};
use crate::utils::structs::{Bank, ResponseData, Transaction};

#[post("/upload_csv", data = "<data>")]
pub async fn upload_csv(
    data: Data<'_>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;
    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while loading the bank. Please try again.".into(),
            ),
            header: Some("No bank selected".into()),
        }));
    }

    let current_bank = current_bank.unwrap();

    let transactions_task = load_transactions_of_bank(current_bank.id, &mut db);

    // Read the CSV file
    let data_stream = match data.open(512.kibibytes()).into_bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            error!("Failed to read CSV file");
            return Ok(Json(ResponseData {
                success: None,
                error: Some("There was an internal error while trying to read the CSV file".into()),
                header: Some("Failed to read CSV file".into()),
            }));
        }
    };

    let cursor = Cursor::new(data_stream.to_vec());
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(cursor);

    let existing_transactions = transactions_task.await.unwrap_or_default();

    let result = extract_and_process_records(
        &mut rdr,
        current_bank.clone(),
        existing_transactions,
        &mut db,
    )
    .await;

    match result {
        Ok(result_string) => Ok(Json(ResponseData {
            success: Some(result_string),
            error: None,
            header: Some("Succesfully parsed the CSV file".to_string()),
        })),
        Err(e) => {
            error!("Failed to insert records: {}", e);
            Ok(Json(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while trying to insert the records".into(),
                ),
                header: Some(e),
            }))
        }
    }
}

async fn extract_and_process_records<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    current_bank: Bank,
    existing_transactions: Vec<Transaction>,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    let mut transactions_to_insert = vec![];

    let csv_converter = load_csv_converter_of_bank(current_bank.id, db).await?;

    if csv_converter.is_none() {
        error!("No CSV converter found");
        return Err("No CSV converter found".to_string());
    }

    let csv_converter = csv_converter.unwrap();

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

        let new_transaction = NewTransaction {
            bank_id: current_bank.id,
            date: date_from_csv,
            counterparty: counterparty_from_csv.to_string(),
            amount: amount_from_csv,
            bank_balance_after,
        };

        transactions_to_insert.push(new_transaction);
    }

    let (succesful_inserts, failed_inserts) =
        insert_transactions(transactions_to_insert, existing_transactions, db).await?;

    info!(
        "Succesfully insertet {} and {} were duplicates",
        succesful_inserts, failed_inserts
    );

    let contract_result = create_contract_from_transactions(current_bank.id, db).await?;

    let result = format!(
        "Succesfully insertet {} and {} were duplicates. {}",
        succesful_inserts, failed_inserts, contract_result
    );

    Ok(result)
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
