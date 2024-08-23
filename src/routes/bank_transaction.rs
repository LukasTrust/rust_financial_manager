use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::serde::json::json;
use rocket::serde::json::Json;
use rocket::{get, State};
use rocket_db_pools::Connection;
use rocket_dyn_templates::Template;

use crate::database::db_connector::DbConn;
use crate::utils::appstate::AppState;
use crate::utils::get_utils::{get_transactions_with_contract, get_user_id};
use crate::utils::structs::ResponseData;
use crate::utils::update_utils::update_transaction_with_contract;
use crate::utils::update_utils::update_transaction_with_hidden;

#[get("/bank/transaction")]
pub async fn bank_transaction(
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    let current_bank = state.get_current_bank(cookie_user_id).await;

    if current_bank.is_none() {
        return Ok(Template::render(
            "bank_trasaction",
            json!(ResponseData {
                success: None,
                error: Some(
                    "There was an internal error while loading the bank. Please try again.".into(),
                ),
                header: Some("No bank selected".into()),
            }),
        ));
    }

    let current_bank = current_bank.unwrap();

    let result = get_transactions_with_contract(current_bank.id, db).await;

    let error = if result.is_err() {
        Some(result.clone().err().unwrap())
    } else {
        None
    };

    let transaction_string = if result.is_ok() {
        result.unwrap()
    } else {
        String::new()
    };

    let mut result = json!(ResponseData {
        success: None,
        error: Some(format!(
            "There was an internal error trying to load the transactions of '{}'.",
            current_bank.name
        )),
        header: error,
    });

    result["transactions"] = json!(transaction_string);

    Ok(Template::render("bank_transaction", json!(result)))
}

#[get("/bank/transaction/remove/<transaction_id>")]
pub async fn transaction_remove(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let result = update_transaction_with_contract(transaction_id, None::<i32>, &mut db).await;

    if result.is_err() {
        return Err(Json(ResponseData {
            success: None,
            error: Some("There was an internal error while removing the contract from the transactions. Please try again.".into()),
            header: Some("Internal error".into()),
        }));
    }

    Ok(Json(ResponseData {
        success: Some("The contract was removed from the transactions.".into()),
        error: None,
        header: Some("Contract removed".into()),
    }))
}

#[get("/bank/transaction/hide/<transaction_id>")]
pub async fn transaction_hide(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let result = update_transaction_with_hidden(transaction_id, true, &mut db).await;

    if result.is_err() {
        return Err(Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while trying to hide the transactions. Please try again."
                    .into(),
            ),
            header: Some("Internal error".into()),
        }));
    }

    Ok(Json(ResponseData {
        success: Some("The transaction will now be hidden.".into()),
        error: None,
        header: Some("Transaction hidden".into()),
    }))
}

#[get("/bank/transaction/show/<transaction_id>")]
pub async fn transaction_show(
    transaction_id: i32,
    mut db: Connection<DbConn>,
) -> Result<Json<ResponseData>, Json<ResponseData>> {
    let result = update_transaction_with_hidden(transaction_id, false, &mut db).await;

    if result.is_err() {
        return Err(Json(ResponseData {
            success: None,
            error: Some(
                "There was an internal error while trying to unhide the transactions. Please try again."
                    .into(),
            ),
            header: Some("Internal error".into()),
        }));
    }

    Ok(Json(ResponseData {
        success: Some("The transaction will now be displayed.".into()),
        error: None,
        header: Some("Transaction shown".into()),
    }))
}
