pub mod fs;
pub mod host;
pub mod instance;
pub mod name;
pub mod topology;

use clap::ArgMatches;
use indexmap::IndexMap;
use log::debug;
use regex::{Captures, RegexBuilder};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::task::cluster::host::hst::{Host, HostConfig, WithHosts};
use crate::task::cluster::host::view::View;
use crate::task::cluster::instance::ins::{Instance, InstanceConfig, Instances};
use crate::task::cluster::instance::Role;
use crate::task::cluster::name::Name;
use crate::task::cluster::topology::{InvalidTopologySet, Topology};
use crate::task::flv::Failover;
use crate::task::inventory::{Child, HostVars, Inventory};
use crate::task::vars::Vars;
use crate::task::AsError;
use crate::task::Validate;
use crate::{DEFAULT_BINARY_PORT, DEFAULT_CFG, DEFAULT_CFG_NAME, DEFAULT_HTTP_PORT};

use self::host::hst::InvalidHost;

use crate::task::flv::{FailoverError, InvalidFailover};
use crate::task::inventory::InventoryError;
use crate::task::state::{State, StateError};
use crate::task::utils::create_file_or_copy;
use crate::task::vars::InvalidVars;

use super::state::Change;
use super::{TypeError, DICT};

/// Cluster is a `genin` specific configuration file
/// ```rust
/// Cluster {
///     // Array of replicasets in free order
///     // topology:
///     // - name: "catalogue"
///     //   type: "storage"
///     //   replicasets_count: 1
///     //   replication_factor: 2
///     //   weight: 10
///     // Array or arrays with hosts parameters
///     // hosts:
///     //     - name: kavkaz
///     //       type: region
///     //       distance: 10
///     //       ports:
///     //         http: 8091
///     //         binary: 3031
///     //       hosts:
///     //         - name: dc-1
///     //           type: datacenter
///     //           hosts:
///     //             - name: server-1
///     //               ip: 10.20.3.100
///     //         - name: dc-2
///     //           type: datacenter
///     //           hosts:
///     //             - name: server-1
///     //               ip: 10.20.4.100
///     //     - name: moscow
///     //       type: region
///     //       distance: 20
///     //       hosts:
///     //         - name: dc-3
///     //           type: datacenter
///     //           ports:
///     //             http: 8091
///     //             binary: 3031
///     //           hosts:
///     //             - name: server-10
///     //               ip: 10.99.3.100
///     hosts: Host, //TODO
///     // Failover coordinator struct.
///     // If cluster should be without failover (`failover_mode: "disabled"`)
///     // this field will be skipped
///     // failover:
///     //     mode: stateful
///     //     state_provider: stateboard
///     //     stateboard_params:
///     //         uri: "10.99.3.100:4001"
///     //         password: "vG?-GG!4sxV8q5:f"
///     failover: Failover,
///     // Ansible cartridge vars in freedom format
///     // vars:
///     //     ansible_user: "admin"
///     //     ansible_password: "'88{bvTp9Gbj<J"m"
///     //     cartridge_bootstrap_vshard: true
///     //     cartridge_app_name: "tarantool-cluster"
///     //     cartridge_cluster_cookie: "tarantool-cluster-cookie"
///     //     wait_cluster_has_no_issues_retries: 20
///     //     instance_start_retries: 20
///     // Although declaring wars does not allow declaring all parameters,
///     // the most important ones will still be added during inventory generation
///     vars: Vars,
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Cluster {
    pub topology: Topology,
    pub hosts: Host,
    pub failover: Failover,
    pub vars: Vars,
    pub metadata: ClusterMetadata,
}

