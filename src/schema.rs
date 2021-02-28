table! {
    sessions (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        user_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        email -> Text,
        password -> Text,
    }
}

joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(
    sessions,
    users,
);
