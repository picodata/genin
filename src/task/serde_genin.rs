use std::error;
use std::fmt;
use std::fmt::Display;

use serde::de::DeserializeOwned;

use crate::task;

pub struct Error<T: fmt::Debug> {
    message: String,
    inner: T,
}

impl<T: task::Validate + fmt::Debug + Default> From<serde_yaml::Error> for Error<T> {
    fn from(err: serde_yaml::Error) -> Self {
        Self {
            message: err.to_string(),
            inner: T::default(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Error<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}({:?})", &self.message, &self.inner)
    }
}

impl<T: fmt::Debug> Display for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({:?})", &self.message, &self.inner)
    }
}

impl<T: fmt::Debug> error::Error for Error<T> {}

struct MallformedContent(String);

impl fmt::Debug for MallformedContent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[allow(unused)]
pub fn from_slice<T>(slice: &[u8]) -> Result<T, Error<impl fmt::Debug>>
where
    T: DeserializeOwned + fmt::Debug + task::Validate,
{
    serde_yaml::from_slice::<T>(slice).map_err(|err| Error {
        message: err.to_string(),
        inner: T::validate(slice).map_err(|err| MallformedContent(err.to_string())),
    })
}
