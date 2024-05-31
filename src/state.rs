use std::{ops::Deref, sync::Arc};

use crate::{
    error::{ErrorVerbosity, ErrorVerbosityProvider},
    extractor::api_key::ApiKeyProvider,
};

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

impl ErrorVerbosityProvider for ApiState {
    fn error_verbosity(&self) -> ErrorVerbosity {
        self.error_verbosity
    }
}

impl ApiKeyProvider for ApiState {
    fn header_name(&self) -> &str {
        &self.api_key_header_name
    }

    // FIX: very expensive operation
    fn validate(&self, key: &str) -> bool {
        self.api_keys.contains(&key.to_string())
    }
}
