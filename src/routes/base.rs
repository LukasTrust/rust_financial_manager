use log::info;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::database::db_mocking::load_user_by_id_mocking;
use crate::routes::error_page::show_error_page;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::loading_utils::{load_banks_of_user, load_user_by_id};
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
) -> Result<Template, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    info!("User is logged in: {}", cookie_user_id);

    let banks = load_banks_of_user(cookie_user_id, &mut db).await;

    if let Err(error) = banks {
        return Ok(Template::render(
            "base",
            json!(ResponseData::new_error(
                error,
                "There was an internal error trying to load the banks of the profile"
            )),
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
) -> Result<Template, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let (user_first_name, user_last_name) = match state.use_mocking {
        true => load_user_by_id_mocking(cookie_user_id)?,
        false => load_user_by_id(cookie_user_id, &mut db).await?,
    };

    state.set_current_bank(cookie_user_id, None).await;

    Ok(Template::render(
        "dashboard",
        json!(ResponseData::new_success(
            String::new(),
            &format!("Welcome, {} {}!", user_first_name, user_last_name)
        )),
    ))
}

/// Display the settings page.
/// The settings page allows the user to manage their bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
#[get("/settings")]
pub async fn settings(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    state.set_current_bank(cookie_user_id, None).await;

    Ok(Template::render("settings", json!({})))
}

/// Display the login page.
/// Remove the user_id cookie to log the user out.
#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Result<Template, Box<Redirect>> {
    info!("User logged out.");
    let cookie = cookies.get_private("user_id");

    if cookie.is_none() {
        return Err(show_error_page(
            "Error validating the login!".to_string(),
            "Please login again.".to_string(),
        ));
    }

    let cookie = cookie.unwrap();

    cookies.remove_private(cookie);
    Ok(Template::render("login", json!({})))
}
