use diesel::result::{DatabaseErrorKind, Error as DieselError};
use log::{error, info};
use rocket::State;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;

use crate::database::{db_connector::DbConn, models::NewBank};
use crate::schema::banks as banks_without_dsl;
use crate::utils::appstate;

use super::appstate::AppState;
use super::display_utils::show_home_or_subview_with_data;
use super::structs::Bank;

pub async fn insert_bank(
    cookie_user_id: i32,
    new_bank: NewBank,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Template {
    let (success_message, error_message) = match diesel::insert_into(banks_without_dsl::table)
        .values(&new_bank)
        .get_result::<Bank>(db)
        .await
    {
        Ok(bank) => {
            info!("Bank inserted: {:?}", bank);

            state.update_banks(cookie_user_id, vec![bank]).await;

            (Some(format!("Bank {} added", new_bank.name)), None)
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            error!("Bank with the name {} already exists", new_bank.name);
            (
                None,
                Some(format!(
                    "A bank with the name {} already exists. Please use a different name.",
                    new_bank.name
                )),
            )
        }
        Err(err) => {
            error!("Internal server error: {}", err);
            (None, Some(format!("Internal server error: {}", err)))
        }
    };

    show_home_or_subview_with_data(
        cookie_user_id,
        state,
        "add_bank".to_string(),
        false,
        false,
        success_message,
        error_message,
    )
    .await
}
