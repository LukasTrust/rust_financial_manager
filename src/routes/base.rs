use ::diesel::{ExpressionMethods, QueryDsl};
use log::info;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::routes::error_page::show_error_page;
use crate::schema::users::{self, first_name, last_name};
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_performance_value_and_graph_data, get_user_id};
use crate::utils::loading_utils::load_banks;
use crate::utils::structs::ResponseData;

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

    let banks = load_banks(cookie_user_id, &mut db).await;

    if let Err(error) = banks {
        return Ok(Template::render(
            "base",
            json!({ "response":
            ResponseData {
                success: None,
                error: Some("There was an internal error trying to load the banks of the profile".into()),
                header: Some(error),
            }}),
        ));
    }

    let banks = banks.unwrap();

    state.set_current_bank(cookie_user_id, None).await;

    Ok(Template::render(
        "base",
        json!({
            "banks": banks,
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

    state.set_current_bank(cookie_user_id, None).await;

    let banks = load_banks(cookie_user_id, &mut db).await;

    if let Err(error) = banks {
        return Ok(Template::render(
            "dashboard",
            json!({ "response":
            ResponseData {
                success: None,
                error: Some("There was an internal error trying to load the banks of the profile".into()),
                header: Some(error),
            }}),
        ));
    }

    let banks = banks.unwrap();

    let result = get_performance_value_and_graph_data(&banks, None, None, db).await;

    if let Err(error) = result {
        return Ok(Template::render(
            "bank",
            json!({"response":
            serde_json::to_string(&ResponseData {
                success: None,
                error: Some("There was an internal error while loading the bank. Please try again.".into()),
                header: Some(error),
            }).unwrap(),}),
        ));
    }

    let (performance_value, graph_data) = result.unwrap();

    Ok(Template::render(
        "dashboard",
        json!({
            "response":
            serde_json::to_string(&ResponseData {
                success: Some(format!("Welcome, {} {}!", user_first_name, user_last_name)),
                error: None,
                header: None,
            }).unwrap(),
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

    state.set_current_bank(cookie_user_id, None).await;

    Ok(Template::render("settings", json!({})))
}

/// Display the login page.
/// Remove the user_id cookie to log the user out.
#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Template {
    info!("User logged out.");
    cookies.remove(Cookie::build("user_id"));
    Template::render("/", json!({}))
}
