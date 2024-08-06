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
use routes::error_page::error_page;
use routes::error_page::not_found;
use routes::home::{dashboard, home, logout, settings};
use routes::login::{login_form, login_user};
use routes::register::{login_form_from_register, register_form, register_user};
use routes::update_csv::{
    update_amount, update_bank_balance_after, update_counterparty, update_date,
};
use routes::upload_csv::upload_csv;
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_state = AppState {
        banks: Arc::new(RwLock::new(HashMap::new())),
        transactions: Arc::new(RwLock::new(HashMap::new())),
        csv_convert: Arc::new(RwLock::new(HashMap::new())),
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
                login_form_from_register,
                // Login
                login_form,
                login_user,
                // Home
                home,
                logout,
                dashboard,
                settings,
                // Bank
                add_bank,
                add_bank_form,
                bank_view,
                update_amount,
                update_bank_balance_after,
                update_counterparty,
                update_date,
                upload_csv,
                // Error page
                error_page
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
        .register("/", catchers![not_found])
}
