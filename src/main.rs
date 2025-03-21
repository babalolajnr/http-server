use std::{fs, path::Path};

use http::{Request, Response, StatusCode};
use server::Server;

pub mod http;
pub mod router;
pub mod server;
pub mod service;

fn main() {
    // Create a new server instance
    let mut server = Server::new("127.0.0.1:8000");

    // Add routes
    server.add_route("/", handle_index);
    server.add_route("/hello", handle_hello);

    // Add middleware
    server.add_middleware(log_middleware);

    // Start the server with out request handler
    if let Err(e) = server.listen() {
        eprintln!("Failed to start server: {}", e);
    }
}

/// Handles the index route by serving the `index.html` file from the `public` directory.
///
/// # Arguments
///
/// * `_request` - A reference to the incoming request.
///
/// # Returns
///
/// * `Response` - The HTTP response containing the file content or a 404 error message.
fn handle_index(_request: &Request) -> Response {
    let file_path = "public/index.html";
    match fs::read(file_path) {
        Ok(content) => {
            let mut response = Response::new(StatusCode::OK);
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
            response
        }
        Err(_) => {
            let mut response = Response::new(StatusCode::NotFound);
            response.set_content_type("text/html");
            response.set_body(b"<html><body><h1>404 - Not Found</h1></body></html>".to_vec());
            response
        }
    }
}

fn handle_hello(_request: &Request) -> Response {
    let mut response = Response::new(StatusCode::OK);
    response.set_content_type("text/plain");
    response.set_body(b"Hello, world!".to_vec());
    response
}

fn log_middleware(request: &mut Request) -> Result<(), Response> {
    println!("Middleware: Logging request to {}", request.path);
    Ok(())
}
