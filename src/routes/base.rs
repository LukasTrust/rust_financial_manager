use ::diesel::{ExpressionMethods, QueryDsl};
use log::info;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, Contract};
use crate::routes::error_page::show_error_page;
use crate::schema::users::{self, first_name, last_name};
use crate::utils::appstate::AppState;
use crate::utils::display_utils::{generate_balance_graph_data, generate_performance_value};
use crate::utils::get_utils::{
    get_banks_of_user, get_first_date_and_last_date_from_bank, get_user_id,
};
use crate::utils::loading_utils::{
    load_banks, load_contracts_of_bank, load_csv_converters_of_bank, load_transactions_of_bank,
};
use crate::utils::structs::Transaction;

/// Display the base page.
/// The base page is the dashboard that displays the user's bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
/// The user's bank accounts and transactions are loaded from the database and displayed on the dashboard.
#[get("/base")]
pub async fn base(
    mut db: Connection<DbConn>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    info!("User is logged in: {}", cookie_user_id);

    let banks_result = load_banks(cookie_user_id, &mut db).await?;

    let mut transactions_map: HashMap<i32, Vec<Transaction>> = HashMap::new();
    let mut csv_converters_map: HashMap<i32, CSVConverter> = HashMap::new();
    let mut contract_map: HashMap<i32, Vec<Contract>> = HashMap::new();

    for bank in banks_result.iter() {
        let transactions_result = load_transactions_of_bank(bank.id, &mut db).await?;
        transactions_map.insert(bank.clone().id, transactions_result);

        if let Some(csv_converter) = load_csv_converters_of_bank(bank.id, &mut db).await? {
            csv_converters_map.insert(bank.id, csv_converter);
        }

        let contracts_result = load_contracts_of_bank(bank.id, &mut db).await?;
        contract_map.insert(bank.id, contracts_result);
    }

    state
        .set_app_state(
            cookie_user_id,
            Some(banks_result.clone()),
            Some(transactions_map.clone()),
            Some(csv_converters_map),
            Some(contract_map.clone()),
            None,
        )
        .await;

    Ok(Template::render(
        "base",
        json!({
            "banks": banks_result,
            "view_name": "dashboard",
        }),
    ))
}

/// Display the login page.
/// The login page is the first page that the user sees when they visit the website.
/// The user is redirected to the dashboard if they are already logged in.
#[get("/dashboard")]
pub async fn dashboard(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let (user_first_name, user_last_name) = users::table
        .filter(users::id.eq(cookie_user_id))
        .select((first_name, last_name))
        .first::<(String, String)>(&mut db)
        .await
        .map_err(|_| {
            info!("User not found: {}", cookie_user_id);
            show_error_page(
                "User not found!".to_string(),
                "Please login again.".to_string(),
            )
        })?;

    state.update_current_bank(cookie_user_id, None).await;

    let transactions_map = state.transactions.read().await;
    let transactions_vec: Vec<Transaction> =
        transactions_map.clone().into_values().flatten().collect();

    let transactions: Option<&Vec<Transaction>> = Some(&transactions_vec);

    let banks = get_banks_of_user(cookie_user_id, state).await;

    let (first_date, last_date) = get_first_date_and_last_date_from_bank(transactions);

    let performance_value =
        generate_performance_value(&banks, &transactions_map, first_date, last_date);

    let graph_data =
        generate_balance_graph_data(&banks, &transactions_map, performance_value.1, None, None)
            .await;

    let contract_map = state.contracts.read().await;
    let contracts_string = serde_json::to_string(
        &contract_map
            .values()
            .flatten()
            .cloned()
            .collect::<Vec<Contract>>(),
    )
    .unwrap();

    Ok(Template::render(
        "dashboard",
        json!({
            "success": format!("Welcome, {} {}!", user_first_name, user_last_name),
            "graph_data": graph_data,
            "performance_value": performance_value.0,
            "contracts": contracts_string
        }),
    ))
}

/// Display the settings page.
/// The settings page allows the user to manage their bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
#[get("/settings")]
pub async fn settings(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    state.update_current_bank(cookie_user_id, None).await;

    Ok(Template::render("settings", json!({})))
}

/// Display the login page.
/// Remove the user_id cookie to log the user out.
#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    info!("User logged out.");
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
