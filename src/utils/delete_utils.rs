use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;

pub async fn delete_contracts_with_ids(
    contract_ids: Vec<i32>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contracts::dsl::*;

    diesel::delete(contracts.filter(id.eq_any(contract_ids.clone())))
        .execute(db)
        .await
        .map_err(|_| "Error deleting contracts")?;

    info!("Contracts IDs {:?} deleted.", contract_ids);

    Ok(())
}

pub async fn delete_contract_history_with_ids(
    contract_history_ids: Vec<i32>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contract_history::dsl::*;

    diesel::delete(contract_history.filter(id.eq_any(contract_history_ids.clone())))
        .execute(db)
        .await
        .map_err(|_| "Error deleting contract histories")?;

    info!("Contract history IDs {:?} deleted.", contract_history_ids);

    Ok(())
}
