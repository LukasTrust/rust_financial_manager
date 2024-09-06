use log::{error, info};
use rocket::tokio::sync::RwLock;
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

use super::structs::Bank;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum Language {
    English,
    German,
}

#[derive(Debug, Clone)]
pub struct AppState {
    current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
    user_language: Arc<RwLock<HashMap<i32, Language>>>,
    localized_strings: Arc<RwLock<HashMap<Language, HashMap<String, String>>>>,
    pub use_mocking: bool,
}

fn load_localized_strings(language_file: &str) -> HashMap<String, String> {
    let file_content = fs::read_to_string(Path::new(language_file)).expect(&format!(
        "Could not read localization file: {}",
        language_file
    ));
    serde_json::from_str(&file_content).expect("Error parsing the localization JSON file")
}

impl AppState {
    pub fn new(use_mocking: bool) -> Self {
        // Load localized strings for each language
        let mut localized_strings = HashMap::new();

        localized_strings.insert(
            Language::English,
            load_localized_strings("static/locales/en.json"),
        );
        localized_strings.insert(
            Language::German,
            load_localized_strings("static/locales/de.json"),
        );

        // Wrap the localized_strings in Arc and RwLock for thread-safe access
        let localized_strings = Arc::new(RwLock::new(localized_strings));

        // Initialize AppState
        AppState {
            current_bank: Arc::new(RwLock::new(HashMap::new())),
            user_language: Arc::new(RwLock::new(HashMap::new())),
            use_mocking,
            localized_strings,
        }
    }

    pub async fn get_user_language(&self, user_id: i32) -> Option<Language> {
        let user_languages = self.user_language.read().await;
        user_languages.get(&user_id).cloned()
    }

    pub async fn set_user_language(&self, user_id: i32, language: Language) {
        let mut user_languages = self.user_language.write().await;
        user_languages.insert(user_id, language.clone());
        info!("User language updated: {:?}", language);
    }

    pub async fn get_current_bank(&self, cookie_user_id: i32) -> Option<Bank> {
        let current_bank_state = self.current_bank.read().await;
        current_bank_state.get(&cookie_user_id).cloned()
    }

    pub async fn localize_message(&self, user_id: i32, key: &str) -> String {
        let start = std::time::Instant::now();
        // Acquire a read lock on the localized strings
        let localized_strings = self.localized_strings.read().await;

        // Get the user's language
        let language = self
            .get_user_language(user_id)
            .await
            .unwrap_or(Language::English);

        // Get the HashMap of localized strings for the given language
        if let Some(strings) = localized_strings.get(&language) {
            if start.elapsed().as_millis() > 10 {
                error!(
                    "Localization took too long: {} ms",
                    start.elapsed().as_millis()
                );
            }
            strings
                .get(key)
                .unwrap_or(&"Unknown key.".to_string())
                .to_string()
        } else {
            if start.elapsed().as_millis() > 10 {
                error!(
                    "Localization took too long: {} ms",
                    start.elapsed().as_millis()
                );
            }
            "Unknown language.".to_string()
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
