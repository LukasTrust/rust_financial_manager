use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_total_amount_paid_of_contract, get_user_id};
use crate::utils::loading_utils::{load_banks, load_contract_history, load_contracts_of_bank};
use crate::utils::structs::{Bank, ContractWithHistory};

#[get("/contract")]
pub async fn contract(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    match current_bank {
        Some(current_bank) => {
            let result = get_contracts_with_history(vec![current_bank], db).await;

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
        None => {
            let banks = load_banks(cookie_user_id, &mut db).await;

            if let Err(e) = banks {
                return Ok(Template::render("contract", json!({ "error": e })));
            }

            let banks = banks.unwrap();

            let result = get_contracts_with_history(banks, db).await;

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
    mut db: Connection<DbConn>,
) -> Result<String, String> {
    let mut contracts_with_history: Vec<ContractWithHistory> = Vec::new();

    for bank in banks {
        let contracts = load_contracts_of_bank(bank.id, &mut db).await;

        match contracts {
            Ok(contracts) => {
                for contract in contracts.iter() {
                    let contract_history = load_contract_history(contract.id, &mut db).await;

                    if contract_history.is_err() {
                        return Err("Error loading contract history.".to_string());
                    }

                    let total_amount_paid =
                        get_total_amount_paid_of_contract(contract.id, &mut db).await?;

                    let contract_with_history = ContractWithHistory {
                        contract: contract.clone(),
                        contract_history: contract_history.unwrap(),
                        total_amount_paid,
                    };

                    contracts_with_history.push(contract_with_history);
                }
            }
            Err(err) => return Err(err),
        }
    }

    Ok(serde_json::to_string(&contracts_with_history).unwrap())
}
