use ::diesel::{ExpressionMethods, QueryDsl};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::tokio::sync::RwLock;
use rocket::{get, post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use std::sync::Arc;
//use std::str::FromStr;

use crate::database::db_connector::DbConn;
use crate::database::models::{Bank, FormBank, NewBank}; //FormTransactions, NewTransactions, TypeOfT};
use crate::schema::banks as banks_without_dsl;
//use crate::schema::transactions::type_of_t;

#[derive(Serialize)]
pub struct Context {
    pub banks: Vec<Bank>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub banks: Arc<RwLock<Vec<Bank>>>,
}

#[get("/home")]
pub async fn home(
    mut db: Connection<DbConn>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
) -> Result<Template, Box<Redirect>> {
    if let Some(user_id_cookie) = cookies.get("user_id") {
        if user_id_cookie.value().parse::<i32>().is_ok() {
            use crate::schema::banks::dsl::*;

            let user_id_cookie = user_id_cookie.value().parse::<i32>().unwrap();
            let result = banks_without_dsl::table
                .filter(user_id.eq(user_id_cookie))
                .load::<Bank>(&mut db)
                .await
                .map_err(|_| Redirect::to("/"))?;

            // Update the global state
            let mut banks_state = state.banks.write().await;
            *banks_state = result.clone();

            let context = Context { banks: result };

            Ok(Template::render("dashboard", &context))
        } else {
            Err(Box::new(Redirect::to("/")))
        }
    } else {
        Err(Box::new(Redirect::to("/")))
    }
}

#[get("/add-bank")]
pub async fn add_bank(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("add_bank", context! { banks })
}

#[post("/add-bank", data = "<bank_form>")]
pub async fn add_bank_form(
    mut db: Connection<DbConn>,
    bank_form: Form<FormBank>,
    cookies: &CookieJar<'_>,
) -> Template {
    let user_id = cookies
        .get("user_id")
        .and_then(|cookie| cookie.value().parse::<i32>().ok());

    match user_id {
        Some(_) => {}
        None => return Template::render("home", context! { error: "Could not find user id" }),
    }

    let new_bank = NewBank {
        user_id: user_id.unwrap(),
        name: bank_form.name.to_string(),
        link: bank_form.link.clone(),
        current_amount: bank_form.current_amount,
        interest_rate: bank_form.interest_rate,
    };

    let result = diesel::insert_into(banks_without_dsl::table)
        .values(&new_bank)
        .execute(&mut db)
        .await;

    match result {
        Ok(_) => Template::render("add_bank", context! { success: "New bank added" }),
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => Template::render(
            "add_bank",
            context! { error: "A bank with this name already exists. Please use a different name." },
        ),
        Err(err) => Template::render(
            "add_bank",
            context! { error: format!("Internal server error {}", err) },
        ),
    }
}

//#[post("/add-transaction", data = "<form>")]
//pub fn add_transaction_form(form: Form<FormTransactions>) -> String {
//    let form = form.into_inner();
//    let type_of_transactions = TypeOfT::from_str(&form.type_of_t).unwrap_or(TypeOfT::Deposit);

//    let bank = NewTransactions {
//        bank_id: todo!(),
//        date: todo!(),
//        counterparty: todo!(),
//        comment: todo!(),
//        amount: todo!(),
//        type_of_t: todo!(),
//    };

//    format!(
//        "Type: {:?}, Date: {}, counterparty: {}, Comment: {}, Amount: {}",
//        type_of_t, form.date, form.counterparty, form.comment, form.amount
//    )
//}

#[get("/dashboard")]
pub async fn dashboard(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("dashboard", context! { banks })
}

#[get("/settings")]
pub async fn settings(state: &State<AppState>) -> Template {
    let banks = state.banks.read().await.clone();
    Template::render("settings", context! {banks})
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove(Cookie::build("user_id"));
    Redirect::to("/")
}
