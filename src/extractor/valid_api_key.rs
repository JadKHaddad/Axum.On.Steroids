use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{ApiError, ApiKeyError, ApiKeyErrorType},
    extractor::api_key::ApiKey,
    traits::{ApiKeyProvider, ErrorVerbosityProvider},
    types::used_api_key::UsedApiKey,
};

/// Extracts and validates the API key from the request headers.
#[derive(Debug, Clone)]
pub struct ValidApiKey(pub UsedApiKey);

#[async_trait]
impl<S> FromRequestParts<S> for ValidApiKey
where
    S: Send + Sync + ApiKeyProvider + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "api_key_validator", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiKey(UsedApiKey { used_api_key }) = ApiKey::from_request_parts(parts, state).await?;

        if !state.validate(&used_api_key) {
            tracing::warn!(%used_api_key, "Rejection. Invalid API key");

            return Err(ApiKeyError::new(verbosity, ApiKeyErrorType::Invalid).into());
        }

        tracing::trace!(%used_api_key, "Validated");

        let used_api_key = used_api_key.to_string();

        Ok(ValidApiKey(UsedApiKey { used_api_key }))
    }
}
