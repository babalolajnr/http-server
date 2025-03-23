mod http;
mod middleware;
mod router;
mod server;
mod service;

use http::{Request, Response, StatusCode};
use router::Router;
use server::new_server;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() {
    // Create a router with routes
    let router = Router::new()
        .get("/", handle_index)
        .get("/hello", handle_hello)
        .get("/users/:id", handle_user)
        .post("/users", handle_create_user)
        .get("/static/*", handle_static)
        .set_not_found_handler(handle_not_found);

    // Create and start the server
    let server = new_server("127.0.0.1:8080", router);

    if let Err(e) = server.listen() {
        eprintln!("Server error: {}", e);
    }
}

async fn handle_index(_request: Request) -> Result<Response, String> {
    // Demonstrate route handling
    let mut response = Response::new(StatusCode::OK);
    response.set_content_type("text/html");
    response.set_body(b"<html><body><h1>Welcome to our Rust HTTP Server</h1><p>Built with Tower-inspired middleware and routing.</p></body></html>".to_vec());
    Ok(response)
}

async fn handle_hello(request: Request) -> Result<Response, String> {
    // Demonstrate query parameter usage
    let name = request.query_param("name").map_or("World", |n| n);

    let mut response = Response::new(StatusCode::OK);
    response.set_content_type("text/plain");
    response.set_body(format!("Hello, {}!", name).into_bytes());
    Ok(response)
}

async fn handle_user(request: Request) -> Result<Response, String> {
    // Demonstrate route parameters
    let user_id = request.param("id").ok_or("Missing user ID")?;

    let mut response = Response::new(StatusCode::OK);
    response.set_content_type("application/json");
    response.set_body(
        format!(
            r#"{{"id": "{}", "name": "User {}", "email": "user{}@example.com"}}"#,
            user_id, user_id, user_id
        )
        .into_bytes(),
    );
    Ok(response)
}

async fn handle_create_user(_request: Request) -> Result<Response, String> {
    // In a real app, we would parse the JSON body with serde
    // For now, let's just pretend we created a user

    let mut response = Response::new(StatusCode::Created);
    response.set_content_type("application/json");
    response.set_body(
        r#"{"id": "new-user-123", "name": "New User", "status": "created"}"#
            .as_bytes()
            .to_vec(),
    );
    Ok(response)
}

async fn handle_static(request: Request) -> Result<Response, String> {
    // Extract the file path from the wildcard
    let path = request.path.strip_prefix("/static/").unwrap_or("");
    let file_path = format!("public/{}", path);

    // Try to read the file
    match fs::read(&file_path) {
        Ok(content) => {
            let mut response = Response::new(StatusCode::OK);

            // Set content type based on file extension
            let content_type = match Path::new(&file_path).extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("png") => "image/png",
                Some("gif") => "image/gif",
                _ => "application/octet-stream",
            };

            response.set_content_type(content_type);
            response.set_body(content);
            Ok(response)
        }
        Err(_) => {
            // File not found
            let mut response = Response::new(StatusCode::NotFound);
            response.set_content_type("text/html");
            response.set_body(b"<html><body><h1>404 - File Not Found</h1></body></html>".to_vec());
            Ok(response)
        }
    }
}

async fn handle_not_found(_request: Request) -> Result<Response, String> {
    let mut response = Response::new(StatusCode::NotFound);
    response.set_content_type("text/html");
    response.set_body(b"<html><body><h1>404 - Not Found</h1><p>The page you're looking for doesn't exist.</p></body></html>".to_vec());
    Ok(response)
}
