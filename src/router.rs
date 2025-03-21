use std::collections::HashMap;

use crate::{
    http::{Request, Response},
    server::RequestHandler,
};

pub(super) type Middleware = fn(&mut Request) -> Result<(), Response>;

#[derive(Debug, Clone)]
pub struct Router {
    pub routes: HashMap<String, RequestHandler>,
    pub middleware: Vec<Middleware>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routes: HashMap::new(),
            middleware: Vec::new(),
        }
    }

    pub fn add_route(&mut self, path: &str, handler: RequestHandler) {
        self.routes.insert(path.to_string(), handler);
    }

    pub fn add_middleware(&mut self, middleware: Middleware) {
        self.middleware.push(middleware);
    }

    pub fn route(&self, request: Request) -> Option<&RequestHandler> {
        self.routes.get(&request.path)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