impl Default for Cluster {
    fn default() -> Self {
        let mut cluster = Cluster {
            metadata: ClusterMetadata {
                paths: vec![DEFAULT_CFG_NAME.into()],
            },
            ..serde_yaml::from_reader(DEFAULT_CFG).expect("cluster is yaml format")
        };
        cluster.vars = cluster.vars.with_failover(cluster.failover.clone());
        cluster
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct ClusterMetadata {
    pub paths: Vec<PathBuf>,
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.hosts)
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Cluster {
    type Error = ClusterError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        match args.try_get_one::<String>("source") {
            Ok(path) => {
                let source = String::from(DEFAULT_CFG_NAME);
                let path = path.unwrap_or(&source);

                let file = File::open(path)?;
                let mut cluster = Cluster {
                    metadata: ClusterMetadata {
                        paths: vec![path.into()],
                    },
                    ..serde_yaml::from_reader(file)?
                };

                cluster.vars = cluster.vars.with_failover(cluster.failover.clone());
                Ok(cluster)
            }
            Err(_) => {
                debug!(
                    "Сluster file will be constructed based on \
                    default values and genin call arguments"
                );
                let failover = Failover::try_from(args)?;
                Ok(Cluster {
                    vars: Vars::from(&failover),
                    failover,
                    ..Cluster::default()
                })
            }
        }
    }
}

impl<'a> TryFrom<&'a PathBuf> for Cluster {
    type Error = ClusterError;

    fn try_from(path: &'a PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path)?;
        Ok(Cluster {
            metadata: ClusterMetadata {
                paths: vec![path.into()],
            },
            ..serde_yaml::from_reader(file)?
        })
    }
}

impl<'a> TryFrom<&'a Inventory> for Cluster {
    type Error = ClusterError;

