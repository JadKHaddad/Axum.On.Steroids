use crate::extractor::basic_auth::ApiBasicAuth;

use super::future::ResponseFuture;
use axum::body::Body as AxumBody;
use http::{Request, Response};
use std::task::{Context, Poll};
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

/// This token will be added to the request extensions to indicate that the request has been
/// processed by the basic auth middleware.
///
/// This is now useful for the extractor that extracts "AuthenticatedBasicAuth",
/// so if ther is no [`BasicAuthToken`] in the extensions, it will return an internal server error,
/// indicating that the request has not been processed by the basic auth middleware.
#[derive(Clone)]
pub struct BasicAuthToken;

impl<S, ReqBody> Service<Request<ReqBody>> for BasicAuth<S>
where
    S: Service<Request<ReqBody>, Response = Response<AxumBody>>,
{
    type Response = Response<AxumBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<ReqBody>) -> Self::Future {
        request.extensions_mut().insert(BasicAuthToken {});

        let (parts, body) = request.into_parts();

        match ApiBasicAuth::from_req_parts(&parts, crate::error::ErrorVerbosity::Full) {
            Ok(_) => {
                let request = Request::from_parts(parts, body);
                let future = self.inner.call(request);

                ResponseFuture::future(future)
            }
            Err(err) => ResponseFuture::api_error(err),
        }
    }
}
