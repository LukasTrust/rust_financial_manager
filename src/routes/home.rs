use ::diesel::{ExpressionMethods, QueryDsl};
use log::info;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{Bank, CSVConverter, Transaction};
use crate::schema::{
    banks as banks_without_dsl, csv_converters as csv_converters_without_dsl,
    transactions as transactions_without_dsl,
};
use crate::structs::AppState;
use crate::utils::{extract_user_id, generate_balance_graph_data};

/// Display the home page.
/// The home page is the dashboard that displays the user's bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
/// The user's bank accounts and transactions are loaded from the database and displayed on the dashboard.
#[get("/home")]
pub async fn home(
    mut db: Connection<DbConn>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(cookie_user_id) => {
            info!("User is logged in: {}", cookie_user_id);

            use crate::schema::banks::dsl::*;
            use crate::schema::csv_converters::dsl::*;
            use crate::schema::transactions::dsl::*;

            let banks_result = banks_without_dsl::table
                .filter(user_id.eq(user_id))
                .load::<Bank>(&mut db)
                .await
                .map_err(|_| Redirect::to("/"))?;

            info!("Banks loaded: {:?}", banks_result);

            let mut transactions_map: HashMap<i32, Vec<Transaction>> = HashMap::new();
            let mut csv_converters_map: HashMap<i32, CSVConverter> = HashMap::new();

            for bank in banks_result.iter() {
                let transactions_result = transactions_without_dsl::table
                    .filter(bank_id.eq(bank.id))
                    .load::<Transaction>(&mut db)
                    .await
                    .map_err(|_| Redirect::to("/"))?;

                info!(
                    "Transactions loaded for bank {}: {:?}",
                    bank.id, transactions_result
                );

                transactions_map.insert(bank.id, transactions_result);

                let csv_converters_result = csv_converters_without_dsl::table
                    .filter(csv_bank_id.eq(bank.id))
                    .first::<CSVConverter>(&mut db)
                    .await
                    .map_err(|_| Redirect::to("/"));

                if csv_converters_result.is_ok() {
                    info!(
                        "CSV converter loaded for bank {}: {:?}",
                        bank.id, csv_converters_result
                    );
                    csv_converters_map.insert(bank.id, csv_converters_result.unwrap());
                }
            }

            let mut banks_state = state.banks.write().await;
            *banks_state = banks_result.clone();

            info!("Banks state updated: {:?}", *banks_state);

            let mut transactions_state = state.transactions.write().await;
            *transactions_state = transactions_map.clone();

            info!("Transactions state updated: {:?}", *transactions_state);

            let mut csv_converters_state = state.csv_convert.write().await;
            *csv_converters_state = csv_converters_map.clone();

            info!("CSV converters state updated: {:?}", *csv_converters_state);

            let plot_data = generate_balance_graph_data(&banks_result, &transactions_map);

            info!("Plot data generated.");

            Ok(Template::render(
                "dashboard",
                context! { banks: banks_result, plot_data: plot_data.to_string() },
            ))
        }
        Err(err) => {
            info!("User is not logged in or parsing user_id failed.");
            Err(Box::new(err))
        }
    }
}

/// Display the login page.
/// The login page is the first page that the user sees when they visit the website.
/// The user is redirected to the dashboard if they are already logged in.
#[get("/dashboard")]
pub async fn dashboard(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(_) => {
            let banks = state.banks.read().await.clone();
            let transactions = state.transactions.read().await.clone();
            let plot_data = generate_balance_graph_data(&banks, &transactions);

            Ok(Template::render(
                "dashboard",
                context! {
                    banks,
                    plot_data: plot_data.to_string()
                },
            ))
        }
        Err(err) => Err(Box::new(err)),
    }
}

/// Display the settings page.
/// The settings page allows the user to manage their bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
#[get("/settings")]
pub async fn settings(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    match extract_user_id(cookies) {
        Ok(_) => {
            let banks = state.banks.read().await.clone();
            Ok(Template::render("settings", context! {banks}))
        }
        Err(err) => Err(Box::new(err)),
    }
}

/// Display the login page.
/// Remove the user_id cookie to log the user out.
#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    info!("User logged out.");
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
