use std::convert::TryFrom;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use clap::ArgMatches;
use indexmap::{IndexMap, IndexSet};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use thiserror::Error;

use crate::task::cluster::hst::v2::Address;
use crate::task::cluster::hst::v2::HostV2;
use crate::task::cluster::ins::v2::InstanceV2;
use crate::task::cluster::name::Name;
use crate::task::flv::Uri;
use crate::task::vars::Vars;
use crate::task::Cluster;
use crate::task::Validate;
use crate::{
    error::{GeninError, GeninErrorKind},
    task::cluster::ins::Role,
};

use super::cluster::ClusterError;
use super::utils::create_file_or_copy;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub all: InventoryParts,
}

impl<'a> TryFrom<&'a ArgMatches> for Inventory {
    type Error = InventoryError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        let path = match args.get_one::<String>("output") {
            Some(path) => path,
            None => "inventory.yml",
        };

        Ok(serde_yaml::from_reader(File::open(path)?)?)
    }
}

impl<'a> TryFrom<&'a [u8]> for Inventory {
    type Error = GeninError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice(value).map_err(|error| {
            GeninError::new(
                GeninErrorKind::Deserialization,
                format!("Deserizalization error {}", error).as_str(),
            )
        })
    }
}

impl<'a> TryFrom<&'a Cluster> for Inventory {
    type Error = InventoryError;

    fn try_from(cluster: &'a Cluster) -> Result<Self, Self::Error> {
        let cl_hosts = cluster.hosts.lower_level_hosts();

        let vars = cluster.vars.clone().with_failover(cluster.failover.clone());

        Ok(Self {
            all: InventoryParts {
                vars,
                // 1. iterate over instances in each host
                // 2. collect all instances as inventory hosts
                hosts: cl_hosts
                    .iter()
                    .flat_map(|host| {
                        debug!(
                            "inserting values from {} to final inventory",
                            host.name.to_string()
                        );
                        // Iterate over all instances
                        host.instances.iter().map(|instance| {
                            (
                                instance.name.clone(),
                                InventoryHost {
                                    stateboard: instance.stateboard.unwrap_or(false),
                                    zone: instance.config.zone.clone(),
                                    cartridge_extra_env: instance.cartridge_extra_env.clone(),
                                    config: InvHostConfig::from((instance, *host)),
                                    vars: instance.vars.clone(),
                                },
                            )
                        })
                    })
                    .collect(),
                children: cl_hosts
                    .iter()
                    .try_fold(IndexMap::new(), |mut accum, host| {
                        host.instances
                            .iter()
                            .filter(|instance| !instance.is_stateboard())
                            .try_for_each(|instance| {
                                let entry = accum
                                    .entry(instance.name.as_replicaset_name())
                                    .or_insert(Child::Replicaset {
                                        vars: ReplicasetVars {
                                            replicaset_alias: instance
                                                .name
                                                .as_replicaset_alias()
                                                .to_string(),
                                            failover_priority: vec![instance.name.to_string()]
                                                .into_iter()
                                                .collect(),
                                            roles: instance.roles.clone(),
                                            all_rw: instance.config.all_rw,
                                            weight: instance.weight,
                                            vshard_group: instance.config.vshard_group.clone(),
                                        },
                                        hosts: vec![(instance.name.to_string(), Value::Null)]
                                            .into_iter()
                                            .collect(),
                                    });
                                entry.extend_failover_priority(instance.name.to_string())?;
                                entry.insert_host(instance.name.to_string(), Value::Null);
                                Ok::<(), InventoryError>(())
                            })?;
                        Ok::<_, InventoryError>(accum)
                    })?
                    .into_iter()
                    .chain(cl_hosts.iter().fold(IndexMap::new(), |mut accum, host| {
                        accum
                            .entry(host.name.clone())
                            .or_insert(Child::Host {
                                vars: HostVars {
                                    ansible_host: host.config.ansible_host(),
                                    additional_config: host.config.additional_config.clone(),
                                },
                                hosts: host
                                    .instances
                                    .iter()
                                    .map(|instance| (instance.name.to_string(), Value::Null))
                                    .collect(),
                            })
                            .extend_hosts(
                                host.instances
                                    .iter()
                                    .map(|instance| (instance.name.to_string(), Value::Null))
                                    .collect(),
                            );
                        accum
                    }))
                    .collect(),
            },
        })
    }
}

