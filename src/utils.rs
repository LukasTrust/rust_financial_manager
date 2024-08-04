use ::diesel::ExpressionMethods;
use chrono::NaiveDate;
use diesel::query_dsl::methods::FilterDsl;
use log::{error, info};
use rocket::State;
use rocket::{http::CookieJar, response::Redirect};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::diesel::AsyncPgConnection;
use rocket_dyn_templates::{context, Template};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::database::models::{Bank, CSVConverter, Transaction};
use crate::routes::error_page::show_error_page;
use crate::structs::AppState;

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

/// Update the application state with new data.
/// The application state is updated with new banks, transactions, CSV converters, and the current bank.
/// All the new data is optional and can be None.
pub async fn update_app_state(
    cookie_user_id: i32,
    state: &State<AppState>,
    new_banks: Option<Vec<Bank>>,
    new_transactions: Option<HashMap<i32, Vec<Transaction>>>,
    new_csv_converters: Option<HashMap<i32, CSVConverter>>,
    new_current_bank: Option<Bank>,
) {
    if let Some(banks) = new_banks {
        let mut banks_state = state.banks.write().await;

        info!(
            "Banks length before update: {}",
            banks_state.values().flatten().count()
        );

        let mut bank_of_user = banks_state.get_mut(&cookie_user_id);

        if bank_of_user.is_none() {
            banks_state.insert(cookie_user_id, vec![]);
        } else {
            for bank in banks.iter() {
                if bank_of_user
                    .as_mut()
                    .unwrap()
                    .iter()
                    .find(|b| b.id == bank.id)
                    .is_none()
                {
                    bank_of_user.as_mut().unwrap().push(bank.clone());
                }
            }
        }

        info!(
            "Banks length after update: {}",
            banks_state.values().flatten().count()
        );
    }

    if let Some(transactions) = new_transactions {
        let mut transactions_state = state.transactions.write().await;

        info!(
            "Transactions length before update: {}",
            transactions_state.values().flatten().count()
        );

        for (bank_id, bank_transactions) in transactions.iter() {
            if let Some(existing_transactions) = (*transactions_state).get_mut(bank_id) {
                for transaction in bank_transactions.iter() {
                    if existing_transactions
                        .iter()
                        .find(|t| t.id == transaction.id)
                        .is_none()
                    {
                        existing_transactions.push(transaction.clone());
                    }
                }
            } else {
                (*transactions_state).insert(*bank_id, bank_transactions.clone());
            }
        }

        info!(
            "Transactions length after update: {}",
            transactions_state.values().flatten().count()
        );
    }

    if let Some(csv_converters) = new_csv_converters {
        let mut csv_converters_state = state.csv_convert.write().await;

        info!(
            "CSV converters state before update: {:?}",
            *csv_converters_state
        );

        for (bank_id, csv_converter) in csv_converters.iter() {
            if let Some(existing_csv_converter) = (*csv_converters_state).get_mut(bank_id) {
                *existing_csv_converter = csv_converter.clone();
            } else {
                (*csv_converters_state).insert(*bank_id, csv_converter.clone());
            }
            *csv_converters_state = csv_converters.clone();

            info!(
                "CSV converters state after update: {:?}",
                *csv_converters_state
            );
        }
    }

    if let Some(current_bank) = new_current_bank {
        let mut current_bank_state = state.current_bank.write().await;

        let bank_of_user = current_bank_state.get(&cookie_user_id);

        if let Some(bank_of_user) = bank_of_user {
            info!("Current bank found: {:?}", bank_of_user);
            if bank_of_user.id != current_bank.id {
                current_bank_state.insert(cookie_user_id, current_bank.clone());
                info!("Current bank updated: {:?}", current_bank);
            }
        }
    }
}

