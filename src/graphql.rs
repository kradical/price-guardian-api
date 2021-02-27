use actix_web::web;
use diesel::prelude::*;
use juniper::{
    graphql_object, EmptySubscription, FieldResult, GraphQLInputObject, GraphQLObject,
    GraphQLUnion, RootNode,
};
use validator::Validate;

use crate::db::PgPool;
use crate::models::{hash_password, verify_password, NewUser, User, UserWithPassword};

#[derive(Clone)]
pub struct Context {
    pub db: PgPool,
}

#[derive(GraphQLObject)]
struct ValidationError {
    field: String,
    code: String,
    message: String,
}

#[derive(GraphQLObject)]
struct ValidationErrors {
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
enum UserResult {
    Ok(User),
    Err(ValidationErrors),
}

#[derive(GraphQLInputObject, Validate)]
struct ChangePasswordUser {
    pub id: i32,
    pub old_password: String,
    #[validate(length(min = 8, message = "new password must be at least 8 characters"))]
    pub new_password: String,
}

fn get_user_duplicate_error() -> ValidationError {
    ValidationError {
        field: "email".to_string(),
        code: "email_duplicate".to_string(),
        message: "a user already exists with this email".to_string(),
    }
}

impl juniper::Context for Context {}

// Queries represent the callable funcitons
pub struct Query;
#[graphql_object(context = Context)]
impl Query {
    fn apiVersion() -> String {
        "0.1.0".to_string()
    }

    async fn user(context: &Context, user_id: i32) -> FieldResult<User> {
        use crate::schema::users::dsl::*;

        let conn = context.db.get()?;

        let find_user = move || -> Result<User, diesel::result::Error> {
            users
                .select((id, email, created_at, updated_at))
                .find(user_id)
                .first(&conn)
        };

        Ok(web::block(find_user).await?)
    }
}

pub struct Mutation;
#[graphql_object(context = Context)]
impl Mutation {
    async fn createUser(context: &Context, input: NewUser) -> FieldResult<UserResult> {
        use crate::schema::users::dsl::*;

        match input.validate() {
            Ok(_) => (),
            Err(e) => return Ok(UserResult::Err(ValidationErrors::new(e))),
        };

        let conn = context.db.get()?;

        let create_user = move || -> Result<UserResult, diesel::result::Error> {
            let new_user = NewUser {
                password: hash_password(&input.password),
                ..input
            };

            let insert_result = diesel::insert_into(users)
                .values(&new_user)
                .returning((id, email, created_at, updated_at))
                .get_result(&conn);

            match insert_result {
                Ok(user) => Ok(UserResult::Ok(user)),
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => Ok(UserResult::Err(ValidationErrors {
                    errors: vec![get_user_duplicate_error()],
                })),
                Err(e) => Err(e),
            }
        };

        Ok(web::block(create_user).await?)
    }

    async fn changePassword(
        context: &Context,
        input: ChangePasswordUser,
    ) -> FieldResult<UserResult> {
        use crate::schema::users::dsl::*;

        match input.validate() {
            Ok(_) => (),
            Err(e) => return Ok(UserResult::Err(ValidationErrors::new(e))),
        };

        let conn = context.db.get()?;

        let change_password = move || -> Result<UserResult, diesel::result::Error> {
            let user_with_pw = users.find(input.id).first::<UserWithPassword>(&conn)?;

            let is_valid = verify_password(&input.old_password, &user_with_pw.password);

            if !is_valid {
                let err = ValidationError {
                    field: "old_password".to_string(),
                    code: "old_password_incorrect".to_string(),
                    message: "incorrect old password".to_string(),
                };

                let errs = ValidationErrors { errors: vec![err] };

                return Ok(UserResult::Err(errs));
            }

            let new_password = hash_password(&input.new_password);

            let user = diesel::update(&user_with_pw)
                .set(password.eq(new_password))
                .returning((id, email, created_at, updated_at))
                .get_result::<User>(&conn)?;

            // Ok(UserResult::Ok(user))
            Ok(UserResult::Ok(user))
        };

        Ok(web::block(change_password).await?)
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
