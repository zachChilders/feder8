mod config;
mod database;
mod handlers;
mod models;
mod services;
mod http;
mod container;

use actix_web::{middleware::Logger, web, App, HttpServer};
use database::{create_configured_mock_database, DatabaseRef};
use container::Container;
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

    // Initialize dependency injection container
    let container = Container::new(config.clone(), db);
    tracing::info!("Dependency injection container initialized");

    let container_clone = container.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(container_clone.config().clone()))
            .app_data(web::Data::new(container_clone.database().clone()))
            .app_data(web::Data::new(container_clone.clone()))
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
