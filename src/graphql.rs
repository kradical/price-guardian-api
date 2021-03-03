use actix_web::web;
use diesel::prelude::*;
use juniper::{
    graphql_object, EmptySubscription, FieldResult, GraphQLInputObject, GraphQLObject,
    GraphQLUnion, RootNode,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::PgPool;
use crate::models::{hash_password, verify_password, FullUser, NewSession, NewUser, Session, User};

pub struct Context {
    pub db: PgPool,
    pub user: Option<User>,
}

#[derive(GraphQLInputObject)]
struct SessionIdInput {
    id: Uuid,
}

#[derive(GraphQLObject)]
struct SessionIdOutput {
    id: Uuid,
}

#[derive(GraphQLInputObject)]
struct UserIdInput {
    id: i32,
}

#[derive(GraphQLObject)]
struct UserIdOutput {
    id: i32,
}

#[derive(GraphQLObject)]
struct AuthenticationError {
    code: String,
    message: String,
}
impl Default for AuthenticationError {
    fn default() -> Self {
        AuthenticationError {
            code: "authentication_error".to_string(),
            message: "Authenticated user required".to_string(),
        }
    }
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
enum UserResponse {
    User(User),
    ValidationError(ValidationErrors),
    AuthenticationError(AuthenticationError),
}

#[derive(GraphQLUnion)]
enum UserIdResponse {
    UserIdOutput(UserIdOutput),
    AuthenticationError(AuthenticationError),
}

#[derive(GraphQLUnion)]
enum SessionIdResponse {
    SessionIdOutput(SessionIdOutput),
    AuthenticationError(AuthenticationError),
}

#[derive(GraphQLUnion)]
enum SessionResult {
    Ok(Session),
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

    async fn user(context: &Context, user_id: i32) -> FieldResult<UserResponse> {
        use crate::schema::users::dsl::*;

        // Authenticate user
        if context.user.is_none() {
            return Ok(UserResponse::AuthenticationError(Default::default()));
        }

        let conn = context.db.get()?;

        let find_user = move || -> Result<User, diesel::result::Error> {
            users
                .select((id, created_at, updated_at, email))
                .find(user_id)
                .first(&conn)
        };

        Ok(UserResponse::User(web::block(find_user).await?))
    }
}

pub struct Mutation;
#[graphql_object(context = Context)]
impl Mutation {
    async fn createUser(context: &Context, input: NewUser) -> FieldResult<UserResponse> {
        use crate::schema::users::dsl::*;

        match input.validate() {
            Ok(_) => (),
            Err(e) => return Ok(UserResponse::ValidationError(ValidationErrors::new(e))),
        };

        let conn = context.db.get()?;

        let create_user = move || -> Result<UserResponse, diesel::result::Error> {
            let new_user = NewUser {
                password: hash_password(&input.password),
                ..input
            };

            let insert_result = diesel::insert_into(users)
                .values(&new_user)
                .returning((id, created_at, updated_at, email))
                .get_result(&conn);

            match insert_result {
                Ok(user) => Ok(UserResponse::User(user)),
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => Ok(UserResponse::ValidationError(ValidationErrors {
                    errors: vec![get_user_duplicate_error()],
                })),
                Err(e) => Err(e),
            }
        };

        Ok(web::block(create_user).await?)
    }

    async fn deleteUser(context: &Context, input: UserIdInput) -> FieldResult<UserIdResponse> {
        use crate::schema::users::dsl::*;

        // Authenticate user
        if context.user.is_none() {
            return Ok(UserIdResponse::AuthenticationError(Default::default()));
        }

        let conn = context.db.get()?;

        let user_id = input.id;

        let delete = move || -> Result<usize, diesel::result::Error> {
            diesel::delete(users.filter(id.eq(user_id))).execute(&conn)
        };

        web::block(delete).await?;

        Ok(UserIdResponse::UserIdOutput(UserIdOutput { id: user_id }))
    }

    async fn changePassword(
        context: &Context,
        input: ChangePasswordUser,
    ) -> FieldResult<UserResponse> {
        use crate::schema::users::dsl::*;

        // Authenticate user
        if context.user.is_none() {
            return Ok(UserResponse::AuthenticationError(Default::default()));
        }

        match input.validate() {
            Ok(_) => (),
            Err(e) => return Ok(UserResponse::ValidationError(ValidationErrors::new(e))),
        };

        let conn = context.db.get()?;

        let change_password = move || -> Result<UserResponse, diesel::result::Error> {
            let user = users.find(input.id).first::<FullUser>(&conn)?;

            let is_valid = verify_password(&input.old_password, &user.password);

            if !is_valid {
                let err = ValidationError {
                    field: "old_password".to_string(),
                    code: "old_password_incorrect".to_string(),
                    message: "incorrect old password".to_string(),
                };

                let errs = ValidationErrors { errors: vec![err] };

                return Ok(UserResponse::ValidationError(errs));
            }

            let new_password = hash_password(&input.new_password);

            let slim_user = diesel::update(&user)
                .set(password.eq(new_password))
                .returning((id, created_at, updated_at, email))
                .get_result::<User>(&conn)?;

            Ok(UserResponse::User(slim_user))
        };

        Ok(web::block(change_password).await?)
    }

    async fn login(context: &Context, input: NewUser) -> FieldResult<SessionResult> {
        use crate::schema::{sessions, users};

        let conn = context.db.get()?;

        let login = move || -> Result<SessionResult, diesel::result::Error> {
            let user = users::table
                .filter(users::email.eq(input.email))
                .first::<FullUser>(&conn)?;

            let is_valid = verify_password(&input.password, &user.password);

            if !is_valid {
                let err = ValidationError {
                    field: "password".to_string(),
                    code: "authentication_error".to_string(),
                    message: "incorrect credentials".to_string(),
                };

                let errs = ValidationErrors { errors: vec![err] };

                return Ok(SessionResult::Err(errs));
            }

            let new_session = NewSession { user_id: user.id };

            let session = diesel::insert_into(sessions::table)
                .values(&new_session)
                .get_result::<Session>(&conn)?;

            Ok(SessionResult::Ok(session))
        };

        Ok(web::block(login).await?)
    }

    async fn logout(context: &Context, input: SessionIdInput) -> FieldResult<SessionIdResponse> {
        use crate::schema::sessions::dsl::*;

        // Authenticate user
        if context.user.is_none() {
            return Ok(SessionIdResponse::AuthenticationError(Default::default()));
        }

        let conn = context.db.get()?;

        let session_id = input.id;

        let logout = move || -> Result<usize, diesel::result::Error> {
            diesel::delete(sessions.filter(id.eq(session_id))).execute(&conn)
        };

        web::block(logout).await?;

        Ok(SessionIdResponse::SessionIdOutput(SessionIdOutput {
            id: session_id,
        }))
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
