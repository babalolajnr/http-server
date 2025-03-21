use std::task::{Context, Poll};

use crate::http::{Request, Response};

/// A trait representing an asynchronous service.
pub trait Service {
    /// The type of response returned by the service.
    type Response;
    /// The type of error that can occur within the service.
    type Error;
    /// The future type returned by the service.
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    /// Polls to check if the service is ready to accept a request.
    ///
    /// # Arguments
    ///
    /// * `cx` - The context of the current task.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating if the service is ready or not.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>>;

    /// Calls the service with a request.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to be processed by the service.
    ///
    /// # Returns
    ///
    /// A future representing the result of the service call.
    fn call(&mut self, request: Request) -> Self::Future;
}

/// A trait representing a layer that wraps a service.
pub trait Layer<S> {
    /// The type of service produced by the layer.
    type Service;

    /// Wraps the given service with the layer.
    ///
    /// # Arguments
    ///
    /// * `service` - The service to be wrapped.
    ///
    /// # Returns
    ///
    /// The wrapped service.
    fn layer(&self, service: S) -> Self::Service;
}

/// A builder for constructing a service with layers.
pub struct ServiceBuilder<S> {
    service: S,
}

impl<S> ServiceBuilder<S> {
    /// Creates a new `ServiceBuilder` with the given service.
    ///
    /// # Arguments
    ///
    /// * `service` - The service to be built.
    ///
    /// # Returns
    ///
    /// A new `ServiceBuilder` instance.
    pub fn new(service: S) -> Self {
        ServiceBuilder { service }
    }

    /// Adds a layer to the service.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to be added.
    ///
    /// # Returns
    ///
    /// A new `ServiceBuilder` with the layer added.
    pub fn layer<L>(self, layer: L) -> ServiceBuilder<L::Service>
    where
        L: Layer<S>,
    {
        ServiceBuilder {
            service: layer.layer(self.service),
        }
    }

    /// Builds the service.
    ///
    /// # Returns
    ///
    /// The constructed service.
    pub fn build(self) -> S {
        self.service
    }
}

/// A service that handles requests using a function.
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

    /// Polls to check if the service is ready to accept a request.
    ///
    /// # Arguments
    ///
    /// * `_cx` - The context of the current task.
    ///
    /// # Returns
    ///
    /// A `Poll` indicating if the service is ready or not.
    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Calls the service with a request.
    ///
    /// # Arguments
    ///
    /// * `request` - The request to be processed by the service.
    ///
    /// # Returns
    ///
    /// A future representing the result of the service call.
    fn call(&mut self, request: Request) -> Self::Future {
        (self.f)(request)
    }
}

/// Creates a new `HandlerService` with the given function.
///
/// # Arguments
///
/// * `f` - The function to handle requests.
///
/// # Returns
///
/// A new `HandlerService` instance.
pub fn service_fn<F, Fut>(f: F) -> HandlerService<F>
where
    F: FnMut(Request) -> Fut,
    Fut: Future<Output = Result<Response, String>>,
{
    HandlerService { f }
}
