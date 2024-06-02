use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use std::convert::Infallible;

use crate::traits::StateProvider;

/// Extracts an optional extractor from the request.
/// 
/// This Extractors never fails, it will always return `None` if the inner extractor fails.
pub struct Optional<X>(pub Option<X>);

#[async_trait]
impl<X, S> FromRequestParts<S> for Optional<X>
where
    X: FromRequestParts<S>,
    S: Send + Sync + StateProvider,
{
    type Rejection = Infallible;

    #[tracing::instrument(name = "optional_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = X::from_request_parts(parts, state).await.ok();

        Ok(Optional(inner))
    }
}
