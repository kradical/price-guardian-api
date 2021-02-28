#[macro_use]
extern crate diesel;
extern crate argonautica;

use actix_cors::Cors;
use actix_web::{
    dev, http::header, middleware, web, App, Error, FromRequest, HttpRequest, HttpResponse,
    HttpServer,
};
use diesel::prelude::*;
use futures::{
    executor::block_on,
    future::{ok, Ready},
};
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler};

mod db;
mod graphql;
mod models;
mod schema;

async fn graphiql_route() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphgl", None).await
}
async fn playground_route() -> Result<HttpResponse, Error> {
    playground_handler("/graphgl", None).await
}
async fn graphql_route(
    req: HttpRequest,
    payload: web::Payload,
    schema: web::Data<graphql::Schema>,
) -> Result<HttpResponse, Error> {
    let mut dev_payload = payload.into_inner();
    let context = graphql::Context::from_request(&req, &mut dev_payload).await?;

    let payload = web::Payload(dev_payload.take());
    graphql_handler(&schema, &context, req, payload).await
}

impl FromRequest for graphql::Context {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let pool = req.app_data::<web::Data<db::PgPool>>().unwrap().as_ref();

        let connection = pool.get().unwrap();
        let current_user = block_on(get_current_user(req, connection));

        ok(graphql::Context {
            db: pool.clone(),
            user: current_user,
        })
    }
}

fn parse_bearer_token(req: &HttpRequest) -> Option<String> {
    let header = req.headers().get("Authorization")?;
    let header_str = header.to_str().ok()?;

    header_str
        .to_lowercase()
        .starts_with("bearer")
        .then(|| header_str[6..header_str.len()].trim().to_string())
}

async fn get_current_user(req: &HttpRequest, conn: db::PgPooledConnection) -> Option<models::User> {
    use crate::schema::users::dsl::*;

    let token = match parse_bearer_token(&req) {
        Some(v) => v,
        None => return None,
    };

    web::block(move || -> Result<models::User, ()> {
        users
            .filter(session_token.eq(token))
            .first::<models::User>(&conn)
            .map_err(|_e| ())
    })
    .await
    .ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let pool = db::build_pool();

    let server = HttpServer::new(move || {
        App::new()
            .data(graphql::schema())
            .data(pool.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(
                web::resource("/graphgl")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql_route)))
    });

    let bind = "127.0.0.1:8080";
    println!("Starting server at: {}", &bind);

    server.bind(bind)?.run().await
}
