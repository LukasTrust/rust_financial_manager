use bcrypt::verify;
use log::{error, info};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;

use crate::utils::loading_utils::load_user_by_email;
use crate::utils::structs::{ErrorResponse, FormUser, SuccessResponse};

/// Display the login form.
/// The form is used to collect user information such as email and password.
#[get("/")]
pub fn login_form() -> Template {
    info!("Login form displayed.");
    Template::render("login", context! {})
}

#[get("/login?<success>")]
pub fn login_from_register(success: String) -> Template {
    info!("Registration successful.");
    Template::render("login", context! { success })
}

/// Login a user.
/// The user information is collected from the login form and compared with the stored information in the database.
/// If the login is successful, the user is redirected to the base page.
/// If the email or password is incorrect, an error message is displayed.
/// If there is an internal server error, an error message is displayed.
#[post("/login", data = "<user_form>")]
pub async fn login_user(
    user_form: Form<FormUser>,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let email_of_user = &user_form.email.to_lowercase();
    let password_of_user = &user_form.password;

    let user = load_user_by_email(email_of_user, &mut db).await?;

    info!("Login attempt for user with email: {}", email_of_user);

    match verify(password_of_user, &user.password) {
        Ok(true) => {
            info!("Login successful for user with email: {}", email_of_user);
            cookies.add_private(Cookie::new("user_id", user.id.to_string()));
            cookies.add_private(Cookie::new("language", user.language));
            Ok(Json(SuccessResponse::new(
                String::new(),
                "Login successful. Redirecting...".to_string(),
            )))
        }
        Ok(false) => {
            info!(
                "Login failed for user with email, password did not match: {} {}",
                email_of_user, password_of_user
            );
            Err(Json(ErrorResponse::new(
                String::new(),
                "Login failed. Either the email or password was incorrect.".to_string(),
            )))
        }
        Err(err) => {
            error!("Login failed, bcrypt error: {}", err);
            Err(Json(ErrorResponse::new(
                String::new(),
                "Login failed. Either the email or password was incorrect.".to_string(),
            )))
        }
    }
}
