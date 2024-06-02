use derivative::Derivative;

/// A struct to hold the used basic auth.
#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct UsedBasicAuth {
    pub username: String,
    #[derivative(Debug(format_with = "crate::utils::mask_fmt"))]
    pub password: Option<String>,
}
