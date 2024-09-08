use log::warn;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use std::vec;

use crate::database::db_connector::DbConn;
use crate::database::models::{Contract, ContractHistory, NewContractHistory};
use crate::utils::appstate::LOCALIZATION;
use crate::utils::delete_utils::delete_contracts_with_ids;
use crate::utils::loading_utils::load_contract_history;
use crate::utils::structs::{ErrorResponse, SuccessResponse};
use crate::utils::update_utils::update_transactions_of_contract_to_new_contract;

use super::appstate::Language;
use super::insert_utiles::insert_contract_histories;
use super::loading_utils::load_last_transaction_of_contract;

pub async fn handle_all_closed_contracts(
    contracts: Vec<Contract>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let contracts_clone = contracts.clone();
    let contract_head = contracts_clone
        .iter()
        .min_by_key(|contract| contract.end_date)
        .unwrap();

    process_contracts(contract_head, contracts, language, db).await
}

pub async fn handle_open_and_closed_contracts(
    open_contracts: Vec<Contract>,
    closed_contracts: Vec<Contract>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let contract_head = get_contract_head(&open_contracts, language, db).await?;

    let mut combined_contracts = open_contracts.clone();
    combined_contracts.extend(closed_contracts.clone());

    process_contracts(contract_head, combined_contracts, language, db).await
}

async fn get_contract_head<'a>(
    contracts: &'a [Contract],
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<&'a Contract, Json<ErrorResponse>> {
    let mut last_transaction_datas = vec![];

    for contract in contracts.iter() {
        let last_transaction = load_last_transaction_of_contract(contract.id, language, db).await?;

        last_transaction_datas.push(last_transaction);
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
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let other_contracts = contracts
        .iter()
        .filter(|contract| contract.id != contract_head.id)
        .collect::<Vec<&Contract>>();

    let contract_head_id = contract_head.id;
    let other_contract_ids: Vec<i32> = other_contracts.iter().map(|contract| contract.id).collect();

    let mut time = std::time::SystemTime::now();
    let mut merged_histories = load_contract_history(contract_head.bank_id, language, db).await?;
    warn!(
        "Time to load header contract history: {:?}",
        time.elapsed().unwrap()
    );

    time = std::time::SystemTime::now();
    for contract in contracts.iter() {
        let mut contract_histories = load_contract_history(contract.bank_id, language, db).await?;

        let last_transaction = load_last_transaction_of_contract(contract.id, language, db).await?;

        let last_transaction_date = last_transaction.date;

        if let Some(latest_history) = contract_histories.last() {
            let changed_at = if let Some(end_date) = contract.end_date {
                end_date
            } else {
                last_transaction_date
            };

            contract_histories.push(ContractHistory {
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

        merged_histories.extend(contract_histories);
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
    insert_contract_histories(&histories_to_insert, language, db).await?;
    warn!(
        "Time to insert contract histories: {:?}",
        time.elapsed().unwrap()
    );

    time = std::time::SystemTime::now();
    update_transactions_of_contract_to_new_contract(
        contract_head_id,
        other_contract_ids.clone(),
        language,
        db,
    )
    .await?;
    warn!("Time to update transactions: {:?}", time.elapsed().unwrap());

    time = std::time::SystemTime::now();
    delete_contracts_with_ids(other_contract_ids, language, db).await?;
    warn!("Time to delete contracts: {:?}", time.elapsed().unwrap());

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(language, "contracts_merged"),
        LOCALIZATION.get_localized_string(language, "contracts_merged_details"),
    )))
}
