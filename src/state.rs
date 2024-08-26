use std::convert::Infallible;
use std::future::Future;
use std::{ops::Deref, sync::Arc};

use serde::de::DeserializeOwned;

use crate::error::ErrorVerbosityProvider;
use crate::extractor::api_key::{ApiKeyProvider, ApiKeyProviderError};
use crate::extractor::basic_auth::{BasicAuthProvider, BasicAuthProviderError};
use crate::extractor::jwt::{JwtProvider, JwtProviderError};
use crate::jwt::{JwkError, JwkRefresher};

use crate::{
    error::ErrorVerbosity,
    types::{used_api_key::UsedApiKey, used_basic_auth::UsedBasicAuth},
};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        error_verbosity: ErrorVerbosity,
        api_key_header_name: String,
        api_keys: Vec<UsedApiKey>,
        basic_auth_users: Vec<UsedBasicAuth>,
        jwk_refresher: JwkRefresher,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(ApiStateInner {
                error_verbosity,
                api_key_header_name,
                api_keys,
                basic_auth_users,
                jwk_refresher,
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
    jwk_refresher: JwkRefresher,
}

impl ErrorVerbosityProvider for ApiState {
    fn error_verbosity(&self) -> ErrorVerbosity {
        self.error_verbosity
    }
}

impl ApiKeyProvider for ApiState {
    type Error = Infallible;

    fn header_name(&self) -> &str {
        &self.api_key_header_name
    }

    async fn validate(&self, key: &str) -> Result<(), ApiKeyProviderError<Self::Error>> {
        for valid_key in self.api_keys.iter() {
            if valid_key.value == key {
                return Ok(());
            }
        }

        Err(ApiKeyProviderError::Invalid)
    }
}

impl BasicAuthProvider for ApiState {
    type Error = Infallible;

    async fn authenticate(
        &self,
        username: &str,
        password: Option<&str>,
    ) -> Result<(), BasicAuthProviderError<Self::Error>> {
        for valid_user in self.basic_auth_users.iter() {
            if valid_user.username == username && valid_user.password.as_deref() == password {
                return Ok(());
            }
        }

        Err(BasicAuthProviderError::Unauthenticated)
    }
}

impl JwtProvider for ApiState {
    type Error = JwkError;

    fn validate<C>(
        &self,
        jwt: &str,
    ) -> impl Future<Output = Result<C, JwtProviderError<Self::Error>>> + Send
    where
        C: DeserializeOwned,
    {
        self.jwk_refresher.validate::<C>(jwt)
    }
}
