use actix_web::{get, web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use crate::config::Config;
use crate::models::Actor;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFingerQuery {
    resource: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFingerResponse {
    pub subject: String,
    pub links: Vec<WebFingerLink>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebFingerLink {
    pub rel: String,
    #[serde(rename = "type")]
    pub link_type: Option<String>,
    pub href: String,
}

#[get("/.well-known/webfinger")]
pub async fn webfinger(
    query: web::Query<WebFingerQuery>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    let resource = &query.resource;
    
    // Parse the resource to extract username
    // Expected format: acct:username@domain
    if let Some(username) = resource.strip_prefix("acct:") {
        if let Some((user, domain)) = username.rsplit_once('@') {
            if domain == config.server_url.replace("http://", "").replace("https://", "") {
                let actor_url = format!("{}/users/{}", config.server_url, user);
                
                let response = WebFingerResponse {
                    subject: resource.clone(),
                    links: vec![
                        WebFingerLink {
                            rel: "self".to_string(),
                            link_type: Some("application/activity+json".to_string()),
                            href: actor_url,
                        },
                        WebFingerLink {
                            rel: "http://webfinger.net/rel/profile-page".to_string(),
                            link_type: Some("text/html".to_string()),
                            href: format!("{}/users/{}", config.server_url, user),
                        },
                    ],
                };
                
                return Ok(HttpResponse::Ok()
                    .content_type("application/jrd+json")
                    .json(response));
            }
        }
    }
    
    Ok(HttpResponse::NotFound().finish())
} 