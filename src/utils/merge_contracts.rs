use log::warn;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::Connection;
use std::vec;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, ContractHistory, NewContractHistory};
use crate::utils::delete_utils::delete_contracts_with_ids;
use crate::utils::loading_utils::load_contract_history;
use crate::utils::structs::ResponseData;
use crate::utils::update_utils::update_transactions_of_contract_to_new_contract;

use super::appstate::AppState;
use super::insert_utiles::insert_contract_histories;
use super::loading_utils::load_last_transaction_data_of_contract;

pub async fn handle_all_closed_contracts(
    contracts: Vec<Contract>,
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Json<ResponseData> {
    let contracts_clone = contracts.clone();
    let contract_head = contracts_clone
        .iter()
        .min_by_key(|contract| contract.end_date)
        .unwrap();

    process_contracts(contract_head, contracts, cookie_user_id, state, db).await
}

pub async fn handle_open_and_closed_contracts(
    open_contracts: Vec<Contract>,
    closed_contracts: Vec<Contract>,
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Json<ResponseData> {
    let contract_head = get_contract_head(&open_contracts, cookie_user_id, state, db).await;

    if let Err(error) = contract_head {
        return error;
    }

    let contract_head = contract_head.unwrap();

    let mut combined_contracts = open_contracts.clone();
    combined_contracts.extend(closed_contracts.clone());

    process_contracts(contract_head, combined_contracts, cookie_user_id, state, db).await
}

async fn get_contract_head<'a>(
    contracts: &'a [Contract],
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Result<&'a Contract, Json<ResponseData>> {
    let mut last_transaction_datas = vec![];

    for contract in contracts.iter() {
        let last_transaction_data = load_last_transaction_data_of_contract(contract.id, db).await;

        if let Err(error) = last_transaction_data {
            return Err(Json(ResponseData::new_error(
                error,
                state
                    .localize_message(cookie_user_id, "error_loading_transactions")
                    .await,
            )));
        }

        last_transaction_datas.push(last_transaction_data.unwrap().unwrap());
    }

    let latest_transaction_data = last_transaction_datas
        .iter()
        .max_by_key(|data| data.date)
        .unwrap();

    let contract_head = contracts
        .iter()
        .find(|contract| {
            let last_transaction_data = last_transaction_datas
                .iter()
                .find(|data| data.contract_id.unwrap() == contract.id)
                .unwrap();

            last_transaction_data.date == latest_transaction_data.date
        })
        .unwrap();

    Ok(contract_head)
}

async fn process_contracts(
    contract_head: &Contract,
    contracts: Vec<Contract>,
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut Connection<DbConn>,
) -> Json<ResponseData> {
    let other_contracts = contracts
        .iter()
        .filter(|contract| contract.id != contract_head.id)
        .collect::<Vec<&Contract>>();

    let contract_head_id = contract_head.id;
    let other_contract_ids: Vec<i32> = other_contracts.iter().map(|contract| contract.id).collect();

    let mut time = std::time::SystemTime::now();
    let header_contract_history = load_contract_history(contract_head.bank_id, db).await;
    warn!(
        "Time to load header contract history: {:?}",
        time.elapsed().unwrap()
    );

    if let Err(error) = header_contract_history {
        return Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_loading_contract_history")
                .await,
        ));
    }

    let mut merged_histories = header_contract_history.unwrap();

    time = std::time::SystemTime::now();
    for contract in contracts.iter() {
        let contract_histories = load_contract_history(contract.bank_id, db).await;

        if let Err(error) = contract_histories {
            return Json(ResponseData::new_error(
                error,
                state
                    .localize_message(cookie_user_id, "error_loading_contract_history")
                    .await,
            ));
        }

        let mut histories = contract_histories.unwrap();

        let last_transaction_date = load_last_transaction_data_of_contract(contract.id, db)
            .await
            .unwrap()
            .unwrap()
            .date;

        if let Some(latest_history) = histories.last() {
            let changed_at = if let Some(end_date) = contract.end_date {
                end_date
            } else {
                last_transaction_date
            };

            histories.push(ContractHistory {
                id: 0,
                contract_id: contract.id,
                old_amount: latest_history.new_amount,
                new_amount: contract.current_amount,
                changed_at,
            });

            if contract.current_amount != contract_head.current_amount {
                merged_histories.push(ContractHistory {
                    id: 0,
                    contract_id: contract_head_id,
                    new_amount: contract_head.current_amount,
                    old_amount: contract.current_amount,
                    changed_at,
                });
            }
        } else if contract.current_amount != contract_head.current_amount {
            merged_histories.push(ContractHistory {
                id: 0,
                contract_id: contract_head_id,
                new_amount: contract_head.current_amount,
                old_amount: contract.current_amount,
                changed_at: last_transaction_date,
            });
        }

        merged_histories.extend(histories);
    }
    warn!(
        "Time to load all contract histories: {:?}",
        time.elapsed().unwrap()
    );

    merged_histories.sort_by_key(|h| h.changed_at);
    merged_histories.dedup_by_key(|h| h.new_amount);

    let mut histories_to_insert: Vec<NewContractHistory> = vec![];

    for history in merged_histories {
        histories_to_insert.push(NewContractHistory {
            contract_id: contract_head_id,
            old_amount: history.old_amount,
            new_amount: history.new_amount,
            changed_at: history.changed_at,
        });
    }

    time = std::time::SystemTime::now();
    let insert_result = insert_contract_histories(&histories_to_insert, db).await;
    warn!(
        "Time to insert contract histories: {:?}",
        time.elapsed().unwrap()
    );

    if let Err(error) = insert_result {
        return Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_inserting_contract_history_details")
                .await,
        ));
    }

    time = std::time::SystemTime::now();
    let result = update_transactions_of_contract_to_new_contract(
        contract_head_id,
        other_contract_ids.clone(),
        db,
    )
    .await;
    warn!("Time to update transactions: {:?}", time.elapsed().unwrap());

    if let Err(error) = result {
        return Json(ResponseData::new_error(
            error,
            state
                .localize_message(
                    cookie_user_id,
                    "error_updating_transaction_of_contract_details",
                )
                .await,
        ));
    }

    time = std::time::SystemTime::now();
    let delete_result = delete_contracts_with_ids(other_contract_ids, db).await;
    warn!("Time to delete contracts: {:?}", time.elapsed().unwrap());

    if let Err(error) = delete_result {
        return Json(ResponseData::new_error(
            error,
            state
                .localize_message(cookie_user_id, "error_deleting_contract")
                .await,
        ));
    }

    Json(ResponseData::new_success(
        state
            .localize_message(cookie_user_id, "contracts_merged")
            .await,
        state
            .localize_message(cookie_user_id, "contracts_merged_details")
            .await,
    ))
}
