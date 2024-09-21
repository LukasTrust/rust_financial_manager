use rocket::http::CookieJar;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::{AppState, LOCALIZATION};
use crate::utils::delete_utils::delte_bank_by_id;
use crate::utils::get_utils::get_user_id_and_language;
use crate::utils::loading_utils::load_current_bank_of_user;
use crate::utils::structs::{ErrorResponse, SuccessResponse};
use crate::utils::translation_utils::get_bank_localized_strings;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank =
        load_current_bank_of_user(cookie_user_id, bank_id, cookie_user_language, &mut db).await?;

    state
        .set_current_bank(cookie_user_id, Some(current_bank.clone()))
        .await;

    let translation_string = get_bank_localized_strings(cookie_user_language);

    Ok(Template::render(
        "bank",
        json!({
            "bank": current_bank,
            "translations": translation_string,
        }),
    ))
}

#[get("/delete_bank")]
pub async fn delete_bank(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let _ = delte_bank_by_id(current_bank.id, cookie_user_language, &mut db).await?;

    state.set_current_bank(cookie_user_id, None).await;

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "deleted_bank"),
        LOCALIZATION.get_localized_string(cookie_user_language, "deleted_bank_details"),
    )))
}
