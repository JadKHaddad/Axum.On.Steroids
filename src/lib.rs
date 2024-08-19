mod claims;
pub mod cli_args;
mod error;
mod extractor;
mod middleware;
mod openid_configuration;
mod route;
pub mod server;
mod state;
mod types;
mod utils;

#[cfg(test)]
mod test;

/// A very convenient macro to map to server error using a one-liner.
/// Instead of writing:
///
/// ```rust
/// pub async fn route(State(state): State<ApiState>) -> Result<(), ApiError> {
///     tokio::fs::read_to_string("non_existent_file.txt")
///         .await
///         .map(|_| ())
///         .map_err(|err| ApiError::from_generic_error(state.error_verbosity(), err))
/// }
/// ```
///
/// You can write:
///
/// ```rust
/// pub async fn route(State(state): State<ApiState>) -> Result<(), ApiError> {
///     tokio::fs::read_to_string("non_existent_file.txt")
///         .await
///         .map(|_| ())
///         .map_err(server_error!(state))
/// }
/// ```
macro_rules! server_error {
    ($state:ident) => {
        |err| ApiError::from_generic_error($state.error_verbosity(), err)
    };
}

use server_error;
