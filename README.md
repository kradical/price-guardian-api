# Price Guardian

Automated price protection tracking.

Track prices and notify if price drops so you can get your well deserved refund (store price protection, visa price protection, etc.)

- backend up and running locally
    - Rust, actix, juniper, diesel, postgres
- frontend up and running locally
    - svelte, graphql, a component lib? snow/webpack?
- deploy backend
    - dockerize -> ecs
    - cloudformation to manage infra.. or terraform.. other tools?
    - deploy infra / code changes on merge to master? through CI? circleci?
- deploy frontend
    - cloudfront / s3

# Setup Instructions

Instructions are for a linux environment. Specifically PopOS 20.

## Install Rust (latest stable)

Instructions taken from: https://www.rust-lang.org/tools/install

If you've never installed Rust:
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

If you've installed Rust in the past:
`rustup update`