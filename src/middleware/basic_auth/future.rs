use crate::error::ApiError;
use axum::{body::Body as AxumBody, response::IntoResponse};
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
        kind: Kind<F>,
    }
}

impl<F> ResponseFuture<F> {
    pub fn future(future: F) -> Self {
        Self {
            kind: Kind::Future { future },
        }
    }

    pub fn api_error(api_error: ApiError) -> Self {
        Self {
            kind: Kind::ApiError {
                api_error: Some(api_error),
            },
        }
    }
}

pin_project! {
    #[project = KindProj]
    enum Kind<F> {
        Future {
            #[pin]
            future: F,
        },
        ApiError {
            api_error: Option<ApiError>,
        },
    }
}

impl<F, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<AxumBody>, E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().kind.project() {
            KindProj::Future { future } => future.poll(cx),
            KindProj::ApiError { api_error } => {
                let response = api_error
                    .take()
                    .expect("future polled after completion")
                    .into_response();
                Poll::Ready(Ok(response))
            }
        }
    }
}
