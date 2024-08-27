use std::{
    fmt::{Debug, Display},
    future::Future,
};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use serde::de::DeserializeOwned;
use validation::JwtValidator;

use crate::{
    error::{ApiError, ErrorVerbosityProvider, InternalServerError, JwtError, JwtErrorType},
    extractor::bearer_token::ApiBearerToken,
    types::used_bearer_token::UsedBearerToken,
};

/// Extracts and validates the claims from the bearer JWT token.
#[derive(Debug)]
pub struct ApiJwt<C>(pub C);

#[async_trait]
impl<C, S> FromRequestParts<S> for ApiJwt<C>
where
    C: DeserializeOwned + Debug,
    S: Send + Sync + JwksProvider + ErrorVerbosityProvider,
    <S as JwksProvider>::Error: Into<anyhow::Error> + Display,
{
    type Rejection = ApiError;

    #[tracing::instrument(name = "jwt_extractor", skip_all)]
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let verbosity = state.error_verbosity();

        let ApiBearerToken(UsedBearerToken { value }) =
            ApiBearerToken::from_request_parts(parts, state).await?;

        let jwks = state.jwks().await.map_err(|err| {
            ApiError::InternalServerError(InternalServerError::from_generic_error(verbosity, err))
        })?;

        let claims = JwtValidator::validate::<C, _, _>(
            &value,
            jwks.as_ref(),
            state.audience(),
            state.issuer(),
            state.validate_nbf(),
        )
        .map_err(|err| {
            tracing::warn!(%err, "Rejection");

            if err.is_expired() {
                return ApiError::Jwt(JwtError::new(verbosity, JwtErrorType::ExpiredSignature));
            }

            ApiError::Jwt(JwtError::new(verbosity, JwtErrorType::Invalid { err }))
        })?;

        tracing::trace!(?claims, "Extracted");

        Ok(ApiJwt(claims))
    }
}

pub mod validation {
    use std::str::FromStr;

    use jsonwebtoken::{
        decode, decode_header,
        jwk::{AlgorithmParameters, JwkSet},
        Algorithm, DecodingKey, Validation,
    };
    use serde::de::DeserializeOwned;

    pub struct JwtValidator;

    impl JwtValidator {
        pub fn validate<C, A, I>(
            jwt: &str,
            jwks: &JwkSet,
            audience: &[A],
            issuer: &[I],
            validate_nbf: bool,
        ) -> Result<C, JwtValidationError>
        where
            C: DeserializeOwned,
            A: ToString,
            I: ToString,
        {
            let header = decode_header(jwt).map_err(JwtValidationError::DecodeHeader)?;
            let kid = header.kid.ok_or(JwtValidationError::NoKid)?;

            let jwk = jwks
                .find(&kid)
                .ok_or(JwtValidationError::NoMatchingJWK { kid })?;
            let AlgorithmParameters::RSA(ref rsa) = jwk.algorithm else {
                return Err(JwtValidationError::UnsupportedAlgorithm);
            };

            let decoding_key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(JwtValidationError::DecodingKey)?;

            let key_algorithm = jwk
                .common
                .key_algorithm
                .ok_or(JwtValidationError::KeyAlgorithmNotFound)?;

            let mut validation = Validation::new(
                Algorithm::from_str(key_algorithm.to_string().as_str()).map_err(|err| {
                    JwtValidationError::ValidationAlgorithm { key_algorithm, err }
                })?,
            );

            validation.set_audience(audience);
            validation.set_issuer(issuer);
            validation.validate_nbf = validate_nbf;

            let token_data = decode::<C>(jwt, &decoding_key, &validation)?;

            Ok(token_data.claims)
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum JwtValidationError {
        #[error("Error decoding header: {0}")]
        DecodeHeader(#[source] jsonwebtoken::errors::Error),
        #[error("Token doesn't have a kid header field")]
        NoKid,
        #[error("No matching JWK found for the given kid: {kid}")]
        NoMatchingJWK { kid: String },
        #[error("JWK algorithm is not supported")]
        UnsupportedAlgorithm,
        #[error("Error creating decoding key: {0}")]
        DecodingKey(#[source] jsonwebtoken::errors::Error),
        #[error("No key algorithm found in JWK")]
        KeyAlgorithmNotFound,
        #[error("Error creating validation algorithm from Key Algorithm: {key_algorithm}, {err}")]
        ValidationAlgorithm {
            key_algorithm: jsonwebtoken::jwk::KeyAlgorithm,
            #[source]
            err: jsonwebtoken::errors::Error,
        },
        #[error("Error validating token: {0}")]
        TokenInvalid(#[from] jsonwebtoken::errors::Error),
    }

    impl JwtValidationError {
        pub fn is_expired(&self) -> bool {
            match self {
                JwtValidationError::TokenInvalid(err) => matches!(
                    err.kind(),
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature
                ),
                _ => false,
            }
        }
    }
}

pub trait JwksProvider {
    type Error;

    /// Returns the JWK set.
    fn jwks(
        &self,
    ) -> impl Future<Output = Result<impl AsRef<jsonwebtoken::jwk::JwkSet>, Self::Error>> + Send;

    /// Returns the audience.
    fn audience(&self) -> &[impl ToString];

    /// Returns the issuer.
    fn issuer(&self) -> &[impl ToString];

    /// Returns whether to validate the nbf claim.
    fn validate_nbf(&self) -> bool;
}
