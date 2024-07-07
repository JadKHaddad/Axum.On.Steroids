use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod get_book;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub isbn: String,
    pub year: u16,
    pub id: i64,
}
