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
            server_url: env::var("SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Shared mutex to prevent parallel tests from interfering with environment variables
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_default_values() {
        // Use a lock to prevent parallel tests from interfering with each other
        let _guard = ENV_LOCK.lock().unwrap();

        // Clear environment variables to test defaults
        let env_vars = [
            "SERVER_NAME",
            "SERVER_URL",
            "PORT",
            "ACTOR_NAME",
            "PRIVATE_KEY_PATH",
            "PUBLIC_KEY_PATH",
        ];
        let original_values: Vec<_> = env_vars.iter().map(|var| env::var(var).ok()).collect();

        for var in &env_vars {
            env::remove_var(var);
        }

        let config = Config::default();

        assert_eq!(config.server_name, "Fediverse Node");
        assert_eq!(config.server_url, "http://localhost:8080");
        assert_eq!(config.port, 8080);
        assert_eq!(config.actor_name, "alice");
        assert_eq!(config.private_key_path, None);
        assert_eq!(config.public_key_path, None);

        // Restore original values
        for (i, var) in env_vars.iter().enumerate() {
            if let Some(value) = &original_values[i] {
                env::set_var(var, value);
            }
        }
    }

    #[test]
    fn test_config_from_environment() {
        // Use a lock to prevent parallel tests from interfering with each other
        let _guard = ENV_LOCK.lock().unwrap();

        // Store original values to restore later
        let original_values: Vec<_> = [
            "SERVER_NAME",
            "SERVER_URL",
            "PORT",
            "ACTOR_NAME",
            "PRIVATE_KEY_PATH",
            "PUBLIC_KEY_PATH",
        ]
        .iter()
        .map(|var| env::var(var).ok())
        .collect();

        // Set environment variables
        env::set_var("SERVER_NAME", "Test Server");
        env::set_var("SERVER_URL", "https://test.example.com");
        env::set_var("PORT", "9090");
        env::set_var("ACTOR_NAME", "testuser");
        env::set_var("PRIVATE_KEY_PATH", "/path/to/private.pem");
        env::set_var("PUBLIC_KEY_PATH", "/path/to/public.pem");

        let config = Config::default();

        assert_eq!(config.server_name, "Test Server");
        assert_eq!(config.server_url, "https://test.example.com");
        assert_eq!(config.port, 9090);
        assert_eq!(config.actor_name, "testuser");
        assert_eq!(
            config.private_key_path,
            Some("/path/to/private.pem".to_string())
        );
        assert_eq!(
            config.public_key_path,
            Some("/path/to/public.pem".to_string())
        );

        // Restore original values or remove if they weren't set
        let env_vars = [
            "SERVER_NAME",
            "SERVER_URL",
            "PORT",
            "ACTOR_NAME",
            "PRIVATE_KEY_PATH",
            "PUBLIC_KEY_PATH",
        ];
        for (i, var) in env_vars.iter().enumerate() {
            if let Some(value) = &original_values[i] {
                env::set_var(var, value);
            } else {
                env::remove_var(var);
            }
        }
    }

    #[test]
    fn test_config_invalid_port_fallback() {
        // Use a lock to prevent parallel tests from interfering with each other
        let _guard = ENV_LOCK.lock().unwrap();

        let original_port = env::var("PORT").ok();

        env::set_var("PORT", "invalid_port");

        let config = Config::default();
        assert_eq!(config.port, 8080); // Should fallback to default

        // Restore original value or remove if it wasn't set
        if let Some(value) = original_port {
            env::set_var("PORT", value);
        } else {
            env::remove_var("PORT");
        }
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            server_name: "Test".to_string(),
            server_url: "http://test.com".to_string(),
            port: 8080,
            actor_name: "test".to_string(),
            private_key_path: Some("/private".to_string()),
            public_key_path: Some("/public".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.server_name, deserialized.server_name);
        assert_eq!(config.server_url, deserialized.server_url);
        assert_eq!(config.port, deserialized.port);
        assert_eq!(config.actor_name, deserialized.actor_name);
        assert_eq!(config.private_key_path, deserialized.private_key_path);
        assert_eq!(config.public_key_path, deserialized.public_key_path);
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        assert_eq!(config.server_name, cloned.server_name);
        assert_eq!(config.server_url, cloned.server_url);
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.actor_name, cloned.actor_name);
        assert_eq!(config.private_key_path, cloned.private_key_path);
        assert_eq!(config.public_key_path, cloned.public_key_path);
    }
}
