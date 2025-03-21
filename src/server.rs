use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

use crate::{
    http::{self, Request, Response, StatusCode},
    router::{Middleware, Router},
};

pub(crate) type RequestHandler = fn(&Request) -> Response;

pub struct Server {
    address: String,
    router: Router,
}

impl Server {
    /// Creates a new `Server` instance with the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - A string slice that holds the address to bind the server to.
    ///
    /// # Returns
    ///
    /// A new `Server` instance.
    pub fn new(address: &str) -> Self {
        Server {
            address: address.to_string(),
            router: Router::new(),
        }
    }

    pub fn add_route(&mut self, path: &str, handler: RequestHandler) {
        self.router.add_route(path, handler);
    }

    pub fn add_middleware(&mut self, middleware: Middleware) {
        self.router.add_middleware(middleware);
    }

    /// Starts the server and listens for incoming connections.
    ///
    /// # Arguments
    ///
    /// * `handler` - A function that handles incoming requests.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the server starts successfully, or an `Err` with a message if it fails.
    pub fn listen(&self) -> Result<(), String> {
        // Create a new TcpListener and bind it to the address
        let listener = TcpListener::bind(&self.address)
            .map_err(|e| format!("Failed to bind to {}: {}", self.address, e))?;

        println!("Server listening on {}", self.address);

        // Use Arc to share router across threads safely
        let router = Arc::new(self.router.clone());

        // Accept incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Clone the Arc to pass to the thread
                    let router = Arc::clone(&router);

                    // Handle each connection in a new thread
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, router) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Failed to establish a connection: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Handles an individual client connection.
    ///
    /// # Arguments
    ///
    /// * `stream` - The TCP stream representing the client connection.
    /// * `handler` - A function that handles the request.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok` if the request is handled successfully, or an `Err` with a message if it fails.
    fn handle_client(mut stream: TcpStream, router: Arc<Router>) -> Result<(), String> {
        // Set timeout to avoid hanging on slow clients
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;

        // Buffer to store incoming data
        let mut buffer = [0; 4096]; // 4KB buffer
        let mut request_data = Vec::new();

        // Read data from the client in chunks
        loop {
            let bytes_read = stream
                .read(&mut buffer)
                .map_err(|e| format!("Error reading from stream: {}", e))?;

            if bytes_read == 0 {
                // Connection closed
                break;
            }

            request_data.extend_from_slice(&buffer[..bytes_read]);

            // Check if we have a complete HTTP request
            if request_data.windows(4).any(|window| window == b"\r\n\r\n") {
                // Found end of headers
                // TODO: Handle chunked transfer encoding and handle content-length validation
                break;
            }

            if request_data.len() >= 1024 * 1024 {
                // 1MB limit
                return Err("Request too large".to_string());
            }
        }

        // Parse the request
        let mut request = match Request::parse(&request_data) {
            Ok(req) => {
                // Log the request
                println!(
                    "{} {}",
                    match req.method {
                        http::Method::GET => "GET",
                        http::Method::POST => "POST",
                        http::Method::PUT => "PUT",
                        http::Method::PATCH => "PATCH",
                        http::Method::DELETE => "DELETE",
                        _ => "OTHER",
                    },
                    req.path
                );

                req
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);

                // Return a 400 Bad Request response
                let mut response = Response::new(StatusCode::BadRequest);
                response.set_content_type("text/plain");
                response.set_body(b"Bad Request".to_vec());
                stream
                    .write_all(&response.to_bytes())
                    .map_err(|e| format!("Failed to send response: {}", e))?;
                return Ok(());
            }
        };

        // Run middlewares
        for middleware in &router.middleware {
            match middleware(&mut request) {
                Ok(_) => continue,
                Err(response) => {
                    stream
                        .write_all(&response.to_bytes())
                        .map_err(|e| format!("Failed to send response: {}", e))?;
                    return Ok(());
                }
            }
        }

        let response = match router.route(request.clone()) {
            Some(handler) => handler(&request),
            None => {
                // Return a 404 Not Found response
                let mut response = Response::new(StatusCode::NotFound);
                response.set_content_type("text/html");
                response.set_body(b"<html><body><h1>404 - Not Found</h1></body></html>".to_vec());
                response
            }
        };
        // Send the response back to the client
        stream
            .write_all(&response.to_bytes())
            .map_err(|e| format!("Failed to send response: {}", e))?;

        Ok(())
    }
}
