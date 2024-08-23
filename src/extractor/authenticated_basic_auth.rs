use axum::{async_trait, extract::FromRequestParts, http::request::Parts};

use crate::{
    error::{
        ApiError, BasicAuthError, BasicAuthErrorType, ErrorVerbosityProvider, InternalServerError,
    },
    extractor::basic_auth::BasicAuthProviderError,
    types::used_basic_auth::UsedBasicAuth,
};

use super::basic_auth::{ApiBasicAuth, BasicAuthProvider};

/// Extracts and authenticates the basic auth from the request headers.
#[derive(Debug, Clone)]
pub struct ApiAuthenticatedBasicAuth(pub UsedBasicAuth);

#[async_trait]
impl<S> FromRequestParts<S> for ApiAuthenticatedBasicAuth
where
    S: Send + Sync + BasicAuthProvider + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "basic_auth_authenticator", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiBasicAuth(UsedBasicAuth { username, password }) =
            ApiBasicAuth::from_request_parts(parts, state).await?;

        state
            .authenticate(&username, password.as_deref())
            .await
            .map_err(|err| {
                tracing::warn!(%username, "Rejection. Invalid basic auth");

                match err {
                    BasicAuthProviderError::Unauthenticated => ApiError::BasicAuth(
                        BasicAuthError::new(verbosity, BasicAuthErrorType::Invalid),
                    ),
                    BasicAuthProviderError::InternalServerError(err) => {
                        ApiError::InternalServerError(InternalServerError::from_generic_error(
                            verbosity, err,
                        ))
                    }
                }
            })?;

        tracing::trace!(%username, "Authenticated");

        Ok(ApiAuthenticatedBasicAuth(UsedBasicAuth {
            username,
            password,
        }))
    }
}
