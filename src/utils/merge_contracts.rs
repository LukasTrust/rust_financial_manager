use chrono::NaiveDate;
use log::warn;
use rocket::serde::json::{json, Json};
use rocket_db_pools::Connection;
use serde_json::Value;
use std::vec;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, ContractHistory, NewContractHistory};
use crate::utils::delete_utils::delete_contracts_with_ids;
use crate::utils::loading_utils::load_contract_history;
use crate::utils::structs::ResponseData;
use crate::utils::update_utils::update_transactions_of_contract_to_new_contract;

use super::get_utils::get_contracts_with_history;
use super::insert_utiles::insert_contract_histories;
use super::loading_utils::load_last_transaction_data_of_contract;

pub async fn handle_all_closed_contracts(
    contracts: Vec<Contract>,
    db: &mut Connection<DbConn>,
) -> Result<Json<Value>, Json<ResponseData>> {
    let contracts_clone = contracts.clone();
    let contract_head = contracts_clone
        .iter()
        .min_by_key(|contract| contract.end_date)
        .unwrap();

    let result = process_contracts(contract_head, contracts, db).await?;

    Ok(Json(result))
}

pub async fn handle_open_and_closed_contracts(
    open_contracts: Vec<Contract>,
    closed_contracts: Vec<Contract>,
    db: &mut Connection<DbConn>,
) -> Result<Json<Value>, Json<ResponseData>> {
    let contract_head = get_contract_head(&open_contracts, db).await?;

    let mut combined_contracts = open_contracts.clone();
    combined_contracts.extend(closed_contracts.clone());

    let result = process_contracts(contract_head, combined_contracts, db).await?;

    Ok(Json(result))
}

async fn get_contract_head<'a>(
    contracts: &'a Vec<Contract>,
    db: &mut Connection<DbConn>,
) -> Result<&'a Contract, Json<ResponseData>> {
    let mut last_transaction_datas = vec![];

    for contract in contracts.iter() {
        let last_transaction_data = load_last_transaction_data_of_contract(contract.id, db).await;

        if let Err(error) = last_transaction_data {
            return Err(Json(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the transaction data.".into(),
                ),
                header: Some(error),
            }));
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
    db: &mut Connection<DbConn>,
) -> Result<Value, Json<ResponseData>> {
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
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while loading the contract history.".into()),
            header: Some(error),
        }));
    }

    let mut merged_histories = header_contract_history.unwrap();

    time = std::time::SystemTime::now();
    for contract in contracts.iter() {
        let contract_histories = load_contract_history(contract.bank_id, db).await;

        if let Err(error) = contract_histories {
            return Err(Json(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the contract history.".into(),
                ),
                header: Some(error),
            }));
        }

        let mut histories = contract_histories.unwrap();

        if let Some(latest_history) = histories.last() {
            if latest_history.new_amount != contract.current_amount {
                histories.push(ContractHistory {
                    id: 0,
                    contract_id: contract.id,
                    old_amount: latest_history.new_amount,
                    new_amount: contract.current_amount,
                    changed_at: contract
                        .end_date
                        .unwrap_or_else(|| NaiveDate::from_ymd_opt(9999, 12, 31).unwrap()),
                });
            }
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
        return Err(Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while inserting the contract histories.".into(),
            ),
            header: Some(error),
        }));
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
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while updating the transactions.".into()),
            header: Some(error),
        }));
    }

    time = std::time::SystemTime::now();
    let delete_result = delete_contracts_with_ids(other_contract_ids, db).await;
    warn!("Time to delete contracts: {:?}", time.elapsed().unwrap());

    if let Err(error) = delete_result {
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while deleting the contracts.".into()),
            header: Some(error),
        }));
    }

    time = std::time::SystemTime::now();
    let result = get_contracts_with_history(contract_head.bank_id, db).await;
    warn!("Time to load contracts: {:?}", time.elapsed().unwrap());

    if let Err(error) = result {
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while loading the contracts.".into()),
            header: Some(error),
        }));
    }

    let contract_string = result.unwrap();

    Ok(json!(contract_string))
}
