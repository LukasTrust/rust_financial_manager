#[macro_use]
extern crate rocket;

use env_logger::Env;
use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::RwLock;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;
use rust_financial_manager::utils::appstate::AppState;
use std::collections::HashMap;
use std::sync::Arc;

use database::db_connector::DbConn;
use routes::add_bank::{add_bank, add_bank_form};
use routes::bank::bank_view;
use routes::bank_contract::{bank_contract, bank_contract_merge};
use routes::bank_transaction::{
    bank_transaction, transaction_add, transaction_allow_contract, transaction_hide,
    transaction_not_allow_contract, transaction_remove, transaction_show,
};
use routes::base::{base, dashboard, logout, settings};
use routes::error_page::error_page;
use routes::error_page::not_found;
use routes::login::{login_form, login_from_register, login_user};
use routes::register::{register_form, register_user};
use routes::update_csv::update_csv;
use routes::update_date_range::update_date_range;
use routes::upload_csv::upload_csv;
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_state = AppState {
        current_bank: Arc::new(RwLock::new(HashMap::new())),
    };

    rocket::build()
        .manage(app_state)
        .attach(DbConn::init())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                // Register
                register_form,
                register_user,
                // Login
                login_form,
                login_user,
                login_from_register,
                // base
                base,
                logout,
                dashboard,
                settings,
                // Add bank
                add_bank,
                add_bank_form,
                // Bank
                bank_view,
                // Update CSV
                update_csv,
                // Upload CSV
                upload_csv,
                // Error page
                error_page,
                // Update date range
                update_date_range,
                // Bank Contract
                bank_contract,
                bank_contract_merge,
                // Bank Transaction
                bank_transaction,
                transaction_remove,
                transaction_add,
                transaction_hide,
                transaction_show,
                transaction_not_allow_contract,
                transaction_allow_contract,
            ],
        )
        .mount("/static", FileServer::from(relative!("static")).rank(11))
        .register("/", catchers![not_found])
}
