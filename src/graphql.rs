use actix_web::web;
use diesel::prelude::*;
use juniper::{
    graphql_object, EmptySubscription, FieldResult, GraphQLObject, GraphQLUnion, RootNode,
};
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

fn get_user_duplicate_error() -> ValidationError {
    ValidationError {
        field: "email".to_string(),
        code: "email_duplicate".to_string(),
        message: "a user already exists with this email".to_string(),
    }
}

pub struct Mutation;
#[graphql_object(context = Context)]
impl Mutation {
    async fn createUser(context: &Context, new_user: NewUser) -> FieldResult<NewUserResult> {
        use crate::schema::users::dsl::*;

        match new_user.validate() {
            Ok(_) => (),
            Err(e) => return Ok(NewUserResult::Err(ValidationErrors::new(e))),
        };

        let conn = context.db.get()?;

        let create_user = move || -> Result<NewUserResult, diesel::result::Error> {
            let new_user_pw_hashed = NewUser {
                password: hash_password(&new_user.password),
                ..new_user
            };

            let insert_result = diesel::insert_into(users)
                .values(&new_user_pw_hashed)
                .returning((id, email, created_at, updated_at))
                .get_result(&conn);

            match insert_result {
                Ok(user) => Ok(NewUserResult::Ok(user)),
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => Ok(NewUserResult::Err(ValidationErrors {
                    errors: vec![get_user_duplicate_error()],
                })),
                Err(e) => Err(e),
            }
        };

        Ok(web::block(create_user).await?)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
