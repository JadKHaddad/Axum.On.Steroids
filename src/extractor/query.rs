use axum::{
    async_trait,
    extract::{FromRequestParts, Query as AxumQuery},
    http::request::Parts,
};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::{ApiError, InternalServerError, QueryError};

/// A Wrapper around [`axum::extract::Query`] that rejects with an [`ApiError`].
///
/// Extracts query parameters from the request.
pub struct ApiQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "query", skip_all)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = AxumQuery::<T>::from_request_parts(parts, _state).await;

        match query {
            Ok(query) => {
                tracing::trace!(query=?query.0, "Extracted");

                Ok(ApiQuery(query.0))
            }
            Err(query_rejection) => {
                tracing::warn!(rejection=?query_rejection, "Rejection");

                let query_error_reason = query_rejection.body_text();

                let query_expected_schema =
                    serde_yaml::to_string(&schema_for!(T)).map_err(InternalServerError::from)?;

                Err(QueryError {
                    query_error_reason,
                    query_expected_schema,
                }
                .into())
            }
        }
    }
}
