use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::{time::Date, FromForm};
use serde::Serialize;

use crate::database::models::{Contract, ContractHistory};

pub type DataTuple = (f64, String, f64, Option<f64>);
pub type DataMap = BTreeMap<NaiveDate, Vec<DataTuple>>;
pub type ContractMapById = HashMap<i64, Vec<Transaction>>;
pub type CounterpartyMap = HashMap<String, ContractMapById>;

#[derive(FromForm)]
pub struct FormUser {
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct FormBank {
    pub name: String,
    pub link: Option<String>,
    pub counterparty_column: Option<i32>,
    pub amount_column: Option<i32>,
    pub bank_balance_after_column: Option<i32>,
    pub date_column: Option<i32>,
}

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Bank {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
}

impl Default for Bank {
    fn default() -> Self {
        Bank {
            id: 0,
            user_id: 0,
            name: "".to_string(),
            link: None,
        }
    }
}

#[derive(FromForm)]
pub struct FormTransactions {
    pub date: Date,
    pub counterparty: String,
    pub amount: f64,
    pub current_amount_after: f64,
}

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Transaction {
    pub id: i32,
    pub bank_id: i32,
    pub contract_id: Option<i32>,
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
    pub bank_balance_after: f64,
    pub is_hidden: bool,
    pub contract_not_allowed: bool,
}

#[derive(Debug)]
pub struct Discrepancy {
    pub transaction_id: i32,
    pub discrepancy_amount: f64,
}

#[derive(Debug, FromForm)]
pub struct DateRangeForm {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PerformanceData {
    transactions_count: usize,
    transactions_average_amount: f64,
    transactions_max_amount: f64,
    transactions_min_amount: f64,
    transactions_net_gain_loss: f64,
    transactions_total_discrepancy: f64,
    contracts_count: usize,
    contracts_average_amount: f64,
    contracts_max_amount: f64,
    contracts_min_amount: f64,
    contracts_amount_per_time_span: f64,
    contracts_amount_per_year: f64,
}

impl PerformanceData {
    pub fn new(
        only_transaction: PerformanceData,
        only_contract: PerformanceData,
        contracts_amount_per_time_span: f64,
    ) -> PerformanceData {
        PerformanceData {
            transactions_count: only_transaction.transactions_count,
            transactions_average_amount: only_transaction.transactions_average_amount,
            transactions_max_amount: only_transaction.transactions_max_amount,
            transactions_min_amount: only_transaction.transactions_min_amount,
            transactions_net_gain_loss: only_transaction.transactions_net_gain_loss,
            transactions_total_discrepancy: only_transaction.transactions_total_discrepancy,
            contracts_count: only_contract.contracts_count,
            contracts_average_amount: only_contract.contracts_average_amount,
            contracts_max_amount: only_contract.contracts_max_amount,
            contracts_min_amount: only_contract.contracts_min_amount,
            contracts_amount_per_time_span,
            contracts_amount_per_year: only_contract.contracts_amount_per_year,
        }
    }

    pub fn new_only_transaction(
        transactions_count: usize,
        transactions_average_amount: f64,
        transactions_max_amount: f64,
        transactions_min_amount: f64,
        transactions_net_gain_loss: f64,
        transactions_total_discrepancy: f64,
    ) -> PerformanceData {
        PerformanceData {
            transactions_count,
            transactions_average_amount,
            transactions_max_amount,
            transactions_min_amount,
            transactions_net_gain_loss,
            transactions_total_discrepancy,
            ..Default::default()
        }
    }

    pub fn new_only_contract(
        contracts_count: usize,
        contracts_average_amount: f64,
        contracts_max_amount: f64,
        contracts_min_amount: f64,
        contracts_amount_per_year: f64,
    ) -> PerformanceData {
        PerformanceData {
            contracts_count,
            contracts_average_amount,
            contracts_max_amount,
            contracts_min_amount,
            contracts_amount_per_year,
            ..Default::default()
        }
    }
}

impl Default for PerformanceData {
    fn default() -> PerformanceData {
        PerformanceData {
            transactions_count: 0,
            transactions_average_amount: 0.0,
            transactions_max_amount: 0.0,
            transactions_min_amount: 0.0,
            transactions_net_gain_loss: 0.0,
            transactions_total_discrepancy: 0.0,
            contracts_count: 0,
            contracts_average_amount: 0.0,
            contracts_max_amount: 0.0,
            contracts_min_amount: 0.0,
            contracts_amount_per_time_span: 0.0,
            contracts_amount_per_year: 0.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    header: String,
    error: String,
}

impl ErrorResponse {
    pub fn new(header: String, error: String) -> ErrorResponse {
        ErrorResponse { header, error }
    }
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    header: String,
    success: String,
}

impl SuccessResponse {
    pub fn new(header: String, success: String) -> SuccessResponse {
        SuccessResponse { header, success }
    }
}

#[derive(Debug, Serialize)]
pub struct ContractWithHistory {
    pub contract: Contract,
    pub contract_history: Vec<ContractHistory>,
    pub total_amount_paid: f64,
    pub last_payment_date: NaiveDate,
}

#[derive(Debug, Serialize)]
pub struct TransactionWithContract {
    pub transaction: Transaction,
    pub contract: Option<Contract>,
}

#[derive(Debug, Serialize, FromForm)]
pub struct ChangePassword {
    pub old_password: String,
    pub new_password: String,
    pub confirm_password: String,
}
