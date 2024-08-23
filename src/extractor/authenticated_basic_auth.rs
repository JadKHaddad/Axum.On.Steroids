use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{ApiError, BasicAuthError, BasicAuthErrorType, ErrorVerbosityProvider},
    state::StateProvider,
    types::used_basic_auth::UsedBasicAuth,
};

use super::basic_auth::ApiBasicAuth;

/// Extracts and authenticates the basic auth from the request headers.
#[derive(Debug, Clone)]
pub struct ApiAuthenticatedBasicAuth(pub UsedBasicAuth);

#[async_trait]
impl<S> FromRequestParts<S> for ApiAuthenticatedBasicAuth
where
    S: Send + Sync + StateProvider + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "basic_auth_authenticator", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiBasicAuth(UsedBasicAuth { username, password }) =
            ApiBasicAuth::from_request_parts(parts, state).await?;

        if !state.basic_auth_authenticate(&username, password.as_deref()) {
            tracing::warn!(%username, "Rejection. Invalid basic auth");

            return Err(BasicAuthError::new(verbosity, BasicAuthErrorType::Invalid).into());
        }

        tracing::trace!(%username, "Authenticated");

        Ok(ApiAuthenticatedBasicAuth(UsedBasicAuth {
            username,
            password,
        }))
    }
}
