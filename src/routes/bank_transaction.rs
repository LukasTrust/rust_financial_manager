use log::{error, info, warn};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::Instant;
use std::vec;

use crate::database::db_connector::DbConn;
use crate::database::models::NewContractHistory;
use crate::utils::appstate::AppState;
use crate::utils::contract_utils::handle_remove_contract;
use crate::utils::get_utils::{get_transactions_with_contract, get_user_id};
use crate::utils::loading_utils::{
    load_contract_history, load_contracts_from_ids, load_transaction_by_id,
};
use crate::utils::structs::ResponseData;
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{
    update_contract_history, update_contract_with_new_amount,
    update_transaction_with_contract_not_allowed, update_transaction_with_hidden,
    update_transactions_with_contract,
};

#[get("/bank/transaction")]
pub async fn bank_transaction(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Box<Redirect>> {
    let start_time = Instant::now();

    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        warn!(
            "Current bank not found, returning default template. Took {:?}",
            start_time.elapsed()
        );
        return Ok(Template::render(
            "bank_transaction",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into(),
                ),
                header: Some("No bank selected".into()),
            }),
        ));
    }

    let current_bank = current_bank.unwrap();

    let result = get_transactions_with_contract(current_bank.id, db).await;

    let error = if result.is_err() {
        Some(result.clone().err().unwrap())
    } else {
        None
    };

    let transaction_string = if let Ok(transactions) = result {
        transactions
    } else {
        String::new()
    };

    let mut response_data = json!(ResponseData {
        success: None,
        error: Some(format!(
            "There was an internal error trying to load the transactions of '{}'.",
            current_bank.name
        )),
        header: error,
    });

    response_data["transactions"] = json!(transaction_string);

    warn!(
        "Bank transaction handling completed in {:?}",
        start_time.elapsed()
    );
    Ok(Template::render("bank_transaction", json!(response_data)))
}

#[get("/bank/transaction/remove_contract/<transaction_id>")]
pub async fn transaction_remove(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = handle_remove_contract(transaction_id, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error removing contract from transaction {}: {}",
            transaction_id, error
        );
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to remove the contract from the transactions. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction contract removal completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData {
        success: Some("The contract was removed from the transactions.".into()),
        error: None,
        header: Some("Contract removed".into()),
    })
}

#[get("/bank/transaction/add_contract/<transaction_id>/<contract_id>")]
pub async fn transaction_add_to_contract(
    transaction_id: i32,
    contract_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result =
        update_transactions_with_contract(vec![transaction_id], Some(contract_id), &mut db).await;

    if let Err(error) = result {
        error!(
            "Error adding contract {} to transaction {}: {}",
            contract_id, transaction_id, error
        );
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while adding the contract to the transactions. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction contract addition completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData {
        success: Some("The contract was added to the transaction.".into()),
        error: None,
        header: Some("Contract added".into()),
    })
}

#[get("/bank/transaction/update_contract_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_update_contract_amount(
    transaction_id: i32,
    contract_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let transaction = get_transaction(transaction_id, &mut db).await;

    if transaction.is_err() {
        return transaction.err().unwrap();
    }

    let transaction = transaction.unwrap();

    let contract = load_contracts_from_ids(vec![contract_id], &mut db).await;

    if let Err(error) = contract {
        error!("Error loading contract {}: {}", contract_id, error);
        return Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while loading the contract. Please try again.".into(),
            ),
            header: Some(error),
        });
    }

    let contract = contract.unwrap();

    if contract.is_empty() {
        info!("Contract {} not found", contract_id);
        return Json(ResponseData {
            success: None,
            error: Some("The contract does not exist.".into()),
            header: Some("Contract not found".into()),
        });
    }

    assert!(contract.len() == 1);

    let mut contract = contract[0].clone();

    contract.current_amount = transaction.amount;

    let result = update_contract_with_new_amount(contract.id, transaction.amount, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error updating contract {} with new amount {}: {}",
            contract.id, transaction.amount, error
        );
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while updating the contract with the new amount. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction contract amount update completed in {:?}",
        start_time.elapsed()
    );

    Json(ResponseData {
        success: Some("The contract amount was updated.".into()),
        error: None,
        header: Some("Contract amount updated".into()),
    })
}

