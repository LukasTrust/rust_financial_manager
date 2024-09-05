use log::{error, info};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::NewContractHistory;
use crate::utils::delete_utils::delete_contract_history_with_ids;
use crate::utils::get_utils::get_transaction;
use crate::utils::insert_utiles::insert_contract_histories;
use crate::utils::update_utils::{
    update_contract_history, update_contract_with_new_amount, update_transactions_with_contract,
};

use super::loading_utils::{
    load_contract_history, load_contracts_from_ids, load_transaction_by_id,
};
use super::structs::ResponseData;

pub async fn handle_remove_contract(
    transaction_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    // Load transaction by ID
    let transaction = load_transaction_by_id(transaction_id, db)
        .await
        .map_err(|error| {
            error!("Failed to load transaction: {}", error);
            error
        })?
        .ok_or_else(|| {
            error!("Transaction not found");
            "Transaction not found".to_string()
        })?;

    // Check if transaction has a contract
    let contract_id = transaction.contract_id.ok_or_else(|| {
        error!("Transaction has no contract");
        "Transaction has no contract".to_string()
    })?;

    // Load contract
    let contract = load_contracts_from_ids(vec![contract_id], db)
        .await
        .map_err(|e| {
            error!("Failed to load contract: {}", e);
            e
        })?
        .into_iter()
        .next()
        .ok_or_else(|| {
            error!("Contract not found");
            "Contract not found".to_string()
        })?;

    // Load contract history
    let contract_histories = load_contract_history(contract.id, db)
        .await
        .map_err(|error| {
            error!("Failed to load contract histories: {}", error);
            error
        })?;

    // Find contract history corresponding to the transaction amount
    if let Some(history) = contract_histories
        .iter()
        .find(|h| h.new_amount == transaction.amount && h.changed_at == transaction.date)
    {
        let history_before = contract_histories
            .iter()
            .filter(|h| h.changed_at < history.changed_at)
            .max_by_key(|h| h.changed_at);

        let history_after = contract_histories
            .iter()
            .filter(|h| h.changed_at > history.changed_at)
            .min_by_key(|h| h.changed_at);

        delete_contract_history_with_ids(vec![history.id], db)
            .await
            .map_err(|e| {
                error!("Failed to delete contract history: {}", e);
                e
            })?;

        match (history_before, history_after) {
            (Some(before), Some(_)) => {
                // Case 1: Both before and after history entries exist
                let mut updated_before = before.clone();
                updated_before.new_amount = history.new_amount;

                update_contract_history(updated_before, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract history: {}", e);
                        e
                    })?;
            }
            (None, Some(after)) => {
                // Case 2: Only after history entry exists
                let mut updated_after = after.clone();

                updated_after.old_amount = history.old_amount;

                update_contract_history(updated_after, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract history: {}", e);
                        e
                    })?;
            }
            (Some(before), None) => {
                // Case 3: Only before history entry exists
                update_contract_with_new_amount(contract.id, before.new_amount, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract: {}", e);
                        e
                    })?;
            }
            (None, None) => {
                // Case 4: No history entries exist
                update_contract_with_new_amount(contract.id, history.old_amount, db)
                    .await
                    .map_err(|e| {
                        error!("Failed to update contract: {}", e);
                        e
                    })?;
            }
        }
    }

    // Update the transaction to remove the contract association
    update_transactions_with_contract(vec![transaction_id], None::<i32>, db)
        .await
        .map_err(|e| {
            error!(
                "Error removing contract from transaction {}: {}",
                transaction_id, e
            );
            e
        })?;

    Ok("Contract removed".into())
}

