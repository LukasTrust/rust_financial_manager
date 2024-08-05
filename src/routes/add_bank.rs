use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel::{ExpressionMethods, QueryDsl};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::database::models::NewBank;
use crate::schema::banks as banks_without_dsl;
use crate::utils::display_utils::show_home_or_subview_with_data;
use crate::utils::get_utils::get_user_id;
use crate::utils::set_utils::set_app_state;
use crate::utils::structs::{AppState, Bank, FormBank};

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
        current_amount: bank_form.current_amount,
    };

    let result = diesel::insert_into(banks_without_dsl::table)
        .values(&new_bank)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => {
            let inserted_bank = banks_without_dsl::table
                .filter(banks_without_dsl::name.eq(&new_bank.name))
                .first::<Bank>(&mut db)
                .await;

            match inserted_bank {
                Ok(inserted_bank) => {
                    set_app_state(
                        cookie_user_id,
                        state,
                        Some(vec![inserted_bank.clone()]),
                        None,
                        None,
                        None,
                    )
                    .await;

                    Ok(show_home_or_subview_with_data(
                        cookie_user_id,
                        state,
                        "add_bank".to_string(),
                        false,
                        false,
                        Some(format!("Bank {} added", inserted_bank.name)),
                        None,
                    )
                    .await)
                }
                Err(err) => Ok(show_home_or_subview_with_data(
                    cookie_user_id,
                    state,
                    "add_bank".to_string(),
                    false,
                    false,
                    None,
                    Some(format!("Internal server error {}", err)),
                )
                .await),
            }
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Ok(show_home_or_subview_with_data(
                cookie_user_id,
                state,
                "add_bank".to_string(),
                false,
                false,
                None,
                Some(format!(
                    "A bank with the name {} already exists. Please use a different name.",
                    new_bank.name
                )),
            )
            .await)
        }
        Err(err) => Ok(show_home_or_subview_with_data(
            cookie_user_id,
            state,
            "add_bank".to_string(),
            false,
            false,
            None,
            Some(format!("Internal server error {}", err)),
        )
        .await),
    }
}
