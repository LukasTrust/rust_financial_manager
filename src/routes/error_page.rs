use log::info;
use rocket::{catch, get, response::Redirect, uri, Request};
use rocket_dyn_templates::{context, Template};

#[get("/error?<error_header>&<error_message>")]
pub fn error_page(error_header: String, error_message: String) -> Template {
    info!("Error page displayed: {}", error_header);
    info!("Error message: {}", error_message);
    Template::render(
        "error_page",
        context! {error_header: error_header, error_message: error_message},
    )
}

#[catch(404)]
pub fn not_found(req: &Request<'_>) -> Redirect {
    info!("404 error: {:?}", req);
    *show_error_page(
        "404 Not Found".to_string(),
        "The page you are looking for does not exist.".to_string(),
    )
}

pub fn show_error_page(error_header: String, error_message: String) -> Box<Redirect> {
    Box::new(Redirect::to(uri!(error_page(
        error_header = error_header,
        error_message = error_message
    ))))
}
