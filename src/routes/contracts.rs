use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_banks_of_user, get_contracts, get_current_bank, get_user_id};
use crate::utils::loading_utils::load_contract_history;
use crate::utils::structs::{Bank, ContractWithHistory};

#[get("/contract")]
pub async fn contract(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = get_current_bank(cookie_user_id, state).await;

    match current_bank {
        Ok(current_bank) => {
            let result = get_contracts_with_history(vec![current_bank], state, db).await;

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

            Ok(Template::render(
                "contract",
                json!({"contracts": contract_string,
                       "error": error}),
            ))
        }
        Err(_) => {
            let banks = get_banks_of_user(cookie_user_id, state).await;

            let result = get_contracts_with_history(banks, state, db).await;

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

            Ok(Template::render(
                "contract",
                json!({"contracts": contract_string,
                       "error": error}),
            ))
        }
    }
}

async fn get_contracts_with_history(
    banks: Vec<Bank>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<String, String> {
    let mut contracts_with_history: Vec<ContractWithHistory> = Vec::new();

    for bank in banks {
        let contracts = get_contracts(bank.id, state).await;

        match contracts {
            Ok(contracts) => {
                for contract in contracts.iter() {
                    let contract_history = load_contract_history(contract.id, &mut db).await;

                    if contract_history.is_err() {
                        return Err("Error loading contract history.".to_string());
                    }

                    let contract_with_history = ContractWithHistory {
                        contract: contract.clone(),
                        contract_history: contract_history.unwrap(),
                    };

                    contracts_with_history.push(contract_with_history);
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(serde_json::to_string(&contracts_with_history).unwrap())
}
