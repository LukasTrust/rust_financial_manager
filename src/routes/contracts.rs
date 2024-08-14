use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_contracts, get_current_bank, get_user_id};
use crate::utils::loading_utils::load_contract_history;
use crate::utils::structs::ContractWithHistory;

#[get("/contract")]
pub async fn contract(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = get_current_bank(cookie_user_id, state).await;

    match current_bank {
        Ok(current_bank) => {
            let contracts = get_contracts(current_bank.id, state).await;

            let mut contracts_with_history: Vec<ContractWithHistory> = Vec::new();

            match contracts {
                Ok(contracts) => {
                    for contract in contracts.iter() {
                        let contract_history = load_contract_history(contract.id, &mut db).await?;

                        let contract_with_history = ContractWithHistory {
                            contract: contract.clone(),
                            contract_history,
                        };

                        contracts_with_history.push(contract_with_history);
                    }

                    return Ok(Template::render(
                        "contract",
                        json!({"contracts": contracts_with_history}),
                    ));
                }
                Err(err) => return Ok(Template::render("contract", json!({ "error": err }))),
            }
        }
        Err(err) => return Ok(Template::render("contract", json!({ "error": err }))),
    }
}
