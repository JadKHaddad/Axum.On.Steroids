use axum::extract::{FromRequest, FromRequestParts};

use crate::error::ApiError;

pub trait ExtractorFromRequestParts<S>: FromRequestParts<S, Rejection = ApiError> {
    type Extracted;

    fn extracted(&self) -> &Self::Extracted;
}

pub trait ExtractorFromRequest<S>: FromRequest<S, Rejection = ApiError> {
    type Extracted;

    fn extracted(&self) -> &Self::Extracted;
}
