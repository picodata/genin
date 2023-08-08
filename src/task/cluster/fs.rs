use std::{fmt::Display, fs::File, io::Read, path::PathBuf};

use clap::ArgMatches;

use crate::error::{GeninError, GeninErrorKind};

//TODO: remove it in next commits
#[allow(unused)]
#[derive(Default)]
pub(in crate::task) struct FsInteraction {
    source: Option<PathBuf>,
    output: Option<PathBuf>,
}

impl<'a> From<&'a ArgMatches> for FsInteraction {
    fn from(args: &'a ArgMatches) -> Self {
        FsInteraction {
            source: args
                .try_get_one::<&str>("source")
                .unwrap_or_default()
                .map(PathBuf::from),
            output: args
                .try_get_one::<&str>("output")
                .unwrap_or_default()
                .map(PathBuf::from),
        }
    }
}

//TODO: remove in future commits
#[allow(unused)]
impl FsInteraction {
    /// Reading source file
    ///
    /// # panic
    /// - source file does not exist
    /// - source file wrong format
    pub fn read(&self) -> Result<Vec<u8>, GeninError> {
        let mut file = File::open(self.source.as_ref().ok_or_else(|| {
            GeninError::new(
                GeninErrorKind::ArgsError,
                "Error while trying to read source file. Source file: None",
            )
        })?)
        .map_err(|error| {
            GeninError::new(
                GeninErrorKind::ArgsError,
                format!(
                    "Error then opening file {}! Err: {}",
                    self.source.as_ref().unwrap().to_str().unwrap(),
                    error
                )
                .as_str(),
            )
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|error| {
            GeninError::new(
                GeninErrorKind::ArgsError,
                format!(
                    "Error then opening file {}! Err: {}",
                    self.source.as_ref().unwrap().to_str().unwrap(),
                    error
                )
                .as_str(),
            )
        })?;
        Ok(buffer)
    }
}

pub trait TryIntoFile {
    type Error;

    fn try_into_file(self) -> Result<Option<File>, Self::Error>;
}

impl TryIntoFile for Option<PathBuf> {
    type Error = std::io::Error;

    fn try_into_file(self) -> Result<Option<File>, Self::Error> {
        if let Some(path) = self {
            return File::open(path).map(Some);
        }

        Ok(None)
    }
}

pub struct IO<I, O> {
    pub input: Option<I>,
    pub output: Option<O>,
}

#[allow(unused)]
impl IO<(), ()> {
    pub fn new() -> Self {
        Self {
            input: None,
            output: None,
        }
    }
}

impl Default for IO<(), ()> {
    fn default() -> Self {
        Self {
            input: Default::default(),
            output: Default::default(),
        }
    }
}

impl<'a> From<&'a ArgMatches> for IO<PathBuf, PathBuf> {
    fn from(args: &'a ArgMatches) -> Self {
        Self {
            input: args
                .try_get_one::<String>("source")
                .transpose()
                .and_then(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str())))),
            output: args
                .try_get_one::<String>("output")
                .transpose()
                .and_then(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str())))),
        }
    }
}

pub trait TryMap<A, B> {
    type Error;
    type Output;

    fn try_map<F>(self, f: F) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Result<Self::Output, Self::Error>;
}

impl<I, O> Display for IO<I, O>
where
    I: Display,
    O: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IO {
                input: Some(input),
                output: Some(output),
            } => write!(f, "{}{}", input, output),
            IO {
                input: Some(input),
                output: None,
            } => write!(f, "{}", input),
            IO {
                input: None,
                output: Some(output),
            } => write!(f, "{}", output),
            _ => write!(f, ""), //TODO
        }
    }
}
