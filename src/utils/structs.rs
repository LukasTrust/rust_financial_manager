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
    pub counterparty: Option<String>,
    pub amount: Option<String>,
    pub bank_balance_after: Option<String>,
    pub date: Option<String>,
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
    pub bank_current_balance_after: f64,
}
