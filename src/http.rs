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
