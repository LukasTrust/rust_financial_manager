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
use crate::utils::appstate::AppState;
use crate::utils::contract_utils::{
    handel_update_amount, handle_remove_contract, handle_set_old_amount,
};
use crate::utils::get_utils::{get_transaction, get_transactions_with_contract, get_user_id};
use crate::utils::structs::ResponseData;
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
            json!(ResponseData::new_error(
                "Noe bank selected".to_string(),
                "There was an internal error while loading the bank. Please try again."
            )),
        ));
    }

    let current_bank = current_bank.unwrap();

    let contract_history_string = get_transactions_with_contract(current_bank.id, db).await;

    let mut result;

    if let Err(error) = contract_history_string {
        result = json!(ResponseData::new_error(
            error,
            &format!(
                "There was an internal error trying to load the contracts of '{}'.",
                current_bank.name
            ),
        ));
    } else {
        result = json!(ResponseData::new_success(
            "Contracts loaded".to_string(),
            "The contracts were successfully loaded.",
        ));
        result["transactions"] = json!(contract_history_string.unwrap());
    }

    warn!(
        "Bank transaction handling completed in {:?}",
        start_time.elapsed()
    );
    Ok(Template::render("bank_transaction", json!(result)))
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
        return Json(ResponseData::new_error(error, "There was an internal error while trying to remove the contract from the transactions. Please try again."));
    }

    warn!(
        "Transaction contract removal completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData::new_success(
        "Contract removed".to_string(),
        "The contract was removed from the transactions.",
    ))
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
        return Json(ResponseData::new_error(error, "There was an internal error while adding the contract to the transactions. Please try again."));
    }

    warn!(
        "Transaction contract addition completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData::new_success(
        "Contract added".to_string(),
        "The contract was added to the transaction.",
    ))
}

#[get("/bank/transaction/update_contract_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_update_contract_amount(
    transaction_id: i32,
    contract_id: i32,
    db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = handel_update_amount(transaction_id, contract_id, db).await;

    if let Err(error) = result {
        return error;
    }

    let result = result.unwrap();

    warn!(
        "Transaction contract amount update completed in {:?}",
        start_time.elapsed()
    );

    Json(ResponseData::new_success(
        result,
        "The contract amount was updated.",
    ))
}

#[get("/bank/transaction/set_old_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    db: Connection<DbConn>,
) -> Json<ResponseData> {
    let start_time = Instant::now();

    let result = handle_set_old_amount(transaction_id, contract_id, db).await;

    warn!(
        "Transaction old amount set completed in {:?}",
        start_time.elapsed()
    );

    result
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
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while trying to hide the transactions. Please try again.",
        ));
    }

    warn!("Transaction hiding completed in {:?}", start_time.elapsed());
    Json(ResponseData::new_error(
        "Transaction hidden".to_string(),
        "The transaction will now be hidden.",
    ))
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
        return Json(ResponseData::new_error(error, "There was an internal error while trying to unhide the transactions. Please try again."));
    }

    warn!(
        "Transaction showing completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData::new_success(
        "Transaction shown".to_string(),
        "The transaction will now be displayed.",
    ))
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
        return Json(ResponseData::new_error(
            "Transaction already has a contract".to_string(),
            "The transaction already has a contract. Please remove it and try again.",
        ));
    }

    let result = update_transaction_with_contract_not_allowed(transaction_id, true, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error setting transaction {} to no contract allowed: {}",
            transaction_id, error
        );
        return Json(ResponseData::new_error(error, "There was an internal error while trying to set the transaction to no contract allowed. Please try again."));
    }

    warn!(
        "Transaction not allow contract completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData::new_success(
        "Transaction set to no contract allowed".to_string(),
        "The transaction will now not be allowed to have a contract.",
    ))
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
        return Json(ResponseData::new_error(error, "There was an internal error while trying to set the transaction to contract allowed. Please try again."));
    }

    warn!(
        "Transaction allow contract completed in {:?}",
        start_time.elapsed()
    );
    Json(ResponseData::new_success(
        "Transaction set to contract allowed".to_string(),
        "The transaction will now be allowed to have a contract.",
    ))
}
