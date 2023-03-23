mod args;
pub mod cluster;
mod flv;
pub mod inventory;
pub mod serde_genin;
pub mod vars;

use log::info;
use serde_yaml::{Mapping, Value};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::fs::{TryMap, IO, UPGRADE_YAML};
use crate::task::{
    cluster::fs::{CLUSTER_YAML, INVENTORY_YAML},
    cluster::Cluster,
    inventory::Inventory,
};

const BOOL: &str = "Bool";
const NUMBER: &str = "Number";
const STRING: &str = "String";
const LIST: &str = "List";
const DICT: &str = "Dict";

/// А function that launches an application and walks it through the state stages.
pub fn run_v2() -> Result<(), Box<dyn Error>> {
    // At first set logging level
    // -v       info
    // -vv      debug
    // -vvv     trace
    let args = args::read();
    std::env::set_var(
        "RUST_LOG",
        match args.get_count("verbosity") {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        },
    );
    env_logger::init();

    info!(
        "Log level {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into())
    );

    // The idea of the first step of creating a task:
    //      - create FsInteration
    //      - map FsInteration as:
    //          - read source from disk
    //          - [map] source deserialized to Data or default Data created (data type depends of
    //          subcomand)
    //          - [map] map data to scheme created from data
    //          - [map] move scheme and data into two closures and return them with fs
    //      - return tupple
    match args.subcommand() {
        Some(("init", args)) => {
            IO::from(args)
                // TODO: make better PathBuf
                .try_into_files(None, Some(CLUSTER_YAML), args.get_flag("force"))?
                .try_map(|IO { output, .. }| {
                    Cluster::try_from(args).map(|cluster| IO {
                        input: Some(cluster),
                        output,
                    })
                })?
                .print_input()
                .serialize_input()?;
        }
        Some(("build", args)) => {
            IO::from(args)
                .try_into_files(
                    Some(CLUSTER_YAML),
                    Some(INVENTORY_YAML),
                    args.get_flag("force"),
                )?
                .deserialize_input::<Cluster>()?
                .print_input()
                .try_map(|IO { input, output }| {
                    Inventory::try_from(&input).map(|inventory| IO {
                        input: Some(inventory),
                        output,
                    })
                })?
                .serialize_input()?;
        }
        Some(("inspect", args)) => {
            IO::from(args)
                .try_into_files(Some(CLUSTER_YAML), None, args.get_flag("force"))?
                .deserialize_input::<Cluster>()?
                .print_input()
                .consume_output();
        }
        Some(("reverse", args)) => {
            IO::from(args)
                .try_into_files(Some(INVENTORY_YAML), None, args.get_flag("force"))?
                .deserialize_input::<Inventory>()?
                .try_map(|IO { input, output }| {
                    Cluster::try_from(&input).map(|cluster| IO {
                        input: Some(cluster),
                        output,
                    })
                })?
                .print_input()
                .serialize_input()?;
        }
        Some(("upgrade", args)) => {
            IO::from(args)
                .try_into_files(
                    Some(CLUSTER_YAML),
                    Some(INVENTORY_YAML),
                    args.get_flag("force"),
                )?
                .deserialize_input::<Cluster>()?
                .try_map(|IO { input, output }| {
                    // 1. read source cluster yaml file what should be upgraded
                    // 2. read cluster yaml which should contains information about upgrade
                    File::open(
                        args.get_one::<String>("new")
                            .unwrap_or(&UPGRADE_YAML.to_string()),
                    )
                    .map_err(|err| GeninError::new(GeninErrorKind::IO, err))
                    .and_then(|mut file| {
                        let mut buffer = Vec::new();
                        file.read_to_end(&mut buffer)
                            .map_err(|err| GeninError::new(GeninErrorKind::IO, err))?;
                        Ok(buffer)
                    })
                    .and_then(|buffer| {
                        serde_yaml::from_slice::<Cluster>(&buffer)
                            .map_err(|err| GeninError::new(GeninErrorKind::Deserialization, err))
                    })
                    .and_then(|new| {
                        input
                            .ok_or_else(|| {
                                GeninError::new(GeninErrorKind::EmptyField, "input file is empty")
                            })
                            .and_then(|input_cluster| input_cluster.try_upgrade(&new))
                    })
                    .map(|upgraded| IO {
                        input: Some(upgraded),
                        output,
                    })
                })?
                .print_input()
                .try_map(|IO { input, output }| {
                    Inventory::try_from(&input).map(|inventory| IO {
                        input: Some(inventory),
                        output,
                    })
                })?
                .serialize_input()?;
        }
        _ => {
            return Err(GeninError::new(GeninErrorKind::ArgsError, "subcommand missing").into());
        }
    }

    Ok(())
}

