/// A struct to hold the used API key.
///
/// Used to define the type of the inner API key.
/// For example, we can use a heapless string here.
#[derive(Debug, Clone)]
pub struct UsedApiKey {
    // TODO: can use a heapless string here.
    pub used_api_key: String,
}
