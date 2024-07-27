#[macro_use]
extern crate rocket;

use rocket::fs::{relative, FileServer};
use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

use database::db_connector::DbConn;
use routes::home::{home, logout};
use routes::login::{login_form, login_user};
use routes::register::{login_form_from_register, register_form, register_user};
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::init())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                login_form,
                register_form,
                register_user,
                login_form_from_register,
                login_user,
                home,
                logout
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
}
