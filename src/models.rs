use diesel::{Insertable, Queryable};
use juniper::{GraphQLInputObject, GraphQLObject};

use crate::schema::users;

#[derive(Queryable, GraphQLObject)]
pub struct User {
    pub id: i32,
    pub email: String,
    // TODO: deal with these fields
    // pub password: String,
    // pub created_at: String,
    // pub updated_at: String,
}

#[derive(Insertable, GraphQLInputObject)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub password: String,
}
