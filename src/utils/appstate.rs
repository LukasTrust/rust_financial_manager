use log::info;
use rocket::tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use super::structs::Bank;

#[derive(Debug, Clone)]
pub struct AppState {
    pub current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
    pub use_mocking: bool,
}

impl AppState {
    pub fn new(use_mocking: bool) -> AppState {
        AppState {
            current_bank: Arc::new(RwLock::new(HashMap::new())),
            use_mocking: use_mocking,
        }
    }

    pub async fn get_current_bank(&self, cookie_user_id: i32) -> Option<Bank> {
        let current_bank_state = self.current_bank.read().await;
        match current_bank_state.get(&cookie_user_id) {
            Some(bank) => Some(bank.clone()),
            None => None,
        }
    }

    pub async fn set_current_bank(&self, cookie_user_id: i32, current_bank: Option<Bank>) {
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
