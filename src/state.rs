use std::{ops::Deref, sync::Arc};

use crate::error::{ErrorVerbosity, ErrorVerbosityProvider};

#[derive(Clone)]
pub struct ApiState {
    inner: Arc<ApiStateInner>,
}

impl ApiState {
    pub fn new(error_verbosity: ErrorVerbosity) -> Self {
        Self {
            inner: Arc::new(ApiStateInner { error_verbosity }),
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
}

impl ErrorVerbosityProvider for ApiState {
    fn error_verbosity(&self) -> ErrorVerbosity {
        self.error_verbosity
    }
}
