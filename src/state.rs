use std::{ops::Deref, sync::Arc};

use crate::{error::ErrorVerbosity, traits::StateProvider};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(
        error_verbosity: ErrorVerbosity,
        api_key_header_name: String,
        api_keys: Vec<String>,
    ) -> Self {
        Self {
            inner: Arc::new(ApiStateInner {
                error_verbosity,
                api_key_header_name,
                api_keys,
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
    api_keys: Vec<String>,
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
            if valid_key == key {
                return true;
            }
        }

        false
    }
}