pub async fn handel_update_amount(
    transaction_id: i32,
    contract_id: i32,
    mut db: Connection<DbConn>,
) -> Result<String, Json<ResponseData>> {
    let transaction = get_transaction(transaction_id, &mut db).await?;

    let contract = load_contracts_from_ids(vec![contract_id], &mut db).await;

    if let Err(error) = contract {
        error!("Error loading contract {}: {}", contract_id, error);
        return Err(Json(ResponseData::new_error(
            error,
            "There was an internal error while loading the contract. Please try again.",
        )));
    }

    let contract = contract.unwrap();

    if contract.is_empty() {
        info!("Contract {} not found", contract_id);
        return Err(Json(ResponseData::new_error(
            "Contract not found".to_string(),
            "The contract does not exist.",
        )));
    }

    assert!(contract.len() == 1);

    let mut contract = contract[0].clone();

    let contract_history = NewContractHistory {
        contract_id: contract.id,
        old_amount: contract.current_amount,
        new_amount: transaction.amount,
        changed_at: transaction.date,
    };

    let result = insert_contract_histories(&vec![contract_history], &mut db).await;

    if let Err(error) = result {
        error!("Error inserting contract history: {}", error);
        return Err(Json(ResponseData::new_error(
            error,
            "There was an internal error while inserting the contract history. Please try again.",
        )));
    }

    contract.current_amount = transaction.amount;

    let result = update_contract_with_new_amount(contract.id, transaction.amount, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error updating contract {} with new amount {}: {}",
            contract.id, transaction.amount, error
        );
        return Err(Json(ResponseData::new_error(error, "There was an internal error while updating the contract with the new amount. Please try again.")));
    }

    Ok("Contract updated".into())
}

pub async fn handle_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let transaction = get_transaction(transaction_id, &mut db).await;

    if let Err(error) = transaction {
        return error;
    }

    let transaction = transaction.unwrap();

    let contract = load_contracts_from_ids(vec![contract_id], &mut db).await;
    if let Err(error) = contract {
        error!("Error loading contract {}: {}", contract_id, error);
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while loading the contract. Please try again.",
        ));
    }

    let contract = contract.unwrap();

    assert!(contract.len() == 1);

    let contract = contract[0].clone();

    let contract_histories = load_contract_history(contract_id, &mut db).await;

    if let Err(error) = contract_histories {
        error!("Error loading contract history {}: {}", contract_id, error);
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while loading the contract history. Please try again.",
        ));
    }

    let contract_histories = contract_histories.unwrap();

    let history_before = contract_histories
        .iter()
        .filter(|h| h.changed_at < transaction.date)
        .max_by_key(|h| h.changed_at);

    let history_after = contract_histories
        .iter()
        .filter(|h| h.changed_at > transaction.date)
        .min_by_key(|h| h.changed_at);

    match (history_before, history_after) {
        (Some(before), Some(after)) => {
            // Case 1: Both before and after history entries exist
            let mut updated_after = after.clone();

            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: before.old_amount,
                new_amount: transaction.amount,
                changed_at: transaction.date,
            };

            updated_after.old_amount = history.new_amount;

            let result = update_contract_history(updated_after, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData::new_error(error, "There was an internal error while updating the contract history. Please try again."));
            }

            let result = insert_contract_histories(&vec![history], &mut db).await;

            if let Err(error) = result {
                return Json(ResponseData::new_error(
                    error,
                    "There was an internal error while updating the contract. Please try again.",
                ));
            }

            return Json(ResponseData::new_success(
                "Contract history updated".to_string(),
                "The contract history was updated.",
            ));
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

            let result = update_contract_history(updated_after, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData::new_error(error, "There was an internal error while updating the contract history. Please try again."));
            }

            let result = insert_contract_histories(&vec![history], &mut db).await;

            if let Err(error) = result {
                return Json(ResponseData::new_error(
                    error,
                    "There was an internal error while updating the contract. Please try again.",
                ));
            }

            return Json(ResponseData::new_success(
                "Contract history updated".to_string(),
                "The contract history was updated.",
            ));
        }
        (Some(before), None) => {
            // Case 3: Only before history entry exists
            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: before.new_amount,
                new_amount: contract.current_amount,
                changed_at: transaction.date,
            };

            let result = insert_contract_histories(&vec![history], &mut db).await;

            if let Err(error) = result {
                return Json(ResponseData::new_error(
                    error,
                    "There was an internal error while updating the contract. Please try again.",
                ));
            }

            return Json(ResponseData::new_success(
                "Contract history updated".to_string(),
                "The contract history was updated.",
            ));
        }
        (None, None) => {
            // Case 4: No history entries exist
            let history = NewContractHistory {
                contract_id: contract.id,
                old_amount: transaction.amount,
                new_amount: contract.current_amount,
                changed_at: transaction.date,
            };

            let result = insert_contract_histories(&vec![history], &mut db).await;

            if let Err(error) = result {
                return Json(ResponseData::new_error(
                    error,
                    "There was an internal error while updating the contract. Please try again.",
                ));
            }

            return Json(ResponseData::new_success(
                "Contract history updated".to_string(),
                "The contract history was updated.",
            ));
        }
    }
}
