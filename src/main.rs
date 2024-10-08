#[macro_use]
extern crate rocket;

use env_logger::Env;
use rocket::fs::FileServer;
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use database::db_connector::DbConn;
use routes::add_bank::{add_bank, add_bank_form};
use routes::bank::bank_view;
use routes::bank_contract::{
    bank_contact_data, bank_contract, bank_contract_delete, bank_contract_merge,
    bank_contract_name_changed, bank_scan_for_new_contracts,
};
use routes::bank_transaction::{
    bank_transaction, transaction_add_to_contract, transaction_allow_contract, transaction_hide,
    transaction_not_allow_contract, transaction_remove, transaction_show,
};
use routes::base::{base, dashboard, logout};
use routes::error_page::error_page;
use routes::error_page::not_found;
use routes::login::{login_form, login_from_register, login_user};
use routes::register::{register_form, register_user};
use routes::update_csv::update_csv;
use routes::update_date_range::update_date_range;
use routes::upload_csv::upload_csv;
use rust_financial_manager::routes::bank::delete_bank;
use rust_financial_manager::routes::bank_transaction::{
    bank_transaction_data, transaction_set_old_amount, transaction_update_contract_amount,
};
use rust_financial_manager::routes::get_data::get_graph_data;
use rust_financial_manager::routes::settings::{
    change_password, delete_account, set_user_language, settings,
};
use rust_financial_manager::utils::appstate::AppState;
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let app_state = AppState::default();

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
                delete_bank,
                // Update CSV
                update_csv,
                // Upload CSV
                upload_csv,
                // Error page
                error_page,
                // Update date range
                update_date_range,
                // Get graph data
                get_graph_data,
                // Bank Contract
                bank_contract,
                bank_contact_data,
                bank_contract_merge,
                bank_contract_delete,
                bank_scan_for_new_contracts,
                bank_contract_name_changed,
                // Bank Transaction
                bank_transaction,
                bank_transaction_data,
                transaction_remove,
                transaction_add_to_contract,
                transaction_set_old_amount,
                transaction_update_contract_amount,
                transaction_hide,
                transaction_show,
                transaction_not_allow_contract,
                transaction_allow_contract,
                // Settings
                set_user_language,
                change_password,
                delete_account
            ],
        )
        .mount("/static", FileServer::from("./static").rank(11))
        .register("/", catchers![not_found])
}
