use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    /// Parses a raw HTTP request into a `Request` object.
    ///
    /// # Arguments
    ///
    /// * `raw` - A byte slice containing the raw HTTP request data.
    ///
    /// # Returns
    ///
    /// * `Ok(Request)` - If the parsing is successful.
    /// * `Err(String)` - If there is an error during parsing, with a description of the error.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The raw data contains invalid UTF-8 sequences.
    /// * The request format is invalid.
    /// * The request line is missing or malformed.
    /// * Any of the required components (method, path, HTTP version) are missing.
    pub fn parse(raw: &[u8]) -> Result<Request, String> {
        // Convert raw bytes to string allowing for partial invalid UTF-8 sequences
        let raw_str = match std::str::from_utf8(raw) {
            Ok(s) => s,
            Err(_) => return Err("Invalid UTF-8 sequence".to_string()),
        };

        // Split into head and body
        let mut parts = raw_str.split("\r\n\r\n");
        let headers_parts = parts.next().ok_or("Invalid request format")?;
        let body_parts = parts.next().unwrap_or("");

        // Parse the request line and headers
        let mut lines = headers_parts.lines();
        let request_line = lines.next().ok_or("Missing request line")?;

        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().ok_or("Missing method")?;
        let path = request_parts.next().ok_or("Missing path")?;
        let version = request_parts.next().ok_or("Missing HTTP version")?;

        // Parse the headers
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
            path: path.to_string(),
            version: Version::from(version),
            body: body_parts.as_bytes().to_vec(),
            headers,
        })
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
