use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use base64::Engine;

use crate::{
    error::{ApiError, BasicAuthError, BasicAuthErrorType},
    traits::StateProvider,
    types::used_basic_auth::UsedBasicAuth,
};

/// Extracts the basic auth from the request headers.
#[derive(Debug, Clone)]
pub struct ApiBasicAuth(pub UsedBasicAuth);

#[async_trait]
impl<S> FromRequestParts<S> for ApiBasicAuth
where
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "basic_auth_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| {
                tracing::warn!("Rejection. Authorization header not found");

                BasicAuthError::new(verbosity, BasicAuthErrorType::Missing)
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. Authorization header contains invalid characters");

                BasicAuthError::new(
                    verbosity,
                    BasicAuthErrorType::InvalidChars {
                        reason: err.to_string(),
                    },
                )
            })?;

        let split = authorization.split_once(' ');
        let encoded_basic = match split {
            Some(("Basic", encoded_basic)) => encoded_basic,
            _ => {
                tracing::warn!("Rejection. Authorization header is not 'Basic'");

                return Err(BasicAuthError::new(verbosity, BasicAuthErrorType::NotBasic).into());
            }
        };

        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded_basic)
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. Authorization header could not be decoded");

                BasicAuthError::new(
                    verbosity,
                    BasicAuthErrorType::Decode {
                        reason: err.to_string(),
                    },
                )
            })?;

        let decoded = String::from_utf8(decoded).map_err(|err| {
            tracing::warn!(%err, "Rejection. Decoded authorization header contains invalid characters");

            BasicAuthError::new(verbosity, BasicAuthErrorType::InvalidChars { reason: err.to_string() })
        })?;

        let (username, password) = match decoded.split_once(':') {
            Some((username, password)) => (username.to_string(), Some(password.to_string())),
            None => (decoded.to_string(), None),
        };

        let used_basic_auth = UsedBasicAuth { username, password };

        tracing::trace!(?used_basic_auth, "Extracted");

        Ok(ApiBasicAuth(used_basic_auth))
    }
}