#[get("/bank/transaction/set_old_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let transaction = get_transaction(transaction_id, &mut db).await;

    if transaction.is_err() {
        return transaction.err().unwrap();
    }

    let transaction = transaction.unwrap();

    let contract_histories = load_contract_history(contract_id, &mut db).await;

    if let Err(error) = contract_histories {
        error!("Error loading contract history {}: {}", contract_id, error);
        return Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while loading the contract history. Please try again."
                    .into(),
            ),
            header: Some(error),
        });
    }

    let contract_histories = contract_histories.unwrap();

    let contract = load_contracts_from_ids(vec![contract_id], &mut db).await;
    if let Err(error) = contract {
        error!("Error loading contract {}: {}", contract_id, error);
        return Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while loading the contract. Please try again.".into(),
            ),
            header: Some(error),
        });
    }

    let contract = contract.unwrap();

    assert!(contract.len() == 1);

    let contract = contract[0].clone();

    let history = NewContractHistory {
        contract_id,
        old_amount: transaction.amount,
        new_amount: contract.current_amount,
        changed_at: transaction.date,
    };

    let history_before = contract_histories
        .iter()
        .filter(|h| h.changed_at < history.changed_at)
        .max_by_key(|h| h.changed_at);

    let history_after = contract_histories
        .iter()
        .filter(|h| h.changed_at > history.changed_at)
        .min_by_key(|h| h.changed_at);

    match (history_before, history_after) {
        (Some(before), Some(_)) => {
            // Case 1: Both before and after history entries exist
            let mut updated_before = before.clone();
            updated_before.new_amount = history.new_amount;

            let result = update_contract_history(updated_before, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData {
                    success: None,
                    error: Some("There was an internal error while updating the contract history. Please try again.".into()),
                    header: Some(error),
                });
            }

            warn!(
                "Transaction old amount set completed in {:?}",
                start_time.elapsed()
            );

            return Json(ResponseData {
                success: Some("The contract history was updated.".into()),
                error: None,
                header: Some("Contract history updated".into()),
            });
        }
        (None, Some(after)) => {
            // Case 2: Only after history entry exists
            let mut updated_after = after.clone();

            updated_after.old_amount = history.old_amount;

            let result = update_contract_history(updated_after, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract history: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData {
                    success: None,
                    error: Some("There was an internal error while updating the contract history. Please try again.".into()),
                    header: Some(error),
                });
            }

            warn!(
                "Transaction old amount set completed in {:?}",
                start_time.elapsed()
            );

            return Json(ResponseData {
                success: Some("The contract history was updated.".into()),
                error: None,
                header: Some("Contract history updated".into()),
            });
        }
        (Some(_), None) => {
            // Case 3: Only before history entry exists
            let result = update_contract_with_new_amount(contract.id, history.old_amount, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData {
                    success: None,
                    error: Some("There was an internal error while updating the contract. Please try again.".into()),
                    header: Some(error),
                });
            }

            warn!(
                "Transaction old amount set completed in {:?}",
                start_time.elapsed()
            );

            return Json(ResponseData {
                success: Some("The contract history was updated.".into()),
                error: None,
                header: Some("Contract history updated".into()),
            });
        }
        (None, None) => {
            // Case 4: No history entries exist
            let result = update_contract_with_new_amount(contract.id, history.old_amount, &mut db)
                .await
                .map_err(|e| {
                    error!("Failed to update contract: {}", e);
                    e
                });

            if let Err(error) = result {
                return Json(ResponseData {
                    success: None,
                    error: Some("There was an internal error while updating the contract. Please try again.".into()),
                    header: Some(error),
                });
            }

            warn!(
                "Transaction old amount set completed in {:?}",
                start_time.elapsed()
            );

            return Json(ResponseData {
                success: Some("The contract history was updated.".into()),
                error: None,
                header: Some("Contract history updated".into()),
            });
        }
    }
}

#[get("/bank/transaction/hide/<transaction_id>")]
pub async fn transaction_hide(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = update_transaction_with_hidden(transaction_id, true, &mut db).await;

    if let Err(error) = result {
        error!("Error hiding transaction {}: {}", transaction_id, error);
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to hide the transactions. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!("Transaction hiding completed in {:?}", start_time.elapsed());
    Json(ResponseData {
        success: Some("The transaction will now be hidden.".into()),
        error: None,
        header: Some("Transaction hidden".into()),
    })
}

#[get("/bank/transaction/show/<transaction_id>")]
pub async fn transaction_show(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = update_transaction_with_hidden(transaction_id, false, &mut db).await;

    if let Err(error) = result {
        error!("Error showing transaction {}: {}", transaction_id, error);
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to unhide the transactions. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction showing completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData {
        success: Some("The transaction will now be displayed.".into()),
        error: None,
        header: Some("Transaction shown".into()),
    })
}

#[get("/bank/transaction/not_allow_contract/<transaction_id>")]
pub async fn transaction_not_allow_contract(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let transaction = match get_transaction(transaction_id, &mut db).await {
        Ok(t) => t,
        Err(e) => return e,
    };

    if transaction.contract_id.is_some() {
        info!("Transaction {} already has a contract", transaction_id);
        return Json(ResponseData {
            success: None,
            error: Some(
                "The transaction already has a contract. Please remove it and try again.".into(),
            ),
            header: Some("Transaction already has a contract".into()),
        });
    }

    let result = update_transaction_with_contract_not_allowed(transaction_id, true, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error setting transaction {} to no contract allowed: {}",
            transaction_id, error
        );
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to set the transaction to no contract allowed. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction not allow contract completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData {
        success: Some("The transaction will now not be allowed to have a contract.".into()),
        error: None,
        header: Some("Transaction set to no contract allowed".into()),
    })
}

#[get("/bank/transaction/allow_contract/<transaction_id>")]
pub async fn transaction_allow_contract(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = update_transaction_with_contract_not_allowed(transaction_id, false, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error setting transaction {} to allow contract: {}",
            transaction_id, error
        );
        return Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to set the transaction to contract allowed. Please try again.".into()),
            header: Some(error),
        });
    }

    warn!(
        "Transaction allow contract completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData {
        success: Some("The transaction will now be allowed to have a contract.".into()),
        error: None,
        header: Some("Transaction set to contract allowed".into()),
    })
}

async fn get_transaction(
    transaction_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<Transaction, Json<ResponseData>> {
    let start_time = Instant::now();

    let transaction = load_transaction_by_id(transaction_id, db).await;

    if let Err(error) = transaction {
        error!("Error loading transaction {}: {}", transaction_id, error);
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while trying to load the transaction. Please try again.".into()),
            header: Some(error),
        }));
    }

    let transaction = transaction.unwrap();

    if transaction.is_none() {
        info!("Transaction {} not found", transaction_id);
        return Err(Json(ResponseData {
            success: None,
            error: Some("The transaction does not exist.".into()),
            header: Some("Transaction not found".into()),
        }));
    }

    warn!("Transaction loaded in {:?}", start_time.elapsed());
    Ok(transaction.unwrap())
}
