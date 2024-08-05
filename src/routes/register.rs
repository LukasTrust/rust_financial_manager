use bcrypt::{hash, DEFAULT_COST};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use log::{error, info};
use regex::Regex;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::{get, post, uri};
use rocket_db_pools::diesel::prelude::RunQueryDsl;
use rocket_db_pools::{diesel, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::database::models::NewUser;
use crate::schema::users;

/// Display the login form after a successful registration.
/// The `succes` query parameter is used to display a success message.
#[get("/login?<succes>")]
pub fn login_form_from_register(succes: String) -> Template {
    info!("Registration successful.");
    Template::render("login", context! { succes })
}

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
) -> Result<Redirect, Template> {
    is_valid_email(&user_form.email.clone())?;

    is_strong_password(&user_form.password.clone())?;

    let hashed_password = match hash(user_form.password.clone(), DEFAULT_COST) {
        Ok(h) => {
            info!("Password hashed.");
            h
        }
        Err(err) => {
            error!("Failed to hash password: {}", err);
            return Err(Template::render(
                "register",
                context! { error: format!("Internal server error. Please try again later.") },
            ));
        }
    };

    let new_user = NewUser {
        first_name: user_form.first_name.clone(),
        last_name: user_form.last_name.clone(),
        email: user_form.email.clone().to_lowercase(),
        password: hashed_password,
    };

    info!("Registering new user with email: {}", new_user.email);

    let result = diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => {
            info!(
                "Registration successful for user with email: {}",
                new_user.email
            );

            Ok(Redirect::to(uri!(login_form_from_register(
                "Registration successful. Please log in."
            ))))
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
            error!("Email already exists.");

            Err(Template::render(
                "register",
                context! { error: "Email already exists. Please use a different email." },
            ))
        }
        Err(err) => {
            error!("Registration failed: {}", err);

            Err(Template::render(
                "register",
                context! { error: "Internal server error. Please try again later." },
            ))
        }
    }
}

/// Check if an email is valid.
/// A valid email must:
/// - Contain only alphanumeric characters, dots, hyphens, and underscores
/// - Have a domain with at least one dot
/// - Have a top-level domain with at least two characters
pub fn is_valid_email(email: &str) -> Result<(), Template> {
    // Regular expression for validating an email address
    let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    // Check if the email matches the regex pattern and doesn't contain consecutive dots
    match re.is_match(email) && !email.contains("..") {
        true => Ok(()),
        false => {
            error!("Email format not valide.");
            Err(Template::render(
                "register",
                context! { error: "Email format not valide." },
            ))
        }
    }
}

/// Check if a password is strong.
/// A strong password must:
/// - Be at least 10 characters long
/// - Contain at least one lowercase letter
/// - Contain at least one uppercase letter
/// - Contain at least one digit
/// - Contain at least one special character
pub fn is_strong_password(password: &str) -> Result<(), Template> {
    // Define password strength criteria
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    // Password is considered strong if it meets all criteria

    match password.len() >= 10 && has_lowercase && has_uppercase && has_digit && has_special {
        true => Ok(()),
        false => {
            error!("Password does not meet the criteria.");
            Err(Template::render(
                "register",
                context! { error: "Password must be at least 10 characters long and contain at least one uppercase letter, one lowercase letter, one digit, and one special character" },
            ))
        }
    }
}
