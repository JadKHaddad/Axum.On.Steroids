use std::{ops::Deref, str::FromStr, sync::Arc};

use anyhow::Context;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, KeyAlgorithm},
    Algorithm, DecodingKey, Validation,
};
use jwks::JwkRefresher;
use serde::de::DeserializeOwned;

use crate::{
    error::{ErrorVerbosity, JwtErrorType},
    openid_configuration::OpenIdConfiguration,
    traits::{JwtValidationErrorProvider, StateProvider},
    types::{used_api_key::UsedApiKey, used_basic_auth::UsedBasicAuth},
};

mod jwks;

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        http_client: reqwest::Client,
        error_verbosity: ErrorVerbosity,
        api_key_header_name: String,
        api_keys: Vec<UsedApiKey>,
        basic_auth_users: Vec<UsedBasicAuth>,
        openid_config: OpenIdConfiguration,
        jwks_time_to_live_in_seconds: u64,
        audience: Vec<String>,
    ) -> anyhow::Result<Self> {
        let jwk_refresher = JwkRefresher::new(
            jwks_time_to_live_in_seconds,
            openid_config.jwks_uri.clone(),
            http_client,
        )
        .await
        .context("Failed to create JwkRefresher")?;

        Ok(Self {
            inner: Arc::new(ApiStateInner {
                error_verbosity,
                api_key_header_name,
                api_keys,
                basic_auth_users,
                openid_config,
                jwk_refresher,
                audience,
            }),
        })
    }
}

impl Deref for ApiState {
    type Target = ApiStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct ApiStateInner {
    error_verbosity: ErrorVerbosity,
    api_key_header_name: String,
    api_keys: Vec<UsedApiKey>,
    basic_auth_users: Vec<UsedBasicAuth>,
    openid_config: OpenIdConfiguration,
    jwk_refresher: JwkRefresher,
    audience: Vec<String>,
}

impl StateProvider for ApiState {
    type JwtValidationError = JwtValidationError;

    fn error_verbosity(&self) -> ErrorVerbosity {
        self.error_verbosity
    }

    fn api_key_header_name(&self) -> &str {
        &self.api_key_header_name
    }

    fn api_key_validate(&self, key: &str) -> bool {
        for valid_key in self.api_keys.iter() {
            if valid_key.value == key {
                return true;
            }
        }

        false
    }

    fn basic_auth_authenticate(&self, username: &str, password: Option<&str>) -> bool {
        for valid_user in self.basic_auth_users.iter() {
            if valid_user.username == username && valid_user.password.as_deref() == password {
                return true;
            }
        }

        false
    }

    async fn jwt_validate<C>(&self, jwt: &str) -> Result<C, Self::JwtValidationError>
    where
        C: DeserializeOwned,
    {
        let jwks_guard = self.jwk_refresher.get().await?.read().await;
        let jwks = jwks_guard.jwks();

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
            Algorithm::from_str(key_algorithm.to_string().as_str())
                .map_err(|err| JwtValidationError::ValidationAlgorithm { key_algorithm, err })?,
        );

        validation.set_audience(&self.audience);
        validation.set_issuer(&[&self.openid_config.issuer]);
        validation.validate_nbf = true;

        let token_data = decode::<C>(jwt, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtValidationError {
    #[error("Error getting jwks: {0}")]
    Jwks(#[from] jwks::JwkError),
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
        key_algorithm: KeyAlgorithm,
        #[source]
        err: jsonwebtoken::errors::Error,
    },
    #[error("Error validating token: {0}")]
    TokenInvalid(#[from] jsonwebtoken::errors::Error),
}

impl JwtValidationErrorProvider for JwtValidationError {
    fn is_expired(&self) -> bool {
        match self {
            JwtValidationError::TokenInvalid(err) => matches!(
                err.kind(),
                jsonwebtoken::errors::ErrorKind::ExpiredSignature
            ),
            _ => false,
        }
    }

    fn into_jwt_error_type(self) -> JwtErrorType {
        JwtErrorType::Invalid { err: self }
    }
}
