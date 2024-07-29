// @generated automatically by Diesel CLI.

diesel::table! {
    banks (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 200]
        link -> Nullable<Varchar>,
        current_amount -> Nullable<Float8>,
        interest_rate -> Nullable<Float8>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        bank_id -> Int4,
        #[max_length = 50]
        type_of_t -> Varchar,
        date -> Date,
        #[max_length = 200]
        counterparty -> Nullable<Varchar>,
        #[max_length = 200]
        comment -> Nullable<Varchar>,
        amount -> Float8,
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
diesel::joinable!(transactions -> banks (bank_id));

diesel::allow_tables_to_appear_in_same_query!(banks, transactions, users,);
