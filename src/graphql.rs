use actix_web::web;
use juniper::{graphql_object, EmptySubscription, FieldResult, RootNode};

use crate::db::PgPool;
use crate::models::{hash_password, NewUser, User};

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
#[graphql_object(context = Context)]
impl Mutation {
    async fn createUser(context: &Context, new_user: NewUser) -> FieldResult<User> {
        use crate::schema::users;
        use diesel::query_dsl::*;

        // TODO
        // - input validation
        //   - email -> unique, add index and handle failure
        //   - password

        let conn = context.db.get().expect("Error getting db connection");

        let user = web::block(move || -> Result<User, diesel::result::Error> {
            let new_user_pw_hashed = NewUser {
                password: hash_password(&new_user.password),
                ..new_user
            };
            let user_returning = (
                users::id,
                users::email,
                users::created_at,
                users::updated_at,
            );
            let user = diesel::insert_into(users::table)
                .values(&new_user_pw_hashed)
                .returning(user_returning)
                .get_result(&conn)?;

            Ok(user)
        })
        .await?;

        Ok(user)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
