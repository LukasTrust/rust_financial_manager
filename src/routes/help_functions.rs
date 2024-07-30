use chrono::NaiveDate;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};

use crate::database::models::{Bank, Transaction};

pub fn generate_balance_graph_data(
    banks: &[Bank],
    transactions: &HashMap<i32, Vec<Transaction>>,
) -> serde_json::Value {
    let mut plot_data = vec![];

    for bank in banks {
        let bank_transactions = transactions.get(&bank.id).unwrap();
        let mut balance = bank.current_amount.unwrap_or(0.0);
        let mut data: BTreeMap<NaiveDate, f64> = BTreeMap::new();

        // Insert today's balance
        let today = chrono::Local::now().naive_local().date();
        data.insert(today, balance);

        for transaction in bank_transactions.iter().rev() {
            match transaction.type_of_t.as_str() {
                "Deposit" => balance -= transaction.amount,
                "Withdraw" => balance += transaction.amount,
                "Interest" => balance -= transaction.amount, // Assuming interest is added to the balance
                _ => (),
            }
            data.entry(transaction.date)
                .and_modify(|e| *e = balance)
                .or_insert(balance);
        }

        data.entry(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
            .or_insert(balance); // Ensure we plot the initial balance at the start

        let series_data: Vec<(String, f64)> = data
            .into_iter()
            .map(|(date, balance)| (date.to_string(), balance))
            .collect();

        plot_data.push(json!({
            "name": bank.name,
            "x": series_data.iter().map(|(date, _)| date.clone()).collect::<Vec<String>>(),
            "y": series_data.iter().map(|(_, balance)| *balance).collect::<Vec<f64>>(),
            "type": "scatter",
            "mode": "lines+markers"
        }));
    }

    json!(plot_data)
}
