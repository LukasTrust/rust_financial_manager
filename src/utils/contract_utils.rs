use log::error;
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::utils::delete_utils::delete_contract_history_with_ids;
use crate::utils::update_utils::{
    update_contract_history, update_contract_with_new_amount, update_transactions_with_contract,
};

use super::loading_utils::{
    load_contract_history, load_contracts_from_ids, load_transaction_by_id,
};

pub async fn handle_remove_contract(
    transaction_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<String, String> {
    // Load transaction by ID
    let transaction = load_transaction_by_id(transaction_id, db)
        .await
        .map_err(|e| {
            error!("Failed to load transaction: {}", e);
            e
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
    let contract_histories = load_contract_history(contract.id, db).await.map_err(|e| {
        error!("Failed to load contract histories: {}", e);
        e
    })?;

    // Find contract history corresponding to the transaction amount
    if let Some(history) = contract_histories
        .iter()
        .find(|h| h.new_amount == transaction.amount)
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
            (Some(_), None) => {
                // Case 3: Only before history entry exists
                update_contract_with_new_amount(contract.id, history.old_amount, db)
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
