CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id serial PRIMARY KEY,
    email CITEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW()
);

SELECT
    diesel_manage_updated_at ('users');

