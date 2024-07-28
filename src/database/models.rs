use diesel::prelude::*;
use rocket::FromForm;

use crate::schema::{banks, users};

#[derive(FromForm)]
pub struct User {
    pub email: String,
    pub password: String,
}

#[derive(FromForm, Insertable, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}
