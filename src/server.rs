use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use crate::http::{self, Request, Response, StatusCode};

type RequestHandler = fn(&Request) -> Response;

pub struct Server {
    address: String,
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
        }
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
    pub fn listen(&self, handler: RequestHandler) -> Result<(), String> {
        // Create a new TcpListener and bind it to the address
        let listener = TcpListener::bind(&self.address)
            .map_err(|e| format!("Failed to bind to {}: {}", self.address, e))?;

        println!("Server listening on {}", self.address);

        // Accept incoming connections
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    // Handle each connection in a new thread
                    thread::spawn(move || {
                        if let Err(e) = Self::handle_client(stream, handler) {
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
    fn handle_client(mut stream: TcpStream, handler: RequestHandler) -> Result<(), String> {
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
        let response = match Request::parse(&request_data) {
            Ok(request) => {
                // Log the request
                println!(
                    "{} {}",
                    match request.method {
                        http::Method::GET => "GET",
                        http::Method::POST => "POST",
                        http::Method::PUT => "PUT",
                        http::Method::PATCH => "PATCH",
                        http::Method::DELETE => "DELETE",
                        _ => "OTHER",
                    },
                    request.path
                );

                // Call the handler function to handle the request
                handler(&request)
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);

                // Return a 400 Bad Request response
                let mut response = Response::new(StatusCode::BadRequest);
                response.set_content_type("text/plain");
                response.set_body(b"Bad Request".to_vec());
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
