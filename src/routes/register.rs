use bcrypt::{hash, DEFAULT_COST};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use regex::Regex;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::{get, post, uri};
use rocket_db_pools::{diesel, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::database::models::RegisterUser;
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
pub async fn register_user(
    mut db: Connection<DbConn>,
    user_form: Form<RegisterUser>,
) -> Result<Redirect, Template> {
    if !is_valid_email(&user_form.email.clone()) {
        return Err(Template::render(
            "register",
            context! { error: "Email format not valide." },
        ));
    }

    if !is_strong_password(&user_form.password.clone()) {
        return Err(Template::render(
            "register",
            context! { error: "Password must be at least 10 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character" },
        ));
    }

    let hashed_password = match hash(user_form.password.clone(), DEFAULT_COST) {
        Ok(h) => h,
        Err(err) => {
            return Err(Template::render(
                "register",
                context! { error: format!("Failed to hash password: {}", err) },
            ))
        }
    };

    let new_user = RegisterUser {
        firstname: user_form.firstname.clone(),
        lastname: user_form.lastname.clone(),
        email: user_form.email.clone().to_lowercase(),
        password: hashed_password,
    };

    let result = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut db)
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
