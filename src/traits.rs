use crate::error::ErrorVerbosity;

pub trait StateProvider {
    /// Returns the error verbosity.
    fn error_verbosity(&self) -> ErrorVerbosity;

    /// Returns the API key header name.
    fn api_key_header_name(&self) -> &str;

    /// Validates the API key.
    fn api_key_validate(&self, key: &str) -> bool;
}
