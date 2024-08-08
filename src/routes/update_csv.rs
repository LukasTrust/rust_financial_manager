use diesel::QueryDsl;
use rocket::form::{Form, FromForm};
use rocket::{http::CookieJar, response::Redirect};
use rocket::{post, State};
use rocket_db_pools::diesel::AsyncPgConnection;
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;
use std::collections::HashMap;

use crate::database::db_connector::DbConn;
use crate::database::models::{CSVConverter, NewCSVConverter};
use crate::schema::csv_converters;
use crate::utils::appstate::AppState;
use crate::utils::display_utils::show_home_or_subview_with_data;
use crate::utils::get_utils::{get_current_bank, get_user_id};

#[derive(FromForm)]
pub struct DateForm {
    date_column: i32,
}

#[derive(FromForm)]
pub struct CounterpartyForm {
    counterparty_column: i32,
}

#[derive(FromForm)]
pub struct AmountForm {
    amount_column: i32,
}

#[derive(FromForm)]
pub struct BankBalanceAfterTransactionForm {
    bank_balance_after_column: i32,
}

#[post("/update_bank_balance_after", data = "<form>")]
pub async fn update_bank_balance_after(
    form: Form<BankBalanceAfterTransactionForm>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    find_bank_and_update(cookie_user_id, state, &mut *db, |converter| {
        converter.bank_balance_after_column = Some(form.bank_balance_after_column.clone());
    })
    .await
}

#[post("/update_date", data = "<form>")]
pub async fn update_date(
    form: Form<DateForm>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    find_bank_and_update(cookie_user_id, state, &mut *db, |converter| {
        converter.date_column = Some(form.date_column.clone());
    })
    .await
}

#[post("/update_counterparty", data = "<form>")]
pub async fn update_counterparty(
    form: Form<CounterpartyForm>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    find_bank_and_update(cookie_user_id, state, &mut *db, |converter| {
        converter.counterparty_column = Some(form.counterparty_column.clone());
    })
    .await
}

#[post("/update_amount", data = "<form>")]
pub async fn update_amount(
    form: Form<AmountForm>,
    cookies: &CookieJar<'_>,
    state: &State<AppState>,
    mut db: Connection<DbConn>,
) -> Result<Template, Redirect> {
    let cookie_user_id = get_user_id(cookies)?;

    find_bank_and_update(cookie_user_id, state, &mut *db, |converter| {
        converter.amount_column = Some(form.amount_column.clone());
    })
    .await
}

async fn find_bank_and_update<F>(
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut AsyncPgConnection,
    update_field: F,
) -> Result<Template, Redirect>
where
    F: Fn(&mut CSVConverter),
{
    let current_bank_id = get_current_bank(cookie_user_id, state).await?.id;

    update_csv(cookie_user_id, state, db, update_field, current_bank_id).await
}

pub async fn update_csv<F>(
    cookie_user_id: i32,
    state: &State<AppState>,
    db: &mut AsyncPgConnection,
    update_field: F,
    current_bank_id: i32,
) -> Result<Template, Redirect>
where
    F: Fn(&mut CSVConverter),
{
    let mut success = None;
    let mut error = None;

    // Obtain a lock on the state (not used in this function, but kept for consistency with the code)
    let mut csv_converters_lock = state.csv_convert.write().await;
    let mut current_csv_converter = csv_converters_lock.get_mut(&current_bank_id).cloned();

    drop(csv_converters_lock);

    if let Some(current_csv_converter) = current_csv_converter.as_mut() {
        update_field(current_csv_converter);

        let result = diesel::update(csv_converters::table.find(current_csv_converter.id))
            .set(current_csv_converter.clone())
            .execute(db)
            .await;

        match result {
            Ok(_) => {
                success = Some("CSV converter updated successfully".to_string());
                state
                    .update_csv_converters(HashMap::from([(
                        current_bank_id,
                        current_csv_converter.clone(),
                    )]))
                    .await;
            }
            Err(err) => {
                error = Some(format!("Internal server error: {}", err));
            }
        }
    } else {
        let mut new_converter = CSVConverter {
            id: 0,
            csv_bank_id: current_bank_id,
            date_column: None,
            counterparty_column: None,
            amount_column: None,
            bank_balance_after_column: None,
        };

        update_field(&mut new_converter);

        let new_converter = NewCSVConverter {
            csv_bank_id: new_converter.csv_bank_id,
            date_column: new_converter.date_column,
            counterparty_column: new_converter.counterparty_column,
            amount_column: new_converter.amount_column,
            bank_balance_after_column: new_converter.bank_balance_after_column,
        };

        let result = diesel::insert_into(csv_converters::table)
            .values(&new_converter)
            .get_result::<CSVConverter>(&mut *db)
            .await;

        match result {
            Ok(converter) => {
                success = Some("CSV converter updated successfully".to_string());

                state
                    .update_csv_converters(HashMap::from([(current_bank_id, converter)]))
                    .await;
            }
            Err(err) => {
                error = Some(format!("Internal server error: {}", err));
            }
        }
    }

    Ok(show_home_or_subview_with_data(
        cookie_user_id,
        state,
        "bank".to_string(),
        true,
        true,
        success,
        error,
    )
    .await)
}
