use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::get_user_id;
use crate::utils::loading_utils::load_current_bank_of_user;
use crate::utils::structs::ResponseData;

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let current_bank =
        load_current_bank_of_user(cookie_user_id, bank_id, language, &mut db).await?;

    state
        .set_current_bank(cookie_user_id, Some(current_bank.clone()))
        .await;

    Ok(Template::render("bank", context! { bank: current_bank }))
}
