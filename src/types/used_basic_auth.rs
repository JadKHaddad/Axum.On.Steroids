/// A struct to hold the used basic auth.
#[derive(Debug, Clone)]
pub struct UsedBasicAuth {
    pub username: String,
    pub password: Option<String>,
}
