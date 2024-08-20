use tower::Layer;

use super::service::BasicAuth;

/// Applies basic authentication to requests via the supplied inner service.
#[derive(Debug, Clone)]
pub struct BasicAuthLayer<P> {
    provider: P,
}

impl<P> BasicAuthLayer<P> {
    pub const fn new(provider: P) -> Self {
        BasicAuthLayer { provider }
    }
}

impl<S, P: Clone> Layer<S> for BasicAuthLayer<P> {
    type Service = BasicAuth<S, P>;

    fn layer(&self, service: S) -> Self::Service {
        BasicAuth::new(service, self.provider.clone())
    }
}
