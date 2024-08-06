use log::info;
use rocket::tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use crate::database::models::CSVConverter;

use super::structs::{Bank, Transaction};

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<HashMap<i32, Vec<Bank>>>>,
    pub transactions: Arc<RwLock<HashMap<i32, Vec<Transaction>>>>,
    pub csv_convert: Arc<RwLock<HashMap<i32, CSVConverter>>>,
    pub current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
}

impl AppState {
    pub async fn set_app_state(
        &self,
        cookie_user_id: i32,
        new_banks: Option<Vec<Bank>>,
        new_transactions: Option<HashMap<i32, Vec<Transaction>>>,
        new_csv_converters: Option<HashMap<i32, CSVConverter>>,
        new_current_bank: Option<Bank>,
    ) {
        if let Some(banks) = new_banks {
            self.update_banks(cookie_user_id, banks).await;
        }

        if let Some(transactions) = new_transactions {
            self.update_transactions(transactions).await;
        }

        if let Some(csv_converters) = new_csv_converters {
            self.update_csv_converters(csv_converters).await;
        }

        if let Some(current_bank) = new_current_bank {
            self.update_current_bank(cookie_user_id, current_bank).await;
        }
    }

    pub async fn update_banks(&self, cookie_user_id: i32, banks: Vec<Bank>) {
        let mut banks_state = self.banks.write().await;

        info!(
            "Banks length before update: {}",
            banks_state.values().flatten().count()
        );

        let bank_of_user = banks_state.entry(cookie_user_id).or_insert_with(Vec::new);

        for bank in banks.iter() {
            if !bank_of_user.iter().any(|b| b.id == bank.id) {
                bank_of_user.push(bank.clone());
            }
        }

        info!(
            "Banks length after update: {}",
            banks_state.values().flatten().count()
        );
    }

    pub async fn update_transactions(&self, new_transactions: HashMap<i32, Vec<Transaction>>) {
        let mut transactions_state = self.transactions.write().await;

        info!(
            "Transactions length before update: {}",
            transactions_state.values().flatten().count()
        );

        for (bank_id, bank_transactions) in new_transactions.iter() {
            let existing_transactions = transactions_state.entry(*bank_id).or_insert_with(Vec::new);

            for transaction in bank_transactions.iter() {
                if !existing_transactions.iter().any(|t| t.id == transaction.id) {
                    existing_transactions.push(transaction.clone());
                }
            }
        }

        info!(
            "Transactions length after update: {}",
            transactions_state.values().flatten().count()
        );
    }

    pub async fn update_csv_converters(&self, new_csv_converters: HashMap<i32, CSVConverter>) {
        let mut csv_converters_state = self.csv_convert.write().await;

        info!(
            "CSV converters state before update: {:?}",
            *csv_converters_state
        );

        for (bank_id, csv_converter) in new_csv_converters.iter() {
            csv_converters_state.insert(*bank_id, csv_converter.clone());
        }

        info!(
            "CSV converters state after update: {:?}",
            *csv_converters_state
        );
    }

    pub async fn update_current_bank(&self, cookie_user_id: i32, current_bank: Bank) {
        let mut current_bank_state = self.current_bank.write().await;

        if let Some(bank_of_user) = current_bank_state.get(&cookie_user_id) {
            info!("Current bank found: {:?}", bank_of_user);
            if bank_of_user.id != current_bank.id {
                current_bank_state.insert(cookie_user_id, current_bank.clone());
                info!("Current bank updated: {:?}", current_bank);
            }
        } else {
            current_bank_state.insert(cookie_user_id, current_bank.clone());
            info!("Current bank set for the first time: {:?}", current_bank);
        }
    }
}
