use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::database::models::CSVConverter;
use crate::schema::{contracts, csv_converters, transactions};

pub async fn update_transactions_with_contract(
    transaction_ids: Vec<i32>,
    contract_id: Option<i32>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    diesel::update(transactions::table.filter(transactions::id.eq_any(transaction_ids.clone())))
        .set(transactions::contract_id.eq(contract_id))
        .execute(db)
        .await
        .map_err(|_| "Error updating transactions")?;

    info!(
        "Transactions IDs {:?} updated with contract ID {:?}.",
        transaction_ids, contract_id
    );

    Ok(())
}

pub async fn update_transactions_of_contract_to_new_contract(
    new_contract_id: i32,
    old_contract_ids: Vec<i32>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    diesel::update(transactions::table.filter(transactions::contract_id.eq_any(old_contract_ids)))
        .set(transactions::contract_id.eq(new_contract_id))
        .execute(db)
        .await
        .map_err(|_| "Error updating transactions")?;

    Ok(())
}

pub async fn update_transaction_with_hidden(
    transactions_id: i32,
    is_hidden_for_updating: bool,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq(transactions_id)))
        .set(is_hidden.eq(is_hidden_for_updating))
        .execute(db)
        .await
        .map_err(|_| "Error updating transaction")?;

    info!(
        "Transaction ID {} updated with hidden status.",
        transactions_id
    );

    Ok(())
}

pub async fn update_transaction_with_contract_not_allowed(
    transaction_id: i32,
    contract_not_allowed_for_updating: bool,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq(transaction_id)))
        .set(contract_not_allowed.eq(contract_not_allowed_for_updating))
        .execute(db)
        .await
        .map_err(|_| "Error updating transaction")?;

    info!(
        "Transaction ID {} updated with has_no_contract status.",
        transaction_id
    );

    Ok(())
}

pub async fn update_contract_with_new_amount(
    contract_id: i32,
    new_amount: f64,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(current_amount.eq(new_amount))
        .execute(db)
        .await
        .map_err(|_| "Error updating contract")?;

    info!(
        "Contract ID {} updated with new amount {}.",
        contract_id, new_amount
    );

    Ok(())
}

pub async fn update_contract_with_new_name(
    contract_id: i32,
    new_name: String,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(name.eq(new_name.clone()))
        .execute(db)
        .await
        .map_err(|_| "Error updating contract")?;

    info!(
        "Contract ID {} updated with new name {}.",
        contract_id, new_name
    );

    Ok(())
}

pub async fn update_contract_with_end_date(
    contract_id: i32,
    end_date_for_update: chrono::NaiveDate,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::contracts::*;

    diesel::update(contracts::table.find(contract_id))
        .set(end_date.eq(end_date_for_update))
        .execute(db)
        .await
        .map_err(|_| "Error updating contract")?;

    Ok(())
}

pub async fn update_csv_converter(
    csv_converter: CSVConverter,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::csv_converters::*;

    diesel::update(csv_converters::table.find(csv_converter.id))
        .set((
            counterparty_column.eq(csv_converter.counterparty_column),
            amount_column.eq(csv_converter.amount_column),
            bank_balance_after_column.eq(csv_converter.bank_balance_after_column),
            date_column.eq(csv_converter.date_column),
        ))
        .execute(db)
        .await
        .map_err(|_| "Error updating CSV converter")?;

    info!("CSV converter updated: {:?}", csv_converter);

    Ok(())
}
