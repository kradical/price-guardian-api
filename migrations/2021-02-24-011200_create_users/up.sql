CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
    id serial PRIMARY KEY,
    email CITEXT NOT NULL,
    password TEXT NOT NULL,
    created_at timestamp NOT NULL DEFAULT NOW(),
    updated_at timestamp NOT NULL DEFAULT NOW()
);

