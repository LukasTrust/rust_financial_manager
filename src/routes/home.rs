use log::info;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, Transaction};
use crate::structs::AppState;
use crate::utils::{
    extract_user_id, load_banks, load_csv_converters, load_transactions,
    show_home_or_subview_with_data, update_app_state,
};

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

            let banks_result = load_banks(cookie_user_id, &mut db).await?;

            let mut transactions_map: HashMap<i32, Vec<Transaction>> = HashMap::new();
            let mut csv_converters_map: HashMap<i32, CSVConverter> = HashMap::new();

            for bank in banks_result.iter() {
                let transactions_result = load_transactions(bank.id, &mut db).await?;

                transactions_map.insert(bank.clone().id, transactions_result);

                let csv_converters_result = load_csv_converters(bank.id, &mut db).await?;

                match csv_converters_result {
                    Some(csv_converter) => {
                        csv_converters_map.insert(bank.id, csv_converter);
                    }
                    None => {}
                }
            }

            update_app_state(
                cookie_user_id,
                state,
                Some(banks_result.clone()),
                Some(transactions_map.clone()),
                Some(csv_converters_map),
                None,
            )
            .await;

            Ok(show_home_or_subview_with_data(
                cookie_user_id,
                state,
                "dashboard".to_string(),
                true,
                false,
                None,
                None,
            )
            .await)
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
        Ok(cookie_user_id) => Ok(show_home_or_subview_with_data(
            cookie_user_id,
            state,
            "dashboard".to_string(),
            true,
            false,
            None,
            None,
        )
        .await),
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
        Ok(cookie_user_id) => Ok(show_home_or_subview_with_data(
            cookie_user_id,
            state,
            "settings".to_string(),
            false,
            false,
            None,
            None,
        )
        .await),
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
