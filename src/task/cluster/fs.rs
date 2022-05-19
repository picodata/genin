use std::{
    fs::File,
    io::{Read, Write},
    panic::{catch_unwind, set_hook, take_hook},
    path::PathBuf,
};

use clap::ArgMatches;
use genin::libs::error::{ConfigError, InternalError, TaskError};

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
            source: get_path(args, "source"),
            output: get_path(args, "output"),
        }
    }
}

impl FsInteraction {
    /// After string args transofrmed to `PathBuf` this function should check
    /// `soure` and `output` existence, and replace to default value
    pub fn check(self, source: Option<&str>, output: Option<&str>) -> Self {
        Self {
            source: self.source.or_else(|| source.map(PathBuf::from)),
            output: self.output.or_else(|| output.map(PathBuf::from)),
        }
    }

    /// Reading source file
    ///
    /// # panic
    /// - source file does not exist
    /// - source file wrong format
    pub fn read(&self) -> Result<Vec<u8>, TaskError> {
        let mut file = File::open(self.source.as_ref().ok_or_else(|| {
            TaskError::InternalError(InternalError::Undefined(
                "Error while trying to read source file. Source file: None".into(),
            ))
        })?)
        .map_err(|e| {
            TaskError::ConfigError(ConfigError::FileContentError(format!(
                "Error then opening file {}! Err: {}",
                self.source.as_ref().unwrap().to_str().unwrap(),
                e
            )))
        })?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| {
            TaskError::ConfigError(ConfigError::FileContentError(format!(
                "Error then opening file {}! Err: {}",
                self.source.as_ref().unwrap().to_str().unwrap(),
                e
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
            f.write_all(data).map_err(|e| {
                TaskError::ConfigError(ConfigError::FileContentError(format!(
                    "Error then writing file {}",
                    e
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

/// After release clap 3.0 `ArgMatches` always panics if arg with `id` does not exists.
/// I think this is strange behaviour. This function should solve it.
fn get_path(args: &ArgMatches, id: &str) -> Option<PathBuf> {
    let hook = take_hook();
    set_hook(Box::new(|_| {}));
    let present = catch_unwind(|| {
        args.is_present(id)
            .then(|| args.value_of(id).map(PathBuf::from))
            .flatten()
    })
    .unwrap_or_default();
    set_hook(hook);
    present
}

#[cfg(test)]
mod test;
