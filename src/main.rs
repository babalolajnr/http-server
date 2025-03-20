use std::{fs, path::Path};

use http::{Request, Response, StatusCode};
use server::Server;

pub mod http;
pub mod server;

fn main() {
    // Create a new server instance
    let server = Server::new("127.0.0.1:8000");

    // Start the server with out request handler
    if let Err(e) = server.listen(handle_request) {
        eprintln!("Failed to start server: {}", e);
    }
}

// This function handles HTTP requests
fn handle_request(request: &Request) -> Response {
    match request.method {
        http::Method::GET => {
            // Handle GET requests
            let path = if request.path == "/" {
                "/index.html"
            } else {
                &request.path
            };

            // Remove the leading `/` from the path and serve from the public directory
            let file_path = format!("public{}", path);

            // Try to read the file
            match fs::read(&file_path) {
                Ok(contents) => {
                    let mut response = Response::new(StatusCode::OK);

                    // Set content type based on file extensions
                    let content_type =
                        match Path::new(&file_path).extension().and_then(|e| e.to_str()) {
                            Some("html") => "text/html",
                            Some("css") => "text/css",
                            Some("js") => "application/javascript",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("png") => "image/png",
                            Some("gif") => "image/gif",
                            _ => "application/octet-stream",
                        };

                    response.set_content_type(content_type);
                    response.set_body(contents);
                    response
                }
                Err(_) => {
                    // File not found
                    let mut response = Response::new(StatusCode::NotFound);
                    response.set_content_type("text/html");
                    response
                        .set_body(b"<html><body><h1>404 - Not Found</h1></body></html>".to_vec());
                    response
                }
            }
        }
        _ => {
            // Return 501 Not Implemented for other methods
            let mut response = Response::new(StatusCode::NotImplemented);
            response.set_content_type("text/plain");
            response.set_body(b"Method not implemented".to_vec());
            response
        }
    }
}
