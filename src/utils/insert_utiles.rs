use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::models::{
    CSVConverter, Contract, ContractHistory, NewCSVConverter, NewContract, NewContractHistory,
};
use crate::database::{db_connector::DbConn, models::NewBank};

use super::structs::Bank;

pub async fn insert_bank(new_bank: NewBank, db: &mut Connection<DbConn>) -> Result<Bank, String> {
    use crate::schema::banks;

    diesel::insert_into(banks::table)
        .values(&new_bank)
        .get_result::<Bank>(db)
        .await
        .map_err(|_| "Error inserting bank".into())
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
        .map_err(|_| "Error inserting csv converter".into())
}

pub async fn insert_contract(
    new_contract: NewContract,
    db: &mut Connection<DbConn>,
) -> Result<Contract, String> {
    use crate::schema::contracts;

    let new_contract = diesel::insert_into(contracts::table)
        .values(&new_contract)
        .get_result::<Contract>(db)
        .await
        .map_err(|_| "Error inserting contract")?;

    info!("Contract inserted: {:?}", new_contract);

    Ok(new_contract)
}

pub async fn insert_contract_history(
    new_contract_history: NewContractHistory,
    db: &mut Connection<DbConn>,
) -> Result<ContractHistory, String> {
    use crate::schema::contract_history;

    let new_contract_history = diesel::insert_into(contract_history::table)
        .values(&new_contract_history)
        .get_result::<ContractHistory>(db)
        .await
        .map_err(|_| "Error inserting contract history")?;

    info!("Contract history inserted: {:?}", new_contract_history);

    Ok(new_contract_history)
}
