use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use axum_auth::AuthBasic;

use crate::{
    error::{ApiError, BasicAuthError},
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
        let auth_basic = AuthBasic::from_request_parts(parts, state).await;

        match auth_basic {
            Ok(AuthBasic((username, password))) => {
                let used_basic_auth = UsedBasicAuth { username, password };

                tracing::trace!(?used_basic_auth, "Extracted");

                Ok(ApiBasicAuth(used_basic_auth))
            }
            Err(auth_basic_rejection) => {
                tracing::warn!(rejection=?auth_basic_rejection, "Rejection");

                let verbosity = state.error_verbosity();

                let basic_auth_error_reason = auth_basic_rejection.1.to_string();

                Err(BasicAuthError {
                    verbosity,
                    basic_auth_error_reason,
                }
                .into())
            }
        }
    }
}
