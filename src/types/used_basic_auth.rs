/// A struct to hold the used basic auth.
#[derive(Debug, Clone)]
pub struct UsedBasicAuth {
    pub username: String,
    // TODO: add mask for password
    pub password: Option<String>,
}
