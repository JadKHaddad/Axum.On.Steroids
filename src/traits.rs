use crate::error::ErrorVerbosity;

pub trait ErrorVerbosityProvider {
    fn error_verbosity(&self) -> ErrorVerbosity;
}

pub trait ApiKeyProvider {
    /// Returns the API key header name.
    fn header_name(&self) -> &str;

    /// Validates the API key.
    fn validate(&self, key: &str) -> bool;
}

pub trait StateProvider: ApiKeyProvider + ErrorVerbosityProvider {}