    fn try_from(inventory: &'a Inventory) -> Result<Self, Self::Error> {
        Ok(Cluster {
            topology: Topology::try_from(Instances::from(
                inventory
                    .all
                    .hosts
                    .iter()
                    .filter(|(_, host)| !host.stateboard)
                    .map(|(name, inventory_host)| {
                        let replicaset_name = if name.len() == 2 {
                            name.clone_with_index("replicaset")
                        } else {
                            name.get_parent_name().clone_with_index("replicaset")
                        };
                        let mut instance = Instance::from((name, inventory_host)).with_roles(
                            inventory
                                .all
                                .children
                                .get(&replicaset_name)
                                .map(|replicaset| match replicaset {
                                    Child::Replicaset { vars, .. } => vars.roles.clone(),
                                    _ => unreachable!(),
                                })
                                .ok_or_else(|| {
                                    ClusterError::Other(format!(
                                        "failed to get replicaset with name {}",
                                        &replicaset_name
                                    ))
                                })?,
                        );

                        instance.config.http_port = None;
                        instance.config.binary_port = None;

                        Ok(instance)
                    })
                    .collect::<Result<Vec<Instance>, ClusterError>>()?,
            ))?,
            hosts: Host::from("cluster").with_hosts(
                inventory
                    .all
                    .children
                    .iter()
                    .filter_map(|(name, replicaset)| match replicaset {
                        Child::Host {
                            vars:
                                HostVars {
                                    ansible_host,
                                    additional_config,
                                },
                            ..
                        } => Some(Host {
                            name: name.clone(),
                            config: HostConfig::from(ansible_host.clone())
                                .with_additional_config(additional_config.clone())
                                .with_ansible_host(ansible_host.clone())
                                .with_ports(
                                    inventory
                                        .all
                                        .hosts
                                        .iter()
                                        .filter(|(_, instance)| !instance.stateboard)
                                        .fold((u16::MAX, u16::MAX), |accum, (_, instance)| {
                                            (
                                                accum.0.min(instance.config.http_port()),
                                                accum.1.min(instance.config.binary_port()),
                                            )
                                        }),
                                ),
                            hosts: Vec::new(),
                            add_queue: IndexMap::default(),
                            delete_queue: IndexMap::default(),
                            instances: Instances::from(
                                inventory
                                    .all
                                    .hosts
                                    .iter()
                                    .filter_map(|(name, instance)| {
                                        let config = HostConfig::from(&instance.config);
                                        debug!(
                                            "ansible_host: {} instance_address: {}",
                                            ansible_host,
                                            config.address()
                                        );
                                        if ansible_host.eq(&config.address()) {
                                            Some(Instance {
                                                name: name.clone(),
                                                stateboard: instance.stateboard.then_some(true),
                                                weight: None,
                                                failure_domains: Default::default(),
                                                roles: Vec::new(),
                                                cartridge_extra_env: instance.vars.clone(),
                                                config: InstanceConfig::from_inventory_host(
                                                    &instance,
                                                ),
                                                vars: instance.vars.clone(),
                                                view: View::default(),
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<Instance>>(),
                            ),
                        }),
                        Child::Replicaset { .. } => None,
                    })
                    .collect::<Vec<Host>>(),
            ),
            failover: inventory
                .all
                .vars
                .cartridge_failover_params
                .clone()
                .ok_or_else(|| {
                    ClusterError::Other(
                        "inventory vars does not have cartridge_failover_params field".into(),
                    )
                })?,
            vars: inventory.all.vars.clone(),
            metadata: ClusterMetadata {
                paths: Default::default(),
            },
        })
    }
}

impl From<State> for Cluster {
    fn from(state: State) -> Self {
        let mut hosts = state.hosts;
        hosts.add_queue = hosts
            .collect_instances()
            .into_iter()
            .map(|instance| (instance.name.clone(), instance))
            .collect();
        hosts.delete_queue = hosts.add_queue.clone();
        Cluster {
            hosts,
            vars: state.vars.with_failover(state.failover.clone()),
            failover: state.failover,
            topology: state.topology,
            metadata: ClusterMetadata {
                paths: vec![PathBuf::from(state.path)],
            },
        }
    }
}

impl<'de> Deserialize<'de> for Cluster {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ClusterHelper {
            Cluster {
                topology: Topology,
                hosts: Vec<Host>,
                #[serde(default)]
                failover: Failover,
                vars: Vars,
            },
            InvalidCluster(Value),
        }

        ClusterHelper::deserialize(deserializer).and_then(|cluster| match cluster {
            ClusterHelper::Cluster {
                topology,
                hosts,
                failover,
                vars,
            } => Ok(Cluster {
                hosts: Host::from("cluster")
                    .with_hosts(hosts)
                    .with_http_port(DEFAULT_HTTP_PORT)
                    .with_binary_port(DEFAULT_BINARY_PORT),
                topology: topology.check_unique().map_err(serde::de::Error::custom)?,
                failover,
                vars,
                metadata: ClusterMetadata {
                    paths: Default::default(),
                },
            }
            .spread()),
            ClusterHelper::InvalidCluster(value) => {
                println!(
                    "Cluster configuration contains errors: {:?}",
                    serde_yaml::from_value::<InvalidCluster>(value)
                        .expect("can't fail because it was already parsed into the similiar type")
                );
                Err(serde::de::Error::custom("Invalid cluster configuration"))
            }
        })
    }
}

impl Serialize for Cluster {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Cluster", 4)?;
        state.serialize_field("topology", &self.topology)?;
        state.serialize_field("hosts", &self.hosts.hosts)?;
        state.serialize_field("failover", &self.failover)?;

        let mut vars = self.vars.clone();
        vars.cartridge_failover_params = None;

        state.serialize_field("vars", &vars)?;
        state.end()
    }
}

impl Validate for Cluster {
    type Type = InvalidCluster;
    type Error = serde_yaml::Error;

    fn validate(bytes: &[u8]) -> Result<Self::Type, Self::Error> {
        serde_yaml::from_str(&check_placeholders(bytes)?)
    }
}

pub fn check_placeholders(slice: &[u8]) -> Result<String, serde_yaml::Error> {
    let text = String::from_utf8_lossy(slice).to_string();
    let reg = RegexBuilder::new(r"(?P<key>^.+:) +(<<.*>>) *(?:# *([^#:]+)$)*")
        .multi_line(true)
        .build()
        .map_err(serde::de::Error::custom)?;
    let captures = reg.captures_iter(&text).collect::<Vec<Captures>>();

    if captures.is_empty() {
        return Ok(text);
    }

    let mut result = format!("\n{}", text);

    for c in captures {
        let placeholder = c.get(2).map(|r| r.as_str().to_string()).unwrap_or_default();
        let comment = c
            .get(3)
            .map(|r| r.as_str().to_string())
            .unwrap_or("Please replace or remove!".to_string());

        result = reg
            .replace(
                &result,
                &format!(
                    "$key Err({})",
                    format!(
                        "The placeholder {} was not replaced! {}",
                        placeholder, comment
                    )
                    .as_error()
                ),
            )
            .to_string();
    }

    Err(serde::de::Error::custom(&result))
}

impl Cluster {
    pub fn spread(self) -> Self {
        let instances = Instances::from(&self.topology);
        let mut hosts = self.hosts.with_instances(instances);
        hosts.with_stateboard(&self.failover);
        let mut hosts = hosts
            .clone()
            .with_add_queue(
                hosts
                    .instances
                    .iter()
                    .map(|instance| (instance.name.clone(), instance.clone()))
                    .collect(),
            )
            .with_delete_queue(
                hosts
                    .instances
                    .iter()
                    .map(|instance| (instance.name.clone(), instance.clone()))
                    .collect(),
            );

        hosts.spread();
        Self { hosts, ..self }
    }

    pub fn merge(
        &mut self,
        new: &mut Cluster,
        idiomatic: bool,
    ) -> Result<Vec<Change>, ClusterError> {
        self.hosts.delete_stateboard();

        std::mem::swap(&mut self.failover, &mut new.failover);
        std::mem::swap(&mut self.vars, &mut new.vars);
        std::mem::swap(&mut self.topology, &mut new.topology);

        let hosts_diff = Host::merge(&mut self.hosts, &mut new.hosts, idiomatic);

        debug!(
            "Instances to Add: {}",
            self.hosts
                .add_queue
                .iter()
                .map(|(name, _)| name.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );
        debug!(
            "Instances to Delete: {}",
            self.hosts
                .delete_queue
                .iter()
                .map(|(name, _)| name.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        self.hosts.add_diff();
        self.hosts.with_stateboard(&self.failover);
        self.hosts.spread();

        self.hosts.remove_diff();
        self.metadata.paths.extend_from_slice(&new.metadata.paths);

        Ok(hosts_diff)
    }

    /// It will traverse the cluster, replacing every instance's zone with its `failure_domain`.
    ///
    /// Note that method is intended to be called after cluster is spread
    /// - that's it, when there may only be single domain name in instance's `failure_domains`.
    pub fn use_failure_domain_as_zone_for_instances(mut self, args: &ArgMatches) -> Self {
        if let Ok(Some(lvl)) = args.try_get_one::<u8>("fd-as-zone") {
            self.hosts.use_failure_domain_as_zone(*lvl);
        }
        self
    }

    pub fn print(self, args: &ArgMatches) -> Self {
        if !args.get_flag("quiet") {
            println!("{self}");
        }

        self
    }

    pub fn write(self, args: &ArgMatches) -> Result<(), ClusterError> {
        let path = PathBuf::from(
            args.get_one::<String>("output")
                .cloned()
                .unwrap_or(DEFAULT_CFG_NAME.to_string()),
        );

        let mut file = create_file_or_copy(path, args.get_flag("force"))?;

        file.write_all(self.as_text_with_comments()?.as_bytes())?;

        Ok(())
    }

    pub fn as_text_with_comments(&self) -> Result<String, ClusterError> {
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
                "# Failover mode (stateful, eventual, disabled, raft)".to_string(),
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
                "# Cookie for connecting to the administrative console of the instances"
                    .to_string(),
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

        let mut text = serde_yaml::to_string(self)?;

        for (key, value) in comments {
            let comment = RegexBuilder::new(&format!("^(?P<spaces>[ /t-]*)(?P<key>{key}:( .*)*)"))
                .multi_line(true)
                .build()
                .unwrap();
            text = comment
                .replace_all(&text, |caps: &Captures| {
                    format!(
                        "{whitespaces}{value}\n{any_symbols}{key}",
                        whitespaces = caps[1].replace('-', " "),
                        any_symbols = &caps[1],
                        key = &caps[2],
                    )
                })
                .to_string();
        }

        Ok(text)
    }

    pub fn write_build_state(self, args: &ArgMatches) -> Result<Self, ClusterError> {
        let state_dir = args
            .get_one::<String>("state-dir")
            .cloned()
            .unwrap_or(".geninstate".into());
        let latest_path = Path::new(&state_dir).join("latest.gz");
        let latest_path = latest_path.to_str().unwrap();

        let mut state = State::builder()
            .uid(self.metadata.paths.clone())?
            .make_build_state()
            .path(latest_path)
            .hosts(&self.hosts)
            .vars(&self.vars)
            .failover(&self.failover)
            .topology(&self.topology)
            .build()?;

        state.dump_by_uid(&state_dir)?;
        state.symlink_latest(latest_path)?;

        Ok(self)
    }

    pub fn write_upgrade_state(
        self,
        args: &ArgMatches,
        hosts_diff: Vec<Change>,
    ) -> Result<Self, ClusterError> {
        let instances_diff: Vec<Change> = self
            .hosts
            .add_queue
            .iter()
            .map(|(name, _)| Change::Added(name.to_string()))
            .chain(
                self.hosts
                    .delete_queue
                    .iter()
                    .map(|(name, _)| Change::Removed(name.to_string())),
            )
            .collect();

        if hosts_diff.is_empty() && instances_diff.is_empty() {
            return Ok(self);
        }

        // if args != export-state -> try open latest
        // if .geninstate not exists -> create dir
        // if latest not exists -> create latest
        // if write state
        let state_dir = args
            .get_one::<String>("state-dir")
            .cloned()
            .unwrap_or(".geninstate".into());

        let path: String = if let Ok(Some(path)) = args.try_get_one::<String>("export-state") {
            path.into()
        } else {
            format!("{state_dir}/latest.gz")
        };

        let mut state = State::builder()
            .uid(self.metadata.paths.clone())?
            .make_upgrade_state()
            .path(&path)
            .instances_changes(instances_diff)
            .hosts_changes(hosts_diff)
            .hosts(&self.hosts)
            .vars(&self.vars)
            .failover(&self.failover)
            .topology(&self.topology)
            .build()?;

        state.dump_by_uid(&state_dir)?;
        state.dump_by_path(&path)?;

        Ok(self)
    }

    pub fn to_inventory(&self) -> Result<Inventory, InventoryError> {
        Inventory::try_from(self)
    }

    pub fn clear_instances(mut self) -> Self {
        self.hosts.clear_instances();
        self
    }
}

#[derive(Error, Debug)]
pub enum ClusterError {
    #[error("unexpected io error")]
    Io(#[from] io::Error),
    #[error("serde error")]
    Serde(#[from] serde_yaml::Error),
    #[error("failover error")]
    Failover(#[from] FailoverError),
    #[error("state error {0}")]
    State(#[from] StateError),
    #[error("other error {0}")]
    Other(String),
}

impl From<String> for ClusterError {
    fn from(err: String) -> Self {
        Self::Other(err)
    }
}

#[derive(Deserialize)]
struct HostHelper {
    name: String,
    #[serde(default)]
    config: HostConfig,
    #[serde(default)]
    hosts: Vec<HostHelper>,
}

impl From<HostHelper> for Host {
    fn from(helper: HostHelper) -> Self {
        let name = Name::from(helper.name.as_str());
        helper.into_host_v2(name)
    }
}

impl HostHelper {
    fn into_host_v2(self, name: Name) -> Host {
        if self.hosts.is_empty() {
            return Host {
                name,
                config: self.config,
                hosts: Vec::default(),
                add_queue: IndexMap::default(),
                delete_queue: IndexMap::default(),
                instances: Instances::default(),
            };
        }

        Host {
            hosts: self
                .hosts
                .into_iter()
                .map(|host| {
                    let children_name = name.clone_with_raw_index(host.name.clone());
                    host.into_host_v2(children_name)
                })
                .collect(),
            name,
            config: self.config,
            add_queue: IndexMap::default(),
            delete_queue: IndexMap::default(),
            instances: Instances::default(),
        }
    }
}

struct TopologyMember {
    name: Name,
    count: usize,
    replicas: usize,
    weight: usize,
    roles: Vec<Role>,
    config: IndexMap<String, Value>,
}

impl<'de> Deserialize<'de> for TopologyMember {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            #[serde(default)]
            count: usize,
            #[serde(default)]
            replicas: usize,
            #[serde(default)]
            weight: usize,
            roles: Vec<Role>,
            #[serde(default)]
            config: IndexMap<String, Value>,
        }

        Helper::deserialize(deserializer).map(
            |Helper {
                 name,
                 count,
                 replicas,
                 weight,
                 roles,
                 config,
                 ..
             }| {
                TopologyMember {
                    name: Name::from(name),
                    count,
                    replicas,
                    weight,
                    roles,
                    config,
                }
            },
        )
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidCluster {
    topology: Value,
    hosts: Value,
    failover: Value,
    vars: Value,
}

impl std::fmt::Debug for InvalidCluster {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // topology: Vec<TopologySet>
        formatter.write_str("\n---\ntopology: ")?;
        match &self.topology {
            Value::Null => {
                formatter.write_str("Missing field 'topology'".as_error().as_str())?;
            }
            Value::Sequence(sequence) => {
                sequence
                    .iter()
                    .try_for_each(|value| -> Result<(), std::fmt::Error> {
                        formatter.write_fmt(format_args!(
                            "{:?}",
                            serde_yaml::from_value::<InvalidTopologySet>(value.clone()).unwrap()
                        ))
                    })?;
            }
            _ => {
                formatter.write_str("Topology must be a list".as_error().as_str())?;
            }
        }

        // hosts: Vec<Host>
        match &self.hosts {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "\nhosts: {}",
                    "Missing field 'hosts'".as_error().as_str()
                ))?;
            }
            Value::Sequence(sequence) => {
                formatter.write_str("\nhosts:")?;
                sequence
                    .iter()
                    .try_for_each(|host| -> Result<(), std::fmt::Error> {
                        formatter.write_fmt(format_args!(
                            "{:?}",
                            serde_yaml::from_value::<InvalidHost>(host.clone())
                                .map(|mut host| {
                                    host.offset = "\n  ".into();
                                    host
                                })
                                .unwrap()
                        ))
                    })?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\nhosts: {}",
                    "Hosts must be a list".as_error().as_str()
                ))?;
            }
        }

        // failover: Failover
        match &self.failover {
            Value::Null => {}
            failover @ Value::Mapping(_) => {
                formatter.write_str("\nfailover: ")?;
                formatter.write_fmt(format_args!(
                    "{:?}",
                    serde_yaml::from_value::<InvalidFailover>(failover.clone()).unwrap()
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\nfailover: {}",
                    self.failover.type_error(DICT).as_error()
                ))?;
            }
        }

        // vars: Vars
        match &self.vars {
            vars @ Value::Mapping(_) => {
                formatter.write_str("\nvars: ")?;
                formatter.write_fmt(format_args!(
                    "{:?}",
                    serde_yaml::from_value::<InvalidVars>(vars.clone()).unwrap()
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\nvars: {}",
                    self.vars.type_error(DICT).as_error()
                ))?;
            }
        }

        formatter.write_str("\n")?;

        Ok(())
    }
}

#[cfg(test)]
mod test;
