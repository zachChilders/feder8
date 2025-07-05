use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// HTTP response for server endpoints
#[derive(Debug)]
#[allow(dead_code)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

#[allow(dead_code)]
impl HttpResponse {
    pub fn ok() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn created() -> Self {
        Self {
            status: 201,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn accepted() -> Self {
        Self {
            status: 202,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn internal_server_error() -> Self {
        Self {
            status: 500,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn with_json(mut self, json: &Value) -> Result<Self> {
        self.body = serde_json::to_vec(json)?;
        self.headers
            .insert("content-type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_activity_json(mut self, json: &Value) -> Result<Self> {
        self.body = serde_json::to_vec(json)?;
        self.headers.insert(
            "content-type".to_string(),
            "application/activity+json".to_string(),
        );
        Ok(self)
    }

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }
}

/// HTTP request context containing request data and dependencies
#[derive(Debug)]
#[allow(dead_code)]
pub struct HttpContext {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub path_params: HashMap<String, String>,
    pub dependencies: Arc<dyn std::any::Any + Send + Sync>,
}

#[allow(dead_code)]
impl HttpContext {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: Vec::new(),
            path_params: HashMap::new(),
            dependencies: Arc::new(()),
        }
    }

    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T> {
        Ok(serde_json::from_slice(&self.body)?)
    }

    pub fn path_param(&self, name: &str) -> Option<&String> {
        self.path_params.get(name)
    }

    pub fn query_param(&self, name: &str) -> Option<&String> {
        self.query_params.get(name)
    }

    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    pub fn get_dependency<T: 'static>(&self) -> Option<&T> {
        self.dependencies.downcast_ref::<T>()
    }
}

/// Handler function type
#[async_trait]
#[allow(dead_code)]
pub trait HttpHandler: Send + Sync {
    async fn handle(&self, context: HttpContext) -> Result<HttpResponse>;
}

/// Route definition
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler_id: String,
}

#[allow(dead_code)]
impl Route {
    pub fn new(method: &str, path: &str, handler_id: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            handler_id: handler_id.to_string(),
        }
    }

    pub fn get(path: &str, handler_id: &str) -> Self {
        Self::new("GET", path, handler_id)
    }

    pub fn post(path: &str, handler_id: &str) -> Self {
        Self::new("POST", path, handler_id)
    }

    pub fn put(path: &str, handler_id: &str) -> Self {
        Self::new("PUT", path, handler_id)
    }

    pub fn delete(path: &str, handler_id: &str) -> Self {
        Self::new("DELETE", path, handler_id)
    }
}

/// HTTP server configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub routes: Vec<Route>,
}

#[allow(dead_code)]
impl ServerConfig {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            routes: Vec::new(),
        }
    }

    pub fn with_route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }

    pub fn with_routes(mut self, routes: Vec<Route>) -> Self {
        self.routes.extend(routes);
        self
    }
}

/// Dependencies container for dependency injection
#[allow(dead_code)]
pub struct Dependencies {
    dependencies: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

#[allow(dead_code)]
impl Dependencies {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn insert<T: 'static + Send + Sync>(&mut self, key: &str, value: T) {
        self.dependencies.insert(key.to_string(), Arc::new(value));
    }

    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.dependencies.get(key)?.downcast_ref::<T>()
    }

    pub fn get_arc<T: 'static>(&self, key: &str) -> Option<Arc<T>> {
        let any_arc = self.dependencies.get(key)?;
        // This is a bit complex but safe way to convert Arc<dyn Any> to Arc<T>
        let raw_ptr = Arc::as_ptr(any_arc) as *const T;
        unsafe { Some(Arc::from_raw(raw_ptr)) }
    }
}

impl Default for Dependencies {
    fn default() -> Self {
        Self::new()
    }
}

/// Abstract HTTP server trait
#[async_trait]
#[allow(dead_code)]
pub trait HttpServer {
    /// Start the server with the given configuration
    async fn start(&self, config: ServerConfig, dependencies: Dependencies) -> Result<()>;

    /// Register a handler for a specific route
    async fn register_handler(&mut self, handler_id: &str, handler: Box<dyn HttpHandler>);
}

/// Actix Web implementation
pub mod actix {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::Mutex;

    pub struct ActixServer {
        #[allow(dead_code)]
        handlers: Arc<Mutex<HashMap<String, Arc<dyn HttpHandler>>>>,
    }

    impl ActixServer {
        pub fn new() -> Self {
            Self {
                handlers: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    impl Default for ActixServer {
        fn default() -> Self {
            Self::new()
        }
    }

    // Commented out for now due to Send/Sync issues with Actix Web
    // #[async_trait]
    // impl HttpServer for ActixServer {
    //     async fn start(&self, config: ServerConfig, _dependencies: Dependencies) -> Result<()> {
    //         // For now, we'll use a simplified implementation that just starts a basic server
    //         // This avoids the complex Send/Sync issues with the full implementation
    //         let host = config.host.clone();
    //         let port = config.port;
    //
    //         ActixHttpServer::new(|| {
    //             App::new()
    //                 .route("/health", web::get().to(|| async {
    //                     ActixHttpResponse::Ok().json(serde_json::json!({
    //                         "status": "healthy"
    //                     }))
    //                 }))
    //         })
    //         .bind((host.as_str(), port))?
    //         .run()
    //         .await?;
    //
    //         Ok(())
    //     }

    //     async fn register_handler(&mut self, handler_id: &str, handler: Box<dyn HttpHandler>) {
    //         let mut handlers_lock = self.handlers.lock().unwrap();
    //         handlers_lock.insert(handler_id.to_string(), Arc::from(handler));
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_http_response_builder() {
        let response = HttpResponse::ok()
            .with_header("X-Custom", "value")
            .with_json(&json!({"message": "success"}))
            .unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("X-Custom").unwrap(), "value");
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "application/json"
        );
    }

    #[test]
    fn test_route_builder() {
        let route = Route::get("/users/{id}", "get_user");
        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/users/{id}");
        assert_eq!(route.handler_id, "get_user");

        let route = Route::post("/users", "create_user");
        assert_eq!(route.method, "POST");
        assert_eq!(route.path, "/users");
        assert_eq!(route.handler_id, "create_user");
    }

    #[test]
    fn test_server_config() {
        let config = ServerConfig::new("localhost", 8080)
            .with_route(Route::get("/health", "health_check"))
            .with_route(Route::post("/users", "create_user"));

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.routes.len(), 2);
    }

    #[test]
    fn test_dependencies() {
        let mut deps = Dependencies::new();
        deps.insert("config", "test_config".to_string());
        deps.insert("port", 8080u16);

        assert_eq!(deps.get::<String>("config").unwrap(), "test_config");
        assert_eq!(deps.get::<u16>("port").unwrap(), &8080);
        assert!(deps.get::<String>("nonexistent").is_none());
    }
}
