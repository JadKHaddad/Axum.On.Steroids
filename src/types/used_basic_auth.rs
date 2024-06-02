use derivative::Derivative;
use serde::Deserialize;

/// A struct to hold the used basic auth.
#[derive(Derivative, Clone, Deserialize)]
#[derivative(Debug)]
pub struct UsedBasicAuth {
    pub username: String,
    #[derivative(Debug(format_with = "crate::utils::mask_fmt"))]
    pub password: Option<String>,
}
