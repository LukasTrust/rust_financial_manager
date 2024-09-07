use crate::utils::appstate::LOCALIZATION;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use log::error;
use rocket::serde::json::Json;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, Contract, ContractHistory, User};
use crate::utils::structs::ResponseData;

use super::appstate::Language;
use super::structs::{Bank, Transaction};

pub async fn load_user_by_email(
    email_for_loading: &str,
    db: &mut Connection<DbConn>,
) -> Result<User, Json<ResponseData>> {
    use crate::schema::users as users_without_dsl;
    use crate::schema::users::dsl::*;

    users_without_dsl::table
        .filter(email.eq(email_for_loading))
        .first::<User>(db)
        .await
        .map_err(|e| {
            error!("Error loading the user: {:?}", e);
            Json(ResponseData::new_error(
                String::new(),
                "Login failed. Either the email or password was incorrect.".to_string(),
            ))
        })
}

pub async fn load_user_by_id(
    user_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<(String, String), Json<ResponseData>> {
    use crate::schema::users as users_without_dsl;
    use crate::schema::users::dsl::*;

    users_without_dsl::table
        .filter(id.eq(user_id_for_loading))
        .select((first_name, last_name))
        .first::<(String, String)>(db)
        .await
        .map_err(|e| {
            error!("Error loading the user: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_user"),
                LOCALIZATION.get_localized_string(language, "error_loading_user_details"),
            ))
        })
}

pub async fn load_current_bank_of_user(
    user_id_for_loading: i32,
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Bank, Json<ResponseData>> {
    use crate::schema::banks as banks_without_dsl;
    use crate::schema::banks::dsl::*;

    banks_without_dsl::table
        .filter(id.eq(bank_id_for_loading))
        .filter(user_id.eq(user_id_for_loading))
        .first::<Bank>(db)
        .await
        .map_err(|e| {
            error!("Error loading the bank: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "no_bank_selected"),
                LOCALIZATION.get_localized_string(language, "no_bank_selected_details"),
            ))
        })
}

/// Load the banks for a user from the database.
/// The banks are loaded from the database using the user ID.
/// The banks are returned as a vector of banks.
/// If the banks cannot be loaded, an error page is displayed.
pub async fn load_banks_of_user(
    user_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Bank>, Json<ResponseData>> {
    use crate::schema::banks as banks_without_dsl;
    use crate::schema::banks::dsl::*;

    banks_without_dsl::table
        .filter(user_id.eq(user_id_for_loading))
        .load::<Bank>(db)
        .await
        .map_err(|e| {
            error!("Error loading banks: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_banks"),
                LOCALIZATION.get_localized_string(language, "error_loading_banks_details"),
            ))
        })
}

/// Load the transactions for a bank from the database.
/// The transactions are loaded from the database using the bank ID.
/// The transactions are returned as a vector of transactions.
/// If the transactions cannot be loaded, an error page is displayed.
pub async fn load_transactions_of_bank(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .load::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading transactions: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_transactions"),
                LOCALIZATION.get_localized_string(language, "error_loading_transactions_details"),
            ))
        })
}

pub async fn load_last_transaction_data_of_bank(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Transaction, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .order_by(date.desc())
        .first::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading last transaction data: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_last_transactions"),
                LOCALIZATION
                    .get_localized_string(language, "error_loading_last_transactions_details"),
            ))
        })
}

pub async fn load_transactions_of_bank_without_contract_and_contract_allowed(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(
            bank_id
                .eq(bank_id_for_loading)
                .and(contract_id.is_null())
                .and(contract_not_allowed.eq(false)),
        )
        .load::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading transactions: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_transactions"),
                LOCALIZATION.get_localized_string(language, "error_loading_transactions_details"),
            ))
        })
}

pub async fn load_transactions_of_contract(
    contract_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(contract_id.eq(contract_id_for_loading))
        .load::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading transactions: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_transactions"),
                LOCALIZATION.get_localized_string(language, "error_loading_transactions_details"),
            ))
        })
}

pub async fn load_transaction_by_id(
    transaction_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Transaction, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(id.eq(transaction_id_for_loading))
        .first::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading transaction data: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_transactions"),
                LOCALIZATION.get_localized_string(language, "error_loading_transactions_details"),
            ))
        })
}

pub async fn load_last_transaction_of_contract(
    contract_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Transaction, Json<ResponseData>> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    transactions_without_dsl::table
        .filter(contract_id.eq(contract_id_for_loading))
        .order_by(date.desc())
        .first::<Transaction>(db)
        .await
        .map_err(|e| {
            error!("Error loading last transaction data: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_last_transactions"),
                LOCALIZATION
                    .get_localized_string(language, "error_loading_last_transactions_details"),
            ))
        })
}

pub async fn load_contracts_of_bank(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, Json<ResponseData>> {
    use crate::schema::contracts as contracts_without_dsl;
    use crate::schema::contracts::dsl::*;

    contracts_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .load::<Contract>(db)
        .await
        .map_err(|e| {
            error!("Error loading contracts: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_contracts"),
                LOCALIZATION.get_localized_string(language, "error_loading_contracts_details"),
            ))
        })
}

pub async fn load_contracts_of_bank_without_end_date(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, Json<ResponseData>> {
    use crate::schema::contracts as contracts_without_dsl;
    use crate::schema::contracts::dsl::*;

    contracts_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading).and(end_date.is_null()))
        .load::<Contract>(db)
        .await
        .map_err(|e| {
            error!("Error loading contracts: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_contracts"),
                LOCALIZATION.get_localized_string(language, "error_loading_contracts_details"),
            ))
        })
}

pub async fn load_contracts_from_ids(
    contract_ids_for_loading: Vec<i32>,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, Json<ResponseData>> {
    use crate::schema::contracts as contracts_without_dsl;
    use crate::schema::contracts::dsl::*;

    contracts_without_dsl::table
        .filter(id.eq_any(contract_ids_for_loading.clone()))
        .load::<Contract>(db)
        .await
        .map_err(|e| {
            error!("Error loading contracts: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_contracts"),
                LOCALIZATION.get_localized_string(language, "error_loading_contracts_details"),
            ))
        })
}

pub async fn load_contract_history(
    contract_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<Vec<ContractHistory>, Json<ResponseData>> {
    use crate::schema::contract_history as contract_history_without_dsl;
    use crate::schema::contract_history::dsl::*;

    contract_history_without_dsl::table
        .filter(contract_id.eq(contract_id_for_loading))
        .load::<ContractHistory>(db)
        .await
        .map_err(|e| {
            error!("Error loading contract history: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_contract_history"),
                LOCALIZATION
                    .get_localized_string(language, "error_loading_contract_history_details"),
            ))
        })
}

pub async fn load_csv_converter_of_bank(
    bank_id_for_loading: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<CSVConverter, Json<ResponseData>> {
    use crate::schema::csv_converters::dsl::*;

    csv_converters
        .filter(bank_id.eq(bank_id_for_loading))
        .first::<CSVConverter>(db)
        .await
        .map_err(|e| {
            error!("Error loading csv converter: {:?}", e);
            Json(ResponseData::new_error(
                LOCALIZATION.get_localized_string(language, "error_loading_csv_converter"),
                LOCALIZATION.get_localized_string(language, "error_loading_csv_converter_details"),
            ))
        })
}
