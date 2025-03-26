use std::collections::HashMap;

use super::{StatusCode, Version};

#[derive(Clone)]
pub struct Response {
    pub version: Version,
    pub status_code: StatusCode,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Response {
    /// Creates a new `Response` with the given status code.
    ///
    /// # Arguments
    ///
    /// * `status_code` - The HTTP status code for the response.
    ///
    /// # Returns
    ///
    /// A new `Response` object with the specified status code, HTTP version set to HTTP/1.1,
    /// a default "Server" header, and an empty body.
    pub fn new(status_code: StatusCode) -> Response {
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "RustHTTP/0.1".to_string());
        headers.insert(
            "Date".to_string(),
            format!("{}", chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT")),
        );

        Response {
            version: Version::HTTP1_1,
            status_code,
            headers,
            body: Vec::new(),
        }
    }

    /// Sets the body of the response and updates the "Content-Length" header.
    ///
    /// # Arguments
    ///
    /// * `body` - A vector of bytes representing the body of the response.
    pub fn set_body(&mut self, body: Vec<u8>) {
        self.body = body;
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
    }

    /// Sets the "Content-Type" header of the response.
    ///
    /// # Arguments
    ///
    /// * `content_type` - A string slice representing the MIME type of the response body.
    pub fn set_content_type(&mut self, content_type: &str) {
        self.headers
            .insert("Content-Type".to_string(), content_type.to_string());
    }

    /// Converts the response to a vector of bytes suitable for sending over a network.
    ///
    /// # Returns
    ///
    /// A vector of bytes representing the entire HTTP response, including the status line,
    /// headers, and body.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        // Status line
        let status_line = format!(
            "{} {} {}\r\n",
            self.version,
            self.status_code as u16,
            self.status_code.reason_phrase()
        );
        response.extend_from_slice(status_line.as_bytes());

        // Headers
        for (key, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", key, value);
            response.extend_from_slice(header_line.as_bytes());
        }

        // Empty line separating headers and body
        response.extend_from_slice(b"\r\n");

        // Body
        response.extend_from_slice(&self.body);

        response
    }
}
