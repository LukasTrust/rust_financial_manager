use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::FromForm;
use serde::{Deserialize, Serialize};

use crate::schema::{banks, csv_converters, transactions, users};

#[derive(FromForm, Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, Insertable, Debug, Deserialize, Serialize, Clone)]
#[diesel(table_name = banks)]
pub struct NewBank {
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
}

#[derive(Insertable, Debug, Queryable)]
#[diesel(table_name = transactions)]
pub struct NewTransactions {
    pub bank_id: i32,
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
    pub bank_balance_after: f64,
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset, Copy)]
#[diesel(table_name = csv_converters)]
pub struct CSVConverter {
    pub id: i32,
    pub bank_id: i32,
    pub date_column: Option<i32>,
    pub counterparty_column: Option<i32>,
    pub amount_column: Option<i32>,
    pub bank_balance_after_column: Option<i32>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = csv_converters)]
pub struct NewCSVConverter {
    pub bank_id: i32,
    pub date_column: Option<i32>,
    pub counterparty_column: Option<i32>,
    pub amount_column: Option<i32>,
    pub bank_balance_after_column: Option<i32>,
}
