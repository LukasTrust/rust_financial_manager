use core::fmt;
use log::info;
use once_cell::sync::Lazy;
use rocket::{serde::json::Json, tokio::sync::RwLock};
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

// Assuming that structs::Bank is correctly imported
use super::structs::{Bank, ErrorResponse};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Copy)]
pub enum Language {
    English,
    German,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            current_bank: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AppState {
    pub async fn get_current_bank(
        &self,
        cookie_user_id: i32,
        cookie_use_language: Language,
    ) -> Result<Bank, Json<ErrorResponse>> {
        let current_bank_state = self.current_bank.read().await;
        let current_bank = current_bank_state.get(&cookie_user_id).cloned();

        match current_bank {
            Some(bank) => Ok(bank),
            None => Err(Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(cookie_use_language, "no_bank_selected"),
                LOCALIZATION.get_localized_string(cookie_use_language, "no_bank_selected_details"),
            ))),
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

pub static LOCALIZATION: Lazy<Localization> = Lazy::new(Localization::new);

pub struct Localization {
    localized_strings: HashMap<Language, HashMap<String, String>>,
}

impl Localization {
    fn new() -> Self {
        let mut localized_strings = HashMap::new();

        localized_strings.insert(
            Language::English,
            Self::load_localized_strings("static/locales/en.json"),
        );
        localized_strings.insert(
            Language::German,
            Self::load_localized_strings("static/locales/de.json"),
        );

        Localization { localized_strings }
    }

    fn load_localized_strings(language_file: &str) -> HashMap<String, String> {
        let file_content = fs::read_to_string(Path::new(language_file))
            .unwrap_or_else(|_| panic!("Could not read localization file: {}", language_file));

        serde_json::from_str(&file_content).expect("Error parsing the localization JSON file")
    }

    pub fn get_localized_string(&self, cookie_use_language: Language, key: &str) -> String {
        self.localized_strings
            .get(&cookie_use_language)
            .and_then(|map| map.get(key))
            .cloned()
            .unwrap_or_else(|| "Unknown key.".to_string())
    }
}
