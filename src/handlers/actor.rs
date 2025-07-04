use actix_web::{get, web, HttpResponse, Result};
use crate::config::Config;
use crate::models::Actor;

#[get("/users/{username}")]
pub async fn get_actor(
    path: web::Path<String>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let username = path.into_inner();
    
    // For now, we'll create a simple actor with a placeholder public key
    // In a real implementation, you'd load this from a database
    let actor = Actor::new(
        format!("{}/users/{}", config.server_url, username),
        username.clone(),
        username,
        &config.server_url,
        "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----".to_string(),
    );
    
    Ok(HttpResponse::Ok()
        .content_type("application/activity+json")
        .json(actor))
} 