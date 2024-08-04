use diesel::QueryDsl;
use rocket::form::{Form, FromForm};
use rocket::{post, State};
use rocket_db_pools::{diesel::prelude::RunQueryDsl, Connection};
use rocket_dyn_templates::Template;
use serde_json::json;

use crate::database::db_connector::DbConn;
use crate::database::models::CSVConverter;
use crate::schema::csv_converters;
use crate::structs::AppState;
use crate::utils::generate_balance_graph_data;

#[derive(FromForm)]
pub struct DateForm {
    date: String,
}

#[derive(FromForm)]
pub struct CounterpartyForm {
    counterparty: String,
}

#[derive(FromForm)]
pub struct AmountForm {
    amount: String,
}

#[derive(FromForm)]
pub struct BankBalanceAfterTransactionForm {
    bank_balance_after: String,
}

#[post("/update_bank_balance_after", data = "<form>")]
pub async fn update_bank_balance_after(
    form: Form<BankBalanceAfterTransactionForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.bank_current_balance_after_conv = Some(form.bank_balance_after.clone());
    })
    .await
}

#[post("/update_date", data = "<form>")]
pub async fn update_date(
    form: Form<DateForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.date_conv = Some(form.date.clone());
    })
    .await
}

#[post("/update_counterparty", data = "<form>")]
pub async fn update_counterparty(
    form: Form<CounterpartyForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.counterparty_conv = Some(form.counterparty.clone());
    })
    .await
}

#[post("/update_amount", data = "<form>")]
pub async fn update_amount(
    form: Form<AmountForm>,
    state: &State<AppState>,
    db: Connection<DbConn>,
) -> Template {
    update_csv_converter(state, db, |converter| {
        converter.amount_conv = Some(form.amount.clone());
    })
    .await
}

async fn update_csv_converter<F>(
    state: &State<AppState>,
    mut db: Connection<DbConn>,
    update_field: F,
) -> Template
where
    F: Fn(&mut CSVConverter),
{
    let current_bank_id;
    {
        let current_bank = state.current_bank.read().await;
        current_bank_id = current_bank.id;
    }

    let mut success = None;
    let mut error = None;

    {
        let mut csv_converters_lock = state.csv_convert.write().await;
        if let Some(current_csv_converter) = csv_converters_lock.get_mut(&current_bank_id) {
            update_field(current_csv_converter);
            let result = diesel::update(csv_converters::table.find(current_csv_converter.id))
                .set(current_csv_converter.clone())
                .execute(&mut db)
                .await;

            match result {
                Ok(_) => success = Some("Update successful"),
                Err(_) => error = Some("Update failed"),
            };
        } else {
            let mut new_csv_converter = CSVConverter {
                id: 0,
                csv_bank_id: current_bank_id,
                date_conv: None,
                counterparty_conv: None,
                amount_conv: None,
                bank_current_balance_after_conv: None,
            };
            update_field(&mut new_csv_converter);
            let result = diesel::insert_into(csv_converters::table)
                .values(new_csv_converter.clone())
                .execute(&mut db)
                .await;

            if result.is_ok() {
                csv_converters_lock.insert(current_bank_id, new_csv_converter);
                success = Some("Insert successful");
            } else {
                error = Some("Insert failed");
            }
        }
    }

    let banks = state.banks.read().await.clone();
    let transactions = state.transactions.read().await.clone();
    let plot_data = generate_balance_graph_data(&banks, &transactions);
    let bank = state.current_bank.read().await.clone();
    let context = json!({
        "banks": banks,
        "bank": bank,
        "plot_data": plot_data.to_string(),
        "success": success,
        "error": error
    });

    Template::render("bank", &context)
}
