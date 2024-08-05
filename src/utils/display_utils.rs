use chrono::NaiveDate;
use log::info;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::database::models::{Bank, Transaction};
use crate::structs::AppState;

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
