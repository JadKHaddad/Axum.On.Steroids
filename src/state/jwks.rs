use std::time::Instant;

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum JWKSError {
    #[error("Failed to fetch JWKS from the JWKS URI: {0}")]
    Fetch(#[source] reqwest::Error),
    #[error("Failed to parse JWKS from the JWKS URI: {0}")]
    Parse(#[source] reqwest::Error),
}

pub struct JWKSRefresher {
    time_to_live_in_seconds: u64,
    jwks_uri: String,
    http_client: reqwest::Client,
    holder: RwLock<JWKSHolder>,
}

impl JWKSRefresher {
    #[tracing::instrument(skip_all)]
    async fn obtain_jwks(jwks_uri: &str, http_client: &reqwest::Client,) -> Result<JwkSet, JWKSError> {
        let jwks = http_client
            .get(jwks_uri)
            .send()
            .await
            .map_err(JWKSError::Fetch)?
            .json::<JwkSet>()
            .await
            .map_err(JWKSError::Parse)?;

        Ok(jwks)
    }

    pub async fn new(
        time_to_live_in_seconds: u64,
        jwks_uri: String,
        http_client: reqwest::Client,
    ) -> Result<Self, JWKSError> {
        let jwks = Self::obtain_jwks(&jwks_uri, &http_client).await?;
        let last_updated = Instant::now();

        Ok(Self {
            time_to_live_in_seconds,
            jwks_uri,
            http_client,
            holder: RwLock::new(JWKSHolder { last_updated, jwks }),
        })
    }

    #[tracing::instrument(skip_all)]
    async fn refresh_jwks(&self) -> Result<(), JWKSError> {
        let jwks = Self::obtain_jwks(&self.jwks_uri, &self.http_client).await?;

        let mut inner = self.holder.write().await;

        inner.jwks = jwks;
        inner.last_updated = Instant::now();

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn get(&self) -> &RwLock<JWKSHolder> {
        let last_updated = self.holder.read().await.last_updated;

        if last_updated.elapsed().as_secs() > self.time_to_live_in_seconds {
            if let Err(err) = self.refresh_jwks().await {
                tracing::error!(%err, "Failed to refresh JWKS");
            }
        }

        &self.holder
    }
}

struct JWKSHolder {
    last_updated: Instant,
    jwks: JwkSet,
}

impl JWKSHolder {
    pub fn jwks(&self) -> &JwkSet {
        &self.jwks
    }
}