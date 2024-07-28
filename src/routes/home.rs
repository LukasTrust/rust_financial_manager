use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_dyn_templates::{context, Template};

#[get("/home")]
pub fn home(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        if user_id_cookie.value().parse::<i32>().is_ok() {
            Ok(Template::render(
                "home",
                context! { main_content: "dashboard" },
            ))
        } else {
            Err(Redirect::to("/"))
        }
    } else {
        Err(Redirect::to("/"))
    }
}

#[get("/dashboard")]
pub fn dashboard() -> Template {
    Template::render("home", context! { main_content: "dashboard" })
}

#[get("/settings")]
pub fn settings() -> Template {
    Template::render("home", context! { main_content: "settings" })
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
