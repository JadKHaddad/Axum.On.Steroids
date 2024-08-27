use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;

use crate::extractor::jwt::JwksProvider;

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
    issuer: Vec<String>,
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
        issuer: Vec<String>,
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

pub struct JwkHolder {
    last_updated: Instant,
    jwks: JwkSet,
}

impl AsRef<JwkSet> for JwkHolder {
    fn as_ref(&self) -> &JwkSet {
        &self.jwks
    }
}

pub struct JwkReadGuard<'a>(tokio::sync::RwLockReadGuard<'a, JwkHolder>);

impl<'a> JwkReadGuard<'a> {
    pub fn new(inner: tokio::sync::RwLockReadGuard<'a, JwkHolder>) -> Self {
        Self(inner)
    }
}

impl<'a> AsRef<JwkSet> for JwkReadGuard<'a> {
    fn as_ref(&self) -> &JwkSet {
        self.0.as_ref()
    }
}

impl JwksProvider for JwkRefresher {
    type Error = JwkError;

    async fn jwks(&self) -> Result<impl AsRef<jsonwebtoken::jwk::JwkSet>, Self::Error> {
        let jwks_guard = self.get().await?.read().await;
        let jwks_guard = JwkReadGuard::new(jwks_guard);

        Ok(jwks_guard)
    }

    fn audience(&self) -> &[impl ToString] {
        &self.audience
    }

    fn issuer(&self) -> &[impl ToString] {
        &self.issuer
    }

    fn validate_nbf(&self) -> bool {
        true
    }
}
