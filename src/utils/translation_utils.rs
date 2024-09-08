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

pub fn get_transactions_localized_strings(language: Language) -> HashMap<&'static str, String> {
    // List all the keys needed for the Transactions view localization.
    let keys = vec![
        "transactions_title",
        "transactions_select_date_range_label",
        "transactions_filter_by_contract_label",
        "transactions_all_contracts_option",
        "transactions_search_placeholder",
        "transactions_toggle_hidden_button",
        "transactions_hide_hidden_text",
        "transactions_show_hidden_text",
        "transactions_icon_header",
        "transactions_counterparty_header",
        "transactions_amount_header",
        "transactions_balance_header",
        "transactions_date_header",
        "transactions_contract_name_header",
        "transactions_contract_amount_header",
    ];

    // Create a HashMap to store the localized strings.
    let mut localized_strings = HashMap::new();

    // Retrieve and insert each localized string into the HashMap.
    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}

pub fn get_bank_localized_strings(language: Language) -> HashMap<&'static str, String> {
    // List all the keys needed for the Bank view localization.
    let keys = vec![
        "bank_transaction_data_title",
        "bank_number_of_transactions_label",
        "bank_average_amount_label",
        "bank_net_gain_loss_label",
        "bank_performance_label",
        "bank_discrepancy_label",
        "bank_contract_data_title",
        "bank_number_of_contracts_label",
        "bank_total_amount_per_year_label",
        "bank_contracts_one_month_label",
        "bank_contracts_three_month_label",
        "bank_contracts_six_month_label",
        "bank_select_date_range_label",
        "bank_upload_button",
        "bank_update_csv_conversion_title",
        "bank_counterparty_column_label",
        "bank_amount_column_label",
        "bank_balance_after_column_label",
        "bank_date_column_label",
        "bank_change_button",
    ];

    // Create a HashMap to store the localized strings.
    let mut localized_strings = HashMap::new();

    // Retrieve and insert each localized string into the HashMap.
    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}

pub fn get_dashboard_localized_strings(language: Language) -> HashMap<&'static str, String> {
    // List all the keys needed for the Dashboard view localization.
    let keys = vec![
        "dashboard_select_date_range_label",
        "dashboard_transaction_data_title",
        "dashboard_number_of_transactions_label",
        "dashboard_average_amount_label",
        "dashboard_net_gain_loss_label",
        "dashboard_performance_label",
        "dashboard_discrepancy_label",
        "dashboard_contract_data_title",
        "dashboard_number_of_contracts_label",
        "dashboard_total_amount_per_year_label",
        "dashboard_contracts_one_month_label",
        "dashboard_contracts_three_month_label",
        "dashboard_contracts_six_month_label",
    ];

    // Create a HashMap to store the localized strings.
    let mut localized_strings = HashMap::new();

    // Retrieve and insert each localized string into the HashMap.
    for key in keys {
        let localized_string = LOCALIZATION.get_localized_string(language, key);
        localized_strings.insert(key, localized_string);
    }

    localized_strings
}
