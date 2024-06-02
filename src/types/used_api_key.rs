use serde::{Deserialize, Serialize};

/// A struct to hold the used API key.
///
/// Used to define the type of the inner API key.
/// For example, we can use a heapless string here.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UsedApiKey {
    // TODO: can use a heapless string here.
    pub api_key: String,
}
