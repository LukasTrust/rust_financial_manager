use super::schema::users;
use diesel::prelude::*;
use rocket::FromForm;

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}

#[derive(FromForm)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(FromForm, Insertable, Debug)]
#[diesel(table_name = users)]
pub struct RegisterUser {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub password: String,
}
