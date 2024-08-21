use crate::{
    error::{ApiError, BasicAuthErrorType, NotFoundError},
    extractor::basic_auth::ApiBasicAuth,
    types::used_basic_auth,
};

use super::{future::ResponseFuture, provider::BasicAuthProvider};
use axum::{body::Body as AxumBody, response::IntoResponse};
use futures::FutureExt;
use http::{Request, Response};
use std::task::{Context, Poll};
use tower::Service;

/// Applies basic authentication to the request.
#[derive(Debug, Clone)]
pub struct BasicAuth<T, P> {
    inner: T,
    provider: P,
}

impl<T, P> BasicAuth<T, P> {
    pub const fn new(inner: T, provider: P) -> Self {
        BasicAuth { inner, provider }
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

impl<S, ReqBody, P> Service<Request<ReqBody>> for BasicAuth<S, P>
where
    P: BasicAuthProvider + Send + Clone + 'static,
    S: Service<Request<ReqBody>, Response = Response<AxumBody>> + Send,
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
            Ok(ApiBasicAuth(used_basic_auth)) => {
                let request = Request::from_parts(parts, body);
                let future = self.inner.call(request);

                let provider = self.provider.clone();

                let boxed = Box::pin(async move {
                    provider
                        .authenticate(
                            &used_basic_auth.username,
                            used_basic_auth.password.as_deref(),
                        )
                        .await
                });

                ResponseFuture::future(boxed, future)
            }
            Err(err) => ResponseFuture::api_error(err),
        }
    }
}
