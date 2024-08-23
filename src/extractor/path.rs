use axum::{
    async_trait,
    extract::{FromRequestParts, Path as AxumPath},
    http::request::Parts,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::error::{ApiError, ErrorVerbosityProvider, PathError};

use super::Extractor;

/// A Wrapper around [`axum::extract::Path`] that rejects with an [`ApiError`].
///
/// Extracts path parameters from the request.
pub struct ApiPath<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "path_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let path = AxumPath::<T>::from_request_parts(parts, state).await;

        match path {
            Ok(path) => {
                tracing::trace!(path=?path.0, "Extracted");

                Ok(ApiPath(path.0))
            }
            Err(path_rejection) => {
                tracing::warn!(rejection=?path_rejection, "Rejection");

                let verbosity = state.error_verbosity();

                Err(PathError::from_path_rejection(verbosity, path_rejection))
            }
        }
    }
}

impl<T> Extractor for ApiPath<T> {
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
