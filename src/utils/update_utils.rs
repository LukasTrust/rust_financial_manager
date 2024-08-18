use ::diesel::ExpressionMethods;
use diesel::QueryDsl;
use log::info;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::db_connector::DbConn;
use crate::database::models::CSVConverter;
use crate::schema::{contracts, csv_converters, transactions};

pub async fn update_transaction_with_contract_id(
    transaction_id: i32,
    contract_id: i32,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    diesel::update(transactions::table.find(transaction_id))
        .set(transactions::contract_id.eq(Some(contract_id)))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating transaction: {}", e))?;

    info!(
        "Transaction ID {} updated with contract ID {}.",
        transaction_id, contract_id
    );
    Ok(())
}

pub async fn update_transaction_remove_contract_id(
    transaction_ids: Vec<i32>,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq_any(transaction_ids.clone())))
        .set(contract_id.eq(None::<i32>))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating transactions: {}", e))?;

    info!(
        "Transaction IDs {:?} updated with contract ID None.",
        transaction_ids
    );

    Ok(())
}

pub async fn update_transaction_with_hidden(
    transactions_ids: Vec<i32>,
    is_hidden_for_updating: bool,
    db: &mut Connection<DbConn>,
) -> Result<(), String> {
    use crate::schema::transactions::dsl::*;

    diesel::update(transactions.filter(id.eq_any(transactions_ids.clone())))
        .set(is_hidden.eq(is_hidden_for_updating))
        .execute(db)
        .await
        .map_err(|e| format!("Error updating transactions: {}", e))?;

    info!(
        "Transaction IDs {:?} updated with hidden {}.",
        transactions_ids, is_hidden_for_updating
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
        .map_err(|e| format!("Error updating contract: {}", e))?;

    info!(
        "Contract ID {} updated with new amount {}.",
        contract_id, new_amount
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
        .map_err(|e| format!("Error updating contract: {}", e))?;

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
        .map_err(|e| format!("Error updating CSV converter: {}", e))?;

    info!("CSV converter updated: {:?}", csv_converter);

    Ok(())
}
