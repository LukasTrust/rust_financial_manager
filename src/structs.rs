use rocket::tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use crate::database::models::{Bank, CSVConverter, Transaction};

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<HashMap<i32, Vec<Bank>>>>,
    pub transactions: Arc<RwLock<HashMap<i32, Vec<Transaction>>>>,
    pub csv_convert: Arc<RwLock<HashMap<i32, CSVConverter>>>,
    pub current_bank: Arc<RwLock<HashMap<i32, Bank>>>,
}
