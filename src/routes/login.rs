use rocket::get;
use rocket_dyn_templates::{context, Template};

#[get("/")]
pub fn login_form() -> Template {
    Template::render("login", context! {})
}
