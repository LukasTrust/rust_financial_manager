use log::{info, warn};
use rocket::http::CookieJar;
use rocket::serde::json::{json, Json};
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use std::time::Instant;
use std::vec;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::{AppState, LOCALIZATION};
use crate::utils::contract_utils::{
    handel_update_amount, handle_remove_contract, handle_set_old_amount,
};
use crate::utils::get_utils::{get_transactions_with_contract, get_user_id};
use crate::utils::loading_utils::load_transaction_by_id;
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
) -> Result<Template, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let current_bank = state.get_current_bank(cookie_user_id).await?;

    let contract_history_string =
        get_transactions_with_contract(current_bank.id, language, db).await?;

    let mut result = json!(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "transactions_loaded"),
        LOCALIZATION.get_localized_string(language, "transactions_loaded_details")
    ));
    result["transactions"] = json!(contract_history_string);

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
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let result = handle_remove_contract(transaction_id, language, &mut db).await;

    warn!(
        "Transaction contract removal completed in {:?}",
        start_time.elapsed()
    );

    result
}

#[get("/bank/transaction/add_contract/<transaction_id>/<contract_id>")]
pub async fn transaction_add_to_contract(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    update_transactions_with_contract(vec![transaction_id], Some(contract_id), language, &mut db)
        .await?;

    warn!(
        "Transaction contract addition completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "transaction_added_to_contract"),
        LOCALIZATION.get_localized_string(language, "transaction_added_to_contract_details"),
    )))
}

#[get("/bank/transaction/update_contract_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_update_contract_amount(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let result = handel_update_amount(transaction_id, contract_id, language, db).await;

    warn!(
        "Transaction contract amount update completed in {:?}",
        start_time.elapsed()
    );
    result
}

#[get("/bank/transaction/set_old_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_set_old_amount(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let result = handle_set_old_amount(transaction_id, contract_id, language, db).await;

    warn!(
        "Transaction old amount set completed in {:?}",
        start_time.elapsed()
    );
    result
}

#[get("/bank/transaction/hide/<transaction_id>")]
pub async fn transaction_hide(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    update_transaction_with_hidden(transaction_id, true, language, &mut db).await?;

    warn!("Transaction hiding completed in {:?}", start_time.elapsed());
    Ok(Json(ResponseData::new_error(
        LOCALIZATION.get_localized_string(language, "transaction_hidden"),
        LOCALIZATION.get_localized_string(language, "transaction_hidden_details"),
    )))
}

#[get("/bank/transaction/show/<transaction_id>")]
pub async fn transaction_show(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    update_transaction_with_hidden(transaction_id, false, language, &mut db).await?;

    warn!(
        "Transaction showing completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "transaction_visible"),
        LOCALIZATION.get_localized_string(language, "transaction_visible_details"),
    )))
}

#[get("/bank/transaction/not_allow_contract/<transaction_id>")]
pub async fn transaction_not_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let transaction = load_transaction_by_id(transaction_id, language, &mut db).await?;

    if transaction.contract_id.is_some() {
        info!("Transaction {} already has a contract", transaction_id);
        return Ok(Json(ResponseData::new_error(
            LOCALIZATION.get_localized_string(
                language,
                "error_setting_transaction_to_contract_not_allowed_has_contract",
            ),
            LOCALIZATION.get_localized_string(
                language,
                "error_setting_transaction_to_contract_not_allowed_has_contract_details",
            ),
        )));
    }

    update_transaction_with_contract_not_allowed(transaction_id, true, language, &mut db).await?;

    warn!(
        "Transaction not allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "transaction_set_to_contract_not_allowed"),
        LOCALIZATION
            .get_localized_string(language, "transaction_set_to_contract_not_allowed_details"),
    )))
}

#[get("/bank/transaction/allow_contract/<transaction_id>")]
pub async fn transaction_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let start_time = Instant::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    update_transaction_with_contract_not_allowed(transaction_id, false, language, &mut db).await?;

    warn!(
        "Transaction allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "transaction_set_to_contract_allowed"),
        LOCALIZATION.get_localized_string(language, "transaction_set_to_contract_allowed_details"),
    )))
}
