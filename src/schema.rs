table! {
    users (id) {
        id -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        email -> Text,
        password -> Text,
        session_token -> Nullable<Text>,
    }
}
