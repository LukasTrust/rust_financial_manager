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
use crate::utils::get_utils::{get_contracts_with_history, get_user_id};
use crate::utils::loading_utils::load_contracts_from_ids;
use crate::utils::merge_contracts::{
    handle_all_closed_contracts, handle_open_and_closed_contracts,
};
use crate::utils::structs::ResponseData;
use crate::utils::update_utils::update_contract_with_new_name;

#[get("/bank/contract")]
pub async fn bank_contract() -> Result<Template, Redirect> {
    Ok(Template::render("bank_contract", json!({})))
}

#[get("/bank/contract/data")]
pub async fn bank_contact_display(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Json<ResponseData>> {
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let current_bank = state.get_current_bank(cookie_user_id).await?;

    let contract_history_string =
        get_contracts_with_history(current_bank.id, language, &mut db).await?;

    let mut result = json!(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "contracts_loaded"),
        LOCALIZATION.get_localized_string(language, "contracts_loaded_details")
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
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let time = std::time::SystemTime::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let contract_id_for_loading = ids.ids.clone();

    let contracts =
        load_contracts_from_ids(contract_id_for_loading.clone(), language, &mut db).await?;

    let mut all_closed = true;

    for contract in contracts.iter() {
        if contract.end_date.is_none() {
            all_closed = false;
            break;
        }
    }

    if all_closed {
        warn!("Time to load contracts: {:?}", time.elapsed().unwrap());
        return Ok(handle_all_closed_contracts(contracts, language, &mut db).await);
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
    Ok(handle_open_and_closed_contracts(open_contracts, closed_contracts, language, &mut db).await)
}

#[post("/bank/contract/delete", format = "json", data = "<ids>")]
pub async fn bank_contract_delete(
    ids: Json<ContractIds>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let time = std::time::SystemTime::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;
    let contract_ids = ids.ids.clone();

    let contracts = load_contracts_from_ids(contract_ids.clone(), language, &mut db).await?;

    let result = delete_contracts_with_ids(contract_ids.clone(), language, &mut db).await?;

    if result != contract_ids.len() {
        return Ok(Json(ResponseData::new_error(
            LOCALIZATION.get_localized_string(language, "error_deleting_contract"),
            LOCALIZATION.get_localized_string(language, "error_deleting_contract_details"),
        )));
    }

    let delete_message = LOCALIZATION.get_localized_string(language, "message_deleted_contract");

    let mut success = String::new();

    for contract in contracts.iter() {
        success += &delete_message.clone().replace("{}", &contract.name);
    }

    warn!("Time to delete contracts: {:?}", time.elapsed().unwrap());
    Ok(Json(ResponseData::new_success(
        "Deleted contracts".to_string(),
        success,
    )))
}

#[get("/bank/contract/scan")]
pub async fn bank_scan_for_new_contracts(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let time = std::time::SystemTime::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    let current_bank = state.get_current_bank(cookie_user_id).await?;

    let result = create_contract_from_transactions(current_bank.id, language, &mut db).await?;

    warn!(
        "Time to scan for new contracts: {:?}",
        time.elapsed().unwrap()
    );
    Ok(Json(ResponseData::new_success(
        "Scanned for new contracts".to_string(),
        result,
    )))
}

#[get("/bank/contract/nameChanged/<id>/<name>")]
pub async fn bank_contract_name_changed(
    id: i32,
    name: &str,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let time = std::time::SystemTime::now();
    let cookie_user_id = get_user_id(cookies)?;
    let language = state.get_user_language(cookie_user_id).await;

    update_contract_with_new_name(id, name.to_string(), language, &mut db).await?;

    warn!(
        "Time to update contract name: {:?}",
        time.elapsed().unwrap()
    );

    Ok(Json(ResponseData::new_success(
        LOCALIZATION.get_localized_string(language, "contract_name_updated"),
        LOCALIZATION.get_localized_string(language, "contract_name_updated_details"),
    )))
}
