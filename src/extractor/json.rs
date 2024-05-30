use axum::{
    async_trait,
    extract::{FromRequest, Json as AxumJson, Request},
};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::{ApiError, BodyError, InternalServerError};

/// A Wrapper around [`axum::extract::Json`] that rejects with an [`ApiError`].
///
/// Extracts the request body as JSON consuming the request.
pub struct ApiJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "json", skip_all)]
    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let json = AxumJson::<T>::from_request(req, _state).await;

        match json {
            Ok(json) => {
                tracing::trace!(json=?json.0, "Extracted");

                Ok(ApiJson(json.0))
            }
            Err(json_rejection) => {
                tracing::warn!(rejection=?json_rejection, "Rejection");

                let body_error_reason = json_rejection.body_text();

                let body_expected_schema =
                    serde_yaml::to_string(&schema_for!(T)).map_err(InternalServerError::from)?;

                Err(BodyError {
                    body_error_reason,
                    body_expected_schema,
                }
                .into())
            }
        }
    }
}
