use std::fmt::Display;
use std::error::Error as StdError;

#[derive(Debug, PartialEq, Eq)]
pub enum GeninErrorKind {
    ArgsError,
    EmptyField,
    SpreadingError,
    Deserialization,
    Serialization,
    UnknownFailureDomain,
    NotApplicable,
    IO,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GeninError {
    err_kind: GeninErrorKind,
    err: String,
}

impl Display for GeninError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.err_kind, self.err)
    }
}

impl StdError for GeninError {}

impl GeninError {
    pub fn new<T: Display>(err_kind: GeninErrorKind, err: T) -> Self {
        Self {
            err_kind,
            err: err.to_string(),
        }
    }
}

