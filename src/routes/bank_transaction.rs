use log::{error, info, warn};
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::Instant;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::contract_utils::{handle_add_update, handle_remove_contract};
use crate::utils::get_utils::{get_transactions_with_contract, get_user_id};
use crate::utils::loading_utils::{load_contracts_from_ids, load_transaction_by_id};
use crate::utils::structs::ResponseData;
use crate::utils::structs::Transaction;
use crate::utils::update_utils::{
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

    let transaction = match get_transaction(transaction_id, &mut db).await {
        Ok(t) => t,
        Err(e) => return e,
    };

    if transaction.contract_not_allowed {
        info!(
            "Transaction {} is not allowed to have a contract",
            transaction_id
        );
        return Json(ResponseData {
            success: None,
            error: Some("The transaction is not allowed to have a contract. Please change the rule for it and try again".into()),
            header: Some("Transaction not allowed to have a contract".into()),
        });
    }

    let contract = match load_contracts_from_ids(vec![contract_id], &mut db).await {
        Ok(c) => c,
        Err(error) => {
            error!("Error loading contract {}: {}", contract_id, error);
            return Json(ResponseData {
                success: None,
                error: Some("There was an internal error while trying to load the contract. Please try again.".into()),
                header: Some(error),
            });
        }
    };

    assert!(contract.len() == 1);

    let contract = contract[0].clone();

    let result_of_contract_update = match handle_add_update(transaction, contract, &mut db).await {
        Ok(res) => res,
        Err(error) => {
            error!("Error updating contract: {}", error);
            return Json(ResponseData {
                success: None,
                error: Some("There was an internal error while trying to update the contract. Please try again.".into()),
                header: Some(error),
            });
        }
    };

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
        success: Some(result_of_contract_update),
        error: None,
        header: Some("Contract added".into()),
    })
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
