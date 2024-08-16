use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::FilterDsl;
use diesel::BoolExpressionMethods;
use log::{error, info};
use rocket::response::Redirect;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::{
    database::models::{CSVConverter, Contract, ContractHistory},
    routes::error_page::show_error_page,
};

use super::structs::{Bank, Transaction};

pub async fn load_bank_of_user(
    cookie_user_id: i32,
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Option<Bank>, String> {
    use crate::schema::banks as banks_without_dsl;
    use crate::schema::banks::dsl::*;

    let bank_result = banks_without_dsl::table
        .filter(id.eq(bank_id_for_loading))
        .filter(user_id.eq(cookie_user_id))
        .first::<Bank>(db)
        .await
        .map_err(|e| {
            error!("Error loading the bank: {:?}", e);
            e.to_string()
        });

    match bank_result {
        Ok(bank) => {
            info!("Bank loaded for user {}: {:?}", cookie_user_id, bank);
            Ok(Some(bank))
        }
        Err(_) => {
            info!(
                "No bank found for user {} with ID {}",
                cookie_user_id, bank_id_for_loading
            );
            Ok(None)
        }
    }
}

/// Load the banks for a user from the database.
/// The banks are loaded from the database using the user ID.
/// The banks are returned as a vector of banks.
/// If the banks cannot be loaded, an error page is displayed.
pub async fn load_banks(
    cookie_user_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Bank>, String> {
    use crate::schema::banks as banks_without_dsl;
    use crate::schema::banks::dsl::*;

    let banks_result = banks_without_dsl::table
        .filter(user_id.eq(cookie_user_id))
        .load::<Bank>(db)
        .await
        .map_err(|e| format!("Error loading banks: {}", e))?;

    info!(
        "Banks count for user {}: {}",
        cookie_user_id,
        banks_result.len()
    );

    Ok(banks_result)
}

/// Load the transactions for a bank from the database.
/// The transactions are loaded from the database using the bank ID.
/// The transactions are returned as a vector of transactions.
/// If the transactions cannot be loaded, an error page is displayed.
pub async fn load_transactions_of_bank(
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, String> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    let transactions_result = transactions_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .load::<Transaction>(db)
        .await
        .map_err(|e| format!("Error loading transactions: {}", e))?;

    info!(
        "Transactions count for bank {}: {}",
        bank_id_for_loading,
        transactions_result.len()
    );
    Ok(transactions_result)
}

pub async fn load_transactions_of_bank_without_contract(
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, String> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    let transactions_result = transactions_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading).and(contract_id.is_null()))
        .load::<Transaction>(db)
        .await
        .map_err(|e| format!("Error loading transactions: {}", e))?;

    info!(
        "Transactions count for bank without a contract {}: {}",
        bank_id_for_loading,
        transactions_result.len()
    );
    Ok(transactions_result)
}

pub async fn load_transactions_of_contract(
    contract_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Transaction>, String> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    let transactions_result = transactions_without_dsl::table
        .filter(contract_id.eq(contract_id_for_loading))
        .load::<Transaction>(db)
        .await
        .map_err(|e| format!("Error loading transactions: {}", e))?;

    info!(
        "Transactions count for contract {}: {}",
        contract_id_for_loading,
        transactions_result.len()
    );

    Ok(transactions_result)
}

pub async fn load_contracts_of_bank(
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, String> {
    use crate::schema::contracts as contracts_without_dsl;
    use crate::schema::contracts::dsl::*;

    let contracts_result = contracts_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .load::<Contract>(db)
        .await
        .map_err(|e| format!("Error loading contracts of bank: {}", e))?;

    info!(
        "Contracts count for bank {}: {}",
        bank_id_for_loading,
        contracts_result.len()
    );

    Ok(contracts_result)
}

pub async fn load_contracts_of_bank_without_end_date(
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, String> {
    use crate::schema::contracts as contracts_without_dsl;
    use crate::schema::contracts::dsl::*;

    let contracts_result = contracts_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading).and(end_date.is_null()))
        .load::<Contract>(db)
        .await
        .map_err(|e| format!("Error loading contracts of bank: {}", e))?;

    info!(
        "Contracts count for bank without end date {}: {}",
        bank_id_for_loading,
        contracts_result.len()
    );

    Ok(contracts_result)
}

pub async fn load_contract_history(
    contract_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Vec<ContractHistory>, Redirect> {
    use crate::schema::contract_history as contract_history_without_dsl;
    use crate::schema::contract_history::dsl::*;

    let contract_history_result = contract_history_without_dsl::table
        .filter(contract_id.eq(contract_id_for_loading))
        .load::<ContractHistory>(db)
        .await
        .map_err(|_| {
            show_error_page(
                "Error loading contract history!".to_string(),
                "".to_string(),
            )
        })?;

    info!(
        "Contract history count for contract {}: {}",
        contract_id_for_loading,
        contract_history_result.len()
    );

    Ok(contract_history_result)
}

pub async fn load_csv_converter_of_bank(
    bank_id_for_loading: i32,
    db: &mut Connection<DbConn>,
) -> Result<Option<CSVConverter>, String> {
    use crate::schema::csv_converters::dsl::*;
    use diesel::result::Error;

    let csv_converters_result = csv_converters
        .filter(bank_id.eq(bank_id_for_loading))
        .first::<CSVConverter>(db)
        .await;

    match csv_converters_result {
        Ok(csv_converter) => {
            info!(
                "CSV converter loaded for bank {}: {:?}",
                bank_id_for_loading, csv_converter
            );
            Ok(Some(csv_converter))
        }
        Err(Error::NotFound) => {
            info!("No CSV converter found for bank {}", bank_id_for_loading);
            Ok(None)
        }
        Err(err) => {
            error!("Error loading CSV converters: {:?}", err);
            Err("Error loading CSV converters!".to_string())
        }
    }
}
