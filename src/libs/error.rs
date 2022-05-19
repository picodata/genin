use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum TaskError {
    InternalError(InternalError),
    ConfigError(ConfigError),
    CommandLineError(CommandLineError),
    UndefinedError,
}

impl Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalError(err) => write!(f, "{}", err),
            Self::ConfigError(err) => write!(f, "{}", err),
            Self::CommandLineError(err) => write!(f, "{}", err),
            Self::UndefinedError => write!(f, "UndefinedError"),
        }
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InternalError(err) => Some(err),
            Self::ConfigError(err) => Some(err),
            Self::CommandLineError(err) => Some(err),
            Self::UndefinedError => None,
        }
    }
}

impl From<InternalError> for TaskError {
    fn from(err: InternalError) -> Self {
        TaskError::InternalError(err)
    }
}

impl From<ConfigError> for TaskError {
    fn from(err: ConfigError) -> Self {
        TaskError::ConfigError(err)
    }
}

impl From<CommandLineError> for TaskError {
    fn from(err: CommandLineError) -> Self {
        TaskError::CommandLineError(err)
    }
}

#[derive(Debug, PartialEq)]
pub enum InternalError {
    InstancesSpreadingError,
    Undefined(String),
}

impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InstancesSpreadingError => write!(f, "InstancesSpreadingError"),
            Self::Undefined(s) => write!(f, "error: {}", s),
        }
    }
}

impl Error for InternalError {}

#[derive(Debug, PartialEq)]
pub enum ConfigError {
    FileNotFoundError(String),
    FileFormatError(String),
    FileContentError(String),
    FileCreationError(String),
    UndefinedError(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFoundError(s) => write!(f, "FileNotFoundError {}", s),
            Self::FileFormatError(s) => write!(f, "FileFormatError {}", s),
            Self::FileContentError(s) => write!(f, "FileContentError {}", s),
            Self::FileCreationError(s) => write!(f, "FileCreationError: {}", s),
            Self::UndefinedError(s) => write!(f, "UndefinedError {}", s),
        }
    }
}

impl Error for ConfigError {}

#[derive(Debug, PartialEq)]
pub enum CommandLineError {
    UnknownSubcommandError(String),
    SubcommandMissingError(String),
}

impl Display for CommandLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownSubcommandError(s) => write!(f, "UnknownSubcommandError: {}", s),
            Self::SubcommandMissingError(s) => write!(f, "SubcommandMissingError: {}", s),
        }
    }
}

impl Error for CommandLineError {}
