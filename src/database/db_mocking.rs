use bcrypt::{hash, DEFAULT_COST};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::serde::json::Json;

use crate::utils::structs::{Bank, ResponseData};

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

pub fn load_user_by_email_mocking(user_email: &str) -> Result<User, Json<ResponseData>> {
    if user_email == "fake_email@mail.com" {
        return Err(Json(ResponseData::new_error(
            String::new(),
            "Login failed. Either the email or password was incorrect.".to_string(),
        )));
    }

    if user_email == "user_exists@mail.com" {
        let hashed_password = match hash("Password123", DEFAULT_COST) {
            Ok(h) => h,
            Err(_) => {
                return Err(Json(ResponseData::new_error(
                    String::new(),
                    "Login failed. Either the email or password was incorrect.".to_string(),
                )));
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
            return Err(Json(ResponseData::new_error(
                String::new(),
                "Login failed. Either the email or password was incorrect.".to_string(),
            )));
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

pub fn load_user_by_id_mocking(user_id: i32) -> Result<(String, String), Json<ResponseData>> {
    if user_id == 0 {
        return Ok(("John".to_string(), "Doe".to_string()));
    }

    Err(Json(ResponseData::new_error(
        "Error loading user".to_string(),
        "There was an internal error while loading the user. Please try again.".to_string(),
    )))
}

pub fn insert_bank_mocking(new_bank: NewBank) -> Result<Bank, Json<ResponseData>> {
    if new_bank.name == "copy_bank" {
        return Err(Json(ResponseData::new_error(
            "Error inserting bank".to_string(),
            "A bank with this name already exists in your profile. Please choose a different bank name."
                .to_string(),
        )));
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
) -> Result<CSVConverter, Json<ResponseData>> {
    if new_csv_converter.bank_id == 0 {
        return Err(Json(ResponseData::new_error(
            "Error inserting csv converter".to_string(),
            "There was an internal error trying to add the csv converter of the new bank. The bank was added but the csv converter was not.".to_string(),
        )));
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
) -> Result<Vec<Bank>, Json<ResponseData>> {
    if new_bank.is_some() {
        let new_bank = new_bank.unwrap();

        if new_bank.name == "error_loading_banks" {
            return Err(Json(ResponseData::new_error(
                "Error loading banks".to_string(),
                "There was an internal error trying to load the banks. Please login again and retry.".to_string(),
            )));
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
