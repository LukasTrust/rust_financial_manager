use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::{time::Date, FromForm};
use serde::Serialize;

#[derive(FromForm)]
pub struct FormUser {
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct FormBank {
    pub name: String,
    pub link: Option<String>,
    pub date_column: Option<i32>,
    pub counterparty_column: Option<i32>,
    pub amount_column: Option<i32>,
    pub bank_balance_after_column: Option<i32>,
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
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
    pub bank_balance_after: f64,
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
}

impl Default for PerformanceData {
    fn default() -> Self {
        PerformanceData {
            total_transactions: 0,
            average_transaction_amount: 0.0,
            net_gain_loss: 0.0,
            performance_percentage: 0.0,
        }
    }
}
