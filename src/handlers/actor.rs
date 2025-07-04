use crate::config::Config;
use crate::database::DatabaseRef;
use crate::models::Actor;
use actix_web::{get, web, HttpResponse, Result};
use tracing::warn;

#[get("/users/{username}")]
pub async fn get_actor(
    path: web::Path<String>,
    config: web::Data<Config>,
    db: web::Data<DatabaseRef>,
) -> Result<HttpResponse> {
    let username = path.into_inner();

    // Load actor from database
    match db.get_actor_by_username(&username).await {
        Ok(Some(db_actor)) => {
            let actor = Actor::new(
                db_actor.id.clone(),
                db_actor.name,
                db_actor.username,
                &config.server_url,
                db_actor.public_key_pem,
            );

            Ok(HttpResponse::Ok()
                .content_type("application/activity+json")
                .json(actor))
        }
        Ok(None) => {
            warn!("Actor not found: {}", username);
            Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "Actor not found"
            })))
        }
        Err(e) => {
            warn!("Database error while fetching actor {}: {}", username, e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error"
            })))
        }
    }
}
