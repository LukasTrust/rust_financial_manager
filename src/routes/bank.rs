use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, State};
use rocket_dyn_templates::Template;

use super::error_page::show_error_page;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::show_base_or_subview_with_data;
use crate::utils::get_utils::{get_banks_of_user, get_user_id};

#[get("/bank/<bank_id>")]
pub async fn bank_view(
    bank_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let banks = get_banks_of_user(cookie_user_id, state).await;
    let bank = banks.iter().find(|&b| b.id == bank_id);

    match bank {
        Some(new_current_bank) => {
            state
                .update_current_bank(cookie_user_id, Some(new_current_bank.clone()))
                .await;

            Ok(show_base_or_subview_with_data(
                cookie_user_id,
                state,
                "bank".to_string(),
                true,
                true,
                None,
                None,
                None,
            )
            .await)
        }
        None => {
            return Err(show_error_page(
                "Bank not found".to_string(),
                "The bank you are looking for does not exist.".to_string(),
            ))
        }
    }
}
