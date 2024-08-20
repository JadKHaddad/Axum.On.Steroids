use std::task::{Context, Poll};

use super::future::ResponseFuture;
use http::{Request, Response};
use tower::Service;

/// Applies basic authentication to the request.
#[derive(Debug, Clone)]
pub struct BasicAuth<T> {
    inner: T,
}

impl<T> BasicAuth<T> {
    pub const fn new(inner: T) -> Self {
        BasicAuth { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for BasicAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Default,
{
    type Response = Response<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let (parts, body) = request.into_parts();

        let request = Request::from_parts(parts, body);

        let future = self.inner.call(request);

        ResponseFuture::new(future)
    }
}
