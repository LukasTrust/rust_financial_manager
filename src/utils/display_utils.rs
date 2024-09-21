use chrono::NaiveDate;
use log::{info, warn};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::{
    database::models::Contract,
    utils::{appstate::LOCALIZATION, structs::DataMap},
};

use super::{
    appstate::Language,
    structs::{Bank, Discrepancy, PerformanceData, Transaction},
};

/// Generate balance graph data for plotting.
/// The balance graph data is generated from the bank accounts and transactions.
/// The balance graph data is used to plot the bank account balances over time.
/// The balance graph data is returned as a JSON value.
pub async fn generate_graph_data(
    banks: &[Bank],
    transactions: &[Transaction],
    discrepancies: &[Discrepancy],
    language: Language,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> String {
    let start = std::time::Instant::now();

    let date_string = LOCALIZATION.get_localized_string(language, "transactions_date_header");
    let amount_string = LOCALIZATION.get_localized_string(language, "transactions_amount_header");
    let balance_string = LOCALIZATION.get_localized_string(language, "new_balance_header");
    let discrepancy_string = LOCALIZATION.get_localized_string(language, "discrepancy_header");
    let heighest_balance_string = LOCALIZATION.get_localized_string(language, "heighest_balance");
    let lowest_balance_string = LOCALIZATION.get_localized_string(language, "lowest_balance");

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

        let filtered_transactions: Vec<&Transaction> = bank_transactions
            .iter()
            .filter(|t| t.date >= *start_date && t.date <= *end_date)
            .collect();

        // Determine the minimum and maximum bank balances after transactions
        let min_balance = filtered_transactions
            .iter()
            .map(|t| t.bank_balance_after)
            .fold(f64::INFINITY, f64::min);
        let max_balance = filtered_transactions
            .iter()
            .map(|t| t.bank_balance_after)
            .fold(f64::NEG_INFINITY, f64::max);

        let mut data: DataMap = BTreeMap::new();

        for transaction in filtered_transactions.iter().rev() {
            let discrepancy_amount = discrepancy_map.get(&transaction.id).cloned();
            data.entry(transaction.date).or_default().push((
                transaction.bank_balance_after,
                transaction.counterparty.clone(),
                transaction.amount,
                discrepancy_amount,
            ));
        }

        let mut series_data = vec![];
        for (date, transactions) in data {
            for (balance, counterparty, amount, discrepancy_amount) in transactions {
                // Adjust marker size and color based on bank balance
                let (color, size) = if balance == min_balance {
                    ("red", 10) // Highlight lowest balance
                } else if balance == max_balance {
                    ("green", 10) // Highlight highest balance
                } else {
                    ("blue", 5) // Default size for other points
                };

                let mut hover_text = if let Some(discrepancy_amount) = discrepancy_amount {
                    format!(
                        "{}<br>{}:{}<br>{}: {} €<br>{}: {} €<br>{}: {} €",
                        counterparty,
                        date_string,
                        date.format("%d.%m.%Y"),
                        amount_string,
                        amount,
                        balance_string,
                        balance,
                        discrepancy_string,
                        discrepancy_amount
                    )
                } else {
                    format!(
                        "{}<br>{}:{}<br>{}: {} €<br>{}: {} €",
                        counterparty,
                        date_string,
                        date.format("%d.%m.%Y"),
                        amount_string,
                        amount,
                        balance_string,
                        balance
                    )
                };

                if balance == min_balance {
                    hover_text = format!("{}<br>{}", lowest_balance_string, hover_text);
                } else if balance == max_balance {
                    hover_text = format!("{}<br>{}", heighest_balance_string, hover_text);
                }

                series_data.push((
                    date.to_string(),
                    balance,
                    hover_text,
                    color.to_string(),
                    size,
                ));
            }
        }

        plot_data.push(json!({
            "name": bank.name,
            "x": series_data.iter().map(|(date, _, _, _, _)| date.clone()).collect::<Vec<String>>(),
            "y": series_data.iter().map(|(_, balance, _, _, _)| *balance).collect::<Vec<f64>>(),
            "text": series_data.iter().map(|(_, _, text, _, _)| text.clone()).collect::<Vec<String>>(),
            "marker": {
                "color": series_data.iter().map(|(_, _, _, color, _)| color.clone()).collect::<Vec<String>>(),
                "size": series_data.iter().map(|(_, _, _, _, size)| *size).collect::<Vec<u32>>(),
            },
            "type": "scatter",
            "mode": "lines+markers",
            "hoverinfo": "text"
        }));
    }

    let plot_data = json!(plot_data);

    warn!("Graph data generation took: {:?}", start.elapsed());

    serde_json::to_string(&plot_data).unwrap()
}

