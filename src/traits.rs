use serde::de::DeserializeOwned;

use crate::error::ErrorVerbosity;

pub trait StateProvider {
    type JwtValidationError: std::error::Error + Send + Sync + 'static;

    /// Returns the error verbosity.
    fn error_verbosity(&self) -> ErrorVerbosity;

    /// Returns the API key header name.
    fn api_key_header_name(&self) -> &str;

    /// Validates the API key.
    fn api_key_validate(&self, key: &str) -> bool;

    /// Authenticates the basic auth.
    fn basic_auth_authenticate(&self, username: &str, password: Option<&str>) -> bool;

    /// Validates the JWT returning the claims.
    async fn jwt_validate<C, E>(&self, jwt: &str) -> Result<C, Self::JwtValidationError>
    where
        C: DeserializeOwned;
}
