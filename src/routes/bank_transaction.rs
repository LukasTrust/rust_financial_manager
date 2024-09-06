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
                state
                    .localize_message(cookie_user_id, "no_bank_selected")
                    .await,
                state
                    .localize_message(cookie_user_id, "no_bank_selected_details")
                    .await,
            )),
        ));
    }

    let current_bank = current_bank.unwrap();

    let contract_history_string = get_transactions_with_contract(current_bank.id, db).await;

    let mut result;

    if let Err(error) = contract_history_string {
        result = json!(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_loading_transactions")
                .await
        ));
    } else {
        result = json!(ResponseData::new_success(
            state
                .localize_message(cookie_user_id, "transactions_loaded")
                .await,
            state
                .localize_message(cookie_user_id, "transactions_loaded_details")
                .await
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
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies).unwrap();

    let result = handle_remove_contract(transaction_id, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error removing contract from transaction {}: {}",
            transaction_id, error
        );
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_removing_transaction_from_contract")
                .await,
        )));
    }

    warn!(
        "Transaction contract removal completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "transaction_removed_from_contract")
            .await,
        state
            .localize_message(cookie_user_id, "transaction_removed_from_contract_details")
            .await,
    )))
}

#[get("/bank/transaction/add_contract/<transaction_id>/<contract_id>")]
pub async fn transaction_add_to_contract(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result =
        update_transactions_with_contract(vec![transaction_id], Some(contract_id), &mut db).await;

    if let Err(error) = result {
        error!(
            "Error adding contract {} to transaction {}: {}",
            contract_id, transaction_id, error
        );
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_adding_transaction_to_contract")
                .await,
        )));
    }

    warn!(
        "Transaction contract addition completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "transaction_added_to_contract")
            .await,
        state
            .localize_message(cookie_user_id, "transaction_added_to_contract_details")
            .await,
    )))
}

#[get("/bank/transaction/update_contract_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_update_contract_amount(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result = handel_update_amount(transaction_id, contract_id, cookie_user_id, state, db).await;

    if let Err(error) = result {
        return Ok(error);
    }

    let result = result.unwrap();

    warn!(
        "Transaction contract amount update completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        result,
        state
            .localize_message(
                cookie_user_id,
                "transaction_contract_amount_updated_details",
            )
            .await,
    )))
}

#[get("/bank/transaction/set_old_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result =
        handle_set_old_amount(transaction_id, contract_id, cookie_user_id, state, db).await;

    warn!(
        "Transaction old amount set completed in {:?}",
        start_time.elapsed()
    );
    Ok(result)
}

#[get("/bank/transaction/hide/<transaction_id>")]
pub async fn transaction_hide(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result = update_transaction_with_hidden(transaction_id, true, &mut db).await;

    if let Err(error) = result {
        error!("Error hiding transaction {}: {}", transaction_id, error);
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_hiding_transaction_details")
                .await,
        )));
    }

    warn!("Transaction hiding completed in {:?}", start_time.elapsed());
    Ok(Json(ResponseData::new_error(
        state
            .localize_message(cookie_user_id, "transaction_hidden")
            .await,
        state
            .localize_message(cookie_user_id, "transaction_hidden_details")
            .await,
    )))
}

#[get("/bank/transaction/show/<transaction_id>")]
pub async fn transaction_show(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result = update_transaction_with_hidden(transaction_id, false, &mut db).await;

    if let Err(error) = result {
        error!("Error showing transaction {}: {}", transaction_id, error);
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_showing_transaction_details")
                .await,
        )));
    }

    warn!(
        "Transaction showing completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "transaction_visible")
            .await,
        state
            .localize_message(cookie_user_id, "transaction_visible_details")
            .await,
    )))
}

#[get("/bank/transaction/not_allow_contract/<transaction_id>")]
pub async fn transaction_not_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let transaction = match get_transaction(transaction_id, cookie_user_id, state, &mut db).await {
        Ok(t) => t,
        Err(e) => return Ok(e),
    };

    if transaction.contract_id.is_some() {
        info!("Transaction {} already has a contract", transaction_id);
        return Ok(Json(ResponseData::new_error(
            state
                .localize_message(
                    cookie_user_id,
                    "error_setting_transaction_to_contract_not_allowed_has_contract",
                )
                .await,
            state
                .localize_message(
                    cookie_user_id,
                    "error_setting_transaction_to_contract_not_allowed_has_contract_details",
                )
                .await,
        )));
    }

    let result = update_transaction_with_contract_not_allowed(transaction_id, true, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error setting transaction {} to no contract allowed: {}",
            transaction_id, error
        );
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(
                    cookie_user_id,
                    "error_setting_transaction_to_contract_not_allowed_internal_error",
                )
                .await,
        )));
    }

    warn!(
        "Transaction not allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "transaction_set_to_contract_not_allowed")
            .await,
        state
            .localize_message(
                cookie_user_id,
                "transaction_set_to_contract_not_allowed_details",
            )
            .await,
    )))
}

#[get("/bank/transaction/allow_contract/<transaction_id>")]
pub async fn transaction_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;

    let result = update_transaction_with_contract_not_allowed(transaction_id, false, &mut db).await;

    if let Err(error) = result {
        error!(
            "Error setting transaction {} to allow contract: {}",
            transaction_id, error
        );
        return Ok(Json(ResponseData::new_error(
            error,
            state
                .localize_message(
                    cookie_user_id,
                    "error_setting_transaction_to_contract_allowed",
                )
                .await,
        )));
    }

    warn!(
        "Transaction allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "transaction_set_to_contract_allowed")
            .await,
        state
            .localize_message(
                cookie_user_id,
                "transaction_set_to_contract_allowed_details",
            )
            .await,
    )))
}
