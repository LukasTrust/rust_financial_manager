use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::{time::Date, FromForm};
use serde::Serialize;
use std::str::FromStr;

use crate::schema::{banks, csv_converters, transactions, users};

#[derive(FromForm)]
pub struct FormUser {
    pub email: String,
    pub password: String,
}

#[derive(FromForm, Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct FormBank {
    pub name: String,
    pub link: Option<String>,
    pub current_amount: f64,
    pub interest_rate: Option<f64>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = banks)]
pub struct NewBank {
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
    pub current_amount: f64,
    pub interest_rate: Option<f64>,
}

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Bank {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
    pub current_amount: Option<f64>,
    pub interest_rate: Option<f64>,
}

#[derive(FromForm)]
pub struct FormTransactions {
    pub date: Date,
    pub counterparty: String,
    pub amount: f64,
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransactions {
    pub bank_id: i32,
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
}

#[derive(Debug, Queryable, Serialize, Clone)]
pub struct Transaction {
    pub id: i32,
    pub bank_id: i32,
    pub type_of_t: String,
    pub date: NaiveDate,
    pub counterparty: Option<String>,
    pub comment: Option<String>,
    pub amount: f64,
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = csv_converters)]
pub struct CSVConverter {
    pub id: i32,
    pub csv_bank_id: i32,
    pub date_conv: Option<String>,
    pub counterparty_conv: Option<String>,
    pub amount_conv: Option<String>,
}
