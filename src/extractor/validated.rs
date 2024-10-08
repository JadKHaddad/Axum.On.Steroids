use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use validator::Validate;

use crate::error::{ApiError, ErrorVerbosityProvider, ValidationError};

use super::Extractor;

/// An extractor that validates the extracted data by another extractor.
pub struct Validated<X>(pub X);

impl<X> Validated<X> {
    fn extract<S>(inner: X, state: &S) -> Result<Self, ApiError>
    where
        X: Extractor,
        S: ErrorVerbosityProvider,
        <X as Extractor>::Extracted: Validate,
    {
        let extracted = inner.extracted();

        match extracted.validate() {
            Ok(_) => {
                tracing::trace!("Validated");

                Ok(Validated(inner))
            }
            Err(errors) => {
                tracing::warn!(?errors, "Validation errors");

                let verbosity = state.error_verbosity();

                Err(ValidationError::from_validation_errors(verbosity, errors).into())
            }
        }
    }
}

#[async_trait]
impl<X, S> FromRequestParts<S> for Validated<X>
where
    X: FromRequestParts<S, Rejection = ApiError>,
    X: Extractor,
    <X as Extractor>::Extracted: Validate,
    S: Send + Sync + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "validated_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = X::from_request_parts(parts, state).await?;

        Self::extract(inner, state)
    }
}

#[async_trait]
impl<X, S> FromRequest<S> for Validated<X>
where
    X: FromRequest<S, Rejection = ApiError>,
    X: Extractor,
    <X as Extractor>::Extracted: Validate,
    S: Send + Sync + ErrorVerbosityProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "validated_extractor", skip_all)]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let inner = X::from_request(req, state).await?;

        Self::extract(inner, state)
    }
}
