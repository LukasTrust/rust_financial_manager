use chrono::NaiveDate;
use log::{error, info};
use rocket::{http::CookieJar, response::Redirect};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::database::models::{Bank, Transaction};
use crate::routes::error_page::show_error_page;

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

    // Return the plot data as JSON
    json!(plot_data)
}

/// Extract the user ID from the user ID cookie.
/// If the user ID cookie is not found or cannot be parsed, an error page is displayed.
/// The user ID is returned if the user ID cookie is found and parsed successfully.
pub fn extract_user_id(cookies: &CookieJar<'_>) -> Result<i32, Redirect> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        info!("User ID cookie found: {:?}", user_id_cookie.value());
        user_id_cookie.value().parse::<i32>().map_err(|_| {
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
