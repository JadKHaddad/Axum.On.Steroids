use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{ApiError, ApiKeyError, ApiKeyErrorType, ErrorVerbosityProvider, InternalServerError},
    extractor::api_key::{ApiKey, ApiKeyProviderError},
    types::used_api_key::UsedApiKey,
};

use super::api_key::ApiKeyProvider;

/// Extracts and validates the API key from the request headers.
#[derive(Debug, Clone)]
pub struct ValidApiKey(pub UsedApiKey);

#[async_trait]
impl<S> FromRequestParts<S> for ValidApiKey
where
    S: Send + Sync + ApiKeyProvider + ErrorVerbosityProvider,
    <S as ApiKeyProvider>::Error: Into<anyhow::Error>,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "api_key_validator", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiKey(UsedApiKey { value: api_key }) =
            ApiKey::from_request_parts(parts, state).await?;

        state.validate(&api_key).await.map_err(|err| {
            tracing::warn!(%api_key, "Rejection. Invalid API key");

            match err {
                ApiKeyProviderError::Invalid => {
                    ApiError::ApiKey(ApiKeyError::new(verbosity, ApiKeyErrorType::Invalid))
                }
                ApiKeyProviderError::InternalServerError(err) => ApiError::InternalServerError(
                    InternalServerError::from_generic_error(verbosity, err),
                ),
            }
        })?;

        tracing::trace!(%api_key, "Validated");

        Ok(ValidApiKey(UsedApiKey { value: api_key }))
    }
}
