use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use validator::Validate;

use crate::{
    error::{ApiError, ValidationError},
    state::StateProvider,
};

use super::extractor::{ExtractorFromRequest, ExtractorFromRequestParts};

/// An extractor that validates the extracted data by another extractor.
pub struct ValidatedFromRequestParts<X>(pub X);

#[async_trait]
impl<X, S> FromRequestParts<S> for ValidatedFromRequestParts<X>
where
    X: FromRequestParts<S, Rejection = ApiError>,
    X: ExtractorFromRequestParts<S>,
    <X as ExtractorFromRequestParts<S>>::Extracted: Validate,
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "validated_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = X::from_request_parts(parts, state).await?;
        let extracted = inner.extracted();

        match extracted.validate() {
            Ok(_) => {
                tracing::trace!("Validated");

                Ok(ValidatedFromRequestParts(inner))
            }
            Err(errors) => {
                tracing::warn!(?errors, "Validation errors");

                let verbosity = state.error_verbosity();

                Err(ValidationError::from_validation_errors(verbosity, errors).into())
            }
        }
    }
}

/// An extractor that validates the extracted data by another extractor.
pub struct ValidatedFromRequest<X>(pub X);

#[async_trait]
impl<X, S> FromRequest<S> for ValidatedFromRequest<X>
where
    X: FromRequest<S, Rejection = ApiError>,
    X: ExtractorFromRequest<S>,
    <X as ExtractorFromRequest<S>>::Extracted: Validate,
    S: Send + Sync + StateProvider,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "validated_extractor", skip_all)]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let inner = X::from_request(req, state).await?;
        let extracted = inner.extracted();

        match extracted.validate() {
            Ok(_) => {
                tracing::trace!("Validated");

                Ok(ValidatedFromRequest(inner))
            }
            Err(errors) => {
                tracing::warn!(?errors, "Validation errors");

                let verbosity = state.error_verbosity();

                Err(ValidationError::from_validation_errors(verbosity, errors).into())
            }
        }
    }
}
