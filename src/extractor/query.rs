use axum::{
    async_trait,
    extract::{FromRequestParts, Query as AxumQuery},
    http::request::Parts,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    error::{ApiError, QueryError},
    state::StateProvider,
};

use super::extractor::ExtractorFromRequestParts;

/// A Wrapper around [`axum::extract::Query`] that rejects with an [`ApiError`].
///
/// Extracts query parameters from the request.
pub struct ApiQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "query_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let query = AxumQuery::<T>::from_request_parts(parts, state).await;

        match query {
            Ok(query) => {
                tracing::trace!(query=?query.0, "Extracted");

                Ok(ApiQuery(query.0))
            }
            Err(query_rejection) => {
                tracing::warn!(rejection=?query_rejection, "Rejection");

                let verbosity = state.error_verbosity();

                Err(QueryError::from_query_rejection::<T>(
                    verbosity,
                    query_rejection,
                ))
            }
        }
    }
}

impl<T, S> ExtractorFromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync + StateProvider,
{
    type Extracted = T;

    fn extracted(&self) -> &Self::Extracted {
        &self.0
    }
}