pub trait Validate {
    type Type: fmt::Debug + Default + 'static;
    type Error: fmt::Debug + ToString;

    fn validate(bytes: &[u8]) -> Result<Self::Type, Self::Error>;

    fn whole_block(bytes: &[u8]) -> String;
}

trait AsError {
    fn as_error(&self) -> String;
}

impl<T: std::fmt::Debug> AsError for T {
    fn as_error(&self) -> String {
        format!("\u{1b}[93m\u{1b}4{:?}\u{1b}[0m", self)
    }
}

trait TypeError {
    fn type_error(&self, expected: &str) -> String;
}

impl TypeError for Value {
    fn type_error(&self, expected: &str) -> String {
        match self {
            Value::Null => format!("Expected type {} got Null", expected),
            Value::Bool(_) => format!("Expected type {} got Bool", expected),
            Value::Number(_) => format!("Expected type {} got Number", expected),
            Value::String(_) => format!("Expected type {} got String", expected),
            Value::Sequence(_) => format!("Expected type {} got List", expected),
            Value::Mapping(_) => format!("Expected type {} got Dict", expected),
        }
    }
}

pub struct ErrSeqMapping<'a> {
    pub offset: String,
    pub value: &'a Vec<Value>,
}

impl<'a> std::fmt::Debug for ErrSeqMapping<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("\n")?;
        self.value.iter().try_for_each(|value| match value {
            Value::String(value) => {
                formatter.write_fmt(format_args!("{}- {}", &self.offset, value))?;
                formatter.write_str("\n")
            }
            Value::Mapping(value) => {
                formatter.write_fmt(format_args!(
                    "{}- {:?}",
                    &self.offset,
                    ErrConfMapping {
                        offset: self.offset.clone(),
                        value,
                    }
                ))?;
                formatter.write_str("\n")
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}- {}",
                    &self.offset,
                    value.type_error(DICT).as_error()
                ))?;
                formatter.write_str("\n")
            }
        })?;

        Ok(())
    }
}

pub struct ErrConfMapping<'a> {
    pub offset: String,
    pub value: &'a Mapping,
}

impl<'a> std::fmt::Debug for ErrConfMapping<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value
            .iter()
            .try_for_each(|(key, value)| match (key, value) {
                (Value::String(key), Value::String(value)) => {
                    formatter.write_str("\n")?;
                    formatter.write_fmt(format_args!("{}  {}: {}", &self.offset, key, value))
                }
                (Value::String(key), Value::Sequence(value)) => {
                    formatter.write_str("\n")?;
                    formatter.write_fmt(format_args!(
                        "{}  {}: {:?}",
                        &self.offset,
                        key,
                        ErrSeqMapping {
                            offset: format!("{}  ", &self.offset),
                            value
                        }
                    ))
                }
                (Value::String(key), Value::Mapping(value)) => {
                    formatter.write_str("\n")?;
                    formatter.write_fmt(format_args!(
                        "{}  {}: {:?}",
                        &self.offset,
                        key,
                        ErrConfMapping {
                            offset: format!("{}  ", &self.offset),
                            value,
                        }
                    ))
                }
                (Value::String(key), value) => {
                    formatter.write_str("\n")?;
                    formatter.write_fmt(format_args!(
                        "{}  - {}: {}",
                        &self.offset,
                        key,
                        value.type_error(DICT).as_error()
                    ))
                }
                _ => {
                    formatter.write_str("\n")?;
                    formatter.write_fmt(format_args!(
                        "{}  - {}",
                        &self.offset,
                        "Errorneous field".as_error(),
                    ))
                }
            })?;

        Ok(())
    }
}
