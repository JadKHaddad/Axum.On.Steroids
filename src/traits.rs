use std::future::Future;

use serde::de::DeserializeOwned;

use crate::error::ErrorVerbosity;

pub trait JwtValidationErrorProvider: std::error::Error {
    /// Indicates if the JWT is expired.
    fn is_expired(&self) -> bool;
}

pub trait StateProvider {
    type JwtValidationError: JwtValidationErrorProvider;

    /// Returns the error verbosity.
    fn error_verbosity(&self) -> ErrorVerbosity;

    /// Returns the API key header name.
    fn api_key_header_name(&self) -> &str;

    /// Validates the API key.
    fn api_key_validate(&self, key: &str) -> bool;

    /// Authenticates the basic auth.
    fn basic_auth_authenticate(&self, username: &str, password: Option<&str>) -> bool;

    /// Validates the JWT returning the claims.
    fn jwt_validate<C>(
        &self,
        jwt: &str,
    ) -> impl Future<Output = Result<C, Self::JwtValidationError>> + Send
    where
        C: DeserializeOwned;
}
