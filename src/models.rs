use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};

use crate::schema::users;

#[derive(Queryable, GraphQLObject)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, GraphQLInputObject)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub password: String,
}
