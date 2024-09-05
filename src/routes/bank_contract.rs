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
use crate::utils::appstate::AppState;
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
) -> Result<Json<Value>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Json(json!(ResponseData::new_error(
            "No bank selected".to_string(),
            "There was an internal error while loading the bank. Please try again."
        ))));
    }

    let current_bank = current_bank.unwrap();

    let contract_history_string = get_contracts_with_history(current_bank.id, &mut db).await;

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
        result["contracts"] = json!(contract_history_string.unwrap());
    }

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ContractIds {
    pub ids: Vec<i32>,
}

#[post("/bank/contract/merge", format = "json", data = "<ids>")]
pub async fn bank_contract_merge(
    ids: Json<ContractIds>,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let time = std::time::SystemTime::now();
    let contract_id_for_loading = ids.ids.clone();

    let contracts = load_contracts_from_ids(contract_id_for_loading.clone(), &mut db).await;

    if let Err(error) = contracts {
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while loading the contracts.",
        ));
    }

    let contracts = contracts.unwrap();

    let mut all_closed = true;

    for contract in contracts.iter() {
        if contract.end_date.is_none() {
            all_closed = false;
            break;
        }
    }

    if all_closed {
        warn!("Time to load contracts: {:?}", time.elapsed().unwrap());
        return handle_all_closed_contracts(contracts, &mut db).await;
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
    handle_open_and_closed_contracts(open_contracts, closed_contracts, &mut db).await
}

#[post("/bank/contract/delete", format = "json", data = "<ids>")]
pub async fn bank_contract_delete(
    ids: Json<ContractIds>,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let contract_ids = ids.ids.clone();

    let contracts = load_contracts_from_ids(contract_ids.clone(), &mut db).await;

    if let Err(error) = contracts {
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while loading the contracts.",
        ));
    }

    let contracts = contracts.unwrap();

    let result = delete_contracts_with_ids(contract_ids, &mut db).await;

    if let Err(error) = result {
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while deleting the contracts.",
        ));
    }

    let mut success = String::new();

    for contract in contracts.iter() {
        success += &format!("Successfully deleted contract '{}'.\n", contract.name);
    }

    Json(ResponseData::new_success(
        "Deleted contracts".to_string(),
        &success,
    ))
}

#[get("/bank/contract/scan")]
pub async fn bank_scan_for_new_contracts(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Box<Redirect>> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Json(ResponseData::new_error(
            "No bank selected".to_string(),
            "There was an internal error while loading the bank. Please try again.",
        )));
    }

    let current_bank = current_bank.unwrap();

    let result = create_contract_from_transactions(current_bank.id, &mut db).await;

    if let Err(error) = result {
        return Ok(Json(ResponseData::new_error(
            error,
            "There was an internal error while scanning for new contracts.",
        )));
    }

    let result = result.unwrap();

    Ok(Json(ResponseData::new_success(
        "Scanned for new contracts".to_string(),
        &result,
    )))
}

#[get("/bank/contract/nameChanged/<id>/<name>")]
pub async fn bank_contract_name_changed(
    id: i32,
    name: &str,
    mut db: Connection<DbConn>,
) -> Json<ResponseData> {
    let result = update_contract_with_new_name(id, name.to_string(), &mut db).await;

    if let Err(error) = result {
        return Json(ResponseData::new_error(
            error,
            "There was an internal error while updating the contract name.",
        ));
    }

    Json(ResponseData::new_success(
        "Updated contract name".to_string(),
        "Successfully updated the contract name.",
    ))
}
