use std::collections::HashMap;

use super::{Method, Version};

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
    pub fn param(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    pub fn query_param(&self, key: &str) -> Option<&String> {
        self.query.get(key)
    }
}
