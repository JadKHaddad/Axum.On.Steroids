use std::{
    future::Future,
    pin::{pin, Pin},
    task::{ready, Context, Poll},
};

use axum::{body::Body as AxumBody, extract::Request, http::Response};
use http_body::Body;
use http_body_util::BodyExt;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct ResponseBodyTraceLayer {}

impl<S> Layer<S> for ResponseBodyTraceLayer {
    type Service = ResponseBodyTraceService<S>;

    fn layer(&self, service: S) -> Self::Service {
        ResponseBodyTraceService { service }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseBodyTraceService<S> {
    service: S,
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for ResponseBodyTraceService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ResBody: Body,
{
    type Response = Response<AxumBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        ResponseFuture {
            inner: self.service.call(request),
        }
    }
}

pin_project! {
    /// Response future of [`ResponseBodyTraceLayer`].
    #[derive(Debug)]
    pub struct ResponseFuture<F> {
        #[pin]
        inner: F,
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
    B: Body,
{
    type Output = Result<Response<AxumBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let response = ready!(self.project().inner.poll(cx)?);

        let (parts, body) = response.into_parts();
        let pinned = pin!(body.collect());

        if let Ok(collected) = ready!(pinned.poll(cx)) {
            let bytes = collected.to_bytes();

            if let Ok(body) = std::str::from_utf8(&bytes) {
                tracing::trace!(%body, "Response Body");
            }

            let response = Response::from_parts(parts, AxumBody::from(bytes));

            return Poll::Ready(Ok(response));
        }

        todo!()
    }
}
