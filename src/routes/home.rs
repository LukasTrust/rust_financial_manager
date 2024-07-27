use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::{get, post};
use rocket_dyn_templates::{context, Template};

#[get("/home")]
pub fn home(cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        let user_id: i32 = user_id_cookie.value().parse().unwrap();
        Ok(Template::render("home", context! { user_id }))
    } else {
        Err(Redirect::to("/"))
    }
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
