use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use crate::error::{ConfigError, InternalError, TaskError};
use clap::ArgMatches;
use log::{debug, trace, warn};

use crate::task::MapSelf;

pub(in crate::task) const CLUSTER_YAML: &str = "cluster.genin.yaml";
pub(in crate::task) const INVENTORY_YAML: &str = "inventory.yaml";

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
                .map(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str()))))
                .flatten(),
            output: args
                .try_get_one::<String>("output")
                .transpose()
                .map(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str()))))
                .flatten(),
        }
    }
}

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
    pub fn read(&self) -> Result<Vec<u8>, TaskError> {
        let mut file = File::open(self.source.as_ref().ok_or_else(|| {
            TaskError::InternalError(InternalError::UndefinedError(
                "Error while trying to read source file. Source file: None".into(),
            ))
        })?)
        .map_err(|err| {
            TaskError::ConfigError(ConfigError::FileContentError(format!(
                "Error then opening file {}! Err: {}",
                self.source.as_ref().unwrap().to_str().unwrap(),
                err
            )))
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|err| {
            TaskError::ConfigError(ConfigError::FileContentError(format!(
                "Error then opening file {}! Err: {}",
                self.source.as_ref().unwrap().to_str().unwrap(),
                err
            )))
        })?;
        Ok(buffer)
    }

    /// Write file on the disk
    pub fn write(&self, data: &[u8]) -> Result<(), TaskError> {
        File::create(self.output.as_ref().ok_or_else(|| {
            TaskError::ConfigError(ConfigError::FileCreationError(format!(
                "Can not create file {}! FsInteraction.taget is none!",
                self.source.as_ref().unwrap().to_str().unwrap(),
            )))
        })?)
        .map_err(|e| {
            TaskError::ConfigError(ConfigError::FileCreationError(format!(
                "Error then creating file {}",
                e
            )))
        })
        .and_then(|mut f| {
            f.write_all(data).map_err(|err| {
                TaskError::ConfigError(ConfigError::FileContentError(format!(
                    "Error then writing file {}",
                    err
                )))
            })
        })
    }
}

impl<T> MapSelf<T> for FsInteraction {
    type Target = T;
    type Error = TaskError;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>,
    {
        func(self)
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

#[cfg(test)]
mod test;
