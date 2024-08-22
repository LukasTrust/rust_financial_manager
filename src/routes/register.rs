use bcrypt::{hash, DEFAULT_COST};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use log::{error, info};
use regex::Regex;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket::{get, post};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::database::models::NewUser;
use crate::schema::users;
use crate::utils::structs::ResponseData;

/// Display the registration form.
/// The form is used to collect user information such as first name, last name, email, and password.
#[get("/register")]
pub fn register_form() -> Template {
    info!("Register form displayed.");
    Template::render("register", context! {})
}

/// Register a new user.
/// The user information is collected from the registration form and stored in the database.
/// If the registration is successful, the user is redirected to the login page with a success message.
/// If the email already exists, an error message is displayed.
/// If the email format is invalid, an error message is displayed.
/// If the password is not strong enough, an error message is displayed.
/// If there is an internal server error, an error message is displayed.
#[post("/register", data = "<user_form>")]
pub async fn register_user(
    mut db: Connection<DbConn>,
    user_form: Form<NewUser>,
) -> Json<ResponseData> {
    if !is_valid_email(&user_form.email) {
        return Json(ResponseData {
            success: None,
            error: Some("Invalid email format. Please use a valid email.".into()),
            header: None,
        });
    }

    if !is_strong_password(&user_form.password) {
        return Json(ResponseData {
            success: None,
            error: Some("Password must be at least 10 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character".into()),
            header: None,        });
    }

    let hashed_password = match hash(user_form.password.clone(), DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return Json(ResponseData {
                success: None,
                error: Some("Internal server error. Please try again later.".into()),
                header: None,
            });
        }
    };

    let new_user = NewUser {
        first_name: user_form.first_name.clone(),
        last_name: user_form.last_name.clone(),
        email: user_form.email.clone().to_lowercase(),
        password: hashed_password,
    };

    let result = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => Json(ResponseData {
            success: Some("Registration successful. Please log in.".into()),
            error: None,
            header: None,
        }),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            Json(ResponseData {
                error: Some("Email already exists. Please use a different email.".into()),
                success: None,
                header: None,
            })
        }
        Err(_) => {
            error!("Registration failed, database error.");
            Json(ResponseData {
                error: Some("Internal server error. Please try again later.".into()),
                success: None,
                header: None,
            })
        }
    }
}

/// Check if an email is valid.
/// A valid email must:
/// - Contain only alphanumeric characters, dots, hyphens, and underscores
/// - Have a domain with at least one dot
/// - Have a top-level domain with at least two characters
pub fn is_valid_email(email: &str) -> bool {
    // Regular expression for validating an email address
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    // Check if the email matches the regex pattern and doesn't contain consecutive dots
    re.is_match(email) && !email.contains("..")
}

/// Check if a password is strong.
/// A strong password must:
/// - Be at least 10 characters long
/// - Contain at least one lowercase letter
/// - Contain at least one uppercase letter
/// - Contain at least one digit
/// - Contain at least one special character
pub fn is_strong_password(password: &str) -> bool {
    // Define password strength criteria
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    // Password is considered strong if it meets all criteria
    password.len() >= 10 && has_lowercase && has_uppercase && has_digit && has_special
}
