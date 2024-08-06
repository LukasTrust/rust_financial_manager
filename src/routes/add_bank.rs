use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::database::models::NewBank;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::show_home_or_subview_with_data;
use crate::utils::get_utils::get_user_id;
use crate::utils::insert_utiles::insert_bank;
use crate::utils::structs::FormBank;

#[get("/add-bank")]
pub async fn add_bank(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    Ok(show_home_or_subview_with_data(
        cookie_user_id,
        state,
        "add_bank".to_string(),
        false,
        false,
        None,
        None,
    )
    .await)
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let new_bank = NewBank {
        user_id: cookie_user_id,
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
    };

    Ok(insert_bank(cookie_user_id, new_bank.clone(), state, &mut db).await)
}
