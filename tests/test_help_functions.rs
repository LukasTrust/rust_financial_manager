use bcrypt::{hash, DEFAULT_COST};
use rocket::fs::{relative, FileServer};
use rocket::local::asynchronous::Client;
use rocket::{catchers, get, tokio};
use rocket_db_pools::{Connection, Database};
use rocket_dyn_templates::Template;
use routes::add_bank::{add_bank, add_bank_form};
use routes::bank::bank_view;
use routes::bank_contract::{
    bank_contact_display, bank_contract, bank_contract_delete, bank_contract_merge,
    bank_contract_name_changed, bank_scan_for_new_contracts,
};
use routes::bank_transaction::{
    bank_transaction, transaction_add_to_contract, transaction_allow_contract, transaction_hide,
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
use rust_financial_manager::database::db_connector::DbConn;
use rust_financial_manager::database::models::{NewBank, NewUser, User};
use rust_financial_manager::routes;
use rust_financial_manager::routes::bank_transaction::{
    transaction_set_old_amount, transaction_update_contract_amount,
};
use rust_financial_manager::routes::get_data::get_graph_data;
use rust_financial_manager::routes::settings::set_user_language;
use rust_financial_manager::utils::appstate::{AppState, Language};
use rust_financial_manager::utils::delete_utils::{delete_bank_by_name, delete_user_by_email};
use rust_financial_manager::utils::insert_utiles::{insert_bank, insert_user};
use rust_financial_manager::utils::loading_utils::load_user_by_email;
use std::env;
use tokio::sync::OnceCell;

// Static variables for the client and user
static CLIENT: OnceCell<Client> = OnceCell::const_new();
static LOADED_USER: OnceCell<User> = OnceCell::const_new();

pub async fn get_test_client() -> &'static Client {
    CLIENT.get_or_init(init_client).await
}

pub fn get_loaded_user() -> Option<&'static User> {
    LOADED_USER.get()
}

async fn init_client() -> Client {
    let app_state = AppState::default();

    let test_rocket = rocket::build()
        .manage(app_state)
        .attach(DbConn::init())
        .attach(Template::fairing())
        .mount("/", rocket::routes![set_up_test_data])
        .mount("/static", FileServer::from(relative!("static")).rank(11))
        .register("/", catchers![not_found]);

    let client = Client::tracked(test_rocket).await.unwrap();

    let result = client.get("/set_up_test_data").dispatch().await;

    assert_eq!(result.status(), rocket::http::Status::Ok);

    let app_state = AppState::default();

    let rocket = rocket::build()
        .manage(app_state)
        .attach(DbConn::init())
        .attach(Template::fairing())
        .mount(
            "/",
            rocket::routes![
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
                // Get graph data
                get_graph_data,
                // Bank Contract
                bank_contract,
                bank_contact_display,
                bank_contract_merge,
                bank_contract_delete,
                bank_scan_for_new_contracts,
                bank_contract_name_changed,
                // Bank Transaction
                bank_transaction,
                transaction_remove,
                transaction_add_to_contract,
                transaction_set_old_amount,
                transaction_update_contract_amount,
                transaction_hide,
                transaction_show,
                transaction_not_allow_contract,
                transaction_allow_contract,
                set_up_test_data,
                // Settings
                set_user_language,
            ],
        )
        .mount("/static", FileServer::from(relative!("static")).rank(11))
        .register("/", catchers![not_found]);

    Client::tracked(rocket).await.unwrap()
}

#[get("/set_up_test_data")]
async fn set_up_test_data(mut db: Connection<DbConn>) {
    let _ = delete_user_by_email("success@mail.com".to_string(), &mut db).await;
    let _ = delete_user_by_email("copy_email@mail.com".to_string(), &mut db).await;
    let _ = delete_user_by_email("user_exists@mail.com".to_string(), &mut db).await;
    let _ = delete_user_by_email("wrong_password@mail.com".to_string(), &mut db).await;
    let _ = delete_bank_by_name("copy_bank".to_string(), &mut db).await;
    let _ = delete_bank_by_name("error_loading_banks".to_string(), &mut db).await;
    let _ = delete_bank_by_name("csv_error".to_string(), &mut db).await;
    let _ = delete_bank_by_name("Test_Bank".to_string(), &mut db).await;

    let user = NewUser {
        first_name: "Copy".to_string(),
        last_name: "Doe".to_string(),
        email: "copy_email@mail.com".to_string(),
        password: "password=S3cureP@ssw0rd!".to_string(),
    };

    let result = insert_user(user, &mut db).await;

    assert!(result.is_ok());

    let hash = hash("Password123".to_string(), DEFAULT_COST).unwrap();

    let user = NewUser {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "user_exists@mail.com".to_string(),
        password: hash,
    };

    let result = insert_user(user, &mut db).await;

    assert!(result.is_ok());

    let inserted_user = load_user_by_email("user_exists@mail.com", &mut db)
        .await
        .unwrap();

    let user = NewUser {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: "wrong_password@mail.com".to_string(),
        password: "wrong".to_string(),
    };

    let result = insert_user(user, &mut db).await;

    assert!(result.is_ok());

    let bank = NewBank {
        user_id: inserted_user.id,
        name: "copy_bank".to_string(),
        link: Some("http://test-bank.com".to_string()),
    };

    let result = insert_bank(bank, Language::English, &mut db).await;

    assert!(result.is_ok());

    // Save the loaded user in the static variable
    LOADED_USER
        .set(inserted_user)
        .expect("Failed to set LOADED_USER");
}
