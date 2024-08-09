use log::info;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::CSVConverter;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::{
    generate_balance_graph_data, generate_performance_value, show_base_or_subview_with_data,
};
use crate::utils::get_utils::{get_banks_of_user, get_user_id};
use crate::utils::loading_utils::{load_banks, load_csv_converters, load_transactions};
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

    for bank in banks_result.iter() {
        let transactions_result = load_transactions(bank.id, &mut db).await?;
        transactions_map.insert(bank.clone().id, transactions_result);

        if let Some(csv_converter) = load_csv_converters(bank.id, &mut db).await? {
            csv_converters_map.insert(bank.id, csv_converter);
        }
    }

    state
        .set_app_state(
            cookie_user_id,
            Some(banks_result.clone()),
            Some(transactions_map.clone()),
            Some(csv_converters_map),
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
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    state.update_current_bank(cookie_user_id, None).await;

    let transactions_map = state.transactions.read().await;

    let banks = get_banks_of_user(cookie_user_id, state).await;

    let first_date = transactions_map
        .values()
        .flatten()
        .map(|transaction| transaction.date)
        .min()
        .unwrap_or_default();

    let last_date = transactions_map
        .values()
        .flatten()
        .map(|transaction| transaction.date)
        .max()
        .unwrap_or_default();

    let graph_data = generate_balance_graph_data(&banks, &transactions_map).await;
    let performance_value =
        generate_performance_value(&banks, &transactions_map, first_date, last_date);

    Ok(Template::render(
        "dashboard",
        json!({
            "graph_data": graph_data,
            "performance_value": performance_value,
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

    Ok(show_base_or_subview_with_data(
        cookie_user_id,
        state,
        "settings".to_string(),
        false,
        false,
        None,
        None,
        None,
    )
    .await)
}

/// Display the login page.
/// Remove the user_id cookie to log the user out.
#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    info!("User logged out.");
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
