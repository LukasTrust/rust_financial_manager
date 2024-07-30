use ::diesel::{ExpressionMethods, QueryDsl};
use chrono::NaiveDate;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use plotters::prelude::*;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::tokio::sync::RwLock;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::database::db_connector::DbConn;
use crate::database::models::{Bank, FormBank, NewBank, Transaction};
use crate::schema::{banks as banks_without_dsl, transactions as transactions_without_dsl};

#[derive(Serialize)]
pub struct Context {
    pub banks: Vec<Bank>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<Vec<Bank>>>,
    pub transactions: Arc<RwLock<HashMap<i32, Vec<Transaction>>>>,
}

#[get("/home")]
pub async fn home(
    mut db: Connection<DbConn>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        if user_id_cookie.value().parse::<i32>().is_ok() {
            use crate::schema::banks::dsl::*;
            use crate::schema::transactions::dsl::*;

            let user_id_cookie = user_id_cookie.value().parse::<i32>().unwrap();
            let banks_result = banks_without_dsl::table
                .filter(user_id.eq(user_id_cookie))
                .load::<Bank>(&mut db)
                .await
                .map_err(|_| Redirect::to("/"))?;

            // Create a HashMap to store transactions by bank_id
            let mut transactions_map: HashMap<i32, Vec<Transaction>> = HashMap::new();

            for bank in banks_result.iter() {
                let transactions_result = transactions_without_dsl::table
                    .filter(bank_id.eq(bank.id))
                    .load::<Transaction>(&mut db)
                    .await
                    .map_err(|_| Redirect::to("/"))?;
                transactions_map.insert(bank.id, transactions_result);
            }
            // Update the global state
            let mut banks_state = state.banks.write().await;
            *banks_state = banks_result.clone();

            let mut transactions_state = state.transactions.write().await;
            *transactions_state = transactions_map.clone();

            let _ = generate_balance_graph(&banks_result, &transactions_map);

            let context = Context {
                banks: banks_result,
            };

            Ok(Template::render("dashboard", &context))
        } else {
            Err(Box::new(Redirect::to("/")))
        }
    } else {
        Err(Box::new(Redirect::to("/")))
    }
}

#[get("/add-bank")]
pub async fn add_bank(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("add_bank", context! { banks })
}

#[get("/dashboard")]
pub async fn dashboard(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("dashboard", context! { banks })
}

#[get("/settings")]
pub async fn settings(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("settings", context! {banks})
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    mut db: Connection<DbConn>,
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
) -> Template {
    let user_id = cookies
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<i32>().ok());

    match user_id {
        Some(_) => {}
        None => return Template::render("home", context! { error: "Could not find user id" }),
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
            context! { error: "A bank with this name already exists. Please use a different name." },
        ),
        Err(err) => Template::render(
            "add_bank",
            context! { error: format!("Internal server error {}", err) },
        ),
    }
}

fn generate_balance_graph(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new("static/balance_graph.svg", (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Bank Account Balances", ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .build_cartesian_2d(
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
                ..NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            0.0..100.0,
        )?; // assuming balance range 0-100, update the date range as needed

    chart.configure_mesh().draw()?;

    for bank in banks {
        let bank_transactions = transactions.get(&bank.id).unwrap();
        let mut balance = bank.current_amount.unwrap_or(0.0);
        let mut data: BTreeMap<NaiveDate, f64> = BTreeMap::new();

        // Insert today's balance
        let today = chrono::Local::now().naive_local();
        data.insert(today.into(), balance);

        for transaction in bank_transactions.iter().rev() {
            match transaction.type_of_t.as_str() {
                "Deposit" => balance -= transaction.amount,
                "Withdraw" => balance += transaction.amount,
                "Interest" => balance -= transaction.amount, // Assuming interest is added to the balance
                _ => (),
            }
            data.entry(transaction.date)
                .and_modify(|e| *e = balance)
                .or_insert(balance);
        }

        data.entry(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            .or_insert(balance); // Ensure we plot the initial balance at the start

        let series_data: Vec<(NaiveDate, f64)> = data.into_iter().collect();

        chart
            .draw_series(LineSeries::new(series_data, &RED))?
            .label(bank.name.clone())
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}
