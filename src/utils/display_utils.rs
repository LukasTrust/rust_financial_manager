use chrono::NaiveDate;
use log::info;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::{database::models::Contract, utils::structs::DataMap};

use super::structs::{Bank, Discrepancy, PerformanceData, Transaction};

/// Generate balance graph data for plotting.
/// The balance graph data is generated from the bank accounts and transactions.
/// The balance graph data is used to plot the bank account balances over time.
/// The balance graph data is returned as a JSON value.
pub async fn generate_graph_data(
    banks: &[Bank],
    transactions: &[Transaction],
    discrepancies: &[Discrepancy],
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> String {
    // Convert the transactions_with_discrepancy into a HashMap for quick lookup
    let discrepancy_map: HashMap<i32, f64> = discrepancies
        .iter()
        .map(|d| (d.transaction_id, d.discrepancy_amount))
        .collect();

    let mut plot_data = vec![];

    info!("Generating graph data for {} banks", banks.len());

    for bank in banks {
        let bank = bank.clone();
        let bank_transactions = transactions
            .iter()
            .filter(|t| t.bank_id == bank.id)
            .cloned()
            .collect::<Vec<Transaction>>();

        // Use a BTreeMap to maintain order and store multiple transactions per day
        let mut data: DataMap = BTreeMap::new();

        // Filter transactions within the date range
        let filtered_transactions: Vec<&Transaction> = bank_transactions
            .iter()
            .filter(|t| t.date >= *start_date && t.date <= *end_date)
            .collect();

        for transaction in filtered_transactions.iter().rev() {
            // Check if the transaction is in the discrepancy map
            let discrepancy_amount = discrepancy_map.get(&transaction.id).cloned();

            data.entry(transaction.date).or_default().push((
                transaction.bank_balance_after,
                transaction.counterparty.clone(),
                transaction.amount,
                discrepancy_amount,
            ));
        }

        // Prepare series data for plotting
        let mut series_data = vec![];
        for (date, transactions) in data {
            for (balance, counterparty, amount, discrepancy_amount) in transactions {
                let color = if discrepancy_amount.is_some() {
                    "red"
                } else {
                    "blue"
                };

                // Adjust hover text based on discrepancy
                let hover_text = if let Some(discrepancy_amount) = discrepancy_amount {
                    format!(
                            "{}<br>Date:{}<br>Amount: {} €<br>New balance: {} €<br>Discrepancy Amount: {} €",
                            counterparty,
                            date.format("%d.%m.%Y"),
                            amount,
                            balance,
                            discrepancy_amount
                        )
                } else {
                    format!(
                        "{}<br>Date:{}<br>Amount: {} €<br>New balance: {} €",
                        counterparty,
                        date.format("%d.%m.%Y"),
                        amount,
                        balance
                    )
                };

                series_data.push((date.to_string(), balance, hover_text, color.to_string()));
            }
        }

        // Add plot data for the bank
        plot_data.push(json!({
            "name": bank.name,
            "x": series_data.iter().map(|(date, _, _, _)| date.clone()).collect::<Vec<String>>(),
            "y": series_data.iter().map(|(_, balance, _, _)| *balance).collect::<Vec<f64>>(),
            "text": series_data.iter().map(|(_, _, text, _)| text.clone()).collect::<Vec<String>>(),
            "marker": {
                "color": series_data.iter().map(|(_, _, _, color)| color.clone()).collect::<Vec<String>>(),
            },
            "type": "scatter",
            "mode": "lines+markers",
            "hoverinfo": "text"
        }));
    }

    // Return the plot data as JSON
    let plot_data = json!(plot_data);

    serde_json::to_string(&plot_data).unwrap()
}

pub fn generate_performance_value(
    banks: &[Bank],
    transactions: &[Transaction],
    contracts: &[Contract],
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> (PerformanceData, Vec<Discrepancy>) {
    let mut total_sum = 0.0;
    let mut total_transactions = 0;
    let mut total_discrepancy = 0.0;
    let mut starting_balance = 0.0;
    let mut ending_balance = 0.0;
    let mut transactions_with_discrepancy = vec![];
    let mut total_contracts = 0;
    let mut one_month_contract_amount = 0.0;
    let mut three_month_contract_amount = 0.0;
    let mut six_month_contract_amount = 0.0;
    let mut total_amount_per_year = 0.0;

    info!("Generating performance data for {} banks", banks.len());

    for bank in banks {
        let contracts_of_bank = contracts
            .iter()
            .filter(|c| c.bank_id == bank.id && c.end_date.is_none())
            .collect::<Vec<&Contract>>();

        total_contracts += contracts_of_bank.len();

        for contract in contracts_of_bank {
            match contract.months_between_payment {
                1 => {
                    total_amount_per_year += contract.current_amount * 12.0;
                    one_month_contract_amount += contract.current_amount
                }
                3 => {
                    total_amount_per_year += contract.current_amount * 4.0;
                    three_month_contract_amount += contract.current_amount
                }
                6 => {
                    total_amount_per_year += contract.current_amount * 2.0;
                    six_month_contract_amount += contract.current_amount
                }
                _ => {}
            }
        }

        let transaction_of_bank = transactions
            .iter()
            .filter(|t| t.bank_id == bank.id)
            .collect::<Vec<&Transaction>>();

        let mut previous_balance = 0.0;

        info!("Processing transactions for bank: {}", bank.name);

        // Filter transactions within the date range
        let mut transactions_for_start_end: Vec<&Transaction> = transaction_of_bank
            .iter()
            .filter(|&t| t.date >= *start_date && t.date <= *end_date)
            .cloned()
            .collect::<Vec<&Transaction>>();

        // Sort filtered transactions by date
        transactions_for_start_end.sort_by(|a, b| {
            match a.date.cmp(&b.date) {
                std::cmp::Ordering::Equal => b.id.cmp(&a.id), // If dates are equal, sort by id in descending order
                other => other,                               // Otherwise, sort by date
            }
        });

        if let Some(first_transaction) = transactions_for_start_end.first() {
            info!("First transaction: {:?}", first_transaction);
            starting_balance += first_transaction.bank_balance_after;
        }

        if let Some(last_transaction) = transactions_for_start_end.last() {
            info!("Last transaction: {:?}", last_transaction);
            ending_balance += last_transaction.bank_balance_after + last_transaction.amount;
        }

        for (index, transaction) in transactions_for_start_end.iter().enumerate() {
            total_sum += transaction.amount;
            total_transactions += 1;

            if index > 0 && index < transactions_for_start_end.len() - 1 {
                let discrepancy =
                    previous_balance - (transaction.bank_balance_after - transaction.amount);
                if !(-0.01..=0.01).contains(&discrepancy) {
                    info!(
                        "Discrepancy found in transaction: {} with amount: {}",
                        transaction.id, discrepancy
                    );
                    total_discrepancy += discrepancy;
                    transactions_with_discrepancy.push(Discrepancy {
                        transaction_id: transaction.id,
                        discrepancy_amount: discrepancy,
                    });
                }
            }

            previous_balance = transaction.bank_balance_after;
        }
    }

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

    let performance_data = PerformanceData {
        total_transactions,
        average_transaction_amount,
        net_gain_loss,
        performance_percentage,
        total_discrepancy,
        total_contracts,
        one_month_contract_amount,
        three_month_contract_amount,
        six_month_contract_amount,
        total_amount_per_year,
    };

    info!("Performance data: {:?}", performance_data);

    (performance_data, transactions_with_discrepancy)
}
