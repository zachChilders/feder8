use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_name: String,
    pub server_url: String,
    pub port: u16,
    pub actor_name: String,
    pub private_key_path: Option<String>,
    pub public_key_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_name: env::var("SERVER_NAME").unwrap_or_else(|_| "Fediverse Node".to_string()),
            server_url: env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            actor_name: env::var("ACTOR_NAME").unwrap_or_else(|_| "alice".to_string()),
            private_key_path: env::var("PRIVATE_KEY_PATH").ok(),
            public_key_path: env::var("PUBLIC_KEY_PATH").ok(),
        }
    }
} 