use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::error;
use rocket::serde::json::Json;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::{
    database::db_connector::DbConn,
    utils::{appstate::LOCALIZATION, structs::ErrorResponse},
};

use super::appstate::Language;

pub async fn delete_contracts_with_ids(
    contract_ids: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ErrorResponse>> {
    use crate::schema::contracts::dsl::*;

    diesel::delete(contracts.filter(id.eq_any(contract_ids.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting contracts with IDs {:?}: {:?}",
                contract_ids, e
            );
            Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(language, "error_deleting_contract"),
                LOCALIZATION.get_localized_string(language, "error_deleting_contract_details"),
            ))
        })
}

pub async fn delete_contract_history_with_ids(
    contract_history_ids: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ErrorResponse>> {
    use crate::schema::contract_history::dsl::*;

    diesel::delete(contract_history.filter(id.eq_any(contract_history_ids.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting contract history with IDs {:?}: {:?}",
                contract_history_ids, e
            );
            Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(language, "error_deleting_contract_history"),
                LOCALIZATION
                    .get_localized_string(language, "error_deleting_contract_history_details"),
            ))
        })
}

pub async fn delete_user_by_email(
    user_email_for_deleting: String,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ErrorResponse>> {
    use crate::schema::users::dsl::*;

    diesel::delete(users.filter(email.eq(user_email_for_deleting.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting user with email '{}': {:?}",
                user_email_for_deleting, e
            );
            Json(ErrorResponse::new(
                "Error deleting user".to_string(),
                "An error occurred while deleting the user.".to_string(),
            ))
        })
}

pub async fn delete_user_by_id(
    user_id_for_deleting: i32,
    user_language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ErrorResponse>> {
    use crate::schema::users::dsl::*;

    diesel::delete(users.filter(id.eq(user_id_for_deleting)))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting user with ID '{}': {:?}",
                user_id_for_deleting, e
            );
            Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(user_language, "error_deleting_user"),
                LOCALIZATION.get_localized_string(user_language, "error_deleting_user_details"),
            ))
        })
}

pub async fn delete_bank_by_name(
    bank_name_for_deleting: String,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ErrorResponse>> {
    use crate::schema::banks::dsl::*;

    diesel::delete(banks.filter(name.eq(bank_name_for_deleting.clone())))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error deleting bank with name '{}': {:?}",
                bank_name_for_deleting, e
            );
            Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(Language::English, "error_deleting_bank"),
                LOCALIZATION.get_localized_string(Language::English, "error_deleting_bank_details"),
            ))
        })
}
