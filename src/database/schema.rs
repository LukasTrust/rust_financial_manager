// @generated automatically by Diesel CLI.

diesel::table! {
    banks (id) {
        id -> Int4,
        userid -> Int4,
        #[max_length = 50]
        name -> Varchar,
        #[max_length = 200]
        link -> Nullable<Varchar>,
        startdate -> Nullable<Date>,
        enddate -> Nullable<Date>,
        currentamount -> Nullable<Float8>,
        interestrate -> Nullable<Float8>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Int4,
        bankid -> Int4,
        #[sql_name = "type"]
        #[max_length = 50]
        type_ -> Varchar,
        date -> Date,
        #[max_length = 200]
        other -> Nullable<Varchar>,
        #[max_length = 200]
        comment -> Nullable<Varchar>,
        amount -> Float8,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 50]
        firstname -> Varchar,
        #[max_length = 50]
        lastname -> Varchar,
        #[max_length = 100]
        email -> Varchar,
        #[max_length = 50]
        password -> Varchar,
    }
}

diesel::joinable!(banks -> users (userid));
diesel::joinable!(transactions -> banks (bankid));

diesel::allow_tables_to_appear_in_same_query!(
    banks,
    transactions,
    users,
);
