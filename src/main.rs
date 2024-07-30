#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::sync::Arc;

use rocket::fs::{relative, FileServer};
use rocket::tokio::sync::RwLock;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use database::db_connector::DbConn;
use routes::home::{add_bank, add_bank_form, bank_view, dashboard, home, logout, settings};
use routes::login::{login_form, login_user};
use routes::register::{login_form_from_register, register_form, register_user};
use rust_financial_manager::routes::home::AppState;
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    let app_state = AppState {
        banks: Arc::new(RwLock::new(vec![])),
        transactions: Arc::new(RwLock::new(HashMap::new())),
        csvConverts: Arc::new(RwLock::new(HashMap::new())),
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
                add_bank,
                add_bank_form,
                bank_view
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
}
