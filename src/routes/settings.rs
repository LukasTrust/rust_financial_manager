use rocket::http::CookieJar;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use serde_json::Value;

use crate::utils::appstate::{AppState, Language, LOCALIZATION};
use crate::utils::get_utils::get_user_id;
use crate::utils::structs::ResponseData;

#[get("/user/set_language/<language>")]
pub async fn set_user_language(
    language: String,
    state: &State<AppState>,
    cookies: &CookieJar<'_>,
) -> Result<Json<&'static str>, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;
    let user_language = state.get_user_language(cookie_user_id).await;

    // Convert the language string to the enum
    let user_language = match language.as_str() {
        "English" => Language::English,
        "German" => Language::German,
        _ => {
            return Err(Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(user_language, "error_invalid_language"),
                LOCALIZATION.get_localized_string(user_language, "error_invalid_language_details"),
            )))
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
) -> Result<Json<Value>, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;

    // Get the user's language preference from the state
    let user_language = state.get_user_language(cookie_user_id).await;

    Ok(Json(json!({ "language": user_language })))
}
