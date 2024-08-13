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
    contract_history (id) {
        id -> Int4,
        contract_id -> Int4,
        old_amount -> Float8,
        new_amount -> Float8,
        changed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    contracts (id) {
        id -> Int4,
        bank_id -> Int4,
        #[max_length = 200]
        name -> Varchar,
        current_amount -> Float8,
        months_between_payment -> Int4,
    }
}

diesel::table! {
    csv_converters (id) {
        id -> Int4,
        bank_id -> Int4,
        date_column -> Nullable<Int4>,
        counterparty_column -> Nullable<Int4>,
        amount_column -> Nullable<Int4>,
        bank_balance_after_column -> Nullable<Int4>,
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
        bank_balance_after -> Float8,
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
diesel::joinable!(contract_history -> contracts (contract_id));
diesel::joinable!(contracts -> banks (bank_id));
diesel::joinable!(csv_converters -> banks (bank_id));
diesel::joinable!(transactions -> banks (bank_id));

diesel::allow_tables_to_appear_in_same_query!(
    banks,
    contract_history,
    contracts,
    csv_converters,
    transactions,
    users,
);
