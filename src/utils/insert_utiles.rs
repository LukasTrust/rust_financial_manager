use diesel::result::Error as DieselError;
use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::models::{
    CSVConverter, Contract, ContractHistory, NewCSVConverter, NewContract, NewContractHistory,
    NewTransaction, NewUser,
};
use crate::database::{db_connector::DbConn, models::NewBank};

use super::structs::{Bank, Transaction};

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

pub async fn insert_bank(new_bank: NewBank, db: &mut Connection<DbConn>) -> Result<Bank, String> {
    use crate::schema::banks;

    diesel::insert_into(banks::table)
        .values(&new_bank)
        .get_result::<Bank>(db)
        .await
        .map_err(|_| "Error inserting bank".to_string())
}

pub async fn insert_csv_converter(
    new_csv_converter: NewCSVConverter,
    db: &mut Connection<DbConn>,
) -> Result<CSVConverter, String> {
    use crate::schema::csv_converters;

    diesel::insert_into(csv_converters::table)
        .values(&new_csv_converter)
        .get_result::<CSVConverter>(db)
        .await
        .map_err(|_| "Error inserting csv converter".to_string())
}

pub async fn insert_contracts(
    new_contracts: &Vec<NewContract>,
    db: &mut Connection<DbConn>,
) -> Result<Vec<Contract>, String> {
    use crate::schema::contracts;

    let inserted_contracts = diesel::insert_into(contracts::table)
        .values(new_contracts)
        .get_results::<Contract>(db)
        .await
        .map_err(|_| "Error inserting contracts")?;

    info!("Contracts inserted");

    Ok(inserted_contracts)
}

pub async fn insert_contract_histories(
    new_contract_histories: &Vec<NewContractHistory>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contract_history;

    diesel::insert_into(contract_history::table)
        .values(new_contract_histories)
        .get_results::<ContractHistory>(db)
        .await
        .map_err(|_| "Error inserting contract histories")?;

    info!("Contract histories inserted");

    Ok(())
}

pub async fn insert_transactions(
    mut new_transactions: Vec<NewTransaction>,
    existing_transactions: Vec<Transaction>,
    db: &mut Connection<DbConn>,
) -> Result<(usize, usize), String> {
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
        .map_err(|_| "Error inserting transactions")?;

    Ok((new_transactions.len(), existing_transactions.len()))
}
