# Price Guardian

Automated price protection tracking.

Track prices and notify if price drops so you can get your well deserved refund (store price protection, visa price protection, etc.)

Examples:
- project layout: https://github.com/golang-standards/project-layout
- https://github.com/jackc/pgx/issues/81#issuecomment-296446179

TODO:
- model out "item"
- error handling (return json, jwt errors, random errors)
- logging
- refactor into cleaner file structure
- add tests
- frontend up and running locally
  - svelte (sveltekit?)
- add item pagination
- how to turn item PATCH into a partial update?
- add price monitoring
  - amazon first, or do they not have a policy?
- model out a price protection policy, allow it to be global (credit card or merchant level) and then applied to specific items (purchased with)
- user / auth system
  - add email confirmation
  - reset password flow
  - add 2FA
  - add oauth signup / login (is amazon possible?? lookup popular options)
- items should be able to have multiple price protection policies
- deploy backend
- deploy frontend

# Setup Instructions

Instructions are for a MacOS environment (M1 arm)

## Install Go

Follow instructions here: https://go.dev/doc/install

## Install air

Air is for hot reloading.

Follow instructions here: https://github.com/cosmtrek/air#installation

## Database setup

1. Install Postgresql and the migration cli:

```
➤ brew install postgres golang-migrate
```

2. Create database

```
➤ createdb price-guardian
```

3. Run migrations

```
➤ migrate -path migrations -database pgx://localhost:5432/price-guardian\?sslmode=disable up
```

4. Create a mgiration

```
➤ migrate create -ext sql NAME
```

## Compile and Run

Run Locally w/ hot reloading:

```
➤ air
```

## Release Build
