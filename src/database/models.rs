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

#[derive(Queryable, Insertable, Debug, Deserialize, Serialize, Clone)]
#[diesel(table_name = banks)]
pub struct NewBank {
    pub user_id: i32,
    pub name: String,
    pub link: Option<String>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = transactions)]
pub struct NewTransactions {
    pub bank_id: i32,
    pub date: NaiveDate,
    pub counterparty: String,
    pub amount: f64,
    pub bank_current_balance_after: f64,
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = csv_converters)]
pub struct CSVConverter {
    pub id: i32,
    pub csv_bank_id: i32,
    pub date_conv: Option<String>,
    pub counterparty_conv: Option<String>,
    pub amount_conv: Option<String>,
    pub bank_current_balance_after_conv: Option<String>,
}
