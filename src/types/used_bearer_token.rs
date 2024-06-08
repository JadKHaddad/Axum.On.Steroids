use serde::{Deserialize, Serialize};

/// A struct to hold the used bearer token.
///
/// Used to define the type of the inner bearer token.
/// For example, we can use a heapless string here.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UsedBearerToken {
    // TODO: can use a heapless string here.
    pub value: String,
}
