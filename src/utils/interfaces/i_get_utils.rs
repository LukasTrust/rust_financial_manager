use async_trait::async_trait;
use chrono::NaiveDate;
use rocket::{http::CookieJar, serde::json::Json};
use rocket_db_pools::Connection;

use crate::database::db_connector::DbConn;
use crate::database::models::Transaction;
use crate::utils::classes::language::Language;
use crate::utils::structs::{Bank, ErrorResponse, PerformanceData};

// Updated trait with instance methods
#[async_trait]
pub trait IGetUtils: Send + Sync {
    fn get_user_id(&self, cookies: &CookieJar<'_>) -> Result<i32, Json<ErrorResponse>>;

    fn get_user_language(&self, cookies: &CookieJar<'_>) -> Language;

    fn get_user_id_and_language(
        &self,
        cookies: &CookieJar<'_>,
    ) -> Result<(i32, Language), Json<ErrorResponse>>;

    fn get_first_date_and_last_date_from_bank(
        &self,
        transactions: Option<&Vec<Transaction>>,
    ) -> (NaiveDate, NaiveDate);

    async fn get_performance_value_and_graph_data(
        &self,
        banks: &Vec<Bank>,
        input_first_date: Option<NaiveDate>,
        input_last_date: Option<NaiveDate>,
        language: Language,
        db: Connection<DbConn>,
    ) -> Result<(PerformanceData, String), Json<ErrorResponse>>;

    async fn get_total_amount_paid_of_contract(
        &self,
        contract_id: i32,
        language: Language,
        db: &mut Connection<DbConn>,
    ) -> Result<f64, Json<ErrorResponse>>;

    async fn get_contracts_with_history(
        &self,
        bank_id: i32,
        language: Language,
        db: &mut Connection<DbConn>,
    ) -> Result<String, Json<ErrorResponse>>;

    async fn get_transactions_with_contract(
        &self,
        bank_id: i32,
        language: Language,
        db: Connection<DbConn>,
    ) -> Result<String, Json<ErrorResponse>>;
}
