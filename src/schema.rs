table! {
    items (id) {
        id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Int4,
        name -> Text,
        price -> Int4,
        protection_ends_at -> Timestamptz,
    }
}

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

joinable!(items -> users (user_id));
joinable!(sessions -> users (user_id));

allow_tables_to_appear_in_same_query!(items, sessions, users);
