use bcrypt::{hash, DEFAULT_COST};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::response::Redirect;

use crate::{routes::error_page::show_error_page, utils::structs::Bank};

use super::models::{CSVConverter, NewBank, NewCSVConverter, NewUser, User};

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

pub fn insert_bank_mocking(new_bank: NewBank) -> Result<Bank, String> {
    if new_bank.name == "copy_bank" {
        return Err("Error inserting bank".into());
    }

    if new_bank.name == "csv_error" {
        return Ok(Bank {
            id: 0,
            user_id: new_bank.user_id,
            name: new_bank.name,
            link: new_bank.link,
        });
    }

    Ok(Bank {
        id: 1,
        user_id: new_bank.user_id,
        name: new_bank.name,
        link: new_bank.link,
    })
}

pub fn insert_csv_converter_mocking(
    new_csv_converter: NewCSVConverter,
) -> Result<CSVConverter, String> {
    if new_csv_converter.bank_id == 0 {
        return Err("Error inserting csv converter".into());
    }

    Ok(CSVConverter {
        id: 1,
        bank_id: new_csv_converter.bank_id,
        counterparty_column: new_csv_converter.counterparty_column,
        amount_column: new_csv_converter.amount_column,
        bank_balance_after_column: new_csv_converter.bank_balance_after_column,
        date_column: new_csv_converter.date_column,
    })
}

pub fn load_banks_of_user_mocking(
    user_id_for_loading: i32,
    new_bank: Option<NewBank>,
) -> Result<Vec<Bank>, String> {
    if new_bank.is_some() {
        let new_bank = new_bank.unwrap();

        if new_bank.name == "error_loading_banks" {
            return Err("Error loading banks".into());
        }

        return Ok(vec![Bank {
            id: 1,
            user_id: user_id_for_loading,
            name: new_bank.name,
            link: new_bank.link,
        }]);
    }

    Ok(vec![Bank {
        id: 1,
        user_id: user_id_for_loading,
        name: "Test_Bank".to_string(),
        link: Some("http://test-bank.com".to_string()),
    }])
}
