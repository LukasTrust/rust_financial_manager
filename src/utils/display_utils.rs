use chrono::NaiveDate;
use rocket::tokio::task;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use super::structs::{Bank, PerformanceData, Transaction};

/// Generate balance graph data for plotting.
/// The balance graph data is generated from the bank accounts and transactions.
/// The balance graph data is used to plot the bank account balances over time.
/// The balance graph data is returned as a JSON value.
pub async fn generate_balance_graph_data(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
) -> String {
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
    let plot_data = json!(plot_data);

    serde_json::to_string(&plot_data).unwrap()
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
    let net_gain_loss = starting_balance - ending_balance;
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
