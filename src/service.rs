use std::task::{Context, Poll};

use crate::http::{Request, Response};

pub trait Service {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;
    fn call(&mut self, request: Request) -> Self::Future;
}

pub trait Layer<S> {
    type Service;

    fn layer(&self, service: S) -> Self::Service;
}

pub struct ServiceBuilder<S> {
    service: S,
}

impl<S> ServiceBuilder<S> {
    pub fn new(service: S) -> Self {
        ServiceBuilder { service }
    }

    pub fn layer<L>(self, layer: L) -> ServiceBuilder<L::Service>
    where
        L: Layer<S>,
    {
        ServiceBuilder {
            service: layer.layer(self.service),
        }
    }

    pub fn build(self) -> S {
        self.service
    }
}

pub struct HandlerService<F> {
    f: F,
}

impl<F, Fut> Service for HandlerService<F>
where
    F: FnMut(Request) -> Fut,
    Fut: Future<Output = Result<Response, String>>,
{
    type Response = Response;
    type Error = String;
    type Future = Fut;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        (self.f)(request)
    }
}

pub fn service_fn<F, Fut>(f: F) -> HandlerService<F>
where
    F: FnMut(Request) -> Fut,
    Fut: Future<Output = Result<Response, String>>,
{
    HandlerService { f }
}
