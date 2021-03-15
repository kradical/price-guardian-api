CREATE EXTENSION IF NOT EXISTS "citext";

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id serial PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    email CITEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

SELECT
    diesel_manage_updated_at ('users');

CREATE TABLE sessions (
    id uuid DEFAULT uuid_generate_v4 () PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    user_id int NOT NULL REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE items (
    id serial PRIMARY KEY,
    created_at timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at timestamp with time zone NOT NULL DEFAULT NOW(),
    user_id int NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    price int NOT NULL,
    protection_ends_at timestamp with time zone NOT NULL
);

SELECT
    diesel_manage_updated_at ('items');

