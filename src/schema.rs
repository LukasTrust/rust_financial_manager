// @generated automatically by Diesel CLI.

diesel::table! {
    banks (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        link -> Nullable<Text>,
    }
}

diesel::table! {
    contract_history (id) {
        id -> Int4,
        contract_id -> Int4,
        old_amount -> Float8,
        new_amount -> Float8,
        changed_at -> Date,
    }
}

diesel::table! {
    contracts (id) {
        id -> Int4,
        bank_id -> Int4,
        name -> Text,
        parse_name -> Text,
        current_amount -> Float8,
        months_between_payment -> Int4,
        end_date -> Nullable<Date>,
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
        contract_id -> Nullable<Int4>,
        date -> Date,
        counterparty -> Text,
        amount -> Float8,
        bank_balance_after -> Float8,
        is_hidden -> Bool,
        contract_not_allowed -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        password -> Text,
        language -> Text,
    }
}

diesel::joinable!(banks -> users (user_id));
diesel::joinable!(contract_history -> contracts (contract_id));
diesel::joinable!(contracts -> banks (bank_id));
diesel::joinable!(csv_converters -> banks (bank_id));
diesel::joinable!(transactions -> banks (bank_id));
diesel::joinable!(transactions -> contracts (contract_id));

diesel::allow_tables_to_appear_in_same_query!(
    banks,
    contract_history,
    contracts,
    csv_converters,
    transactions,
    users,
);
