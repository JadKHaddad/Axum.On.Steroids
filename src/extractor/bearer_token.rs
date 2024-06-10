use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{
    error::{ApiError, BearerError, BearerErrorType, ErrorVerbosity},
    traits::StateProvider,
    types::used_bearer_token::UsedBearerToken,
};

/// Extracts the bearer token from the request headers.
#[derive(Debug, Clone)]
pub struct ApiBearerToken(pub UsedBearerToken);

impl ApiBearerToken {
    fn extract_authorization(parts: &Parts, verbosity: ErrorVerbosity) -> Result<&str, ApiError> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| {
                tracing::warn!("Rejection. Authorization header not found");

                BearerError::new(verbosity, BearerErrorType::AuthMissing)
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. Authorization header contains invalid characters");

                BearerError::new(verbosity, BearerErrorType::AuthInvalidChars { err })
            })?;

        Ok(authorization)
    }

    fn extract_bearer_token(
        authorization: &str,
        verbosity: ErrorVerbosity,
    ) -> Result<&str, ApiError> {
        let split = authorization.split_once(' ');
        let bearer_token = match split {
            Some(("Bearer", bearer_token)) => bearer_token,
            _ => {
                tracing::warn!("Rejection. Authorization header is invalid Bearer");

                return Err(BearerError::new(verbosity, BearerErrorType::InvalidBearer).into());
            }
        };

        Ok(bearer_token)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiBearerToken
where
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "bearer_token_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let authorization = Self::extract_authorization(parts, verbosity)?;
        let bearer_token = Self::extract_bearer_token(authorization, verbosity)?;

        let used_bearer_token = UsedBearerToken {
            value: bearer_token.to_string(),
        };

        tracing::trace!(?used_bearer_token, "Extracted");

        Ok(ApiBearerToken(used_bearer_token))
    }
}
