use http::Response;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
       future: F
    }
}

impl<F> ResponseFuture<F> {
    pub fn new(future: F) -> Self {
        ResponseFuture { future }
    }
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(cx)
    }
}
