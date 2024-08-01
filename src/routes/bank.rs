use chrono::NaiveDate;
use csv::ReaderBuilder;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::sql_types::Record;
use diesel::{ExpressionMethods, QueryDsl};
use rocket::data::{Data, ToByteUnit};
use rocket::form::{Form, FromForm};
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
        interest_rate: bank_form.interest_rate,
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
        .delimiter(b',')
        .from_reader(cursor);

    let mut header_indices: Vec<usize> = Vec::new();

    // Iterate over records to find the headers
    for (i, result) in rdr.records().enumerate() {
        let record = result;

        if record.is_err() {
            return render_template_with_error(state, Some("Failed to read CSV file")).await;
        }

        // If we have already found all headers, we can stop
        if header_indices.len() == headers_to_extract.len() {
            break;
        }

        // Check if this record has any of the headers we are looking for
        for (_, field) in record.iter().enumerate() {
            let array = field.as_slice().split(';');
            for (j, value) in array.enumerate() {
                if headers_to_extract.contains(&value.to_string()) {
                    header_indices.push(j);
                    print!("{}: {} ", value, j);
                }
            }
        }
    }

    render_template_with_error(state, None).await
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
