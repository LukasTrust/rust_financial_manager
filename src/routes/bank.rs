use chrono::{Datelike, NaiveDate};
use csv::ReaderBuilder;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{ExpressionMethods, QueryDsl};
use rocket::data::{Data, ToByteUnit};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;

use super::error_page::show_error_page;
use crate::database::db_connector::DbConn;
use crate::database::models::{
    Bank, CSVConverter, FormBank, NewBank, NewTransactions, Transaction,
};
use crate::schema::{banks as banks_without_dsl, transactions};
use crate::structs::AppState;
use crate::utils::{
    extract_user_id, generate_balance_graph_data, show_home_or_subview_with_data, update_app_state,
};

#[get("/add-bank")]
pub async fn add_bank(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(cookie_user_id) => Ok(show_home_or_subview_with_data(
            cookie_user_id,
            state,
            "add_bank".to_string(),
            false,
            false,
            None,
            None,
        )
        .await),
        Err(err) => Err(Box::new(err)),
    }
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(cookie_user_id) => {
            let new_bank = NewBank {
                user_id: cookie_user_id,
                name: bank_form.name.to_string(),
                link: bank_form.link.clone(),
                current_amount: bank_form.current_amount,
            };

            let result = diesel::insert_into(banks_without_dsl::table)
                .values(&new_bank)
                .execute(&mut db)
                .await;

            match result {
                Ok(_) => {
                    let inserted_bank = banks_without_dsl::table
                        .filter(banks_without_dsl::name.eq(&new_bank.name))
                        .first::<Bank>(&mut db)
                        .await;

                    match inserted_bank {
                        Ok(inserted_bank) => {
                            update_app_state(
                                cookie_user_id,
                                state,
                                Some(vec![inserted_bank.clone()]),
                                None,
                                None,
                                None,
                            )
                            .await;

                            Ok(show_home_or_subview_with_data(
                                cookie_user_id,
                                state,
                                "add_bank".to_string(),
                                false,
                                false,
                                Some(format!("Bank {} added", inserted_bank.name)),
                                None,
                            )
                            .await)
                        }
                        Err(err) => Ok(show_home_or_subview_with_data(
                            cookie_user_id,
                            state,
                            "add_bank".to_string(),
                            false,
                            false,
                            None,
                            Some(format!("Internal server error {}", err)),
                        )
                        .await),
                    }
                }
                Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                    Ok(show_home_or_subview_with_data(
                        cookie_user_id,
                        state,
                        "add_bank".to_string(),
                        false,
                        false,
                        None,
                        Some(format!(
                            "A bank with the name {} already exists. Please use a different name.",
                            new_bank.name
                        )),
                    )
                    .await)
                }
                Err(err) => Ok(show_home_or_subview_with_data(
                    cookie_user_id,
                    state,
                    "add_bank".to_string(),
                    false,
                    false,
                    None,
                    Some(format!("Internal server error {}", err)),
                )
                .await),
            }
        }
        Err(err) => Err(Box::new(err)),
    }
}

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(cookie_user_id) => {
            let banks = state
                .banks
                .read()
                .await
                .clone()
                .get(&cookie_user_id)
                .cloned()
                .unwrap_or_default();

            let bank = banks.iter().find(|&b| b.id == bank_id);

            match bank {
                Some(new_current_bank) => {
                    update_app_state(
                        cookie_user_id,
                        state,
                        None,
                        None,
                        None,
                        Some(new_current_bank.clone()),
                    )
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
    match extract_user_id(cookies) {
        Ok(cookie_user_id) => {
            let current_bank_id = {
                let current_bank = state
                    .current_bank
                    .read()
                    .await
                    .get(&cookie_user_id)
                    .cloned()
                    .unwrap();

                current_bank.id
            };

            let mut csv_converters_lock = state.csv_convert.write().await;
            let error = validate_csv_converters(&csv_converters_lock, current_bank_id);

            if let Some(err) = error {
                return Ok(render_template_with_error(state, Some(err)).await);
            }

            let csv_converter = csv_converters_lock.get_mut(&current_bank_id).unwrap();
            let headers_to_extract = get_headers_to_extract(csv_converter);

            // Read the CSV file
            let data_stream = match data.open(512.kibibytes()).into_bytes().await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(render_template_with_error(
                        state,
                        Some("Failed to read data stream"),
                    )
                    .await)
                }
            };

            let cursor = Cursor::new(data_stream.to_vec());
            let mut rdr = ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .from_reader(cursor);

            let all_transactions = transactions::table
                .load::<Transaction>(&mut db)
                .await
                .unwrap();

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
) -> Result<(i32, i32), String> {
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
            Err(_) => return Err("Failed to read CSV file".to_string()),
        };

        let mut date_from_csv = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
        let mut counterparty_from_csv = "";
        let mut amount_from_csv = 0.0;
        let mut bank_current_balance_after = 0.0;

        let date_index = *headers_map.get("Date").ok_or("Date header missing")?;
        let counterparty_index = *headers_map
            .get("Counterparty")
            .ok_or("Counterparty header missing")?;
        let amount_index = *headers_map.get("Amount").ok_or("Amount header missing")?;
        let bank_current_balance_after_index = *headers_map
            .get("Bank current balance after")
            .ok_or("Bank current balance after header missing")?;

        for (j, value) in record.as_slice().split(';').enumerate() {
            match j {
                idx if idx == date_index => {
                    date_from_csv = NaiveDate::parse_from_str(value, "%d.%m.%Y")
                        .map_err(|e| format!("Failed to parse date '{}': {}", value, e))?;
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

                    // Print out the value after processing
                    println!("Parsing amount from value: '{}'", processed_value);
                    amount_from_csv = processed_value
                        .replace(',', ".") // Convert comma to dot for parsing
                        .parse::<f64>()
                        .map_err(|e| {
                            format!("Failed to parse amount '{}': {}", processed_value, e)
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

                    // Print out the value after processing
                    println!(
                        "Parsing bank current balance after from value: '{}'",
                        processed_value
                    );
                    bank_current_balance_after = processed_value
                        .replace(',', ".") // Convert comma to dot for parsing
                        .parse::<f64>()
                        .map_err(|e| {
                            format!(
                                "Failed to parse bank current balance after '{}': {}",
                                processed_value, e
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
) -> Result<(HashMap<String, usize>, i32), String> {
    let mut header_indices = HashMap::new();
    let mut header_row = -1;

    for (i, result) in rdr.records().enumerate() {
        header_row += 1;
        let record = match result {
            Ok(rec) => rec,
            Err(_) => return Err("Failed to read CSV file".to_string()),
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
        return Err("Not all required headers were found".to_string());
    }

    Ok((header_indices, header_row - 2))
}

fn validate_csv_converters(
    csv_converters: &HashMap<i32, CSVConverter>,
    bank_id: i32,
) -> Option<&'static str> {
    let csv_converter = csv_converters.get(&bank_id)?;
    if csv_converter.date_conv.is_none()
        || csv_converter.counterparty_conv.is_none()
        || csv_converter.amount_conv.is_none()
    {
        return Some("Please set all CSV converters before uploading a CSV file");
    }
    None
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

async fn render_template_with_error(state: &State<AppState>, error: Option<&str>) -> Template {
    let banks = state.banks.read().await.clone();
    let transactions = state.transactions.read().await.clone();
    let plot_data = generate_balance_graph_data(&banks, &transactions);
    let bank = state.current_bank.read().await.clone();
    let context = json!({
        "banks": banks,
        "bank": bank,
        "plot_data": plot_data.to_string(),
        "error": error
    });

    Template::render("bank", &context)
}

async fn render_template_with_success(
    state: &State<AppState>,
    success: String,
    all_transactions: Vec<Transaction>,
) -> Template {
    let banks = state.banks.read().await.clone();
    let mut transactions = state.transactions.write().await;

    let current_bank_id = state.current_bank.read().await.id;
    if let Some(bank_transactions) = transactions.get_mut(&current_bank_id) {
        bank_transactions.clear();
        for transaction in &all_transactions {
            if transaction.bank_id == current_bank_id {
                bank_transactions.push(transaction.clone());
            }
        }
    } else {
        transactions.insert(current_bank_id, all_transactions.clone());
    }

    let plot_data = generate_balance_graph_data(&banks, &transactions);
    let bank = state.current_bank.read().await.clone();
    let context = json!({
        "banks": banks,
        "bank": bank,
        "plot_data": plot_data.to_string(),
        "success": success
    });

    Template::render("bank", &context)
}
