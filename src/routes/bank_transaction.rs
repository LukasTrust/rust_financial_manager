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
use crate::utils::get_utils::{
    get_transactions_with_contract, get_user_id_and_language, get_user_language,
};
use crate::utils::loading_utils::load_transaction_by_id;
use crate::utils::structs::{ErrorResponse, SuccessResponse};
use crate::utils::translation_utils::get_transactions_localized_strings;
use crate::utils::update_utils::{
    update_transaction_with_contract_not_allowed, update_transaction_with_hidden,
    update_transactions_with_contract,
};

#[get("/bank/transaction")]
pub async fn bank_transaction(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let contract_history_string =
        get_transactions_with_contract(current_bank.id, cookie_user_language, db).await?;

    let mut result = json!(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "transactions_loaded"),
        LOCALIZATION.get_localized_string(cookie_user_language, "transactions_loaded_details")
    ));
    result["transactions"] = json!(contract_history_string);

    warn!(
        "Bank transaction handling completed in {:?}",
        start_time.elapsed()
    );

    let translation_string = get_transactions_localized_strings(cookie_user_language);
    result["translations"] = json!(translation_string);

    Ok(Template::render("bank_transaction", json!(result)))
}

#[get("/bank/transaction/remove_contract/<transaction_id>")]
pub async fn transaction_remove(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    let result = handle_remove_contract(transaction_id, cookie_user_language, &mut db).await;

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
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    update_transactions_with_contract(
        vec![transaction_id],
        Some(contract_id),
        cookie_user_language,
        &mut db,
    )
    .await?;

    warn!(
        "Transaction contract addition completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "transaction_added_to_contract"),
        LOCALIZATION.get_localized_string(
            cookie_user_language,
            "transaction_added_to_contract_details",
        ),
    )))
}

#[get("/bank/transaction/update_contract_amount/<transaction_id>/<contract_id>")]
pub async fn transaction_update_contract_amount(
    transaction_id: i32,
    contract_id: i32,
    cookies: &CookieJar<'_>,
    db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    let result = handel_update_amount(transaction_id, contract_id, cookie_user_language, db).await;

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
    db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    let result = handle_set_old_amount(transaction_id, contract_id, cookie_user_language, db).await;

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
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    update_transaction_with_hidden(transaction_id, true, cookie_user_language, &mut db).await?;

    warn!("Transaction hiding completed in {:?}", start_time.elapsed());
    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "transaction_hidden"),
        LOCALIZATION.get_localized_string(cookie_user_language, "transaction_hidden_details"),
    )))
}

#[get("/bank/transaction/show/<transaction_id>")]
pub async fn transaction_show(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    update_transaction_with_hidden(transaction_id, false, cookie_user_language, &mut db).await?;

    warn!(
        "Transaction showing completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "transaction_visible"),
        LOCALIZATION.get_localized_string(cookie_user_language, "transaction_visible_details"),
    )))
}

#[get("/bank/transaction/not_allow_contract/<transaction_id>")]
pub async fn transaction_not_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    let transaction = load_transaction_by_id(transaction_id, cookie_user_language, &mut db).await?;

    if transaction.contract_id.is_some() {
        info!("Transaction {} already has a contract", transaction_id);
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(
                cookie_user_language,
                "error_setting_transaction_to_contract_not_allowed_has_contract",
            ),
            LOCALIZATION.get_localized_string(
                cookie_user_language,
                "error_setting_transaction_to_contract_not_allowed_has_contract_details",
            ),
        )));
    }

    update_transaction_with_contract_not_allowed(
        transaction_id,
        true,
        cookie_user_language,
        &mut db,
    )
    .await?;

    warn!(
        "Transaction not allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(
            cookie_user_language,
            "transaction_set_to_contract_not_allowed",
        ),
        LOCALIZATION.get_localized_string(
            cookie_user_language,
            "transaction_set_to_contract_not_allowed_details",
        ),
    )))
}

#[get("/bank/transaction/allow_contract/<transaction_id>")]
pub async fn transaction_allow_contract(
    transaction_id: i32,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let start_time = Instant::now();
    let cookie_user_language = get_user_language(cookies);

    update_transaction_with_contract_not_allowed(
        transaction_id,
        false,
        cookie_user_language,
        &mut db,
    )
    .await?;

    warn!(
        "Transaction allow contract completed in {:?}",
        start_time.elapsed()
    );
    Ok(Json(SuccessResponse::new(
        LOCALIZATION
            .get_localized_string(cookie_user_language, "transaction_set_to_contract_allowed"),
        LOCALIZATION.get_localized_string(
            cookie_user_language,
            "transaction_set_to_contract_allowed_details",
        ),
    )))
}
