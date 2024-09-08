use log::warn;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::{json, Json};
use rocket::{get, post, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;
use serde::Deserialize;
use serde_json::Value;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::{AppState, LOCALIZATION};
use crate::utils::create_contract::create_contract_from_transactions;
use crate::utils::delete_utils::delete_contracts_with_ids;
use crate::utils::get_utils::{
    get_contracts_with_history, get_user_id_and_language, get_user_language,
};
use crate::utils::loading_utils::load_contracts_from_ids;
use crate::utils::merge_contracts::{
    handle_all_closed_contracts, handle_open_and_closed_contracts,
};
use crate::utils::structs::{ErrorResponse, SuccessResponse};
use crate::utils::translation_utils::get_bank_contract_localized_strings;
use crate::utils::update_utils::update_contract_with_new_name;

#[get("/bank/contract")]
pub async fn bank_contract(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    let cookie_user_language = get_user_language(cookies);

    let localized_strings = get_bank_contract_localized_strings(cookie_user_language);

    Ok(Template::render(
        "bank_contract",
        json!({ "translations": localized_strings }),
    ))
}

#[get("/bank/contract/data")]
pub async fn bank_contact_display(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let contract_history_string =
        get_contracts_with_history(current_bank.id, cookie_user_language, &mut db).await?;

    let mut result = json!(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "contracts_loaded"),
        LOCALIZATION.get_localized_string(cookie_user_language, "contracts_loaded_details")
    ));
    result["contracts"] = serde_json::Value::String(contract_history_string);

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ContractIds {
    pub ids: Vec<i32>,
}

#[post("/bank/contract/merge", format = "json", data = "<ids>")]
pub async fn bank_contract_merge(
    ids: Json<ContractIds>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let time = std::time::SystemTime::now();
    let cookie_user_language = get_user_language(cookies);

    let contract_id_for_loading = ids.ids.clone();

    let contracts = load_contracts_from_ids(
        contract_id_for_loading.clone(),
        cookie_user_language,
        &mut db,
    )
    .await?;

    let mut all_closed = true;

    for contract in contracts.iter() {
        if contract.end_date.is_none() {
            all_closed = false;
            break;
        }
    }

    if all_closed {
        warn!("Time to load contracts: {:?}", time.elapsed().unwrap());
        return handle_all_closed_contracts(contracts, cookie_user_language, &mut db).await;
    }

    let mut open_contracts = Vec::new();
    let mut closed_contracts = Vec::new();

    for contract in contracts.iter() {
        if contract.end_date.is_none() {
            open_contracts.push(contract.clone());
        } else {
            closed_contracts.push(contract.clone());
        }
    }

    warn!("Time to load contracts: {:?}", time.elapsed().unwrap());
    handle_open_and_closed_contracts(
        open_contracts,
        closed_contracts,
        cookie_user_language,
        &mut db,
    )
    .await
}

#[post("/bank/contract/delete", format = "json", data = "<ids>")]
pub async fn bank_contract_delete(
    ids: Json<ContractIds>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let time = std::time::SystemTime::now();
    let cookie_user_language = get_user_language(cookies);

    let contract_ids = ids.ids.clone();

    let contracts =
        load_contracts_from_ids(contract_ids.clone(), cookie_user_language, &mut db).await?;

    let result =
        delete_contracts_with_ids(contract_ids.clone(), cookie_user_language, &mut db).await?;

    if result != contract_ids.len() {
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(cookie_user_language, "error_deleting_contract"),
            LOCALIZATION
                .get_localized_string(cookie_user_language, "error_deleting_contract_details"),
        )));
    }

    let delete_message =
        LOCALIZATION.get_localized_string(cookie_user_language, "message_deleted_contract");

    let mut success = String::new();

    for contract in contracts.iter() {
        success += &delete_message.clone().replace("{}", &contract.name);
    }

    warn!("Time to delete contracts: {:?}", time.elapsed().unwrap());
    Ok(Json(SuccessResponse::new(
        "Deleted contracts".to_string(),
        success,
    )))
}

#[get("/bank/contract/scan")]
pub async fn bank_scan_for_new_contracts(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let time = std::time::SystemTime::now();
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let current_bank = state
        .get_current_bank(cookie_user_id, cookie_user_language)
        .await?;

    let result =
        create_contract_from_transactions(current_bank.id, cookie_user_language, &mut db).await?;

    warn!(
        "Time to scan for new contracts: {:?}",
        time.elapsed().unwrap()
    );
    Ok(Json(SuccessResponse::new(
        "Scanned for new contracts".to_string(),
        result,
    )))
}

#[get("/bank/contract/nameChanged/<id>/<name>")]
pub async fn bank_contract_name_changed(
    id: i32,
    name: &str,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let time = std::time::SystemTime::now();
    let cookie_user_language = get_user_language(cookies);

    update_contract_with_new_name(id, name.to_string(), cookie_user_language, &mut db).await?;

    warn!(
        "Time to update contract name: {:?}",
        time.elapsed().unwrap()
    );

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "contract_name_updated"),
        LOCALIZATION.get_localized_string(cookie_user_language, "contract_name_updated_details"),
    )))
}
