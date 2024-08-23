use std::{str::FromStr, time::Instant};

use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet, KeyAlgorithm},
    Algorithm, DecodingKey, Validation,
};
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

pub trait JwkProvider {
    fn jwk_refresher(&self) -> &JwkRefresher;
}

#[derive(Debug, thiserror::Error)]
pub enum JwkError {
    #[error("Failed to fetch Jwk from the Jwks URI: {0}")]
    Fetch(#[source] reqwest::Error),
    #[error("Failed to parse Jwk from the Jwks URI: {0}")]
    Parse(#[source] reqwest::Error),
}

pub struct JwkRefresher {
    time_to_live_in_seconds: u64,
    jwks_uri: String,
    http_client: reqwest::Client,
    holder: RwLock<JwkHolder>,
    issuer: String,
    audience: Vec<String>,
}

impl JwkRefresher {
    #[tracing::instrument(skip_all)]
    async fn obtain_jwks(
        jwks_uri: &str,
        http_client: &reqwest::Client,
    ) -> Result<JwkSet, JwkError> {
        tracing::debug!("Obtaining Jwks");

        let jwks = http_client
            .get(jwks_uri)
            .send()
            .await
            .map_err(JwkError::Fetch)?
            .json::<JwkSet>()
            .await
            .map_err(JwkError::Parse)?;

        Ok(jwks)
    }

    pub async fn new(
        time_to_live_in_seconds: u64,
        jwks_uri: String,
        issuer: String,
        audience: Vec<String>,
        http_client: reqwest::Client,
    ) -> Result<Self, JwkError> {
        let jwks = Self::obtain_jwks(&jwks_uri, &http_client).await?;
        let last_updated = Instant::now();

        Ok(Self {
            time_to_live_in_seconds,
            jwks_uri,
            issuer,
            audience,
            http_client,
            holder: RwLock::new(JwkHolder { last_updated, jwks }),
        })
    }

    #[tracing::instrument(skip_all)]
    async fn refresh_jwks(&self) -> Result<(), JwkError> {
        tracing::debug!("Refreshing Jwks");

        let jwks = Self::obtain_jwks(&self.jwks_uri, &self.http_client).await?;

        let mut inner = self.holder.write().await;

        inner.jwks = jwks;
        inner.last_updated = Instant::now();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn get(&self) -> Result<&RwLock<JwkHolder>, JwkError> {
        let last_updated = self.holder.read().await.last_updated;

        if last_updated.elapsed().as_secs() > self.time_to_live_in_seconds {
            self.refresh_jwks().await?;
        }

        Ok(&self.holder)
    }

    pub async fn jwt_validate<C>(&self, jwt: &str) -> Result<C, JwtValidationError>
    where
        C: DeserializeOwned,
    {
        let jwks_guard = self.get().await?.read().await;
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
        validation.set_issuer(&[&self.issuer]);
        validation.validate_nbf = true;

        let token_data = decode::<C>(jwt, &decoding_key, &validation)?;

        Ok(token_data.claims)
    }
}

pub struct JwkHolder {
    last_updated: Instant,
    jwks: JwkSet,
}

impl JwkHolder {
    pub fn jwks(&self) -> &JwkSet {
        &self.jwks
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtValidationError {
    #[error("Error getting jwks: {0}")]
    Jwks(#[from] JwkError),
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
