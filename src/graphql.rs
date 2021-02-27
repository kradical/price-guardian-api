use actix_web::web;
use diesel::prelude::*;
use juniper::{graphql_object, EmptySubscription, GraphQLObject, GraphQLUnion, RootNode};
use validator::Validate;

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

#[derive(GraphQLObject)]
pub struct ValidationError {
    field: String,
    code: String,
    message: String,
}

#[derive(GraphQLObject)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    fn new(validation_errors: validator::ValidationErrors) -> ValidationErrors {
        let errors = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |err| {
                    let message = match &err.message {
                        Some(m) => m.to_string(),
                        None => "".to_string(),
                    };

                    ValidationError {
                        field: field.to_string(),
                        code: err.code.to_string(),
                        message,
                    }
                })
            })
            .collect();

        ValidationErrors { errors }
    }
}

#[derive(GraphQLUnion)]
pub enum NewUserResult {
    Ok(User),
    Err(ValidationErrors),
}

pub struct Mutation;
#[graphql_object(context = Context)]
impl Mutation {
    async fn createUser(context: &Context, new_user: NewUser) -> NewUserResult {
        match new_user.validate() {
            Ok(_) => (),
            Err(e) => return NewUserResult::Err(ValidationErrors::new(e)),
        };

        use crate::schema::users::dsl::*;

        let conn = context.db.get().expect("Error getting db connection");

        let user = web::block(move || -> Result<User, diesel::result::Error> {
            let new_user_pw_hashed = NewUser {
                password: hash_password(&new_user.password),
                ..new_user
            };
            let user_returning = (id, email, created_at, updated_at);
            let user = diesel::insert_into(users)
                .values(&new_user_pw_hashed)
                .returning(user_returning)
                .get_result(&conn)?;

            Ok(user)
        })
        .await
        .unwrap();

        NewUserResult::Ok(user)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
