use tower::Layer;

use super::service::BasicAuth;

/// Applies basic authentication to requests via the supplied inner service.
#[derive(Debug, Clone)]
pub struct BasicAuthLayer {}

impl BasicAuthLayer {
    pub const fn new() -> Self {
        BasicAuthLayer {}
    }
}

impl<S> Layer<S> for BasicAuthLayer {
    type Service = BasicAuth<S>;

    fn layer(&self, service: S) -> Self::Service {
        BasicAuth::new(service)
    }
}
