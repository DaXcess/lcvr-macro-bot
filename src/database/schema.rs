// @generated automatically by Diesel CLI.

diesel::table! {
    attachment (id) {
        id -> Integer,
        macro_id -> Integer,
        link -> Text,
    }
}

diesel::table! {
    #[sql_name = "macro"]
    macro_ (id) {
        id -> Integer,
        name -> Text,
        description -> Text,
        content -> Text,
    }
}

diesel::joinable!(attachment -> macro_ (macro_id));

diesel::allow_tables_to_appear_in_same_query!(
    attachment,
    macro_,
);
