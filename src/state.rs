use crate::error::ErrorVerbosity;

pub trait ApiState {
    fn error_verbosity(&self) -> ErrorVerbosity;
}
