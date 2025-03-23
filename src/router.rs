use std::{collections::HashMap, pin::Pin, sync::Arc, task::Poll};

/// The router module provides routing functionality for HTTP requests.
/// It includes definitions for route patterns, path segments, and the router itself.
use crate::{
    http::{Method, Request, Response, StatusCode},
    service::Service,
};

/// Type alias for middleware functions.
pub(super) type Middleware = fn(&mut Request) -> Result<(), Response>;

/// Represents a route pattern with segments.
pub struct RoutePattern {
    segments: Vec<PathSegment>,
}

/// Enum representing different types of path segments.
enum PathSegment {
    Exact(String),
    Param(String),
    Wildcard,
}

impl RoutePattern {
    /// Creates a new `RoutePattern` from a pattern string.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A string slice that holds the pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// let pattern = RoutePattern::new("/users/:id");
    /// ```
    pub fn new(pattern: &str) -> Self {
        let segments = pattern
            .split('/')
            .filter(|segment| !segment.is_empty())
            .map(|segment| {
                if segment == "*" {
                    PathSegment::Wildcard
                } else if let Some(param) = segment.strip_prefix(':') {
                    PathSegment::Param(param.to_string())
                } else {
                    PathSegment::Exact(segment.to_string())
                }
            })
            .collect();

        RoutePattern { segments }
    }

    /// Checks if the given path matches the route pattern.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `HashMap` of parameters if the path matches, or `None` if it doesn't.
    pub fn matches(&self, path: &str) -> Option<HashMap<String, String>> {
        let path_segments = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        // Quick check for segment count
        if path_segments.len() != self.segments.len()
            && !self
                .segments
                .iter()
                .any(|s| matches!(s, PathSegment::Wildcard))
        {
            return None;
        }

        let mut params = HashMap::new();
        let mut path_index = 0;

        for segment in self.segments.iter() {
            match segment {
                PathSegment::Exact(expected) => {
                    if path_index >= path_segments.len() || path_segments[path_index] != expected {
                        return None;
                    }
                    path_index += 1;
                }
                PathSegment::Param(name) => {
                    if path_index >= path_segments.len() {
                        return None;
                    }
                    params.insert(name.clone(), path_segments[path_index].to_string());
                    path_index += 1;
                }
                PathSegment::Wildcard => {
                    // Wildcard matches all remaining segments
                    return Some(params);
                }
            }
        }

        // Check if we've consumed all path segments
        if path_index == path_segments.len() {
            Some(params)
        } else {
            None
        }
    }
}

/// Type alias for handler functions.
type HandlerFn =
    dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response, String>> + Send>> + Send + Sync;

/// Represents a route with a pattern, method, and handler.
pub struct Route {
    pattern: RoutePattern,
    method: Option<Method>,
    handler: Arc<HandlerFn>,
}

/// Represents the router with a collection of routes and a not-found handler.
pub struct Router {
    pub routes: Vec<Route>,
    pub not_found_handler: Arc<HandlerFn>,
}

impl Router {
    /// Creates a new `Router` with a default 404 handler.
    pub fn new() -> Self {
        // Default 404 handler
        let not_found_handler = Arc::new(
            |_| -> Pin<Box<dyn Future<Output = Result<Response, String>> + Send>> {
                Box::pin(async {
                    let mut response = Response::new(StatusCode::NotFound);
                    response.set_content_type("text/html");
                    response
                        .set_body(b"<html><body><h1>404 - Not Found</h1></body></html>".to_vec());
                    Ok(response)
                })
            },
        );

        Router {
            routes: Vec::new(),
            not_found_handler,
        }
    }

    /// Adds a route to the router.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A string slice that holds the route pattern.
    /// * `method` - An optional HTTP method for the route.
    /// * `handler` - A function that handles the request.
    ///
    /// # Examples
    ///
    /// ```
    /// router.route("/users/:id", Some(Method::GET), handler);
    /// ```
    pub fn route<F, Fut>(mut self, pattern: &str, method: Option<Method>, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, String>> + Send + 'static,
    {
        let handler = Arc::new(move |req| {
            let fut = handler(req);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Result<Response, String>> + Send>>
        });

        self.routes.push(Route {
            pattern: RoutePattern::new(pattern),
            method,
            handler,
        });

        self
    }

    /// Adds a GET route to the router.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A string slice that holds the route pattern.
    /// * `handler` - A function that handles the request.
    ///
    /// # Examples
    ///
    /// ```
    /// router.get("/users/:id", handler);
    /// ```
    pub fn get<F, Fut>(self, pattern: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, String>> + Send + 'static,
    {
        self.route(pattern, Some(Method::GET), handler)
    }

    /// Adds a POST route to the router.
    ///
    /// # Arguments
    ///
    /// * `pattern` - A string slice that holds the route pattern.
    /// * `handler` - A function that handles the request.
    ///
    /// # Examples
    ///
    /// ```
    /// router.post("/users", handler);
    /// ```
    pub fn post<F, Fut>(self, pattern: &str, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, String>> + Send + 'static,
    {
        self.route(pattern, Some(Method::POST), handler)
    }

    /// Sets the not-found handler for the router.
    ///
    /// # Arguments
    ///
    /// * `handler` - A function that handles the request.
    ///
    /// # Examples
    ///
    /// ```
    /// router.set_not_found_handler(handler);
    /// ```
    pub fn set_not_found_handler<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, String>> + Send + 'static,
    {
        self.not_found_handler = Arc::new(move |req| {
            let fut = handler(req);
            Box::pin(fut) as Pin<Box<dyn Future<Output = Result<Response, String>> + Send>>
        });
        self
    }

    /// Handles an incoming request and returns a response.
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming request.
    ///
    /// # Returns
    ///
    /// A `Future` that resolves to a `Result` containing the response or an error message.
    pub async fn handle(&self, req: Request) -> Result<Response, String> {
        // Extract path from request
        let path = &req.path;

        // Find matching route
        for route in &self.routes {
            if let Some(method) = &route.method {
                if &req.method != method {
                    continue;
                }
            }

            if let Some(params) = route.pattern.matches(path) {
                let mut req = req.clone();
                req.params = params;
                return (route.handler)(req).await;
            }
        }

        // No route found, use the 404 handler
        (self.not_found_handler)(req).await
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

// Implement a Service for the Router
impl Service for Router {
    type Response = Response;
    type Error = String;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let router = self.clone();
        Box::pin(async move { router.handle(request).await })
    }
}

impl Clone for Router {
    fn clone(&self) -> Self {
        Router {
            routes: self.routes.clone(),
            not_found_handler: self.not_found_handler.clone(),
        }
    }
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Route {
            pattern: self.pattern.clone(),
            method: self.method.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl Clone for RoutePattern {
    fn clone(&self) -> Self {
        RoutePattern {
            segments: self.segments.clone(),
        }
    }
}

impl Clone for PathSegment {
    fn clone(&self) -> Self {
        match self {
            PathSegment::Exact(s) => PathSegment::Exact(s.clone()),
            PathSegment::Param(s) => PathSegment::Param(s.clone()),
            PathSegment::Wildcard => PathSegment::Wildcard,
        }
    }
}
