use diesel::result::{DatabaseErrorKind, Error as DieselError};
use log::{error, info};
use rocket::State;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::{db_connector::DbConn, models::NewBank};
use crate::schema::banks as banks_without_dsl;

use super::appstate::AppState;
use super::structs::Bank;

pub async fn insert_bank(
    cookie_user_id: i32,
    new_bank: NewBank,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Result<i32, String> {
    match diesel::insert_into(banks_without_dsl::table)
        .values(&new_bank)
        .get_result::<Bank>(db)
        .await
    {
        Ok(bank) => {
            info!("Bank inserted: {:?}", bank);

            state.update_banks(cookie_user_id, vec![bank.clone()]).await;

            Ok(bank.id)
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            error!("Bank with the name {} already exists", new_bank.name);
            Err(format!(
                "A bank with the name {} already exists. Please use a different name.",
                new_bank.name
            ))
        }
        Err(err) => {
            error!("Internal server error: {}", err);
            Err(format!("Internal server error: {}", err))
        }
    }
}
