use once_cell::sync::Lazy;
use std::{collections::HashMap, fs, path::Path};

use super::language::Language;

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
