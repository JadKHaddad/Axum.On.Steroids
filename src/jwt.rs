use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;

use crate::extractor::jwt::{validation::JwtValidator, JwtProvider, JwtProviderError};

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
}

impl JwtProvider for JwkRefresher {
    type Error = JwkError;

    async fn validate<C>(&self, jwt: &str) -> Result<C, JwtProviderError<Self::Error>>
    where
        C: DeserializeOwned,
    {
        let jwks_guard = self
            .get()
            .await
            .map_err(JwtProviderError::InternalServerError)?
            .read()
            .await;

        let jwks = jwks_guard.as_ref();

        Ok(JwtValidator::validate(
            jwt,
            jwks,
            &self.audience,
            &[&self.issuer],
            true,
        )?)
    }
}

pub struct JwkHolder {
    last_updated: Instant,
    jwks: JwkSet,
}

impl AsRef<JwkSet> for JwkHolder {
    fn as_ref(&self) -> &JwkSet {
        &self.jwks
    }
}
