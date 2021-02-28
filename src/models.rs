use argonautica::{Hasher, Verifier};
use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};
use std::env;
use uuid::Uuid;
use validator::Validate;

use crate::schema::{sessions, users};

#[derive(Debug, Clone, Identifiable, Queryable)]
#[table_name = "users"]
pub struct FullUser {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Queryable, GraphQLObject)]
pub struct User {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
}

#[derive(Debug, Insertable, GraphQLInputObject, Validate)]
#[table_name = "users"]
pub struct NewUser {
    #[validate(email(message = "email is invalid"))]
    pub email: String,
    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Queryable, GraphQLObject)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub user_id: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    pub user_id: i32,
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
