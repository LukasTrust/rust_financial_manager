use rocket::{
    get,
    http::{Cookie, CookieJar},
    serde::json::Json,
};
use rocket_db_pools::Connection;

use crate::{
    database::db_connector::DbConn,
    utils::{
        appstate::{Language, LOCALIZATION},
        get_utils::get_user_id_and_language,
        structs::{ErrorResponse, SuccessResponse},
        update_utils::update_user_with_language,
    },
};

#[get["/user/set_language/<new_language>"]]
pub async fn set_user_language(
    new_language: &str,
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let old_language = cookies.get_private("language");

    if old_language.is_none() {
        return Err(Json(ErrorResponse::new(
            LOCALIZATION.get_localized_string(cookie_user_language, "error_language_not_found"),
            LOCALIZATION
                .get_localized_string(cookie_user_language, "error_language_not_found_details"),
        )));
    }

    let old_language = old_language.unwrap();

    cookies.remove_private(old_language);

    cookies.add_private(Cookie::new("language", new_language.to_string()));

    let language = match new_language {
        "English" => Language::English,
        "German" => Language::German,
        _ => Language::English,
    };

    update_user_with_language(cookie_user_id, language, &mut db).await?;

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(language, "new_language_set"),
        LOCALIZATION.get_localized_string(language, "new_language_set_details"),
    )))
}
