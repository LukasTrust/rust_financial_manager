use chrono::{Duration, Utc};
use log::error;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::NewContractHistory;
use crate::utils::appstate::LOCALIZATION;
use crate::utils::delete_utils::delete_contract_history_with_ids;
use crate::utils::insert_utiles::insert_contract_histories;
use crate::utils::update_utils::{
    update_contract_history, update_contract_with_new_amount, update_transactions_with_contract,
};

use super::appstate::Language;
use super::delete_utils::delete_contracts_with_ids;
use super::loading_utils::{
    load_contract_history, load_contracts_from_ids, load_last_transaction_of_contract,
    load_transaction_by_id,
};
use super::structs::{ErrorResponse, SuccessResponse};
use super::update_utils::update_contract_with_end_date;

pub async fn handle_remove_contract(
    transaction_id: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    // Load transaction by ID
    let transaction = load_transaction_by_id(transaction_id, language, db).await?;

    // Check if transaction has a contract
    let contract_id = transaction.contract_id.ok_or_else(|| {
        error!("Transaction has no contract");
        Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(language, "error_transaction_has_no_contract"),
            LOCALIZATION
                .get_localized_string(language, "error_transaction_has_no_contract_details"),
        ))
    })?;

    // Update the transaction to remove the contract association
    update_transactions_with_contract(vec![transaction_id], None::<i32>, language, db).await?;

    let last_transaction_of_contract =
        load_last_transaction_of_contract(contract_id, language, db).await;

    if last_transaction_of_contract.is_err() {
        delete_contracts_with_ids(vec![contract_id], language, db).await?;

        return Ok(Json(SuccessResponse::new(
            LOCALIZATION.get_localized_string(language, "delete_contract_with_no_transactions"),
            LOCALIZATION
                .get_localized_string(language, "delete_contract_with_no_transactions_details"),
        )));
    }

    let last_transaction_of_contract = last_transaction_of_contract.unwrap();
    let new_date = last_transaction_of_contract.date + Duration::days(60);
    let now = Utc::now().naive_utc().date();

    let new_end_date = match new_date < now {
        true => Some(last_transaction_of_contract.date),
        false => None,
    };

    update_contract_with_end_date(contract_id, new_end_date, language, db).await?;

    // Load contract
    let contract = load_contracts_from_ids(vec![contract_id], language, db).await?;

    if contract.len() != 1 {
        error!("Contract not found");
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(language, "error_contract_not_found"),
            LOCALIZATION.get_localized_string(language, "error_contract_not_found_details"),
        )));
    }

    let contract = contract[0].clone();

    // Load contract history
    let contract_histories = load_contract_history(contract.id, language, db).await?;

    // Find contract history corresponding to the transaction amount
    if let Some(history) = contract_histories
        .iter()
        .find(|h| h.new_amount == transaction.amount && h.changed_at == transaction.date)
    {
        update_contract_with_new_amount(contract.id, history.old_amount, language, db).await?;

        let history_before = contract_histories
            .iter()
            .filter(|h| h.changed_at < history.changed_at)
            .max_by_key(|h| h.changed_at);

        let history_after = contract_histories
            .iter()
            .filter(|h| h.changed_at > history.changed_at)
            .min_by_key(|h| h.changed_at);

        delete_contract_history_with_ids(vec![history.id], language, db).await?;

        match (history_before, history_after) {
            (Some(before), Some(_)) => {
                // Case 1: Both before and after history entries exist
                let mut updated_before = before.clone();
                updated_before.new_amount = history.new_amount;

                update_contract_history(updated_before, language, db).await?;
            }
            (None, Some(after)) => {
                // Case 2: Only after history entry exists
                let mut updated_after = after.clone();

                updated_after.old_amount = history.old_amount;

                update_contract_history(updated_after, language, db).await?;
            }
            (Some(_), None) => {}
            (None, None) => {}
        }
    }

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(language, "transaction_removed_from_contract"),
        LOCALIZATION.get_localized_string(language, "transaction_removed_from_contract_details"),
    )))
}

