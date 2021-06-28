#[macro_use]
extern crate diesel;
extern crate argon2;

mod models;
mod schema;
mod handlers;
mod utils;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{middleware, web, App, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use handlers::*;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "simple-auth-server=debug,actix_web=info,actix_server=info");
    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("Database url must be provided.");

    //create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create a pool.");
    let domain: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    //start http server
    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            //enable loggger
            .wrap(middleware::Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(utils::password_hash::SECRET_KEY.as_bytes())
                    .name("auth")
                    .path("/")
                    .domain(domain.as_str())
                    .max_age(86400) // 24 hours in secnods
                    .secure(false),
            ))
            //limit the maximum size of data that the server will accept
            .data(web::JsonConfig::default().limit(4096))
            //handle everything under /api route
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/auth")
                            .route(web::post().to(auth_handler::sign_in))
                            .route(web::delete().to(auth_handler::sign_out))
                            .route(web::get().to(auth_handler::get_me))
                    )
                    .service(
                        web::resource("/invitation")
                            .route(web::post().to(invitation_handler::post_invitation)),
                    )
                    .service(
                        web::resource("/register/{invitation_id}")
                            .route(web::post().to(register_handler::register_user))
                    )
            )
    })
    .bind("127.0.0.1:9100")?
    .run()
    .await
}