impl<'a> TryFrom<&'a Option<Cluster>> for Inventory {
    type Error = GeninError;

    fn try_from(cluster: &'a Option<Cluster>) -> Result<Self, Self::Error> {
        let (cl_hosts, vars) = if let Some(cluster) = cluster {
            (
                cluster.hosts.lower_level_hosts(),
                cluster.vars.clone().with_failover(cluster.failover.clone()),
            )
        } else {
            return Err(GeninError::new(
                GeninErrorKind::EmptyField,
                "Failed to create inventory from cluster. Cluster is empty.",
            ));
        };

        Ok(Self {
            all: InventoryParts {
                vars,
                // 1. iterate over instances in each host
                // 2. collect all instances as inventory hosts
                hosts: cl_hosts
                    .iter()
                    .flat_map(|host| {
                        debug!(
                            "inserting values from {} to final inventory",
                            host.name.to_string()
                        );
                        // Iterate over all instances
                        host.instances.iter().map(|instance| {
                            (
                                instance.name.clone(),
                                InventoryHost {
                                    stateboard: instance.stateboard.unwrap_or(false),
                                    zone: instance.config.zone.clone(),
                                    cartridge_extra_env: instance.cartridge_extra_env.clone(),
                                    config: InvHostConfig::from((instance, *host)),
                                    vars: instance.vars.clone(),
                                },
                            )
                        })
                    })
                    .collect(),
                children: cl_hosts
                    .iter()
                    .try_fold(IndexMap::new(), |mut accum, host| {
                        host.instances
                            .iter()
                            .filter(|instance| !instance.is_stateboard())
                            .try_for_each(|instance| {
                                let entry = accum
                                    .entry(instance.name.as_replicaset_name())
                                    .or_insert(Child::Replicaset {
                                        vars: ReplicasetVars {
                                            replicaset_alias: instance
                                                .name
                                                .as_replicaset_alias()
                                                .to_string(),
                                            failover_priority: vec![instance.name.to_string()]
                                                .into_iter()
                                                .collect(),
                                            roles: instance.roles.clone(),
                                            all_rw: instance.config.all_rw,
                                            weight: instance.weight,
                                            vshard_group: instance.config.vshard_group.clone(),
                                        },
                                        hosts: vec![(instance.name.to_string(), Value::Null)]
                                            .into_iter()
                                            .collect(),
                                    });
                                entry.extend_failover_priority(instance.name.to_string())?;
                                entry.insert_host(instance.name.to_string(), Value::Null);
                                Ok::<(), GeninError>(())
                            })?;
                        Ok(accum)
                    })?
                    .into_iter()
                    .chain(cl_hosts.iter().fold(IndexMap::new(), |mut accum, host| {
                        accum
                            .entry(host.name.clone())
                            .or_insert(Child::Host {
                                vars: HostVars {
                                    ansible_host: host.config.ansible_host(),
                                    additional_config: host.config.additional_config.clone(),
                                },
                                hosts: host
                                    .instances
                                    .iter()
                                    .map(|instance| (instance.name.to_string(), Value::Null))
                                    .collect(),
                            })
                            .extend_hosts(
                                host.instances
                                    .iter()
                                    .map(|instance| (instance.name.to_string(), Value::Null))
                                    .collect(),
                            );
                        accum
                    }))
                    .collect(),
            },
        })
    }
}

impl Inventory {
    pub fn write(&self, args: &ArgMatches) -> Result<(), InventoryError> {
        let path = PathBuf::from(
            args.try_get_one::<String>("output")
                .unwrap_or_default()
                .unwrap_or(&"inventory.yml".into())
                .to_owned(),
        );

        let file = create_file_or_copy(path, args.get_flag("force"))?;

        serde_yaml::to_writer(file, &self)?;

        Ok(())
    }

