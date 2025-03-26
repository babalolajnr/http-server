use std::collections::HashMap;

use super::{Method, Request, Version};

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
