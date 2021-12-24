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

COMMIT;
