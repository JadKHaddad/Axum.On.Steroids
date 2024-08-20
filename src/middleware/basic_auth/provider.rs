use std::future::Future;

pub trait BasicAuthProvider {
    fn authenticate(&self, username: &str, passowrd: Option<&str>) -> impl Future<Output = bool>;
}

#[derive(Debug, Clone)]
pub struct DummyAuthProvider;

impl BasicAuthProvider for DummyAuthProvider {
    async fn authenticate(&self, _username: &str, _passowrd: Option<&str>) -> bool {
        true
    }
}
