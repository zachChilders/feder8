mod config;
mod database;
mod handlers;
mod models;
mod services;

use actix_web::{middleware::Logger, web, App, HttpServer};
use database::{create_configured_mock_database, DatabaseRef};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = config::Config::default();

    tracing::info!("Starting Fediverse server on port {}", config.port);
    tracing::info!("Server URL: {}", config.server_url);
    tracing::info!("Actor name: {}", config.actor_name);

    // Initialize database (using mock for now)
    let db: DatabaseRef = Arc::new(create_configured_mock_database());
    tracing::info!("Database initialized (using mock)");

    let config_clone = config.clone();
    let db_clone = db.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(config_clone.clone()))
            .app_data(web::Data::new(db_clone.clone()))
            .service(handlers::webfinger::webfinger)
            .service(handlers::actor::get_actor)
            .service(handlers::inbox::inbox)
            .service(handlers::outbox::get_outbox)
            .service(handlers::outbox::post_outbox)
    })
    .bind(("127.0.0.1", config.port))?
    .run()
    .await
}
