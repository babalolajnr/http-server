use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use futures_executor::block_on;

use crate::http::parser::parse;
use crate::http::{Response, StatusCode};
use crate::router::Router;
use crate::service::{Service, ServiceBuilder};

pub struct Server<S> {
    address: String,
    service: S,
}

impl<S> Server<S>
where
    S: Service<Response = Response, Error = String> + Send + Clone + 'static,
    S::Future: Send + 'static,
{
    pub fn new(address: &str, service: S) -> Self {
        Server {
            address: address.to_string(),
            service,
        }
    }

    pub fn listen(&self) -> Result<(), String> {
        // Create a TCP listener
        let listener = TcpListener::bind(&self.address)
            .map_err(|e| format!("Failed to bind to {}: {}", self.address, e))?;

        println!("Server listening on {}", self.address);

        // Accept connections and process them
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Clone the service for each connection
                    let mut service = self.service.clone();

                    // Handle each connection in a new thread
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, &mut service) {
                            eprintln!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }

        Ok(())
    }

    fn handle_client(mut stream: TcpStream, service: &mut S) -> Result<(), String> {
        // Set timeout to avoid hanging on slow clients
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;

        // Buffer to store the incoming data
        let mut buffer = [0; 4096]; // 4KB buffer
        let mut request_data = Vec::new();

        // Read data from the client in chunks
        loop {
            let bytes_read = stream
                .read(&mut buffer)
                .map_err(|e| format!("Error reading from stream: {}", e))?;

            if bytes_read == 0 {
                break; // Connection was closed
            }

            request_data.extend_from_slice(&buffer[..bytes_read]);

            // Check if we have a complete HTTP request
            if request_data.windows(4).any(|window| window == b"\r\n\r\n") {
                // Found the end of headers
                // For simplicity we don't handle chunked encoding or content-length validation here
                break;
            }

            if request_data.len() > 1024 * 1024 {
                // 1MB limit
                return Err("Request too large".to_string());
            }
        }

        // Parse the request
        let request = match parse(&request_data) {
            Ok(req) => req,
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

        // Make sure service is ready
        match block_on(futures::future::poll_fn(|cx| service.poll_ready(cx))) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Service not ready: {}", e);

                // Return a 503 Service Unavailable response
                let mut response = Response::new(StatusCode::ServiceUnavailable);
                response.set_content_type("text/plain");
                response.set_body(b"Service Unavailable".to_vec());
                stream
                    .write_all(&response.to_bytes())
                    .map_err(|e| format!("Failed to send response: {}", e))?;
                return Ok(());
            }
        }

        // Process the request through the service
        let response_future = service.call(request);
        let response = match block_on(response_future) {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Error processing request: {}", e);

                // Return a 500 Internal Server Error response
                let mut response = Response::new(StatusCode::InternalServerError);
                response.set_content_type("text/plain");
                response.set_body(b"Internal Server Error".to_vec());
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

// Helper to create a server with a router and middleware
pub fn new_server(
    address: &str,
    router: Router,
) -> Server<impl Service<Response = Response, Error = String> + Send + Clone + 'static> {
    // Create a service with middleware
    let service = ServiceBuilder::new(router)
        .layer(crate::middleware::LogLayer)
        .layer(crate::middleware::CorsLayer)
        .service();

    Server::new(address, service)
}
