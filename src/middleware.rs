use std::{
    pin::Pin,
    task::{Context, Poll},
};

use serde::de::DeserializeOwned;

use crate::{
    http::{Method, Request, Response},
    service::{Layer, Service},
};

/// Middleware to log requests
pub struct LogLayer;

impl<S> Layer<S> for LogLayer {
    type Service = LogMiddleware<S>;

    /// Wraps the given service with the logging middleware.
    fn layer(&self, service: S) -> Self::Service {
        LogMiddleware { inner: service }
    }
}

/// Middleware service that logs requests and responses.
#[derive(Clone)]
pub struct LogMiddleware<S> {
    inner: S,
}

impl<S> Service for LogMiddleware<S>
where
    S: Service<Response = Response, Error = String> + Send,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    /// Checks if the service is ready to accept a request.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Handles the incoming request, logs it, and then logs the response or error.
    fn call(&mut self, req: Request) -> Self::Future {
        println!(
            "Request: {} {}",
            match req.method {
                Method::Get => "GET",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
                _ => "OTHER",
            },
            req.path
        );

        let future = self.inner.call(req);

        Box::pin(async move {
            let result = future.await;
            match &result {
                Ok(response) => {
                    println!("Response: {}", response.status_code as u16);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
            result
        })
    }
}

/// Middleware to handle Cross-Origin Resource Sharing (CORS)
pub struct CorsLayer;

impl<S> Layer<S> for CorsLayer {
    type Service = CorsMiddleware<S>;

    /// Wraps the given service with the CORS middleware.
    fn layer(&self, service: S) -> Self::Service {
        CorsMiddleware { inner: service }
    }
}

/// Middleware service that adds CORS headers to responses.
#[derive(Clone)]
pub struct CorsMiddleware<S> {
    inner: S,
}

impl<S> Service for CorsMiddleware<S>
where
    S: Service<Response = Response, Error = String> + Send,
    S::Future: Send + 'static,
{
    type Response = S::Response;

    type Error = S::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    /// Checks if the service is ready to accept a request.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Handles the incoming request and adds CORS headers to the response.
    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);

        Box::pin(async move {
            let mut response = future.await?;

            response
                .headers
                .insert("Access-Control-Allow-Origin".to_owned(), "*".to_string());
            response.headers.insert(
                "Access-Control-Allow-Methods".to_owned(),
                "GET, POST, PUT, DELETE, OPTIONS".to_string(),
            );

            response.headers.insert(
                "Access-Control-Allow-Headers".to_owned(),
                "Content-Type, Authorization".to_string(),
            );

            Ok(response)
        })
    }
}

/// Helper function to extract request body as JSON
///
/// # Arguments
///
/// * `request` - A reference to the incoming request.
///
/// # Returns
///
/// * `Result<T, String>` - The deserialized JSON body or an error message.
pub async fn json_extractor<T: DeserializeOwned>(request: &Request) -> Result<T, String> {
    let body = &request.body;
    let result: T =
        serde_json::from_slice(body).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(result)
}
