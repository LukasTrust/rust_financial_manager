use log::{error, info};
use rocket::{http::CookieJar, response::Redirect};

use crate::routes::error_page::show_error_page;

/// Extract the user ID from the user ID cookie.
/// If the user ID cookie is not found or cannot be parsed, an error page is displayed.
/// The user ID is returned if the user ID cookie is found and parsed successfully.
pub fn extract_user_id(cookies: &CookieJar<'_>) -> Result<i32, Redirect> {
    if let Some(cookie_user_id) = cookies.get_private("user_id") {
        info!("User ID cookie found: {:?}", cookie_user_id.value());
        cookie_user_id.value().parse::<i32>().map_err(|_| {
            error!("Error parsing user ID cookie.");
            show_error_page(
                "Error validating the login!".to_string(),
                "Please login again.".to_string(),
            )
        })
    } else {
        error!("No user ID cookie found.");
        Err(show_error_page(
            "Error validating the login!".to_string(),
            "Please login again.".to_string(),
        ))
    }
}
