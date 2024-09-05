use std::collections::{BTreeMap, HashMap};

use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::{time::Date, FromForm};
use serde::Serialize;

use crate::database::models::{Contract, ContractHistory};

pub type DataTuple = (f64, String, f64, Option<f64>);
pub type DataList = Vec<DataTuple>;
pub type DataMap = BTreeMap<NaiveDate, DataList>;
pub type ContractData = (f64, NaiveDate, i32);
pub type ContractList = Vec<ContractData>;
pub type ContractMapById = HashMap<i64, ContractList>;
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
    pub total_transactions: usize,
    pub average_transaction_amount: f64,
    pub net_gain_loss: f64,
    pub performance_percentage: f64,
    pub total_discrepancy: f64,
    pub total_contracts: usize,
    pub one_month_contract_amount: f64,
    pub three_month_contract_amount: f64,
    pub six_month_contract_amount: f64,
    pub total_amount_per_year: f64,
}

impl Default for PerformanceData {
    fn default() -> Self {
        PerformanceData {
            total_transactions: 0,
            average_transaction_amount: 0.0,
            net_gain_loss: 0.0,
            performance_percentage: 0.0,
            total_discrepancy: 0.0,
            total_contracts: 0,
            one_month_contract_amount: 0.0,
            three_month_contract_amount: 0.0,
            six_month_contract_amount: 0.0,
            total_amount_per_year: 0.0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResponseData {
    header: String,
    success: Option<String>,
    error: Option<String>,
}

impl ResponseData {
    pub fn new_success(header: String, success: &str) -> Self {
        ResponseData {
            header: header,
            success: Some(success.to_string()),
            error: None,
        }
    }

    pub fn new_error(header: String, error: &str) -> Self {
        ResponseData {
            header: header,
            success: None,
            error: Some(error.to_string()),
        }
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
