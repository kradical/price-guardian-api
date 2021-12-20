# Price Guardian

Automated price protection tracking.

Track prices and notify if price drops so you can get your well deserved refund (store price protection, visa price protection, etc.)

Examples:
- project layout: https://github.com/golang-standards/project-layout

TODO:
- backend up and running locally
  - Go, fiber
- frontend up and running locally
  - svelte/sveltekit
- deploy backend
  - dockerize -> ecs
  - cloudformation to manage infra.. or terraform.. other tools?
  - deploy infra / code changes on merge to master? through CI? circleci?
- deploy frontend
  - cloudfront / s3

General todos:

- user / auth system
- reset password flow
- confirm email flow
  - remove permissions on unconfirmed email account ??
- add 2FA
- add oauth signup / login (is amazon possible?? lookup popular options)
- logging
- add tests
- add price monitoring
  - amazon first, or do they not have a policy?
- model out a price protection policy, allow it to be global (credit card or merchant level) and then applied to specific items (purchased with)
- items should be able to have multiple price protection policies

# Setup Instructions

Instructions are for a MacOS environment (M1 arm)

## Install Go

Follow instructions here: https://go.dev/doc/install

## Install air

Air is for hot reloading.

Follow instructions here: https://github.com/cosmtrek/air#installation

## Database setup

Install Postgresql and the trimmings:

```
➤ brew install postgres
```

## Compile and Run

Run Locally w/ hot reloading:

```
➤ air
```

Release Build:
