use ::diesel::{ExpressionMethods, QueryDsl};
use bcrypt::verify;
use log::{error, info};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::schema::users::dsl::*;
use crate::utils::structs::FormUser;

/// Display the login form.
/// The form is used to collect user information such as email and password.
#[get("/")]
pub fn login_form() -> Template {
    info!("Login form displayed.");
    Template::render("login", context! {})
}

/// Login a user.
/// The user information is collected from the login form and compared with the stored information in the database.
/// If the login is successful, the user is redirected to the home page.
/// If the email or password is incorrect, an error message is displayed.
/// If there is an internal server error, an error message is displayed.
#[post("/login", data = "<user_form>")]
pub async fn login_user(
    mut db: Connection<DbConn>,
    user_form: Form<FormUser>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Template> {
    let email_of_user = &user_form.email.to_lowercase();
    let password_of_user = &user_form.password;

    let result = users
        .filter(email.eq(email_of_user))
        .select((id, password))
        .first::<(i32, String)>(&mut db)
        .await;

    info!("Login attempt for user with email: {}", email_of_user);

    match result {
        Ok((user_id, stored_password)) => match verify(password_of_user, &stored_password) {
            Ok(true) => {
                info!("Login successful for user with email: {}", email_of_user);
                cookies.add_private(Cookie::new("user_id", user_id.to_string()));
                Ok(Redirect::to("/home"))
            }
            Ok(false) => {
                info!(
                    "Login failed for user with email, password did not match: {}",
                    email_of_user
                );
                Err(Template::render(
                    "login",
                    context! { error: "Login failed. Either the email or password was incorrect." },
                ))
            }
            Err(err) => {
                error!("Login failed, bcrypt error: {}", err);
                Err(Template::render(
                    "login",
                    context! { error: "Login failed. Internal server error. Please try again later." },
                ))
            }
        },
        Err(err) => {
            error!("Login failed, database error {}", err);
            Err(Template::render(
                "login",
                context! { error: "Login failed. Either the email or password was incorrect." },
            ))
        }
    }
}
