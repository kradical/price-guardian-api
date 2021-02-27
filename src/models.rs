use argonautica::Hasher;
use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};
use std::env;
use validator::Validate;

use crate::schema::users;

#[derive(Queryable, GraphQLObject)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
