use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::uri;
use rocket::{get, post, FromForm};
use rocket_dyn_templates::{context, Template};
use rocket_sync_db_pools::diesel;

use crate::database::db_connector::DbConn;
use crate::database::models::NewUser;
use crate::database::schema::users;

#[get("/login?<message>")]
pub fn login_form_from_register(message: String) -> Template {
    Template::render("login", context! { message })
}

#[get("/register")]
pub fn register_form() -> Template {
    Template::render("register", context! {})
}

#[post("/register", data = "<user_form>")]
pub async fn register_user(db: DbConn, user_form: Form<NewUserForm>) -> Result<Redirect, Template> {
    let new_user = NewUser {
        firstname: user_form.firstname.clone(),
        lastname: user_form.lastname.clone(),
        email: user_form.email.clone(),
        password: user_form.password.clone(),
    };

    let result = db
        .run(move |conn| {
            diesel::insert_into(users::table)
                .values(&new_user)
                .execute(conn)
        })
        .await;

    match result {
        Ok(_) => Ok(Redirect::to(uri!(login_form_from_register(
            "Registration successful. Please log in."
        )))),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Err(Template::render(
                "register",
                context! { error: "Email already exists. Please use a different email." },
            ))
        }
        Err(_) => Err(Template::render(
            "register",
            context! { error: "Internal server error. Please try again later." },
        )),
    }
}

#[derive(FromForm)]
pub struct NewUserForm {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}
