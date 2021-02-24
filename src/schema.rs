table! {
    users (id) {
        id -> Int4,
        email -> Citext,
        password -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
