// TODO: this is being passed around like crazy and it's produced by ValidApiKey extractor and ApiKey extractor.
// We don't know where it came from, so we don't know if it's valid or not.

/// A struct to hold the used API key.
///
/// Used to pass the API key arround as an Extension.
#[derive(Debug, Clone)]
pub struct UsedApiKey {
    pub used_api_key: String,
}
