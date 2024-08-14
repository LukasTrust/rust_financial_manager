use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::schema::transactions;

pub async fn update_transaction_with_contract_id(
    transaction_id: i32,
    contract_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    diesel::update(transactions::table.find(transaction_id))
        .set(transactions::contract_id.eq(Some(contract_id)))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating transaction: {}", e))?;

    info!(
        "Transaction ID {} updated with contract ID {}.",
        transaction_id, contract_id
    );
    Ok(())
}
