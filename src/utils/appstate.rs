use log::info;
use once_cell::sync::Lazy;
use rocket::{serde::json::Json, tokio::sync::RwLock};
use serde::Serialize;
use std::{collections::HashMap, fs, path::Path, sync::Arc};

// Assuming that structs::Bank is correctly imported
use super::structs::{Bank, ResponseData};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Copy)]
pub enum Language {
    English,
    German,
}

#[derive(Debug, Clone)]
pub struct AppState {
    current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
    user_language: Arc<RwLock<HashMap<i32, Language>>>,
    pub use_mocking: bool,
}

impl AppState {
    pub fn new(use_mocking: bool) -> Self {
        AppState {
            current_bank: Arc::new(RwLock::new(HashMap::new())),
            user_language: Arc::new(RwLock::new(HashMap::new())),
            use_mocking,
        }
    }

    pub async fn get_user_language(&self, user_id: i32) -> Language {
        let user_languages = self.user_language.read().await;

        let user_language = user_languages.get(&user_id).cloned();

        user_language.unwrap_or(Language::English)
    }

    pub async fn set_user_language(&self, user_id: i32, language: Language) {
        let mut user_languages = self.user_language.write().await;
        user_languages.insert(user_id, language.clone());
        info!("User language updated: {:?}", language);
    }

    pub async fn get_current_bank(&self, cookie_user_id: i32) -> Result<Bank, Json<ResponseData>> {
        let current_bank_state = self.current_bank.read().await;
        let current_bank = current_bank_state.get(&cookie_user_id).cloned();

        match current_bank {
            Some(bank) => Ok(bank),
            None => {
                let language = self.get_user_language(cookie_user_id).await;
                Err(Json(ResponseData::new_error(
                    LOCALIZATION.get_localized_string(language, "no_bank_selected"),
                    LOCALIZATION.get_localized_string(language, "no_bank_selected_details"),
                )))
            }
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
        let file_content = fs::read_to_string(Path::new(language_file)).expect(&format!(
            "Could not read localization file: {}",
            language_file
        ));
        serde_json::from_str(&file_content).expect("Error parsing the localization JSON file")
    }

    pub fn get_localized_string(&self, language: Language, key: &str) -> String {
        self.localized_strings
            .get(&language)
            .and_then(|map| map.get(key))
            .cloned()
            .unwrap_or_else(|| "Unknown key.".to_string())
    }
}
