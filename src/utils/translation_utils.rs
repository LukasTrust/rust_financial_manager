use std::collections::HashMap;

use super::appstate::{Language, LOCALIZATION};

pub fn get_base_localized_strings(language: Language) -> HashMap<&'static str, String> {
    let keys = vec![
        "base_dashboard",
        "base_add_new_bank",
        "base_settings",
        "base_logout",
        "base_contracts",
        "base_transactions",
    ];

    let mut localized_strings = HashMap::new();

    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}

pub fn get_add_bank_localized_strings(language: Language) -> HashMap<&'static str, String> {
    let keys = vec![
        "add_bank_title",
        "add_bank_form_name_label",
        "add_bank_form_link_label",
        "add_bank_csv_headers_title",
        "add_bank_counterparty_column_label",
        "add_bank_amount_column_label",
        "add_bank_balance_after_column_label",
        "add_bank_date_column_label",
        "add_bank_submit_button",
    ];

    let mut localized_strings = HashMap::new();

    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}

pub fn get_bank_contract_localized_strings(language: Language) -> HashMap<&'static str, String> {
    let keys = vec![
        "bank_contract_title",
        "bank_contract_merge_selected_button",
        "bank_contract_delete_selected_button",
        "bank_contract_scan_button",
        "bank_contract_toggle_closed_contracts_button",
        "bank_contract_show_open_contracts_text",
        "bank_contract_show_closed_contracts_text",
    ];

    let mut localized_strings = HashMap::new();

    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}
