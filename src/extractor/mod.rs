pub mod api_key;
pub mod authenticated_basic_auth;
pub mod basic_auth;
pub mod bearer_token;
pub mod json;
pub mod jwt;
pub mod optional;
pub mod path;
pub mod query;
pub mod valid_api_key;
pub mod validated;

pub trait Extractor {
    type Extracted;

    fn extracted(&self) -> &Self::Extracted;

    fn extracted_mut(&mut self) -> &mut Self::Extracted;

    fn into_extracted(self) -> Self::Extracted;
}
