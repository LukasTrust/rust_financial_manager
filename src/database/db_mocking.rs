use diesel::result::{DatabaseErrorKind, Error as DieselError};

use super::models::NewUser;

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
