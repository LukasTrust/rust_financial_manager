use rocket::http::ContentType;
use rocket::local::asynchronous::{Client, LocalResponse};
use rocket::{routes, Build, Rocket};
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;
use rust_financial_manager::utils::structs::FormUser;
use urlencoding::encode;

use rust_financial_manager::database::db_connector::DbConn;
use rust_financial_manager::database::models::NewUser;
use rust_financial_manager::routes::delete_user::delete_user;
use rust_financial_manager::routes::home::home;
use rust_financial_manager::routes::login::{login_form, login_user};
use rust_financial_manager::routes::register::{
    login_form_from_register, register_form, register_user,
};

// Helper function to create a Rocket instance for testing
pub fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount(
            "/",
            routes![
                // Register routes
                login_form_from_register,
                register_form,
                register_user,
                delete_user,
                // Login routes
                login_user,
                login_form,
                // Home routes
                home
            ],
        )
        .attach(Template::fairing())
        .attach(DbConn::init())
}

// Helper function to create a test client asynchronously
pub async fn test_client() -> Client {
    Client::tracked(rocket())
        .await
        .expect("valid rocket instance")
}

pub fn form_encoded_register(body: &NewUser) -> String {
    format!(
        "first_name={}&last_name={}&email={}&password={}",
        encode(&body.first_name),
        encode(&body.last_name),
        encode(&body.email),
        encode(&body.password)
    )
}

pub fn form_encoded_login(body: &FormUser) -> String {
    format!(
        "email={}&password={}",
        encode(&body.email),
        encode(&body.password)
    )
}

pub async fn user_login(client: &Client, form: FormUser) -> LocalResponse {
    let form_body = form_encoded_login(&form);

    client
        .post("/login")
        .header(ContentType::Form)
        .body(form_body)
        .dispatch()
        .await
}

pub async fn user_register(client: &Client, form: NewUser) -> LocalResponse {
    let form_body = form_encoded_register(&form);

    client
        .post("/register")
        .header(ContentType::Form)
        .body(form_body)
        .dispatch()
        .await
}
