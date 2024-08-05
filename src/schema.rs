// @generated automatically by Diesel CLI.

diesel::table! {
    banks (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 200]
        link -> Nullable<Varchar>,
    }
}

diesel::table! {
    csv_converters (id) {
        id -> Int4,
        csv_bank_id -> Int4,
        #[max_length = 200]
        date_conv -> Nullable<Varchar>,
        #[max_length = 200]
        counterparty_conv -> Nullable<Varchar>,
        #[max_length = 200]
        amount_conv -> Nullable<Varchar>,
        #[max_length = 200]
        bank_current_balance_after_conv -> Nullable<Varchar>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        bank_id -> Int4,
        date -> Date,
        #[max_length = 200]
        counterparty -> Varchar,
        amount -> Float8,
        bank_current_balance_after -> Float8,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 50]
        first_name -> Varchar,
        #[max_length = 50]
        last_name -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 200]
        password -> Varchar,
    }
}

diesel::joinable!(banks -> users (user_id));
diesel::joinable!(csv_converters -> banks (csv_bank_id));
diesel::joinable!(transactions -> banks (bank_id));

diesel::allow_tables_to_appear_in_same_query!(banks, csv_converters, transactions, users,);
