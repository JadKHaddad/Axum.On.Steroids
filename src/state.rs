use std::{ops::Deref, sync::Arc};

use crate::error::ErrorVerbosityProvider;
use crate::extractor::api_key::ApiKeyProvider;
use crate::jwt::JwkRefresher;

use crate::{
    error::ErrorVerbosity,
    types::{used_api_key::UsedApiKey, used_basic_auth::UsedBasicAuth},
};

/// Describes our state to axum.
///
/// This trait is crate private and therefore has no unnecessary generics.
pub trait StateProvider {
    /// Authenticates the basic auth.
    fn basic_auth_authenticate(&self, username: &str, password: Option<&str>) -> bool;

    /// Validates the JWT returning the claims.
    fn jwk_refresher(&self) -> &JwkRefresher;
}

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
}

impl StateProvider for ApiState {
    fn basic_auth_authenticate(&self, username: &str, password: Option<&str>) -> bool {
        for valid_user in self.basic_auth_users.iter() {
            if valid_user.username == username && valid_user.password.as_deref() == password {
                return true;
            }
        }

        false
    }

    fn jwk_refresher(&self) -> &JwkRefresher {
        &self.jwk_refresher
    }
}
