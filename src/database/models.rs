use chrono::NaiveDate;
use diesel::prelude::*;
use rocket::{time::Date, FromForm};

use crate::schema::{banks, transactions, users};

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
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
    pub start_date: Date,
    pub end_date: Date,
    pub current_amount: f64,
    pub interest_rate: Option<f64>,
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = banks)]
pub struct NewBank {
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub current_amount: f64,
    pub interest_rate: Option<f64>,
}

#[derive(FromForm)]
pub struct FormTransactions {
    pub bank_id: i32,
    pub type_of_t: String,
    pub date: Date,
    pub other: String,
    pub comment: String,
    pub amount: f64,
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransactions {
    pub bank_id: i32,
    pub type_of_t: String,
    pub date: NaiveDate,
    pub other: String,
    pub comment: String,
    pub amount: f64,
}
