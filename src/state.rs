use std::{ops::Deref, sync::Arc};

use crate::{
    error::ErrorVerbosity,
    traits::StateProvider,
    types::{used_api_key::UsedApiKey, used_basic_auth::UsedBasicAuth},
};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(
        error_verbosity: ErrorVerbosity,
        api_key_header_name: String,
        api_keys: Vec<UsedApiKey>,
        basic_auth_users: Vec<UsedBasicAuth>,
    ) -> Self {
        Self {
            inner: Arc::new(ApiStateInner {
                error_verbosity,
                api_key_header_name,
                api_keys,
                basic_auth_users,
            }),
        }
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
}

impl StateProvider for ApiState {
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
}
