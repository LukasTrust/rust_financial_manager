use bcrypt::{hash, DEFAULT_COST};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::response::Redirect;

use crate::routes::error_page::show_error_page;

use super::models::{NewUser, User};

pub fn insert_user_mocking(new_user: NewUser) -> Result<usize, DieselError> {
    if new_user.email == "copy_email@mail.com" {
        return Err(DieselError::DatabaseError(
            DatabaseErrorKind::UniqueViolation,
            Box::new(String::new()),
        ));
    }
    if new_user.email == "internal_error@mail.com" {
        return Err(DieselError::DatabaseError(
            DatabaseErrorKind::Unknown,
            Box::new(String::new()),
        ));
    }

    Ok(1)
}

pub fn load_user_by_email_mocking(user_email: &str) -> Result<User, DieselError> {
    if user_email == "fake_email@mail.com" {
        return Err(DieselError::NotFound);
    }

    if user_email == "user_exists@mail.com" {
        let hashed_password = match hash("Password123", DEFAULT_COST) {
            Ok(h) => h,
            Err(_) => {
                return Err(DieselError::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    Box::new(String::new()),
                ));
            }
        };
        return Ok(User {
            id: 1,
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "user_exists@mail.com".to_string(),
            password: hashed_password,
        });
    }

    let hashed_password = match hash("WrongPassword", DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => {
            return Err(DieselError::DatabaseError(
                DatabaseErrorKind::UniqueViolation,
                Box::new(String::new()),
            ));
        }
    };

    Ok(User {
        id: 1,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        email: user_email.to_string(),
        password: hashed_password,
    })
}

pub fn load_user_by_id_mocking(user_id: i32) -> Result<(String, String), Box<Redirect>> {
    if user_id == 0 {
        return Ok(("John".to_string(), "Doe".to_string()));
    }

    Err(show_error_page(
        "User not found!".to_string(),
        "Please login again.".to_string(),
    ))
}
