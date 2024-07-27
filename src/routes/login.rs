use bcrypt::verify;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{context, Template};

use crate::database::db_connector::{find_user_id_and_password, DbConn};
use crate::database::models::LoginUser;

#[get("/")]
pub fn login_form() -> Template {
    Template::render("login", context! {})
}

#[post("/login", data = "<user_form>")]
pub async fn login_user(
    db: Connection<DbConn>,
    user_form: Form<LoginUser>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, Template> {
    let email_of_user = &user_form.email.to_lowercase();
    let password_of_user = &user_form.password;

    let result = find_user_id_and_password(email_of_user.to_string(), db).await;

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
