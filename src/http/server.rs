use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// HTTP response for server endpoints
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

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
        self.headers.insert("content-type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_activity_json(mut self, json: &Value) -> Result<Self> {
        self.body = serde_json::to_vec(json)?;
        self.headers.insert("content-type".to_string(), "application/activity+json".to_string());
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
pub struct HttpContext {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub path_params: HashMap<String, String>,
    pub dependencies: Arc<dyn std::any::Any + Send + Sync>,
}

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
pub trait HttpHandler: Send + Sync {
    async fn handle(&self, context: HttpContext) -> Result<HttpResponse>;
}

/// Route definition
#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler_id: String,
}

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
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub routes: Vec<Route>,
}

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
pub struct Dependencies {
    dependencies: HashMap<String, Arc<dyn std::any::Any + Send + Sync>>,
}

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
pub trait HttpServer: Send + Sync {
    /// Start the server with the given configuration
    async fn start(&self, config: ServerConfig, dependencies: Dependencies) -> Result<()>;

    /// Register a handler for a specific route
    fn register_handler(&mut self, handler_id: &str, handler: Box<dyn HttpHandler>);
}

/// Actix Web implementation
pub mod actix {
    use super::*;
    use actix_web::{web, App, HttpServer as ActixHttpServer, HttpRequest, HttpResponse as ActixHttpResponse};
    use std::sync::Mutex;
    use std::collections::HashMap;

    pub struct ActixServer {
        handlers: Arc<Mutex<HashMap<String, Box<dyn HttpHandler>>>>,
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

    #[async_trait]
    impl HttpServer for ActixServer {
        async fn start(&self, config: ServerConfig, dependencies: Dependencies) -> Result<()> {
            let handlers = self.handlers.clone();
            let deps = Arc::new(dependencies);
            
            ActixHttpServer::new(move || {
                let mut app = App::new();
                
                // Add routes
                for route in &config.routes {
                    let handler_id = route.handler_id.clone();
                    let handlers_clone = handlers.clone();
                    let deps_clone = deps.clone();
                    
                    app = app.route(&route.path, web::to(move |req: HttpRequest, body: web::Bytes| {
                        let handler_id = handler_id.clone();
                        let handlers = handlers_clone.clone();
                        let deps = deps_clone.clone();
                        
                        async move {
                            // Create context from Actix request
                            let mut context = HttpContext::new(req.method().as_str(), req.path());
                            context.body = body.to_vec();
                            
                            // Extract path parameters
                            for (key, value) in req.match_info().iter() {
                                context.path_params.insert(key.to_string(), value.to_string());
                            }
                            
                            // Extract headers
                            for (name, value) in req.headers() {
                                if let Ok(value_str) = value.to_str() {
                                    context.headers.insert(name.to_string(), value_str.to_string());
                                }
                            }
                            
                            // Extract query parameters
                            for (key, value) in req.query_string().split('&').filter_map(|pair| {
                                let mut parts = pair.split('=');
                                Some((parts.next()?, parts.next()?))
                            }) {
                                context.query_params.insert(key.to_string(), value.to_string());
                            }
                            
                            // Set dependencies
                            context.dependencies = deps.dependencies.get("main").cloned().unwrap_or_else(|| Arc::new(()));
                            
                            // Get handler and execute
                            let handlers_lock = handlers.lock().unwrap();
                            if let Some(handler) = handlers_lock.get(&handler_id) {
                                match handler.handle(context).await {
                                    Ok(response) => {
                                        let mut actix_response = ActixHttpResponse::build(
                                            actix_web::http::StatusCode::from_u16(response.status).unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
                                        );
                                        
                                        for (name, value) in response.headers {
                                            actix_response.insert_header((name, value));
                                        }
                                        
                                        actix_response.body(response.body)
                                    }
                                    Err(e) => {
                                        ActixHttpResponse::InternalServerError().json(serde_json::json!({
                                            "error": e.to_string()
                                        }))
                                    }
                                }
                            } else {
                                ActixHttpResponse::NotFound().json(serde_json::json!({
                                    "error": "Handler not found"
                                }))
                            }
                        }
                    }));
                }
                
                app
            })
            .bind((config.host.as_str(), config.port))?
            .run()
            .await?;
            
            Ok(())
        }

        fn register_handler(&mut self, handler_id: &str, handler: Box<dyn HttpHandler>) {
            let mut handlers = self.handlers.lock().unwrap();
            handlers.insert(handler_id.to_string(), handler);
        }
    }
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
        assert_eq!(response.headers.get("content-type").unwrap(), "application/json");
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