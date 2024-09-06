use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use serde_json::Value;

use crate::utils::appstate::{AppState, Language};
use crate::utils::get_utils::get_user_id;

use super::error_page::show_error_page;

#[get("/user/set_language/<language>")]
pub async fn set_user_language(
    language: String,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<&'static str>, Box<Redirect>> {
    // Get the user's ID from the cookie
    let cookie_user_id = get_user_id(cookies)?;

    // Convert the language string to the enum
    let user_language = match language.as_str() {
        "English" => Language::English,
        "German" => Language::German,
        _ => {
            return Err(show_error_page(
                "Language not supported".to_string(),
                "The requested message is not supported.".to_string(),
            ))
        }
    };

    // Update the user's language preference in the state
    state.set_user_language(cookie_user_id, user_language).await;

    Ok(Json("Language preference updated successfully"))
}

#[get("/user/get_language")]
pub async fn get_user_language(
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<Value>, Box<Redirect>> {
    // Get the user's ID from the cookie
    let cookie_user_id = get_user_id(cookies)?;

    // Get the user's language preference from the state
    let user_language = state
        .get_user_language(cookie_user_id)
        .await
        .unwrap_or(Language::English);

    Ok(Json(json!({ "language": user_language })))
}
