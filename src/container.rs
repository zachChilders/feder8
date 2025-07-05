use crate::config::Config;
use crate::database::DatabaseRef;
use crate::http::{HttpClient, ReqwestClient};
use crate::services::delivery::DeliveryService;
use std::sync::Arc;
use std::time::Duration;

/// Dependency injection container that manages all application dependencies
pub struct Container {
    config: Config,
    database: DatabaseRef,
    http_client: Arc<dyn HttpClient>,
    delivery_service: Arc<DeliveryService>,
}

impl Container {
    /// Create a new container with default implementations
    pub fn new(config: Config, database: DatabaseRef) -> Self {
        // Create HTTP client
        let http_client: Arc<dyn HttpClient> = Arc::new(ReqwestClient::with_timeout(Duration::from_secs(30)));
        
        // Create delivery service with injected HTTP client
        let delivery_service = Arc::new(DeliveryService::new(config.clone(), http_client.clone()));
        
        Self {
            config,
            database,
            http_client,
            delivery_service,
        }
    }

    /// Create a new container with custom HTTP client
    pub fn with_http_client(config: Config, database: DatabaseRef, http_client: Arc<dyn HttpClient>) -> Self {
        let delivery_service = Arc::new(DeliveryService::new(config.clone(), http_client.clone()));
        
        Self {
            config,
            database,
            http_client,
            delivery_service,
        }
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the database reference
    pub fn database(&self) -> &DatabaseRef {
        &self.database
    }

    /// Get the HTTP client
    pub fn http_client(&self) -> &Arc<dyn HttpClient> {
        &self.http_client
    }

    /// Get the delivery service
    pub fn delivery_service(&self) -> &Arc<DeliveryService> {
        &self.delivery_service
    }

    /// Create a clone of the container for use in different contexts
    pub fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            database: self.database.clone(),
            http_client: self.http_client.clone(),
            delivery_service: self.delivery_service.clone(),
        }
    }
}

/// Builder pattern for creating containers with different configurations
pub struct ContainerBuilder {
    config: Option<Config>,
    database: Option<DatabaseRef>,
    http_client: Option<Arc<dyn HttpClient>>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            database: None,
            http_client: None,
        }
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_database(mut self, database: DatabaseRef) -> Self {
        self.database = Some(database);
        self
    }

    pub fn with_http_client(mut self, http_client: Arc<dyn HttpClient>) -> Self {
        self.http_client = Some(http_client);
        self
    }

    pub fn build(self) -> Result<Container, String> {
        let config = self.config.ok_or("Config is required")?;
        let database = self.database.ok_or("Database is required")?;
        
        match self.http_client {
            Some(http_client) => Ok(Container::with_http_client(config, database, http_client)),
            None => Ok(Container::new(config, database)),
        }
    }
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::create_configured_mock_database;
    use crate::http::HttpClient;
    use crate::http::{HttpRequest, HttpResponse as HttpClientResponse, StatusCode};
    use anyhow::Result;
    use std::collections::HashMap;

    // Mock HTTP client for testing
    struct MockHttpClient;

    #[async_trait::async_trait]
    impl HttpClient for MockHttpClient {
        async fn send(&self, _request: HttpRequest) -> Result<HttpClientResponse> {
            Ok(HttpClientResponse {
                status: StatusCode(200),
                headers: HashMap::new(),
                body: b"OK".to_vec(),
            })
        }
    }

    fn create_test_config() -> Config {
        Config {
            server_name: "Test Server".to_string(),
            server_url: "https://test.example.com".to_string(),
            port: 8080,
            actor_name: "testuser".to_string(),
            private_key_path: None,
            public_key_path: None,
        }
    }

    #[test]
    fn test_container_creation() {
        let config = create_test_config();
        let database = Arc::new(create_configured_mock_database());
        let container = Container::new(config.clone(), database);

        assert_eq!(container.config().server_name, config.server_name);
        assert_eq!(container.config().server_url, config.server_url);
        assert_eq!(container.config().port, config.port);
        assert_eq!(container.config().actor_name, config.actor_name);
    }

    #[test]
    fn test_container_with_custom_http_client() {
        let config = create_test_config();
        let database = Arc::new(create_configured_mock_database());
        let http_client: Arc<dyn HttpClient> = Arc::new(MockHttpClient);
        let container = Container::with_http_client(config.clone(), database, http_client);

        assert_eq!(container.config().server_name, config.server_name);
        assert_eq!(container.config().server_url, config.server_url);
        assert_eq!(container.config().port, config.port);
        assert_eq!(container.config().actor_name, config.actor_name);
    }

    #[test]
    fn test_container_builder() {
        let config = create_test_config();
        let database = Arc::new(create_configured_mock_database());
        let http_client: Arc<dyn HttpClient> = Arc::new(MockHttpClient);

        let container = ContainerBuilder::new()
            .with_config(config.clone())
            .with_database(database)
            .with_http_client(http_client)
            .build()
            .unwrap();

        assert_eq!(container.config().server_name, config.server_name);
        assert_eq!(container.config().server_url, config.server_url);
        assert_eq!(container.config().port, config.port);
        assert_eq!(container.config().actor_name, config.actor_name);
    }

    #[test]
    fn test_container_builder_missing_config() {
        let database = Arc::new(create_configured_mock_database());
        
        let result = ContainerBuilder::new()
            .with_database(database)
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Config is required");
    }

    #[test]
    fn test_container_builder_missing_database() {
        let config = create_test_config();
        
        let result = ContainerBuilder::new()
            .with_config(config)
            .build();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Database is required");
    }

    #[test]
    fn test_container_clone() {
        let config = create_test_config();
        let database = Arc::new(create_configured_mock_database());
        let container = Container::new(config.clone(), database);
        let cloned_container = container.clone();

        assert_eq!(container.config().server_name, cloned_container.config().server_name);
        assert_eq!(container.config().server_url, cloned_container.config().server_url);
        assert_eq!(container.config().port, cloned_container.config().port);
        assert_eq!(container.config().actor_name, cloned_container.config().actor_name);
    }
}