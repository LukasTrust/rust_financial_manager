use rocket::http::CookieJar;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_user_id, get_user_language};
use crate::utils::loading_utils::load_current_bank_of_user;
use crate::utils::structs::ErrorResponse;
use crate::utils::translation_utils::get_bank_localized_strings;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Json<ErrorResponse>> {
    let cookie_user_id = get_user_id(cookies)?;
    let cookie_user_language = get_user_language(cookies);

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
