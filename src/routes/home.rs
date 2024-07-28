use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_dyn_templates::{context, Template};

#[get("/home")]
pub fn home(cookies: &CookieJar<'_>) -> Result<Template, Box<Redirect>> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        match user_id_cookie.value().parse::<i32>() {
            Ok(user_id) => Ok(Template::render("home", context! { user_id })),
            Err(_) => Err(Box::new(Redirect::to("/"))),
        }
    } else {
        Err(Box::new(Redirect::to("/")))
    }
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
