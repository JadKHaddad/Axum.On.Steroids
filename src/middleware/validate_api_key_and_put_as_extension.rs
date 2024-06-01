use axum::{extract::Request, middleware::Next, response::IntoResponse};

use crate::extractor::valid_api_key::ValidApiKey;

/// Validates the API key and puts it as an extension for the next layers.
///
/// Next layers can extract the API key from the extension. See [`crate::route::api_key_protected::api_key_from_extension`] for an example.
pub async fn validate_api_key_and_put_as_extension(
    ValidApiKey(key): ValidApiKey,
    mut req: Request,
    next: Next,
) -> impl IntoResponse {
    req.extensions_mut().insert(key);

    next.run(req).await
}
