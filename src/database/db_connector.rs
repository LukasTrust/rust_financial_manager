use ::diesel::{ExpressionMethods, QueryDsl};
use diesel::result::Error as DieselError;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};

use crate::database::schema::users::dsl::*;
use rocket_db_pools::{diesel, Database};

#[derive(Database)]
#[database("postgres_db")]
pub struct DbConn(diesel::PgPool);

pub async fn find_user_id_and_password(
    email_of_user: String,
    mut db: Connection<DbConn>,
) -> Result<(i32, String), DieselError> {
    // Query the database for the user with the specified email
    let user: (i32, String) = users
        .filter(email.eq(email_of_user))
        .select((id, password))
        .first::<(i32, String)>(&mut db)
        .await?;

    // Return the user ID and password
    Ok(user)
}
