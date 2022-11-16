use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::ArgMatches;
use log::{debug, trace, warn};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{GeninError, GeninErrorKind};

pub const CLUSTER_YAML: &str = "cluster.genin.yaml";
pub const INVENTORY_YAML: &str = "inventory.yaml";
pub const UPGRADE_YAML: &str = "upgrade.genin.yaml";

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

//TODO: remove in future commits
#[allow(unused)]
impl FsInteraction {
    /// After string args transofrmed to `PathBuf` this function should check
    /// `soure` and `output` existence, and replace to default value
    pub fn check(self, source: Option<&str>, output: Option<&str>, force: bool) -> Self {
        Self {
            source: self
                .source
                .or_else(|| source.map(PathBuf::from))
                .and_then(|path| {
                    path.clone()
                        .to_str()
                        .map(|path_str| (path, path_str.to_string()))
                })
                .and_then(|(path, path_str)| right_ext(path, path_str, false)),
            output: self
                .output
                .or_else(|| output.map(PathBuf::from))
                .and_then(|path| {
                    path.clone()
                        .to_str()
                        .map(|path_str| (path, path_str.to_string()))
                })
                .map(|(path, path_str)| {
                    force
                        .then(|| {
                            debug!("output file will be overrided because flag force defined");
                            path.clone()
                        })
                        .unwrap_or_else(|| as_copy(path, path_str))
                }),
        }
    }

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

#[inline]
/// Check that file with current extension exists and replace path if it
/// exists with related extension.
fn right_ext(path: PathBuf, path_str: String, second_try: bool) -> Option<PathBuf> {
    match (
        path.is_file(),
        path.extension().and_then(|e| e.to_str()),
        second_try,
    ) {
        (false, Some("yml"), false) => {
            let new_path_str = path_str.replace(".yml", ".yaml");
            trace!("file_exists: false, extension: .yml, second_try: false");
            warn!(
                "file {} does not exists, trying to open {}",
                path_str, &new_path_str
            );
            right_ext(PathBuf::from(&new_path_str), new_path_str, true)
        }
        (false, Some("yaml"), false) => {
            let new_path_str = path_str.replace(".yaml", ".yml");
            trace!("file_exists: false, extension: .yaml, second_try: false");
            warn!(
                "file {} does not exists, trying to open {}",
                path_str, &new_path_str
            );
            right_ext(PathBuf::from(&new_path_str), new_path_str, true)
        }
        (false, _, true) => None,
        _ => Some(path),
    }
}

#[inline]
/// Check that target file not exists and return concatenated path with copy suffix
fn as_copy(path: PathBuf, path_str: String) -> PathBuf {
    match (path.is_file(), path.extension()) {
        (true, Some(e)) => {
            let ext = format!(".{}", e.to_str().unwrap_or_default());
            let new_path_str = format!("{}.copy{}", path_str.replace(&ext, ""), ext);
            warn!(
                "the target file {} already exists so \
                the new file will be saved with the name {}",
                path_str, new_path_str
            );
            as_copy(PathBuf::from(&path_str), new_path_str)
        }
        _ => path,
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

impl IO<PathBuf, PathBuf> {
    pub fn try_into_files(
        self,
        source: Option<&str>,
        output: Option<&str>,
        force: bool,
    ) -> Result<IO<File, File>, Box<dyn Error>> {
        Ok(IO {
            input: self
                .input
                .or_else(|| source.map(PathBuf::from))
                .and_then(|path| {
                    path.clone()
                        .to_str()
                        .map(|path_str| (path, path_str.to_string()))
                })
                .and_then(|(path, path_str)| right_ext(path, path_str, false))
                .map(File::open)
                .transpose()?,
            output: self
                .output
                .or_else(|| output.map(PathBuf::from))
                .and_then(|path| {
                    path.clone()
                        .to_str()
                        .map(|path_str| (path, path_str.to_string()))
                })
                .map(|(path, path_str)| {
                    force
                        .then(|| {
                            debug!("output file will be overrided because flag force defined");
                            path.clone()
                        })
                        .unwrap_or_else(|| as_copy(path, path_str))
                })
                .map(File::create)
                .transpose()?,
        })
    }
}

impl<I: Read, O> IO<I, O> {
    pub fn deserialize_input<T: DeserializeOwned>(self) -> Result<IO<T, O>, Box<dyn Error>> {
        Ok(IO {
            input: Some(serde_yaml::from_reader(self.input.ok_or_else(|| {
                GeninError::new(
                    GeninErrorKind::EmptyField,
                    "IO struct has empty input field",
                )
            })?)?),
            output: self.output,
        })
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

impl<I, O, A, B> TryMap<A, B> for IO<I, O> {
    type Error = GeninError;
    type Output = IO<A, B>;

    fn try_map<F>(self, function: F) -> Result<Self::Output, Self::Error>
    where
        Self: Sized,
        F: FnOnce(Self) -> Result<Self::Output, Self::Error>,
    {
        function(self)
    }
}

impl<I: Display, O> IO<I, O> {
    pub fn print_input(self) -> Self {
        if let Some(input) = self.input.as_ref() {
            println!("{}", input)
        }
        self
    }
}

impl<I: Serialize, O: Write> IO<I, O> {
    pub fn serialize_input(self) -> Result<IO<I, ()>, Box<dyn Error>> {
        if let IO {
            input,
            output: Some(mut writer),
        } = self
        {
            serde_yaml::to_writer(&mut writer, &input)?;
            Ok(IO {
                input,
                output: Some(()),
            })
        } else {
            Err(GeninError::new(
                GeninErrorKind::Serialization,
                "failed to serialize input because output file is None",
            )
            .into())
        }
    }
}

impl<I, O> IO<I, O> {
    pub fn consume_output(self) -> IO<I, String> {
        IO {
            input: self.input,
            output: Option::<String>::None,
        }
    }
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

//#[cfg(test)]
//mod test;
