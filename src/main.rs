#[macro_use]
extern crate diesel;
extern crate argonautica;

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error, HttpResponse, HttpServer};
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
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    context: web::Data<graphql::Context>,
    schema: web::Data<graphql::Schema>,
) -> Result<HttpResponse, Error> {
    graphql_handler(&schema, &context, req, payload).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    let pool = db::build_pool();
    let schema_context = graphql::Context { db: pool.clone() };

    let server = HttpServer::new(move || {
        App::new()
            .data(graphql::schema())
            .data(schema_context.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
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

    server.bind(bind).unwrap().run().await
}
