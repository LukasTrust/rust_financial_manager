use ::diesel::ExpressionMethods;
use chrono::NaiveDate;
use diesel::QueryDsl;
use log::error;
use rocket::serde::json::Json;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, ContractHistory};
use crate::schema::{contract_history, contracts, csv_converters, transactions};
use crate::utils::appstate::LOCALIZATION;

use super::appstate::Language;
use super::structs::ResponseData;

pub async fn update_transactions_with_contract(
    transaction_ids: Vec<i32>,
    contract_id: Option<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    diesel::update(transactions::table.filter(transactions::id.eq_any(transaction_ids.clone())))
        .set(transactions::contract_id.eq(contract_id))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating transactions with contract: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION
                    .get_localized_string(language, "error_updating_transaction_of_contract"),
                LOCALIZATION.get_localized_string(
                    language,
                    "error_updating_transaction_of_contract_details",
                ),
            ))
        })
}

pub async fn update_transactions_of_contract_to_new_contract(
    new_contract_id: i32,
    old_contract_ids: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    diesel::update(transactions::table.filter(transactions::contract_id.eq_any(old_contract_ids)))
        .set(transactions::contract_id.eq(new_contract_id))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error updating transactions of contract to new contract: {:?}",
                e
            );
            Json(ResponseData::new_error(
                LOCALIZATION
                    .get_localized_string(language, "error_updating_transaction_of_contract"),
                LOCALIZATION.get_localized_string(
                    language,
                    "error_updating_transaction_of_contract_details",
                ),
            ))
        })
}

pub async fn update_transaction_with_hidden(
    transactions_id: i32,
    is_hidden_for_updating: bool,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq(transactions_id)))
        .set(is_hidden.eq(is_hidden_for_updating))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating transaction with hidden status: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_transaction"),
                LOCALIZATION.get_localized_string(language, "error_updating_transaction_details"),
            ))
        })
}

pub async fn update_transaction_with_contract_not_allowed(
    transaction_id: i32,
    contract_not_allowed_for_updating: bool,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq(transaction_id)))
        .set(contract_not_allowed.eq(contract_not_allowed_for_updating))
        .execute(db)
        .await
        .map_err(|e| {
            error!(
                "Error updating transaction with contract not allowed status: {:?}",
                e
            );
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_transaction"),
                LOCALIZATION.get_localized_string(language, "error_updating_transaction_details"),
            ))
        })
}

pub async fn update_contract_with_new_amount(
    contract_id: i32,
    new_amount: f64,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(current_amount.eq(new_amount))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating contract with new amount: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_contract"),
                LOCALIZATION
                    .get_localized_string(language, "error_updating_contract_amount_details"),
            ))
        })
}

pub async fn update_contract_with_new_name(
    contract_id: i32,
    new_name: String,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(name.eq(new_name.clone()))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating contract with new name: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_contract"),
                LOCALIZATION.get_localized_string(language, "error_updating_contract_name_details"),
            ))
        })
}

pub async fn update_contract_with_end_date(
    contract_id: i32,
    end_date_for_update: Option<NaiveDate>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(end_date.eq(end_date_for_update))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating contract with end date: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_contract"),
                LOCALIZATION
                    .get_localized_string(language, "error_updating_contract_end_date_details"),
            ))
        })
}

pub async fn update_csv_converter(
    csv_converter: CSVConverter,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    use crate::schema::csv_converters::*;

    diesel::update(csv_converters::table.find(csv_converter.id))
        .set((
            counterparty_column.eq(csv_converter.counterparty_column),
            amount_column.eq(csv_converter.amount_column),
            bank_balance_after_column.eq(csv_converter.bank_balance_after_column),
            date_column.eq(csv_converter.date_column),
        ))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating CSV converter: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_csv_converter"),
                LOCALIZATION.get_localized_string(language, "error_updating_csv_converter_details"),
            ))
        })
}

pub async fn update_contract_history(
    contract_history: ContractHistory,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<usize, Json<ResponseData>> {
    diesel::update(contract_history::table.find(contract_history.id))
        .set((
            contract_history::contract_id.eq(contract_history.contract_id),
            contract_history::old_amount.eq(contract_history.old_amount),
            contract_history::new_amount.eq(contract_history.new_amount),
            contract_history::changed_at.eq(contract_history.changed_at),
        ))
        .execute(db)
        .await
        .map_err(|e| {
            error!("Error updating contract history: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_updating_contract_history"),
                LOCALIZATION
                    .get_localized_string(language, "error_updating_contract_history_details"),
            ))
        })
}
