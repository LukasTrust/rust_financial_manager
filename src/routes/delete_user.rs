use diesel::prelude::*;
use rocket::response::Redirect;
use rocket::{delete, get, uri};
use rocket_db_pools::{diesel, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::database::schema::users;

#[delete("/delete_user/<email>")]
pub async fn delete_user(mut db: Connection<DbConn>, email: String) -> Result<Redirect, Template> {
    // Attempt to delete the user
    match diesel::delete(users::table.filter(users::email.eq(email)))
        .execute(&mut db)
        .await
    {
        Ok(0) => {
            // No rows were affected; user may not exist
            Err(Template::render(
                "error",
                context! { error: "User not found." },
            ))
        }
        Ok(_) => {
            // Deletion successful
            Ok(Redirect::to(uri!(login_form_from_delete(
                "User deleted successfully."
            ))))
        }
        Err(_) => {
            // An error occurred
            Err(Template::render(
                "error",
                context! { error: "Internal server error. Please try again later." },
            ))
        }
    }
}

#[get("/login?<message>")]
pub fn login_form_from_delete(message: String) -> Template {
    Template::render("login", context! { message })
}
