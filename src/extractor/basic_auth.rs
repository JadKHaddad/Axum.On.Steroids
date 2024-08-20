use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use base64::Engine;

use crate::{
    error::{ApiError, BasicAuthError, BasicAuthErrorType, ErrorVerbosity},
    state::StateProvider,
    types::used_basic_auth::UsedBasicAuth,
};

/// Extracts the basic auth from the request headers.
#[derive(Debug, Clone)]
pub struct ApiBasicAuth(pub UsedBasicAuth);

impl ApiBasicAuth {
    fn extract_authorization(parts: &Parts, verbosity: ErrorVerbosity) -> Result<&str, ApiError> {
        let authorization = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| {
                tracing::warn!("Rejection. Authorization header not found");

                BasicAuthError::new(verbosity, BasicAuthErrorType::AuthMissing)
            })?
            .to_str()
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. Authorization header contains invalid characters");

                BasicAuthError::new(verbosity, BasicAuthErrorType::AuthInvalidChars { err })
            })?;

        Ok(authorization)
    }

    fn extract_encoded_basic(
        authorization: &str,
        verbosity: ErrorVerbosity,
    ) -> Result<&str, ApiError> {
        let split = authorization.split_once(' ');
        let encoded_basic = match split {
            Some(("Basic", encoded_basic)) => encoded_basic,
            _ => {
                tracing::warn!("Rejection. Authorization header is invalid Basic");

                return Err(
                    BasicAuthError::new(verbosity, BasicAuthErrorType::InvalidBasic).into(),
                );
            }
        };

        Ok(encoded_basic)
    }

    fn decode(encoded_basic: &str, verbosity: ErrorVerbosity) -> Result<String, ApiError> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded_basic)
            .map_err(|err| {
                tracing::warn!(%err, "Rejection. Authorization header could not be decoded");

                BasicAuthError::new(verbosity, BasicAuthErrorType::Decode { err })
            })?;

        let decoded = String::from_utf8(decoded).map_err(|err| {
            tracing::warn!(%err, "Rejection. Decoded authorization header contains invalid characters");

            BasicAuthError::new(verbosity, BasicAuthErrorType::AuthInvalidUTF8 { err })
        })?;

        Ok(decoded)
    }

    fn split(basic_auth: String) -> (String, Option<String>) {
        match basic_auth.split_once(':') {
            Some((username, password)) => (username.to_string(), Some(password.to_string())),
            None => (basic_auth.to_string(), None),
        }
    }

    pub fn from_req_parts(parts: &Parts, verbosity: ErrorVerbosity) -> Result<Self, ApiError> {
        let authorization = Self::extract_authorization(parts, verbosity)?;
        let encoded_basic = Self::extract_encoded_basic(authorization, verbosity)?;
        let decoded = Self::decode(encoded_basic, verbosity)?;
        let (username, password) = Self::split(decoded);

        let used_basic_auth = UsedBasicAuth { username, password };

        tracing::trace!(?used_basic_auth, "Extracted");

        Ok(ApiBasicAuth(used_basic_auth))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiBasicAuth
where
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "basic_auth_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        Self::from_req_parts(parts, verbosity)
    }
}
