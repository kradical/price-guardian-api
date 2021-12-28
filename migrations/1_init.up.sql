BEGIN;

CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  email CITEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW()
);

SELECT pgx_manage_updated_at('users');

CREATE TABLE items (
  id serial PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT NOW(),
  updated_at timestamptz NOT NULL DEFAULT NOW(),
  user_id int NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  name text NOT NULL,
  price int NOT NULL
);

SELECT pgx_manage_updated_at('items');

COMMIT;
