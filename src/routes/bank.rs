use chrono::{Datelike, NaiveDate};
use csv::{ReaderBuilder, StringRecord};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::Record;
use diesel::{ExpressionMethods, QueryDsl};
use rocket::data::{Data, ToByteUnit};
use rocket::form::{Form, FromForm};
use rocket::futures::stream::ForEach;
use rocket::http::hyper::header;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};
use serde_json::json;
use std::collections::HashMap;
use std::io::Cursor;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, FormBank, NewBank, NewTransactions, Transaction};
use crate::routes::home::AppState;
use crate::schema::{banks as banks_without_dsl, csv_converters, transactions};

use super::help_functions::generate_balance_graph_data;

#[get("/add-bank")]
pub async fn add_bank(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("add_bank", context! { banks })
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    mut db: Connection<DbConn>,
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Template {
    let user_id = cookies
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<i32>().ok());
    let banks = state.banks.read().await.clone();

    match user_id {
        Some(_) => {}
        None => {
            return Template::render(
                "home",
                context! {banks: banks, error: "Could not find user id" },
            )
        }
    }

    let new_bank = NewBank {
        user_id: user_id.unwrap(),
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
        current_amount: bank_form.current_amount,
    };

    let result = diesel::insert_into(banks_without_dsl::table)
        .values(&new_bank)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => Template::render("add_bank", context! { success: "New bank added" }),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Template::render(
            "add_bank",
            context! {banks: banks, error: "A bank with this name already exists. Please use a different name." },
        ),
        Err(err) => Template::render(
            "add_bank",
            context! {banks: banks, error: format!("Internal server error {}", err) },
        ),
    }
}

#[get("/bank/<bank_id>")]
pub async fn bank_view(bank_id: i32, state: &State<AppState>) -> Result<Template, Redirect> {
    // Retrieve banks and transactions from state
    let banks = state.banks.read().await.clone();
    let transactions = state.transactions.read().await.clone();

    // Find the requested bank
    let bank = banks.iter().find(|&b| b.id == bank_id);

    if let Some(bank) = bank {
        // Fetch the transactions for the found bank
        let bank_transactions = transactions.get(&bank_id).unwrap_or(&Vec::new()).clone();

        // Generate plot data based on the bank's transactions
        let plot_data = generate_balance_graph_data(
            &vec![bank.clone()],
            &HashMap::from([(bank.id, bank_transactions)]),
        );

        let mut current_bank = state.current_bank.write().await;
        *current_bank = bank.clone();

        let context = json!({
            "banks": banks,
            "bank": bank,
            "plot_data": plot_data.to_string()
        });

        Ok(Template::render("bank", &context))
    } else {
        // Redirect to home if bank is not found
        Err(Redirect::to("/"))
    }
}

#[post("/upload_csv", data = "<data>")]
pub async fn upload_csv(
    data: Data<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Template {
    let current_bank_id = {
        let current_bank = state.current_bank.read().await;
        current_bank.id
    };

    let mut csv_converters_lock = state.csv_convert.write().await;
    let error = validate_csv_converters(&csv_converters_lock, current_bank_id);

    if let Some(err) = error {
        return render_template_with_error(state, Some(err)).await;
    }

    let csv_converter = csv_converters_lock.get_mut(&current_bank_id).unwrap();
    let headers_to_extract = get_headers_to_extract(csv_converter);

    // Read the CSV file
    let data_stream = match data.open(512.kibibytes()).into_bytes().await {
        Ok(bytes) => bytes,
        Err(_) => {
            return render_template_with_error(state, Some("Failed to read data stream")).await
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

    match extract_and_process_records(&mut rdr, &headers_to_extract, current_bank_id, db).await {
        Ok(inserts) => {
            render_template_with_success(
                state,
                format!(
                    "Succesfully insertet {} and {} were duplicates",
                    inserts.0, inserts.1
                ),
                all_transactions,
            )
            .await
        }
        Err(err) => render_template_with_error(state, Some(&err)).await,
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

        let date_index = *headers_map.get("Date").ok_or("Date header missing")?;
        let counterparty_index = *headers_map
            .get("Counterparty")
            .ok_or("Counterparty header missing")?;
        let amount_index = *headers_map.get("Amount").ok_or("Amount header missing")?;

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
        };

        let result = diesel::insert_into(transactions::table)
            .values(&new_transaction)
            .execute(&mut db)
            .await;

        match result {
            Ok(_) => succesful_inserts += 1,
            Err(err) => {
                println!("Failed to insert transaction: {:?}", err);
                failed_inserts += 1
            }
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

#[derive(FromForm)]
pub struct DateForm {
    date: String,
}

#[derive(FromForm)]
pub struct CounterpartyForm {
    counterparty: String,
}

#[derive(FromForm)]
pub struct AmountForm {
    amount: String,
}

#[post("/update_date", data = "<form>")]
pub async fn update_date(
    form: Form<DateForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.date_conv = Some(form.date.clone());
    })
    .await
}

#[post("/update_counterparty", data = "<form>")]
pub async fn update_counterparty(
    form: Form<CounterpartyForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.counterparty_conv = Some(form.counterparty.clone());
    })
    .await
}

#[post("/update_amount", data = "<form>")]
pub async fn update_amount(
    form: Form<AmountForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.amount_conv = Some(form.amount.clone());
    })
    .await
}

async fn update_csv_converter<F>(
    state: &State<AppState>,
    mut db: Connection<DbConn>,
    update_field: F,
) -> Template
where
    F: Fn(&mut CSVConverter),
{
    let current_bank_id;
    {
        let current_bank = state.current_bank.read().await;
        current_bank_id = current_bank.id;
    }

    let mut success = None;
    let mut error = None;

    {
        let mut csv_converters_lock = state.csv_convert.write().await;
        if let Some(current_csv_converter) = csv_converters_lock.get_mut(&current_bank_id) {
            update_field(current_csv_converter);
            let result = diesel::update(csv_converters::table.find(current_csv_converter.id))
                .set(current_csv_converter.clone())
                .execute(&mut db)
                .await;

            match result {
                Ok(_) => success = Some("Update successful"),
                Err(_) => error = Some("Update failed"),
            };
        } else {
            let mut new_csv_converter = CSVConverter {
                id: 0,
                csv_bank_id: current_bank_id,
                date_conv: None,
                counterparty_conv: None,
                amount_conv: None,
            };
            update_field(&mut new_csv_converter);
            let result = diesel::insert_into(csv_converters::table)
                .values(new_csv_converter.clone())
                .execute(&mut db)
                .await;

            if result.is_ok() {
                csv_converters_lock.insert(current_bank_id, new_csv_converter);
                success = Some("Insert successful");
            } else {
                error = Some("Insert failed");
            }
        }
    }

    let banks = state.banks.read().await.clone();
    let transactions = state.transactions.read().await.clone();
    let plot_data = generate_balance_graph_data(&banks, &transactions);
    let bank = state.current_bank.read().await.clone();
    let context = json!({
        "banks": banks,
        "bank": bank,
        "plot_data": plot_data.to_string(),
        "success": success,
        "error": error
    });

    Template::render("bank", &context)
}