/// Display the home page or a subview with data.
/// The view to show is passed as a parameter.
/// The success message and error message are optional and are displayed on the page.
pub async fn show_home_or_subview_with_data(
    cookie_user_id: i32,
    state: &State<AppState>,
    view_to_show: String,
    generate_graph_data: bool,
    generate_only_current_bank: bool,
    success_message: Option<String>,
    error_message: Option<String>,
) -> Template {
    let banks = state
        .banks
        .read()
        .await
        .get(&cookie_user_id)
        .cloned()
        .unwrap_or_default();

    let transactions = state.transactions.read().await.clone();

    let current_bank = state
        .current_bank
        .read()
        .await
        .get(&cookie_user_id)
        .cloned()
        .unwrap_or_default();

    let plot_data = if generate_graph_data {
        match generate_only_current_bank {
            true => {
                info!("Generating balance graph data for current bank only.");

                generate_balance_graph_data(&[current_bank.clone()], &transactions)
            }
            false => {
                info!("Generating balance graph data for all banks.");
                generate_balance_graph_data(&banks, &transactions)
            }
        }
    } else {
        serde_json::Value::String("".to_string())
    };

    Template::render(
        view_to_show,
        context! {
            banks: banks,
            bank: current_bank,
            plot_data: plot_data.to_string(),
            success: success_message.unwrap_or_default(),
            error: error_message.unwrap_or_default(),
        },
    )
}

/// Generate balance graph data for plotting.
/// The balance graph data is generated from the bank accounts and transactions.
/// The balance graph data is used to plot the bank account balances over time.
/// The balance graph data is returned as a JSON value.
pub fn generate_balance_graph_data(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
) -> serde_json::Value {
    let mut plot_data = vec![];

    for bank in banks {
        if let Some(bank_transactions) = transactions.get(&bank.id) {
            let mut data: BTreeMap<NaiveDate, f64> = BTreeMap::new();

            // Insert today's balance from the bank's current amount
            let today = chrono::Local::now().naive_local().date();
            if let Some(current_amount) = bank.current_amount {
                data.insert(today, current_amount);
            }

            // Process transactions in reverse chronological order
            for transaction in bank_transactions.iter().rev() {
                data.entry(transaction.date)
                    .and_modify(|e| *e = transaction.bank_current_balance_after)
                    .or_insert(transaction.bank_current_balance_after);
            }

            // Ensure we plot the initial balance at the start of 2023
            if let Some(start_date) = NaiveDate::from_ymd_opt(2023, 1, 1) {
                if let Some(&initial_balance) = data.values().next() {
                    data.entry(start_date).or_insert(initial_balance);
                }
            }

            // Prepare series data for plotting
            let series_data: Vec<(String, f64)> = data
                .into_iter()
                .map(|(date, balance)| (date.to_string(), balance))
                .collect();

            // Add plot data for the bank
            plot_data.push(json!({
                "name": bank.name,
                "x": series_data.iter().map(|(date, _)| date.clone()).collect::<Vec<String>>(),
                "y": series_data.iter().map(|(_, balance)| *balance).collect::<Vec<f64>>(),
                "type": "scatter",
                "mode": "lines+markers"
            }));
        }
    }

    info!("Plot data generated: {:?}", plot_data);

    // Return the plot data as JSON
    json!(plot_data)
}

/// Extract the user ID from the user ID cookie.
/// If the user ID cookie is not found or cannot be parsed, an error page is displayed.
/// The user ID is returned if the user ID cookie is found and parsed successfully.
pub fn extract_user_id(cookies: &CookieJar<'_>) -> Result<i32, Redirect> {
    if let Some(cookie_user_id) = cookies.get("user_id") {
        info!("User ID cookie found: {:?}", cookie_user_id.value());
        cookie_user_id.value().parse::<i32>().map_err(|_| {
            error!("Error parsing user ID cookie.");
            show_error_page(
                "Error validating the login!".to_string(),
                "Please login again.".to_string(),
            )
        })
    } else {
        error!("No user ID cookie found.");
        Err(show_error_page(
            "Error validating the login!".to_string(),
            "Please login again.".to_string(),
        ))
    }
}
