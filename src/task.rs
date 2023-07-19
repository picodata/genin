mod args;
pub mod cluster;
mod flv;
pub mod inventory;
pub mod serde_genin;
pub mod vars;

use log::info;
use regex::{Captures, RegexBuilder};
use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

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
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}: {}", record.level(), record.args()))
        .init();

    info!(
        "Log level {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into())
    );

    // Cluster init comments

    let comments = [
        (
            "topology".into(),
            "# List of replicasets as an array".into(),
        ),
        (
            "replicasets_count".to_string(),
            "# How many masters we want, by default equal 1".to_string(),
        ),
        ("roles".into(), "# Array of roles for this instance".into()),
        (
            "replication_factor".to_string(),
            "# Number of replicas in replicaset, default 0".to_string(),
        ),
        (
            "address".to_string(),
            "# Host or instance address (maybe IP or URI)".to_string(),
        ),
        (
            "http_port".to_string(),
            "# Specify http port to start counting from".to_string(),
        ),
        (
            "binary_port".to_string(),
            "# Specify binary port to start counting from".to_string(),
        ),
        (
            "weight".to_string(),
            "# Vshard replicaset weight (matters only if `vshard-storage` role is enabled)"
                .to_string(),
        ),
        (
            "all_rw".to_string(),
            "# A flag indicating that all servers in the replicaset should be read-write"
                .to_string(),
        ),
        (
            "zone".to_string(),
            "# Zone parameter for ansible cartridge playbook".to_string(),
        ),
        (
            "hosts".into(),
            "# List of regions, datacenters, and servers".into(),
        ),
        (
            "config".to_string(),
            "# Config with arbitrary key-values pairs".to_string(),
        ),
        (
            "vshard_group".to_string(),
            "# Vshard group for vshard-storage".to_string(),
        ),
        (
            "additional_config".to_string(),
            "# Additional parameters to be added to the host config".to_string(),
        ),
        (
            "cartridge_extra_env".to_string(),
            "# Environment variables for instance service (systemd service)".to_string(),
        ),
        (
            "vars".to_string(),
            "# Ansible vars to be added to hosts".to_string(),
        ),
        ("failover".into(), "# Failover management options".into()),
        (
            "mode".to_string(),
            "# Failover mode (stateful, eventual, disabled)".to_string(),
        ),
        (
            "state_provider".to_string(),
            "# What is serve failover (stateboard, stateful)".to_string(),
        ),
        (
            "stateboard_params".to_string(),
            "# Params for chosen in state_provider failover type".to_string(),
        ),
        (
            "uri".to_string(),
            "# Uri on which the stateboard will be available".to_string(),
        ),
        ("password".to_string(), "# Stateboard password".to_string()),
        (
            "vars".into(),
            "# Vars similar to those configured in the cartridge inventory".into(),
        ),
        (
            "ansible_user".to_string(),
            "# Username under which the ansible will connect to the servers".to_string(),
        ),
        (
            "ansible_password".to_string(),
            "# Ansible user password".to_string(),
        ),
        ("cartridge_app_name".into(), "# Application name".into()),
        (
            "cartridge_cluster_cookie".to_string(),
            "# Cookie for connecting to the administrative console of the instances".to_string(),
        ),
        (
            "cartridge_package_path".to_string(),
            "# Path to the application package".to_string(),
        ),
        (
            "cartridge_bootstrap_vshard".into(),
            "# Indicates if vshard must be bootstrapped on the cluster".into(),
        ),
    ]
    .into_iter()
    .collect::<HashMap<String, String>>();

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
            IO::from(args)
                // TODO: make better PathBuf
                .try_into_files(None, Some(CLUSTER_YAML), args.get_flag("force"))?
                .try_map(|IO { output, .. }| {
                    Cluster::try_from(args).map(|cluster| IO {
                        input: Some(cluster),
                        output,
                    })
                })?
                .print_input(args.get_flag("quiet"))
                .try_map(|io| {
                    if let IO {
                        input: Some(cluster),
                        output: Some(mut file),
                    } = io
                    {
                        let mut text = serde_yaml::to_string(&cluster)
                            .map_err(|err| GeninError::new(GeninErrorKind::Deserialization, err))?;

                        for (key, value) in comments {
                            let comment = RegexBuilder::new(&format!(
                                "^(?P<spaces>[ /t-]*)(?P<key>{key}:( .*)*)"
                            ))
                            .multi_line(true)
                            .build()
                            .unwrap();
                            text = comment
                                .replace_all(&text, |caps: &Captures| {
                                    //println!("caps 0: {}", &caps[0]);
                                    //println!("caps 0: {}", &caps[0]);
                                    format!(
                                        "{whitespaces}{value}\n{any_symbols}{key}",
                                        whitespaces = caps[1].replace('-', " "),
                                        any_symbols = &caps[1],
                                        key = &caps[2],
                                    )
                                })
                                //.replace_all(&text, &format!("$key {value}"))
                                .to_string();
                        }
                        file.write(text.as_bytes())
                            .map_err(|err| GeninError::new(GeninErrorKind::Deserialization, err))?;

                        return Ok(IO {
                            input: Some(()),
                            output: Some(()),
                        });
                    }
                    Err(GeninError::new(GeninErrorKind::EmptyField, "TODO"))
                })?;
        }
        Some(("build", args)) => {
            IO::from(args)
                .try_into_files(
                    Some(CLUSTER_YAML),
                    Some(INVENTORY_YAML),
                    args.get_flag("force"),
                )?
                .deserialize_input::<Cluster>()?
                .print_input(args.get_flag("quiet"))
                .try_map(|IO { mut input, output }| {
                    if args.get_flag("fd-as-zone") {
                        input
                            .iter_mut()
                            .for_each(Cluster::use_failure_domain_as_zone_for_instances);
                    }
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
            IO {
                input: args
                    .try_get_one::<String>("old")
                    .transpose()
                    .and_then(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str())))),
                output: args
                    .try_get_one::<String>("output")
                    .transpose()
                    .and_then(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str())))),
            }
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
            .print_input(args.get_flag("quiet"))
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
                        "Erroneous field".as_error(),
                    ))
                }
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod test;
