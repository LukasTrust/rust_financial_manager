use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use log::{error, info};
use rocket::fs::TempFile;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::tokio::io::AsyncReadExt;
use rocket::{post, State};
use rocket_db_pools::Connection;
use std::io::Cursor;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewTransaction};
use crate::utils::appstate::{AppState, Language, LOCALIZATION};
use crate::utils::create_contract::create_contract_from_transactions;
use crate::utils::get_utils::get_user_id_and_language;
use crate::utils::insert_utiles::insert_transactions;
use crate::utils::loading_utils::{load_csv_converter_of_bank, load_transactions_of_bank};
use crate::utils::structs::{Bank, ErrorResponse, SuccessResponse, Transaction};

#[post("/upload_csv", data = "<file>")]
pub async fn upload_csv(
    file: TempFile<'_>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let transactions_task =
        load_transactions_of_bank(current_bank.id, cookie_user_language, &mut db);

    // Open the file and read its contents into a Vec<u8>
    let mut buffer = Vec::new();
    let mut temp_file = file.open().await.map_err(|_| {
        error!("Failed to open temporary file");
        Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(cookie_user_language, "error_reading_csv_file"),
            LOCALIZATION
                .get_localized_string(cookie_user_language, "error_reading_csv_file_details"),
        ))
    })?;
    temp_file.read_to_end(&mut buffer).await.map_err(|_| {
        error!("Failed to read CSV file content");
        Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(cookie_user_language, "error_reading_csv_file"),
            LOCALIZATION
                .get_localized_string(cookie_user_language, "error_reading_csv_file_details"),
        ))
    })?;

    // Read the CSV data from the buffer
    let cursor = Cursor::new(buffer);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(cursor);

    let existing_transactions = transactions_task.await.unwrap_or_default();

    let result = extract_and_process_records(
        &mut rdr,
        current_bank.clone(),
        existing_transactions,
        cookie_user_language,
        &mut db,
    )
    .await?;

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "csv_file_read"),
        result,
    )))
}

async fn extract_and_process_records<R: std::io::Read>(
    rdr: &mut csv::Reader<R>,
    current_bank: Bank,
    existing_transactions: Vec<Transaction>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<String, Json<ErrorResponse>> {
    let mut transactions_to_insert = vec![];

    let csv_converter = load_csv_converter_of_bank(current_bank.id, language, db).await?;

    validate_csv_converters(csv_converter, language)?;

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
                return Err(Json(ErrorResponse::new(
                    LOCALIZATION.get_localized_string(language, "error_reading_csv_file"),
                    LOCALIZATION.get_localized_string(language, "error_reading_csv_file_details"),
                )));
            }
        };

        let mut date_from_csv = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let mut counterparty_from_csv = "";
        let mut amount_from_csv = 0.0;
        let mut bank_balance_after = 0.0;

        for (j, value) in record.as_slice().split(';').enumerate() {
            match j {
                idx if idx == date_index => {
                    date_from_csv = NaiveDate::parse_from_str(value, "%d.%m.%Y").map_err(|e| {
                        error!("Failed to parse date: {}", e);
                        Json(ErrorResponse::new(
                            LOCALIZATION.get_localized_string(language, "error_parsing_date"),
                            LOCALIZATION
                                .get_localized_string(language, "error_parsing_date_details"),
                        ))
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
                            error!("Failed to parse amount: {}", e);
                            Json(ErrorResponse::new(
                                LOCALIZATION.get_localized_string(language, "error_parsing_amount"),
                                LOCALIZATION
                                    .get_localized_string(language, "error_parsing_amount_details"),
                            ))
                        })?;
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
                        .map_err(|e| {
                            error!("Failed to parse bank balance after: {}", e);
                            Json(ErrorResponse::new(
                                LOCALIZATION.get_localized_string(
                                    language,
                                    "error_parsing_bank_balance_after",
                                ),
                                LOCALIZATION.get_localized_string(
                                    language,
                                    "error_parsing_bank_balance_after_details",
                                ),
                            ))
                        })?;
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
        insert_transactions(transactions_to_insert, existing_transactions, language, db).await?;

    info!(
        "Succesfully insertet {} and {} were duplicates",
        succesful_inserts, failed_inserts
    );

    let contract_result = create_contract_from_transactions(current_bank.id, language, db).await?;

    let mut local_string =
        LOCALIZATION.get_localized_string(language, "transactions_inserted_details");

    local_string = local_string.replace("{success}", &succesful_inserts.to_string());
    local_string = local_string.replace("{error}", &failed_inserts.to_string());
    local_string = local_string.replace("{contracts}", &contract_result);

    Ok(local_string)
}

fn validate_csv_converters(
    csv_converter: CSVConverter,
    language: Language,
) -> Result<(), Json<ErrorResponse>> {
    if csv_converter.date_column.is_none()
        || csv_converter.counterparty_column.is_none()
        || csv_converter.amount_column.is_none()
        || csv_converter.bank_balance_after_column.is_none()
    {
        error!("CSV converter not set up");
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(language, "csv_converter_not_set_up"),
            LOCALIZATION.get_localized_string(language, "csv_converter_not_set_up_details"),
        )));
    }
    info!("CSV converter found");

    Ok(())
}
