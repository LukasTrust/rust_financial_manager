use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::FromForm;
use serde::{Deserialize, Serialize};

use crate::schema::{banks, contract_history, contracts, csv_converters, transactions, users};

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
pub struct NewTransaction {
    pub bank_id: i32,
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
    pub bank_balance_after: f64,
}

#[derive(Queryable, Debug, Clone, AsChangeset, Copy)]
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

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = contracts)]
pub struct NewContract {
    pub bank_id: i32,
    pub name: String,
    pub parse_name: String,
    pub current_amount: f64,
    pub months_between_payment: i32,
}

#[derive(Queryable, Insertable, Debug, Clone, Serialize)]
#[diesel(table_name = contracts)]
pub struct Contract {
    pub id: i32,
    pub bank_id: i32,
    pub name: String,
    pub parse_name: String,
    pub current_amount: f64,
    pub months_between_payment: i32,
    pub end_date: Option<NaiveDate>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = contract_history)]
pub struct NewContractHistory {
    pub contract_id: i32,
    pub old_amount: f64,
    pub new_amount: f64,
    pub changed_at: NaiveDate,
}

#[derive(Queryable, Debug, Clone, Serialize)]
#[diesel(table_name = contract_history)]
pub struct ContractHistory {
    pub id: i32,
    pub contract_id: i32,
    pub old_amount: f64,
    pub new_amount: f64,
    pub changed_at: NaiveDate,
}
