use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode, ScalarValue};

use crate::db::PgPool;
use crate::models::{NewUser, User};

#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
}

impl juniper::Context for Context {}

// Queries represent the callable funcitons
pub struct Query;
#[graphql_object(context = Context)]
impl Query {
    fn apiVersion() -> String {
        "0.1.0".to_string()
    }
}

pub struct Mutation;
#[graphql_object(
    context = Context,
    scalar = S,
)]
impl<S: ScalarValue + std::fmt::Display> Mutation {
    fn createUser(context: &Context, new_user: NewUser) -> FieldResult<User, S> {
        use crate::schema::users::dsl::*;
        use diesel::query_dsl::*;

        let conn = context
            .db
            .get()
            .map_err(|_e| "Error getting db connection")?;
        diesel::insert_into(users)
            .values(&new_user)
            .execute(&conn)
            // .get_result(&conn)
            .map_err(|_e| "Error inserting user")?;

        Ok(User {
            email: new_user.email,
            id: 123,
        })
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
