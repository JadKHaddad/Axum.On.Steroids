use crate::error::ErrorVerbosity;

pub trait StateProvider {
    /// Returns the error verbosity.
    fn error_verbosity(&self) -> ErrorVerbosity;

    /// Returns the API key header name.
    fn header_name(&self) -> &str;

    /// Validates the API key.
    fn validate(&self, key: &str) -> bool;
}
