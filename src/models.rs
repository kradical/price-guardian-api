use argonautica::{Hasher, Verifier};
use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};
use std::env;
use validator::Validate;

use crate::schema::users;

#[derive(Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub password: String,
    pub session_token: Option<String>,
}

#[derive(Identifiable, Queryable, GraphQLObject)]
#[table_name = "users"]
pub struct TokenUser {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub session_token: Option<String>,
}

impl From<User> for TokenUser {
    fn from(user: User) -> TokenUser {
        TokenUser {
            id: user.id,
            created_at: user.created_at,
            updated_at: user.updated_at,
            email: user.email,
            session_token: user.session_token,
        }
    }
}

#[derive(Queryable, GraphQLObject)]
#[graphql(name = "User")]
pub struct SlimUser {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
}

#[derive(Insertable, GraphQLInputObject, Validate)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(email(message = "email is invalid"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
}

pub fn hash_password(password: &str) -> String {
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let mut hasher = Hasher::default();
    hasher
        .with_password(password)
        .with_secret_key(secret_key)
        .hash()
        .unwrap()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let mut verifier = Verifier::default();
    verifier
        .with_password(password)
        .with_hash(hash)
        .with_secret_key(secret_key)
        .verify()
        .unwrap()
}
