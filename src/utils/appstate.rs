use log::info;
use rocket::tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use crate::database::models::{CSVConverter, Contract};

use super::structs::{Bank, Transaction};

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<HashMap<i32, Vec<Bank>>>>,
    pub transactions: Arc<RwLock<HashMap<i32, Vec<Transaction>>>>,
    pub csv_convert: Arc<RwLock<HashMap<i32, CSVConverter>>>,
    pub contracts: Arc<RwLock<HashMap<i32, Vec<Contract>>>>,
    pub current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
}

impl AppState {
    pub async fn set_app_state(
        &self,
        cookie_user_id: i32,
        new_banks: Option<Vec<Bank>>,
        new_transactions: Option<HashMap<i32, Vec<Transaction>>>,
        new_csv_converters: Option<HashMap<i32, CSVConverter>>,
        new_contracts: Option<HashMap<i32, Vec<Contract>>>,
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

        if let Some(contracts) = new_contracts {
            self.update_contracts(contracts).await;
        }

        if let Some(current_bank) = new_current_bank {
            self.update_current_bank(cookie_user_id, Some(current_bank))
                .await;
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

    pub async fn update_contracts(&self, new_contracts: HashMap<i32, Vec<Contract>>) {
        let mut contracts_state = self.contracts.write().await;

        info!(
            "Contracts length before update: {}",
            contracts_state.values().flatten().count()
        );

        for (bank_id, bank_contract) in new_contracts.iter() {
            let existing_contract = contracts_state.entry(*bank_id).or_insert_with(Vec::new);

            for contract in bank_contract.iter() {
                if !existing_contract.into_iter().any(|c| c.id == contract.id) {
                    existing_contract.push(contract.clone());
                }
            }
        }

        info!(
            "Contracts length after update: {}",
            contracts_state.values().flatten().count()
        );
    }

    pub async fn update_csv_converters(&self, new_csv_converters: HashMap<i32, CSVConverter>) {
        let mut csv_converters_state = self.csv_convert.write().await;

        info!(
            "CSV converters length before update: {}",
            csv_converters_state.len()
        );

        for (bank_id, csv_converter) in new_csv_converters.iter() {
            csv_converters_state.insert(*bank_id, csv_converter.clone());
        }

        info!(
            "CSV converters length after update: {:?}",
            csv_converters_state.len()
        );
    }

    pub async fn update_current_bank(&self, cookie_user_id: i32, current_bank: Option<Bank>) {
        let mut current_bank_state = self.current_bank.write().await;

        if let Some(bank_of_user) = current_bank_state.get(&cookie_user_id) {
            match current_bank {
                Some(bank) => {
                    if bank_of_user.id != bank.id {
                        current_bank_state.insert(cookie_user_id, bank.clone());
                        info!("Current bank updated: {:?}", bank);
                    }
                }
                None => {
                    current_bank_state.remove(&cookie_user_id);
                    info!("Current bank removed");
                }
            }
        } else {
            match current_bank {
                Some(bank) => {
                    current_bank_state.insert(cookie_user_id, bank.clone());
                    info!("Current bank set for the first time: {:?}", bank);
                }
                None => {
                    info!("Current bank not set");
                }
            }
        }
    }
}
