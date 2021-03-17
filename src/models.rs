use argonautica::{Hasher, Verifier};
use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};
use std::env;
use uuid::Uuid;
use validator::Validate;

use crate::schema::{items, sessions, users};

#[derive(Clone, Identifiable, Queryable)]
#[table_name = "users"]
pub struct FullUser {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email: String,
    pub password: String,
}

#[derive(Queryable)]
pub struct User {
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

#[derive(Queryable, GraphQLObject)]
pub struct Session {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub user_id: i32,
}

#[derive(Insertable)]
#[table_name = "sessions"]
pub struct NewSession {
    pub user_id: i32,
}

#[derive(Queryable, GraphQLObject)]
pub struct Item {
    pub id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: i32,
    pub name: String,
    pub price: i32,
    pub protection_ends_at: DateTime<Utc>,
}

#[derive(Insertable, GraphQLInputObject, Validate)]
#[table_name = "items"]
pub struct NewItem {
    pub user_id: i32,
    #[validate(length(max = 250, message = "name must be less than 250 characters"))]
    pub name: String,
    #[validate(range(min = 0, message = "price must be more than 0"))]
    pub price: i32,
    pub protection_ends_at: DateTime<Utc>,
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
