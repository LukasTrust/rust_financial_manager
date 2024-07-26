#[macro_use]
extern crate rocket;

use database::db_connector::DbConn;
use rocket::fs::{relative, FileServer};
use rocket_dyn_templates::Template;
use routes::login::login_form;
use routes::register::{login_form_from_register, register_form, register_user};
use rust_financial_manager::{database, routes};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                login_form,
                register_form,
                register_user,
                login_form_from_register
            ],
        )
        .mount("/static", FileServer::from(relative!("static")))
}
