use log::info;
use rocket::State;
use std::collections::HashMap;

use crate::database::models::{Bank, CSVConverter, Transaction};
use crate::structs::AppState;

/// Update the application state with new data.
/// The application state is updated with new banks, transactions, CSV converters, and the current bank.
/// All the new data is optional and can be None.
pub async fn set_app_state(
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
