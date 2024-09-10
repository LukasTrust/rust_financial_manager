use bcrypt::{hash, verify, DEFAULT_COST};
use rocket::{
    form::Form,
    get,
    http::{Cookie, CookieJar},
    post,
    serde::json::{json, Json},
    State,
};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::{
    database::db_connector::DbConn,
    utils::{
        appstate::{AppState, Language, LOCALIZATION},
        delete_utils::delete_user_by_id,
        get_utils::get_user_id_and_language,
        loading_utils::load_user_by_id,
        structs::{ChangePassword, ErrorResponse, SuccessResponse},
        translation_utils::get_settings_localized_strings,
        update_utils::{update_user_password, update_user_with_language},
    },
};

use super::register::is_strong_password;

/// Display the settings page.
/// The settings page allows the user to manage their bank accounts and transactions.
/// The user is redirected to the login page if they are not logged in.
#[get("/settings")]
pub async fn settings(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    state.set_current_bank(cookie_user_id, None).await;

    let localized_strings = get_settings_localized_strings(cookie_user_language);

    Ok(Template::render(
        "settings",
        json!({"translations": localized_strings,}),
    ))
}

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

#[post("/change_password", data = "<change_password>")]
pub async fn change_password(
    cookies: &CookieJar<'_>,
    change_password: Form<ChangePassword>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    let user = load_user_by_id(cookie_user_id, cookie_user_language, &mut db).await?;

    match verify(change_password.old_password.clone(), &user.password) {
        Ok(true) => {
            if change_password.new_password != change_password.confirm_password {
                return Err(Json(ErrorResponse::new(
                    LOCALIZATION.get_localized_string(
                        cookie_user_language,
                        "error_new_passwords_do_not_match",
                    ),
                    LOCALIZATION.get_localized_string(
                        cookie_user_language,
                        "error_new_passwords_do_not_match_details",
                    ),
                )));
            }

            if !is_strong_password(&change_password.new_password) {
                return Err(Json(ErrorResponse::new(
                    LOCALIZATION.get_localized_string(cookie_user_language, "error_weak_password"),
                    LOCALIZATION
                        .get_localized_string(cookie_user_language, "error_weak_password_details"),
                )));
            }
            let hashed_password = match hash(change_password.new_password.clone(), DEFAULT_COST) {
                Ok(h) => h,
                Err(_) => {
                    return Err(Json(ErrorResponse::new(
                        LOCALIZATION
                            .get_localized_string(cookie_user_language, "error_password_hashing"),
                        LOCALIZATION.get_localized_string(
                            cookie_user_language,
                            "error_password_hashing_details",
                        ),
                    )));
                }
            };

            update_user_password(
                cookie_user_id,
                hashed_password,
                cookie_user_language,
                &mut db,
            )
            .await?;

            return Ok(Json(SuccessResponse::new(
                LOCALIZATION.get_localized_string(cookie_user_language, "password_changed"),
                LOCALIZATION.get_localized_string(cookie_user_language, "password_changed_details"),
            )));
        }
        Ok(false) => {
            return Err(Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(
                    cookie_user_language,
                    "error_old_password_does_not_match",
                ),
                LOCALIZATION.get_localized_string(
                    cookie_user_language,
                    "error_old_password_does_not_match_details",
                ),
            )));
        }
        Err(_) => {
            return Err(Json(ErrorResponse::new(
                LOCALIZATION.get_localized_string(cookie_user_language, "error_password_hashing"),
                LOCALIZATION
                    .get_localized_string(cookie_user_language, "error_password_hashing_details"),
            )));
        }
    }
}

#[get("/delete_account")]
pub async fn delete_account(
    cookies: &CookieJar<'_>,
    mut db: Connection<DbConn>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let (cookie_user_id, cookie_user_language) = get_user_id_and_language(cookies)?;

    delete_user_by_id(cookie_user_id, cookie_user_language, &mut db).await?;

    cookies.remove_private(Cookie::build("user_id"));

    Ok(Json(SuccessResponse::new(
        LOCALIZATION.get_localized_string(cookie_user_language, "account_deleted"),
        LOCALIZATION.get_localized_string(cookie_user_language, "account_deleted_details"),
    )))
}
