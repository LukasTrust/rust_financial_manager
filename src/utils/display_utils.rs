use chrono::NaiveDate;
use log::info;
use rocket::{tokio::task, State};
use rocket_dyn_templates::{context, Template};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use super::appstate::AppState;
use super::get_utils::{get_banks_of_user, get_current_bank};
use super::structs::{Bank, PerformanceData, Transaction};

/// Display the base page or a subview with data.
/// The view to show is passed as a parameter.
/// The success message and error message are optional and are displayed on the page.
pub async fn show_base_or_subview_with_data(
    cookie_user_id: i32,
    state: &State<AppState>,
    view_to_show: String,
    generate_graph_data: bool,
    generate_only_current_bank: bool,
    success_message: Option<String>,
    error_message: Option<String>,
    performance_data: Option<PerformanceData>,
) -> Template {
    let banks = get_banks_of_user(cookie_user_id, state).await;

    let transactions = state.transactions.read().await.clone();

    let mut current_bank = None;

    let plot_data = if generate_graph_data {
        match generate_only_current_bank {
            true => {
                let current_bank_result = get_current_bank(cookie_user_id, state).await;

                if current_bank_result.is_err() {
                    let error = "No bank selected. Please select a bank.".to_string();
                    return Template::render(
                        "base",
                        context! {
                            banks: banks,
                            plot_data: "".to_string(),
                            success: success_message.unwrap_or_default(),
                            error: error,
                            performance_data: performance_data.unwrap_or_default(),
                        },
                    );
                }

                let bank_of_user = current_bank_result.unwrap();
                current_bank = Some(bank_of_user.clone());

                info!("Generating balance graph data for current bank only.");

                generate_balance_graph_data(&[bank_of_user], &transactions).await
            }
            false => {
                info!("Generating balance graph data for all banks.");
                generate_balance_graph_data(&banks, &transactions).await
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
            performance_data: performance_data.unwrap_or_default(),
        },
    )
}

/// Generate balance graph data for plotting.
/// The balance graph data is generated from the bank accounts and transactions.
/// The balance graph data is used to plot the bank account balances over time.
/// The balance graph data is returned as a JSON value.
pub async fn generate_balance_graph_data(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
) -> serde_json::Value {
    let mut tasks = vec![];

    for bank in banks {
        let bank = bank.clone();
        let transactions = transactions.clone();

        let task = task::spawn(async move {
            let mut plot_data = vec![];

            if let Some(bank_transactions) = transactions.get(&bank.id) {
                let mut data: BTreeMap<NaiveDate, (f64, String, f64)> = BTreeMap::new();

                // Process transactions in reverse chronological order
                for transaction in bank_transactions.iter().rev() {
                    data.entry(transaction.date)
                        .and_modify(|e| {
                            *e = (
                                transaction.bank_balance_after,
                                transaction.counterparty.clone(),
                                transaction.amount,
                            )
                        })
                        .or_insert((
                            transaction.bank_balance_after,
                            transaction.counterparty.clone(),
                            transaction.amount,
                        ));
                }

                // Ensure we plot the initial balance at the start of 2023
                if let Some(start_date) = NaiveDate::from_ymd_opt(2023, 1, 1) {
                    if let Some(&(initial_balance, ref initial_counterparty, initial_amount)) =
                        data.values().next()
                    {
                        let initial_counterparty = initial_counterparty.clone();
                        data.entry(start_date).or_insert((
                            initial_balance,
                            initial_counterparty,
                            initial_amount,
                        ));
                    }
                }

                // Prepare series data for plotting
                let series_data: Vec<(String, f64, String)> = data
                    .into_iter()
                    .map(|(date, (balance, counterparty, amount))| {
                        (
                            date.to_string(),
                            balance,
                            format!(
                                "{}<br>Date:{}<br>Amount: {} €<br>New balance: {} €",
                                counterparty,
                                date.format("%d.%m.%Y").to_string(),
                                amount,
                                balance
                            ),
                        )
                    })
                    .collect();

                // Add plot data for the bank
                plot_data.push(json!({
                    "name": bank.name,
                    "x": series_data.iter().map(|(date, _, _)| date.clone()).collect::<Vec<String>>(),
                    "y": series_data.iter().map(|(_, balance, _)| *balance).collect::<Vec<f64>>(),
                    "text": series_data.iter().map(|(_, _, text)| text.clone()).collect::<Vec<String>>(),
                    "type": "scatter",
                    "mode": "lines+markers",
                    "hoverinfo": "text"
                }));
            }

            plot_data
        });

        tasks.push(task);
    }

    let mut plot_data = vec![];

    for task in tasks {
        if let Ok(data) = task.await {
            plot_data.extend(data);
        }
    }

    // Return the plot data as JSON
    json!(plot_data)
}

pub fn generate_performance_value(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> PerformanceData {
    let mut total_sum = 0.0;
    let mut total_transactions = 0;
    let mut starting_balance = None;
    let mut ending_balance = None;

    for bank in banks {
        if let Some(transaction_of_bank) = transactions.get(&bank.id) {
            for transaction in transaction_of_bank {
                if transaction.date >= start_date && transaction.date <= end_date {
                    total_sum += transaction.amount;
                    total_transactions += 1;

                    // Determine the starting balance
                    if starting_balance.is_none() {
                        starting_balance =
                            Some(transaction.bank_balance_after - transaction.amount);
                    }

                    // Continuously update the ending balance
                    ending_balance = Some(transaction.bank_balance_after);
                }
            }
        }
    }

    let starting_balance = starting_balance.unwrap_or(0.0);
    let ending_balance = ending_balance.unwrap_or(starting_balance); // If no transactions in range, balance doesn't change
    let net_gain_loss = ending_balance - starting_balance;
    let performance_percentage = if starting_balance != 0.0 {
        (net_gain_loss / starting_balance) * 100.0
    } else {
        0.0
    };
    let average_transaction_amount = if total_transactions > 0 {
        total_sum / total_transactions as f64
    } else {
        0.0
    };

    PerformanceData {
        total_transactions,
        average_transaction_amount,
        net_gain_loss,
        performance_percentage,
    }
}
