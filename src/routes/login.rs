use ::diesel::{ExpressionMethods, QueryDsl};
use bcrypt::verify;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::DbConn;
use crate::database::models::FormUser;
use crate::schema::users::dsl::*;

#[get("/")]
pub fn login_form() -> Template {
    Template::render("login", context! {})
}

#[post("/login", data = "<user_form>")]
pub async fn login_user(
    mut db: Connection<DbConn>,
    user_form: Form<FormUser>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Template> {
    let email_of_user = &user_form.email.to_lowercase();
    let password_of_user = &user_form.password;

    let result = users
        .filter(email.eq(email_of_user))
        .select((id, password))
        .first::<(i32, String)>(&mut db)
        .await;

    match result {
        Ok((user_id, stored_password)) => match verify(password_of_user, &stored_password) {
            Ok(true) => {
                cookies.add(Cookie::new("user_id", user_id.to_string()));
                Ok(Redirect::to("/home"))
            }
            Ok(false) => Err(Template::render(
                "login",
                context! { error: "Login failed. Either the email or password was incorrect." },
            )),
            Err(_) => Err(Template::render(
                "login",
                context! { error: "Login failed. Internal server error. Please try again later." },
            )),
        },
        Err(_) => Err(Template::render(
            "login",
            context! { error: "Login failed. Either the email or password was incorrect." },
        )),
    }
}
