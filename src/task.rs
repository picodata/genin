mod args;
pub mod cluster;
mod flv;
pub mod inventory;
pub mod serde_genin;
pub mod state;
pub mod utils;
pub mod vars;

use log::info;
use serde_yaml::{Mapping, Value};
use std::convert::TryFrom;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use std::{fmt, io};

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::fs::{TryMap, IO};
use crate::task::cluster::ClusterError;
use crate::task::state::State;
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
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();

    info!(
        "Log level {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into())
    );

    // The idea of the first step of creating a task:
    //      - create FsInteraction
    //      - map FsInteraction as:
    //          - read source from disk
    //          - [map] source deserialized to Data or default Data created (data type depends of
    //          subcommand)
    //          - [map] map data to scheme created from data
    //          - [map] move scheme and data into two closures and return them with fs
    //      - return tuple
    match args.subcommand() {
        Some(("init", args)) => {
            Cluster::try_from(args)?
                .print(args)
                .clear_instances()
                .write(args)?;
        }
        Some(("build", args)) => {
            Cluster::try_from(args)?
                .use_failure_domain_as_zone_for_instances(args)
                .print(args)
                .write_build_state(args)?
                .to_inventory()?
                .write(args)?;
        }
        Some(("inspect", args)) => {
            IO::from(args)
                .try_into_files(Some(CLUSTER_YAML), None, args.get_flag("force"))?
                .deserialize_input::<Cluster>()?
                .print_input(true)
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
                .print_input(args.get_flag("quiet"))
                .serialize_input()?;
        }
        Some(("upgrade", args)) => {
            let mut old: Cluster = if args.get_flag("from-latest-state") {
                State::from_latest(args)?.into()
            } else {
                match args.get_one::<String>("old") {
                    Some(old_path) if old_path.ends_with(".tgz") => {
                        State::try_from(&PathBuf::from(old_path))?.into()
                    }
                    Some(old_path) => Cluster::try_from(&PathBuf::from(old_path))?,
                    None => Cluster::try_from(&PathBuf::from("cluster.genin.yml"))?,
                }
            };

            old.hosts.clear_view();

            let mut new = if let Some(new) = args.get_one::<String>("new") {
                Cluster::try_from(&PathBuf::from(new))?
            } else {
                return Err(ClusterError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    "missing --new option",
                ))
                .into());
            };

            let hosts_diff = old.merge(&mut new)?;

            old.print(args)
                .write_upgrade_state(args, hosts_diff)?
                .to_inventory()?
                .write(args)?;
        }
        Some(("list-state", args)) => {
            let path = match args.get_one::<String>("state-dir") {
                Some(dir) => PathBuf::from(dir),
                None => PathBuf::from(".geninstate"),
            };

            let dirs = match path.read_dir() {
                Ok(dirs) => dirs,
                Err(err) if err.kind() == io::ErrorKind::NotFound => {
                    panic!("directory containing genin state not found or empty");
                }
                Err(e) => {
                    panic!("{}", e);
                }
            };

            let mut entries = dirs
                .into_iter()
                .filter_map(
                    |entry| match entry.as_ref().map(|entry| entry.file_type()) {
                        Ok(Ok(file_type)) if file_type.is_file() => entry.ok(),
                        _ => None,
                    },
                )
                .collect::<Vec<std::fs::DirEntry>>();

            entries.sort_by_key(|entry| entry.metadata().unwrap().modified().unwrap());
            entries.reverse();

            for (id, entry) in entries
                .into_iter()
                .take(
                    args.get_one::<usize>("number")
                        .map(|num| num + 1)
                        .unwrap_or(11),
                )
                .enumerate()
            {
                if id != 1 {
                    let state = State::try_from(&entry.path())?;
                    state.print_kind();
                    state.print_changes();
                }
            }
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
        format!("\u{1b}[31m\u{1b}4{:?}\u{1b}[0m", self)
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
                        "Erroneous field".as_error(),
                    ))
                }
            })?;

        Ok(())
    }
}
