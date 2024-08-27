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
use crate::utils::get_utils::{get_contracts_with_history, get_user_id};
use crate::utils::loading_utils::load_contracts_from_ids;
use crate::utils::merge_contracts::{
    handle_all_closed_contracts, handle_open_and_closed_contracts,
};
use crate::utils::structs::ResponseData;

#[get("/bank/contract")]
pub async fn bank_contract(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Template::render(
            "bank_contract",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into()
                ),
                header: Some("No bank selected".into()),
            }),
        ));
    }

    let current_bank = current_bank.unwrap();

    let result = get_contracts_with_history(current_bank.id, &mut db).await;

    let error = if result.is_err() {
        Some(result.clone().err().unwrap())
    } else {
        None
    };

    let contract_string = if result.is_ok() {
        result.unwrap()
    } else {
        String::new()
    };

    let mut result = json!(ResponseData {
        success: None,
        error: if error.is_none() {
            None
        } else {
            Some(format!(
                "There was an internal error trying to load the contracts of '{}'.",
                current_bank.name
            ))
        },
        header: error,
    });

    result["contracts"] = json!(contract_string);

    Ok(Template::render("bank_contract", json!(result)))
}

#[derive(Deserialize)]
pub struct ContractIds {
    pub ids: Vec<i32>,
}

#[post("/bank/contract/merge", format = "json", data = "<ids>")]
pub async fn bank_contract_merge(
    ids: Json<ContractIds>,
    mut db: Connection<DbConn>,
) -> Result<Json<Value>, Json<ResponseData>> {
    let time = std::time::SystemTime::now();
    let contract_id_for_loading = ids.ids.clone();

    let contracts = load_contracts_from_ids(contract_id_for_loading.clone(), &mut db).await;

    if let Err(error) = contracts {
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while loading the contracts.".into()),
            header: Some(error),
        }));
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
    return handle_open_and_closed_contracts(open_contracts, closed_contracts, &mut db).await;
}
