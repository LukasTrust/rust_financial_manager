use ::diesel::ExpressionMethods;
use diesel::query_dsl::methods::FilterDsl;
use log::{error, info};
use rocket::response::Redirect;
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::diesel::AsyncPgConnection;

use crate::{database::models::CSVConverter, routes::error_page::show_error_page};

use super::structs::{Bank, Transaction};

/// Load the transactions for a bank from the database.
/// The transactions are loaded from the database using the bank ID.
/// The transactions are returned as a vector of transactions.
/// If the transactions cannot be loaded, an error page is displayed.
pub async fn load_transactions(
    bank_id_for_loading: i32,
    db: &mut AsyncPgConnection,
) -> Result<Vec<Transaction>, Redirect> {
    use crate::schema::transactions as transactions_without_dsl;
    use crate::schema::transactions::dsl::*;

    let transactions_result = transactions_without_dsl::table
        .filter(bank_id.eq(bank_id_for_loading))
        .load::<Transaction>(db)
        .await
        .map_err(|_| show_error_page("Error loading transactions!".to_string(), "".to_string()));

    match transactions_result {
        Ok(transactions_result) => {
            info!(
                "Transactions count for bank {}: {}",
                bank_id_for_loading,
                transactions_result.len()
            );
            Ok(transactions_result)
        }
        Err(err) => {
            error!("Error loading transactions: {:?}", err);
            Err(err)
        }
    }
}

/// Load the banks for a user from the database.
/// The banks are loaded from the database using the user ID.
/// The banks are returned as a vector of banks.
/// If the banks cannot be loaded, an error page is displayed.
pub async fn load_banks(
    cookie_user_id: i32,
    db: &mut AsyncPgConnection,
) -> Result<Vec<Bank>, Redirect> {
    use crate::schema::banks as banks_without_dsl;
    use crate::schema::banks::dsl::*;

    let banks_result = banks_without_dsl::table
        .filter(user_id.eq(cookie_user_id))
        .load::<Bank>(db)
        .await
        .map_err(|_| show_error_page("Error loading banks!".to_string(), "".to_string()));

    match banks_result {
        Ok(banks_result) => {
            info!(
                "Banks count for user {}: {}",
                cookie_user_id,
                banks_result.len()
            );
            Ok(banks_result)
        }
        Err(err) => {
            error!("Error loading banks: {:?}", err);
            Err(err)
        }
    }
}

/// Load the CSV converters for a bank from the database.
/// The CSV converters are loaded from the database using the bank ID.
/// The CSV converters are returned as a CSVConverter struct.
/// If the CSV converters cannot be loaded, an error page is displayed.
pub async fn load_csv_converters(
    bank_id_for_loading: i32,
    db: &mut AsyncPgConnection,
) -> Result<Option<CSVConverter>, Redirect> {
    use crate::schema::csv_converters::dsl::*;
    use diesel::result::Error;

    let csv_converters_result = csv_converters
        .filter(csv_bank_id.eq(bank_id_for_loading))
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
            Err(show_error_page(
                "Error loading CSV converters!".to_string(),
                "".to_string(),
            ))
        }
    }
}
