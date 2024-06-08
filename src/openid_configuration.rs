use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenIdConfiguration {
    pub issuer: String,
    pub jwks_uri: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub request_parameter_supported: bool,
    pub request_uri_parameter_supported: bool,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub claims_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
}
