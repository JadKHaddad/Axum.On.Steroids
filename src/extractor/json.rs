use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Json as AxumJson, Request},
};
use schemars::{schema_for, JsonSchema};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

use crate::{
    error::{ApiError, InternalServerError, JsonBodyError, JsonBodyErrorType},
    traits::StateProvider,
};

/// A Wrapper around [`axum::extract::Json`] that rejects with an [`ApiError`].
///
/// Extracts the request body as JSON consuming the request.
pub struct ApiJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned + JsonSchema + Debug + Send,
    S: Send + Sync + StateProvider,
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
                let json_body_error_type = match json_rejection {
                    JsonRejection::JsonDataError(_) => JsonBodyErrorType::DataError,
                    JsonRejection::JsonSyntaxError(_) => JsonBodyErrorType::SyntaxError,
                    JsonRejection::MissingJsonContentType(_) => {
                        JsonBodyErrorType::MissingJsonContentType
                    }
                    _ => {
                        return Err(InternalServerError::from_generic_error(
                            state.error_verbosity(),
                            json_rejection,
                        )
                        .into())
                    }
                };

                tracing::warn!(rejection=?json_rejection, "Rejection");

                let verbosity = state.error_verbosity();

                let json_body_error_reason = json_rejection.body_text();

                let json_body_expected_schema = serde_yaml::to_string(&schema_for!(T))
                    .map_err(|err| InternalServerError::from_generic_error(verbosity, err))?;

                Err(JsonBodyError::new(
                    verbosity,
                    json_body_error_type,
                    json_body_error_reason,
                    json_body_expected_schema,
                )
                .into())
            }
        }
    }
}
