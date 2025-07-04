mod config;
mod handlers;
mod models;
mod services;

use actix_web::{middleware::Logger, web, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let config = config::Config::default();

    tracing::info!("Starting Fediverse server on port {}", config.port);
    tracing::info!("Server URL: {}", config.server_url);
    tracing::info!("Actor name: {}", config.actor_name);

    let config_clone = config.clone();
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(config_clone.clone()))
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
