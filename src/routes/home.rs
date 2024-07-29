use std::str::FromStr;

use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_dyn_templates::{context, Template};

use crate::database::models::{FormTransactions, TypeOfT};

#[get("/home")]
pub fn home(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        if user_id_cookie.value().parse::<i32>().is_ok() {
            Ok(Template::render("dashboard", context! {}))
        } else {
            Err(Redirect::to("/"))
        }
    } else {
        Err(Redirect::to("/"))
    }
}

#[get("/add-bank")]
pub fn add_bank() -> Template {
    Template::render("add_bank", context! {})
}

#[post("/add-bank", data = "<form>")]
pub fn add_bank_form(form: Form<FormTransactions>) -> String {
    let form = form.into_inner();
    let type_of_t = TypeOfT::from_str(&form.type_of_t).unwrap_or(TypeOfT::Deposit); // Default or handle error

    format!(
        "Type: {:?}, Date: {}, counterparty: {}, Comment: {}, Amount: {}",
        type_of_t, form.date, form.counterparty, form.comment, form.amount
    )
}

#[get("/dashboard")]
pub fn dashboard() -> Template {
    Template::render("dashboard", context! {})
}

#[get("/settings")]
pub fn settings() -> Template {
    Template::render("settings", context! {})
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
