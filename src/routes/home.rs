use ::diesel::{ExpressionMethods, QueryDsl};
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::tokio::sync::RwLock;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::db_connector::DbConn;
use crate::database::models::{Bank, CSVConverter, Transaction};
use crate::routes::help_functions::generate_balance_graph_data;
use crate::schema::{
    banks as banks_without_dsl, csv_converters as csv_converters_without_dsl,
    transactions as transactions_without_dsl,
};

#[derive(Serialize)]
pub struct Context {
    pub banks: Vec<Bank>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<Vec<Bank>>>,
    pub transactions: Arc<RwLock<HashMap<i32, Vec<Transaction>>>>,
    pub csv_convert: Arc<RwLock<HashMap<i32, CSVConverter>>>,
    pub current_bank: Arc<RwLock<Bank>>,
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
            use crate::schema::csv_converters::dsl::*;
            use crate::schema::transactions::dsl::*;

            let user_id_cookie = user_id_cookie.value().parse::<i32>().unwrap();
            let banks_result = banks_without_dsl::table
                .filter(user_id.eq(user_id_cookie))
                .load::<Bank>(&mut db)
                .await
                .map_err(|_| Redirect::to("/"))?;

            let mut transactions_map: HashMap<i32, Vec<Transaction>> = HashMap::new();
            let mut csv_converters_map: HashMap<i32, CSVConverter> = HashMap::new();

            for bank in banks_result.iter() {
                let transactions_result = transactions_without_dsl::table
                    .filter(bank_id.eq(bank.id))
                    .load::<Transaction>(&mut db)
                    .await
                    .map_err(|_| Redirect::to("/"))?;
                transactions_map.insert(bank.id, transactions_result);

                let csv_converters_result = csv_converters_without_dsl::table
                    .filter(csv_bank_id.eq(bank.id))
                    .first::<CSVConverter>(&mut db)
                    .await
                    .map_err(|_| Redirect::to("/"));

                if csv_converters_result.is_ok() {
                    csv_converters_map.insert(bank.id, csv_converters_result.unwrap());
                }
            }

            let mut banks_state = state.banks.write().await;
            *banks_state = banks_result.clone();

            let mut transactions_state = state.transactions.write().await;
            *transactions_state = transactions_map.clone();

            let mut csv_converters_state = state.csv_convert.write().await;
            *csv_converters_state = csv_converters_map.clone();

            let plot_data = generate_balance_graph_data(&banks_result, &transactions_map);

            let context = json!({
                "banks": banks_result,
                "plot_data": plot_data.to_string()
            });

            Ok(Template::render("dashboard", &context))
        } else {
            Err(Box::new(Redirect::to("/")))
        }
    } else {
        Err(Box::new(Redirect::to("/")))
    }
}

#[get("/dashboard")]
pub async fn dashboard(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    let transactions = state.transactions.read().await.clone();
    let plot_data = generate_balance_graph_data(&banks, &transactions);

    Template::render(
        "dashboard",
        context! {
            banks,
            plot_data: plot_data.to_string()
        },
    )
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
