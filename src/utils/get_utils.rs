use chrono::NaiveDate;
use log::error;
use rocket::{http::CookieJar, serde::json::Json};
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;

use super::{
    appstate::Language,
    display_utils::{generate_graph_data, generate_performance_value},
    loading_utils::{
        load_contract_history, load_contracts_of_bank, load_last_transaction_of_contract,
        load_transactions_of_bank, load_transactions_of_contract,
    },
    structs::{
        Bank, ContractWithHistory, ErrorResponse, PerformanceData, Transaction,
        TransactionWithContract,
    },
};

pub fn get_user_id(cookies: &CookieJar<'_>) -> Result<i32, Json<ErrorResponse>> {
    let user_id = cookies
        .get_private("user_id")
        .and_then(|cookie| cookie.value().parse().ok());

    if user_id.is_none() {
        error!("User ID not found in cookies.");
        return Err(Json(ErrorResponse::new(
            "User ID not found in cookies.".to_string(),
            "Please login again.".to_string(),
        )));
    }

    Ok(user_id.unwrap())
}

pub fn get_user_language(cookies: &CookieJar<'_>) -> Language {
    let language = cookies.get_private("language").and_then(|cookie| {
        let language = cookie.value();

        match language {
            "English" => Some(Language::English),
            "German" => Some(Language::German),
            _ => None,
        }
    });

    if language.is_none() {
        error!("Language not found in cookies.");
        return Language::English;
    }

    language.unwrap()
}

pub fn get_user_id_and_language(
    cookies: &CookieJar<'_>,
) -> Result<(i32, Language), Json<ErrorResponse>> {
    let user_id = get_user_id(cookies)?;
    let language = get_user_language(cookies);

    Ok((user_id, language))
}

pub fn get_first_date_and_last_date_from_bank(
    transactions: Option<&Vec<Transaction>>,
) -> (NaiveDate, NaiveDate) {
    let mut first_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
    let mut last_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

    if transactions.is_some() {
        let transactions = transactions.unwrap();

        first_date = transactions
            .iter()
            .min_by_key(|t| t.date)
            .map(|t| t.date)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());

        last_date = transactions
            .iter()
            .max_by_key(|t| t.date)
            .map(|t| t.date)
            .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap());
    }

    (first_date, last_date)
}

pub async fn get_performance_value_and_graph_data(
    banks: &Vec<Bank>,
    input_first_date: Option<NaiveDate>,
    input_last_date: Option<NaiveDate>,
    language: Language,
    mut db: Connection<DbConn>,
) -> Result<(PerformanceData, String), Json<ErrorResponse>> {
    let mut all_transactions = Vec::new();
    let mut all_contracts = Vec::new();

    for bank in banks {
        let transactions = load_transactions_of_bank(bank.id, language, &mut db).await?;

        all_transactions.extend(transactions);

        let contracts = load_contracts_of_bank(bank.id, language, &mut db).await?;

        all_contracts.extend(contracts);
    }

    let (first_date, last_date);

    if input_first_date.is_none() || input_last_date.is_none() {
        (first_date, last_date) = get_first_date_and_last_date_from_bank(Some(&all_transactions));
    } else {
        first_date = input_first_date.unwrap();
        last_date = input_last_date.unwrap();
    }

    let performance_value =
        generate_performance_value(&all_transactions, &all_contracts, &first_date, &last_date);

    let graph_data = generate_graph_data(
        banks,
        &all_transactions,
        &performance_value.1,
        language,
        &first_date,
        &last_date,
    )
    .await;

    Ok((performance_value.0, graph_data))
}

pub async fn get_total_amount_paid_of_contract(
    contract_id: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<f64, Json<ErrorResponse>> {
    let transactions = load_transactions_of_contract(contract_id, language, db).await?;

    Ok(transactions.iter().map(|t| t.amount).sum())
}

pub async fn get_contracts_with_history(
    bank_id: i32,
    language: Language,
    db: &mut Connection<DbConn>,
) -> Result<String, Json<ErrorResponse>> {
    let mut contracts_with_history: Vec<ContractWithHistory> = Vec::new();

    let contracts = load_contracts_of_bank(bank_id, language, db).await?;

    for contract in contracts.iter() {
        let mut contract_history = load_contract_history(contract.id, language, db).await?;

        contract_history.sort_by(|a, b| b.changed_at.cmp(&a.changed_at));

        let total_amount_paid =
            get_total_amount_paid_of_contract(contract.id, language, db).await?;

        let last_transaction = load_last_transaction_of_contract(contract.id, language, db).await?;

        let last_payment_date = last_transaction.date;

        let contract_with_history = ContractWithHistory {
            contract: contract.clone(),
            contract_history,
            total_amount_paid,
            last_payment_date,
        };

        contracts_with_history.push(contract_with_history);
    }

    Ok(serde_json::to_string(&contracts_with_history).unwrap())
}

pub async fn get_transactions_with_contract(
    bank_id: i32,
    language: Language,
    mut db: Connection<DbConn>,
) -> Result<String, Json<ErrorResponse>> {
    let transactions = load_transactions_of_bank(bank_id, language, &mut db).await?;

    let mut transactions_with_contract = Vec::new();
    let contracts = load_contracts_of_bank(bank_id, language, &mut db).await?;

    for transaction in transactions.iter() {
        let contract = if transaction.contract_id.is_some() {
            contracts
                .iter()
                .find(|c| c.id == transaction.contract_id.unwrap())
        } else {
            None
        };

        let transaction_with_contract = TransactionWithContract {
            transaction: transaction.clone(),
            contract: contract.cloned(),
        };

        transactions_with_contract.push(transaction_with_contract);
    }

    Ok(serde_json::to_string(&transactions_with_contract).unwrap())
}
