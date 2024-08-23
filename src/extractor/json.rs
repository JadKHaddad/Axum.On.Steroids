use axum::{
    async_trait,
    extract::{FromRequest, Json as AxumJson, Request},
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::{ApiError, ErrorVerbosityProvider, JsonBodyError};

use super::Extractor;

/// A Wrapper around [`axum::extract::Json`] that rejects with an [`ApiError`].
///
/// Extracts the request body as JSON consuming the request.
pub struct ApiJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "json_extractor", skip_all)]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let json = AxumJson::<T>::from_request(req, state).await;

        match json {
            Ok(json) => {
                tracing::trace!(json=?json.0, "Extracted");

                Ok(ApiJson(json.0))
            }
            Err(json_rejection) => {
                tracing::warn!(rejection=?json_rejection, "Rejection");

                let verbosity = state.error_verbosity();

                Err(JsonBodyError::from_json_rejection::<T>(
                    verbosity,
                    json_rejection,
                ))
            }
        }
    }
}

impl<T> Extractor for ApiJson<T> {
    type Extracted = T;

    fn extracted(&self) -> &Self::Extracted {
        &self.0
    }

    fn extracted_mut(&mut self) -> &mut Self::Extracted {
        &mut self.0
    }

    fn into_extracted(self) -> Self::Extracted {
        self.0
    }
}
