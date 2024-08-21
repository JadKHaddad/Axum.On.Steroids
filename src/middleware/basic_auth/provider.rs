use std::future::Future;

pub trait BasicAuthProvider {
    fn authenticate(
        &self,
        username: &str,
        passowrd: Option<&str>,
    ) -> impl Future<Output = bool> + Send;
}

#[derive(Debug, Clone)]
pub struct DummyAuthProvider;

impl BasicAuthProvider for DummyAuthProvider {
    async fn authenticate(&self, username: &str, _passowrd: Option<&str>) -> bool {
        username == "admin"
    }
}
