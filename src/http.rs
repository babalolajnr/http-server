use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "CONNECT" => Method::CONNECT,
            "OPTIONS" => Method::OPTIONS,
            "TRACE" => Method::TRACE,
            "PATCH" => Method::PATCH,
            _ => panic!("Invalid method"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    UNKNOWN,
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.0" => Version::HTTP1_0,
            "HTTP/1.1" => Version::HTTP1_1,
            "HTTP/2.0" => Version::HTTP2_0,
            _ => Version::UNKNOWN,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::HTTP1_0 => write!(f, "HTTP/1.0"),
            Version::HTTP1_1 => write!(f, "HTTP/1.1"),
            Version::HTTP2_0 => write!(f, "HTTP/2.0"),
            Version::UNKNOWN => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
}

impl Request {
    /// Parses a raw HTTP request into a `Request` object.
    ///
    /// # Arguments
    ///
    /// * `raw` - A byte slice containing the raw HTTP request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Request` object or an error message.
    pub fn parse(raw: &[u8]) -> Result<Request, String> {
        // Convert raw bytes to string, allowing for partial invalid UTF-8 sequences
        let raw_str = String::from_utf8_lossy(raw);

        // Split into headers and body
        let mut parts = raw_str.split("\r\n\r\n");
        let headers_part = parts.next().ok_or("Invalid request format")?;
        let body_part = parts.next().unwrap_or("");

        // Parse the request line and headers
        let mut lines = headers_part.lines();
        let request_line = lines.next().ok_or("Missing request line")?;

        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().ok_or("Missing method")?;
        let path_with_query = request_parts.next().ok_or("Missing path")?;
        let version = request_parts.next().ok_or("Missing HTTP version")?;

        // Parse path and query parameters
        let mut query = HashMap::new();
        let path = if let Some(q_idx) = path_with_query.find('?') {
            let path = &path_with_query[..q_idx];
            let query_str = &path_with_query[q_idx + 1..];

            // Parse query string
            for pair in query_str.split('&') {
                if let Some(eq_idx) = pair.find('=') {
                    let key = pair[..eq_idx].to_string();
                    let value = pair[eq_idx + 1..].to_string();
                    query.insert(key, value);
                } else if !pair.is_empty() {
                    query.insert(pair.to_string(), "".to_string());
                }
            }

            path.to_string()
        } else {
            path_with_query.to_string()
        };

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines {
            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        Ok(Request {
            method: Method::from(method),
            path,
            version: Version::from(version),
            headers,
            body: body_part.as_bytes().to_vec(),
            params: HashMap::new(), // Will be filled by the router
            query,
        })
    }

    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatusCode {
    OK = 200,
    Created = 201,
    Accepted = 202,
    NoContent = 204,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
}

impl StatusCode {
    pub fn reason_phrase(&self) -> &str {
        match self {
            StatusCode::OK => "OK",
            StatusCode::Created => "Created",
            StatusCode::Accepted => "Accepted",
            StatusCode::NoContent => "No Content",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_from_str() {
        assert_eq!(Method::from("GET"), Method::GET);
        assert_eq!(Method::from("POST"), Method::POST);
        assert_eq!(Method::from("PUT"), Method::PUT);
        assert_eq!(Method::from("DELETE"), Method::DELETE);
        assert_eq!(Method::from("HEAD"), Method::HEAD);
        assert_eq!(Method::from("CONNECT"), Method::CONNECT);
        assert_eq!(Method::from("OPTIONS"), Method::OPTIONS);
        assert_eq!(Method::from("TRACE"), Method::TRACE);
        assert_eq!(Method::from("PATCH"), Method::PATCH);
    }

    #[test]
    #[should_panic(expected = "Invalid method")]
    fn test_method_from_invalid_str() {
        Method::from("INVALID");
    }
}
