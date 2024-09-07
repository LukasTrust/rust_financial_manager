use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::error;
use rocket::serde::json::Json;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::{database::db_connector::DbConn, utils::appstate::LOCALIZATION};

use super::{appstate::Language, structs::ResponseData};

pub async fn delete_contracts_with_ids(
    contract_ids: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::contracts::dsl::*;

    diesel::delete(contracts.filter(id.eq_any(contract_ids.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting contracts with IDs {:?}: {:?}",
                contract_ids, e
            );
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_deleting_contract"),
                LOCALIZATION.get_localized_string(language, "error_deleting_contract_details"),
            ))
        })
}

pub async fn delete_contract_history_with_ids(
    contract_history_ids: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::contract_history::dsl::*;

    diesel::delete(contract_history.filter(id.eq_any(contract_history_ids.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting contract history with IDs {:?}: {:?}",
                contract_history_ids, e
            );
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_deleting_contract_history"),
                LOCALIZATION
                    .get_localized_string(language, "error_deleting_contract_history_details"),
            ))
        })
}