pub async fn handel_update_amount(
    transaction_id: i32,
    contract_id: i32,
    language: Language,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let transaction = load_transaction_by_id(transaction_id, language, &mut db).await?;

    let contract = load_contracts_from_ids(vec![contract_id], language, &mut db).await?;

    if contract.len() != 1 {
        error!("Contract not found");
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(language, "error_contract_not_found"),
            LOCALIZATION.get_localized_string(language, "error_contract_not_found_details"),
        )));
    }

    let mut contract = contract[0].clone();

    update_transactions_with_contract(vec![transaction_id], Some(contract.id), language, &mut db)
        .await?;

    let contract_history = NewContractHistory {
        contract_id: contract.id,
        old_amount: contract.current_amount,
        new_amount: transaction.amount,
        changed_at: transaction.date,
    };

    insert_contract_histories(&vec![contract_history], language, &mut db).await?;

    contract.current_amount = transaction.amount;

    update_contract_with_new_amount(contract.id, transaction.amount, language, &mut db).await?;

    if contract.end_date.is_some() && contract.end_date.unwrap() < transaction.date {
        let new_date = transaction.date + Duration::days(60);
        let now = Utc::now().naive_utc().date();

        let new_end_date = match new_date < now {
            true => Some(transaction.date),
            false => None,
        };

        update_contract_with_end_date(contract.id, new_end_date, language, &mut db).await?;
    }

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(language, "contract_updated"),
        LOCALIZATION.get_localized_string(language, "contract_updated_details"),
    )))
}

pub async fn handle_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    language: Language,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let transaction = load_transaction_by_id(transaction_id, language, &mut db).await?;

    let contract = load_contracts_from_ids(vec![contract_id], language, &mut db).await?;

    if contract.len() != 1 {
        error!("Contract not found");
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(language, "error_contract_not_found"),
            LOCALIZATION.get_localized_string(language, "error_contract_not_found_details"),
        )));
    }

    let contract = contract[0].clone();

    let contract_histories = load_contract_history(contract_id, language, &mut db).await?;

    let history_before = contract_histories
        .iter()
        .filter(|h| h.changed_at < transaction.date)
        .max_by_key(|h| h.changed_at);

    let history_after = contract_histories
        .iter()
        .filter(|h| h.changed_at > transaction.date)
        .min_by_key(|h| h.changed_at);

    update_transactions_with_contract(vec![transaction_id], Some(contract_id), language, &mut db)
        .await?;

    match (history_before, history_after) {
        (Some(before), Some(after)) => {
            // Case 1: Both before and after history entries exist
            let mut updated_after = after.clone();

            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: before.new_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            updated_after.old_amount = transaction.amount;

            update_contract_history(updated_after, language, &mut db).await?;

            insert_contract_histories(&vec![history], language, &mut db).await?;

            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(language, "contract_history_updated"),
                LOCALIZATION.get_localized_string(language, "contract_history_updated_details"),
            )))
        }
        (None, Some(after)) => {
            // Case 2: Only after history entry exists
            let mut updated_after = after.clone();

            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: updated_after.old_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            updated_after.old_amount = history.new_amount;

            update_contract_history(updated_after, language, &mut db).await?;

            insert_contract_histories(&vec![history], language, &mut db).await?;

            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(language, "contract_history_updated"),
                LOCALIZATION.get_localized_string(language, "contract_history_updated_details"),
            )))
        }
        (Some(before), None) => {
            let mut updated_before = before.clone();

            // Case 3: Only before history entry exists
            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: transaction.amount,
                new_amount: contract.current_amount,
                changed_at: transaction.date,
            };

            updated_before.new_amount = transaction.amount;

            update_contract_history(updated_before, language, &mut db).await?;

            insert_contract_histories(&vec![history], language, &mut db).await?;

            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(language, "contract_history_updated"),
                LOCALIZATION.get_localized_string(language, "contract_history_updated_details"),
            )))
        }
        (None, None) => {
            // Case 4: No history entries exist
            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: transaction.amount,
                new_amount: contract.current_amount,
                changed_at: transaction.date,
            };

            insert_contract_histories(&vec![history], language, &mut db).await?;

            Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(language, "contract_history_updated"),
                LOCALIZATION.get_localized_string(language, "contract_history_updated_details"),
            )))
        }
    }
}