pub fn generate_performance_value(
    transactions: &[Transaction],
    contracts: &[Contract],
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> (PerformanceData, Vec<Discrepancy>) {
    let start = std::time::Instant::now();

    info!("start_date: {}", start_date);
    info!("end_date: {}", end_date);

    // Filter transactions within the date range
    let mut filtered_transactions: Vec<&Transaction> = transactions
        .iter()
        .filter(|t| t.date >= *start_date && t.date <= *end_date)
        .collect();

    let transactions_count = filtered_transactions.len();

    let mut open_contracts: Vec<&Contract> = contracts
        .iter()
        .filter(|c| c.start_date >= *start_date && c.end_date.map_or(true, |d| d >= *end_date))
        .collect();

    let contracts_count = open_contracts.len();

    if (transactions_count == 0) || (contracts_count == 0) {
        return (PerformanceData::default(), vec![]);
    } else if transactions_count == 0 {
        return handle_only_contracts(contracts_count, &mut open_contracts);
    } else if contracts_count == 0 {
        return handle_only_transactions(transactions_count, &mut filtered_transactions);
    }

    let result_only_transactions =
        handle_only_transactions(transactions_count, &mut filtered_transactions);

    let transactions_with_discrepancy = result_only_transactions.1;

    let result_only_contracts = handle_only_contracts(contracts_count, &mut open_contracts);

    let contracts_amount_per_time_span: f64 =
        filtered_transactions.iter().fold(0.0, |acc, transaction| {
            if let Some(contract_id) = transaction.contract_id {
                if open_contracts.iter().any(|c| c.id == contract_id) {
                    acc + transaction.amount
                } else {
                    acc
                }
            } else {
                acc
            }
        });

    let performance_data = PerformanceData::new(
        result_only_transactions.0,
        result_only_contracts.0,
        contracts_amount_per_time_span,
    );

    warn!("Performance value calculation took: {:?}", start.elapsed());

    (performance_data, transactions_with_discrepancy)
}

fn handle_only_transactions(
    transactions_count: usize,
    filtered_transactions: &mut [&Transaction],
) -> (PerformanceData, Vec<Discrepancy>) {
    let mut transactions_total_amount = 0.0;
    let mut transactions_max_amount = f64::MIN;
    let mut transactions_min_amount = f64::MAX;

    // Calculate total, max, and min amounts in a single pass
    for transaction in filtered_transactions.iter() {
        let amount = transaction.amount;
        transactions_total_amount += amount;
        transactions_max_amount = transactions_max_amount.max(amount);
        transactions_min_amount = transactions_min_amount.min(amount);
    }

    let transactions_average_amount = transactions_total_amount / transactions_count as f64;

    // Find first and last transactions by date
    filtered_transactions.sort_by(|a, b| a.date.cmp(&b.date));
    let first_transaction = filtered_transactions.first().unwrap();

    let last_transaction = filtered_transactions.last().unwrap();

    let transactions_net_gain_loss =
        last_transaction.bank_balance_after - first_transaction.bank_balance_after;

    let mut transactions_total_discrepancy = 0.0;
    let mut transactions_with_discrepancy = vec![];

    // Sort by date, then by ID descending for same-date transactions
    filtered_transactions.sort_by(|a, b| {
        match a.date.cmp(&b.date) {
            std::cmp::Ordering::Equal => b.id.cmp(&a.id), // Sort by ID descending if dates are equal
            other => other,                               // Otherwise, sort by date
        }
    });

    let mut previous_balance = filtered_transactions.first().unwrap().bank_balance_after;

    // Calculate discrepancies
    for (_, transaction) in filtered_transactions.iter().enumerate().skip(1) {
        let discrepancy = previous_balance - (transaction.bank_balance_after - transaction.amount);
        if !(-0.01..=0.01).contains(&discrepancy) {
            transactions_total_discrepancy += discrepancy;
            transactions_with_discrepancy.push(Discrepancy {
                transaction_id: transaction.id,
                discrepancy_amount: discrepancy,
            });
        }
        previous_balance = transaction.bank_balance_after;
    }

    let performance_data = PerformanceData::new_only_transaction(
        transactions_count,
        transactions_average_amount,
        transactions_max_amount,
        transactions_min_amount,
        transactions_net_gain_loss,
        transactions_total_discrepancy,
    );

    (performance_data, transactions_with_discrepancy)
}

fn handle_only_contracts(
    contracts_count: usize,
    open_contracts: &mut [&Contract],
) -> (PerformanceData, Vec<Discrepancy>) {
    let positive_amounts: Vec<f64> = open_contracts
        .iter()
        .filter(|c| c.current_amount > 0.0)
        .map(|c| c.current_amount)
        .collect();

    let contracts_total_positive_amount: f64 = positive_amounts.iter().sum();

    let negative_amounts: Vec<f64> = open_contracts
        .iter()
        .filter(|c| c.current_amount < 0.0)
        .map(|c| c.current_amount)
        .collect();

    let contracts_total_negative_amount: f64 = negative_amounts.iter().sum();

    let contracts_amount_per_year: f64 = open_contracts
        .iter()
        .map(|contract| {
            let months_between = contract.months_between_payment as f64;
            contract.current_amount / months_between * 12.0
        })
        .sum();

    let contracts_amount_per_month = contracts_amount_per_year / 12.0;

    let performance_data = PerformanceData::new_only_contract(
        contracts_count,
        contracts_total_positive_amount,
        contracts_total_negative_amount,
        contracts_amount_per_month,
        contracts_amount_per_year,
    );

    (performance_data, vec![])
}
