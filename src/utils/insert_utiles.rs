use diesel::result::Error as DieselError;
use log::{error, info};
use rocket::serde::json::Json;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use super::appstate::Language;
use super::structs::{Bank, Transaction};
use crate::database::models::{
    CSVConverter, Contract, ContractHistory, NewCSVConverter, NewContract, NewContractHistory,
    NewTransaction, NewUser,
};
use crate::database::{db_connector::DbConn, models::NewBank};
use crate::utils::appstate::LOCALIZATION;
use crate::utils::structs::ResponseData;

pub async fn insert_user(
    new_user: NewUser,
    db: &mut Connection<DbConn>,
) -> Result<usize, DieselError> {
    use crate::schema::users;

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(db)
        .await
}

pub async fn insert_bank(
    new_bank: NewBank,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Bank, Json<ResponseData>> {
    use crate::schema::banks;

    diesel::insert_into(banks::table)
        .values(&new_bank)
        .get_result::<Bank>(db)
        .await
        .map_err(|e| {
            error!("Error inserting bank: {:?}", e);
            // Handle the specific error kind for a unique violation
            if let DieselError::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) = e
            {
                return Json(ResponseData::new_error(
                    LOCALIZATION.get_localized_string(language, "error_inserting_bank"),
                    LOCALIZATION.get_localized_string(
                        language,
                        "error_inserting_bank_details_already_exists",
                    ),
                ));
            }
            // Handle any other kind of error
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_inserting_bank"),
                LOCALIZATION.get_localized_string(language, "error_inserting_bank_details"),
            ))
        })
}

pub async fn insert_csv_converter(
    new_csv_converter: NewCSVConverter,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<CSVConverter, Json<ResponseData>> {
    use crate::schema::csv_converters;

    diesel::insert_into(csv_converters::table)
        .values(&new_csv_converter)
        .get_result::<CSVConverter>(db)
        .await
        .map_err(|e| {
            error!("Error inserting csv converter: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_inserting_csv"),
                LOCALIZATION.get_localized_string(language, "error_inserting_csv_details"),
            ))
        })
}

pub async fn insert_contracts(
    new_contracts: &Vec<NewContract>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, Json<ResponseData>> {
    use crate::schema::contracts;

    diesel::insert_into(contracts::table)
        .values(new_contracts)
        .get_results::<Contract>(db)
        .await
        .map_err(|e| {
            error!("Error inserting contracts: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_inserting_contract"),
                LOCALIZATION.get_localized_string(language, "error_inserting_contract_details"),
            ))
        })
}

pub async fn insert_contract_histories(
    new_contract_histories: &Vec<NewContractHistory>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<ContractHistory>, Json<ResponseData>> {
    use crate::schema::contract_history;

    diesel::insert_into(contract_history::table)
        .values(new_contract_histories)
        .get_results::<ContractHistory>(db)
        .await
        .map_err(|e| {
            error!("Error inserting contract histories: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_inserting_contract_histories"),
                LOCALIZATION
                    .get_localized_string(language, "error_inserting_contract_histories_details"),
            ))
        })
}

pub async fn insert_transactions(
    mut new_transactions: Vec<NewTransaction>,
    existing_transactions: Vec<Transaction>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<(usize, usize), Json<ResponseData>> {
    use crate::schema::transactions;

    info!(
        "New transactions before filtering: {:?}",
        new_transactions.len()
    );

    for transaction in &existing_transactions {
        new_transactions.retain(|new_transaction| {
            new_transaction.date != transaction.date
                || new_transaction.counterparty != transaction.counterparty
                || new_transaction.amount != transaction.amount
                || new_transaction.bank_balance_after != transaction.bank_balance_after
        });
    }

    info!(
        "New transactions after filtering: {:?}",
        new_transactions.len()
    );

    diesel::insert_into(transactions::table)
        .values(&new_transactions)
        .get_results::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error inserting transactions: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_inserting_transactions"),
                LOCALIZATION.get_localized_string(language, "error_inserting_transactions_details"),
            ))
        })?;

    Ok((new_transactions.len(), existing_transactions.len()))
}
