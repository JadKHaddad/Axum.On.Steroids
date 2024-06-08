use std::fmt::Debug;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;

use crate::{
    error::{ApiError, JwtError, JwtErrorType},
    extractor::bearer_token::ApiBearerToken,
    traits::StateProvider,
    types::used_bearer_token::UsedBearerToken,
};

/// Extracts and validates the claims from the bearer JWT token.
#[derive(Debug)]
pub struct ApiJwt<C>(pub C);

#[async_trait]
impl<C, S> FromRequestParts<S> for ApiJwt<C>
where
    C: DeserializeOwned + Debug,
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "jwt_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiBearerToken(UsedBearerToken { value }) =
            ApiBearerToken::from_request_parts(parts, state).await?;

        let claims = state.jwt_validate::<C>(&value).await.map_err(|err| {
            tracing::warn!(%err, "Rejection");

            JwtError::new(
                verbosity,
                JwtErrorType::Invalid {
                    reason: err.to_string(),
                },
            )
        })?;

        tracing::trace!(?claims, "Extracted");

        Ok(ApiJwt(claims))
    }
}
