use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq, Clone)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Connect,
    Options,
    Trace,
    Patch,
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "GET" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "HEAD" => Method::Head,
            "CONNECT" => Method::Connect,
            "OPTIONS" => Method::Options,
            "TRACE" => Method::Trace,
            "PATCH" => Method::Patch,
            _ => panic!("Invalid method"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Version {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    Unknown,
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.0" => Version::HTTP1_0,
            "HTTP/1.1" => Version::HTTP1_1,
            "HTTP/2.0" => Version::HTTP2_0,
            _ => Version::Unknown,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::HTTP1_0 => write!(f, "HTTP/1.0"),
            Version::HTTP1_1 => write!(f, "HTTP/1.1"),
            Version::HTTP2_0 => write!(f, "HTTP/2.0"),
            Version::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: Method,
    pub path: String,
    version: Version,
    headers: HashMap<String, String>,
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
        let mut parts = raw_str.splitn(2, "\r\n\r\n");
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
        let (path, query) = if let Some(q_idx) = path_with_query.find('?') {
            let path = &path_with_query[..q_idx];
            let query_str = &path_with_query[q_idx + 1..];
            let query = query_str
                .split('&')
                .filter_map(|pair| {
                    let mut split = pair.splitn(2, '=');
                    let key = split.next()?.to_string();
                    let value = split.next().unwrap_or("").to_string();
                    Some((key, value))
                })
                .collect();
            (path.to_string(), query)
        } else {
            (path_with_query.to_string(), HashMap::new())
        };

        // Parse headers
        let headers = lines
            .filter_map(|line| {
                let mut split = line.splitn(2, ':');
                let key = split.next()?.trim().to_string();
                let value = split.next()?.trim().to_string();
                Some((key, value))
            })
            .collect();

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

#[allow(dead_code)]
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
        assert_eq!(Method::from("GET"), Method::Get);
        assert_eq!(Method::from("POST"), Method::Post);
        assert_eq!(Method::from("PUT"), Method::Put);
        assert_eq!(Method::from("DELETE"), Method::Delete);
        assert_eq!(Method::from("HEAD"), Method::Head);
        assert_eq!(Method::from("CONNECT"), Method::Connect);
        assert_eq!(Method::from("OPTIONS"), Method::Options);
        assert_eq!(Method::from("TRACE"), Method::Trace);
        assert_eq!(Method::from("PATCH"), Method::Patch);
    }

    #[test]
    #[should_panic(expected = "Invalid method")]
    fn test_method_from_invalid_str() {
        Method::from("INVALID");
    }
}