    pub fn try_into_cluster(&self) -> Result<Cluster, InventoryError> {
        Ok(Cluster::try_from(self)?)
    }
}

#[derive(Error, Debug)]
pub enum InventoryError {
    #[error("unexpected io error")]
    Io(#[from] io::Error),
    #[error("serde error")]
    Serde(#[from] serde_yaml::Error),
    #[error("genin error")]
    Genin(#[from] GeninError),
    #[error("cluster error")]
    Cluster(#[from] ClusterError),
}

impl Validate for Inventory {
    type Type = String;
    type Error = String;

    fn validate(_bytes: &[u8]) -> Result<Self::Type, Self::Error> {
        Ok(String::default())
    }

    fn whole_block(bytes: &[u8]) -> String {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InventoryParts {
    pub vars: Vars,
    pub hosts: IndexMap<Name, InventoryHost>,
    pub children: IndexMap<Name, Child>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InventoryHost {
    #[serde(default, skip_serializing_if = "InventoryHost::not_stateboard")]
    pub stateboard: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub cartridge_extra_env: IndexMap<String, Value>,
    pub config: InvHostConfig,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub vars: IndexMap<String, Value>,
}

impl InventoryHost {
    pub fn not_stateboard(stateboard: &bool) -> bool {
        !stateboard
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum InvHostConfig {
    Instance {
        advertise_uri: Uri,
        http_port: u16,
        #[serde(flatten, skip_serializing_if = "IndexMap::is_empty")]
        additional_config: IndexMap<String, Value>,
    },
    Stateboard(IndexMap<String, Value>),
}

impl<'a> From<(&'a InstanceV2, &'a HostV2)> for InvHostConfig {
    fn from(pair: (&'a InstanceV2, &'a HostV2)) -> Self {
        if !pair.0.is_stateboard() {
            InvHostConfig::Instance {
                advertise_uri: Uri {
                    address: pair.1.config.address.clone(),
                    port: pair.0.config.binary_port.unwrap(),
                },
                http_port: pair.0.config.http_port.unwrap(),
                additional_config: pair.0.config.additional_config.clone(),
            }
        } else {
            InvHostConfig::Stateboard(pair.0.config.additional_config.clone())
        }
    }
}

impl InvHostConfig {
    pub fn http_port(&self) -> u16 {
        if let InvHostConfig::Instance { http_port, .. } = self {
            *http_port
        } else {
            unreachable!()
        }
    }

    pub fn binary_port(&self) -> u16 {
        if let InvHostConfig::Instance { advertise_uri, .. } = self {
            advertise_uri.port
        } else {
            unreachable!()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Child {
    Replicaset {
        vars: ReplicasetVars,
        hosts: IndexMap<String, Value>,
    },
    Host {
        vars: HostVars,
        hosts: IndexMap<String, Value>,
    },
}

impl Child {
    pub fn extend_failover_priority(&mut self, name: String) -> Result<(), GeninError> {
        match self {
            Child::Replicaset { vars, .. } => {
                vars.failover_priority.insert(name);
                vars.failover_priority.sort();
                Ok(())
            }
            Child::Host { .. } => Err(GeninError::new(
                GeninErrorKind::NotApplicable,
                "unable to extend failover_priority for child type Child::Host",
            )),
        }
    }

    pub fn extend_hosts(&mut self, new_hosts: IndexMap<String, Value>) {
        match self {
            Self::Host { hosts, .. } => {
                hosts.extend(new_hosts);
            }
            Self::Replicaset { hosts, .. } => {
                hosts.extend(new_hosts);
            }
        }
    }

    pub fn insert_host(&mut self, name: String, value: Value) {
        match self {
            Self::Replicaset { hosts, .. } => {
                hosts.insert(name, value);
            }
            Self::Host { hosts, .. } => {
                hosts.insert(name, value);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ReplicasetVars {
    pub replicaset_alias: String,
    pub failover_priority: IndexSet<String>,
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_rw: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vshard_group: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct HostVars {
    pub ansible_host: Address,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_config: IndexMap<String, Value>,
}

#[cfg(test)]
mod test;
