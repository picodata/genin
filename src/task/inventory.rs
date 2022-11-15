use indexmap::{IndexMap, IndexSet};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::task::vars::Vars;
use crate::task::Cluster;
use crate::{
    error::{GeninError, GeninErrorKind},
    task::cluster::ins::{is_false, Role},
};

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct Inventory {
    pub(in crate::task) all: InventoryParts,
}

impl<'a> TryFrom<&'a [u8]> for Inventory {
    type Error = GeninError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice(value).map_err(|error| {
            GeninError::new(
                GeninErrorKind::DeserializationError,
                format!("Deserizalization error {}", error).as_str(),
            )
        })
    }
}

impl<'a> TryFrom<&'a Option<Cluster>> for Inventory {
    type Error = GeninError;

    fn try_from(cluster: &'a Option<Cluster>) -> Result<Self, Self::Error> {
        let (cl_hosts, vars) = if let Some(cluster) = cluster {
            (
                cluster.hosts().first().unwrap().lower_level_hosts(),
                cluster
                    .vars()
                    .clone()
                    .with_failover(cluster.failover().clone()),
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
                        host.instances.iter().map(|instance| {
                            (
                                instance.name.to_string(),
                                InventoryHost {
                                    stateboard: instance.stateboard.unwrap_or(false),
                                    zone: instance.zone.clone(),
                                    config: { instance.config.additional_config() },
                                },
                            )
                        })
                    })
                    .collect(),
                children: cl_hosts
                    .iter()
                    .fold(IndexMap::new(), |mut accum, host| {
                        host.instances.iter().for_each(|instance| {
                            let entry = accum
                                .entry(format!("{}-replicaset", instance.name.get_parent()))
                                .or_insert(InventoryReplicaset {
                                    vars: InventoryVars::ReplicasetInventoryVars {
                                        replicaset_alias: instance.name.get_parent().to_string(),
                                        weight: instance.weight,
                                        failover_priority: vec![instance.name.to_string()]
                                            .into_iter()
                                            .collect(),
                                        roles: instance.roles.clone(),
                                    },
                                    hosts: vec![(instance.name.to_string(), Value::Null)]
                                        .into_iter()
                                        .collect(),
                                });
                            entry.extend_failover_priority(instance.name.to_string());
                            entry.insert_host(instance.name.to_string(), Value::Null);
                        });
                        accum
                    })
                    .into_iter()
                    .chain(cl_hosts.iter().fold(IndexMap::new(), |mut accum, host| {
                        accum
                            .entry(host.name.to_string())
                            .or_insert(InventoryReplicaset {
                                vars: InventoryVars::HostInventoryVars(
                                    vec![(
                                        String::from("ansible_host"),
                                        Value::String(host.config.address_to_string()),
                                    )]
                                    .into_iter()
                                    .collect(),
                                ),
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

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct InventoryParts {
    vars: Vars,
    hosts: IndexMap<String, InventoryHost>,
    children: IndexMap<String, InventoryReplicaset>,
}

#[derive(Serialize, Deserialize)]
pub struct InventoryHost {
    #[serde(default, skip_serializing_if = "is_false")]
    stateboard: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    zone: Option<String>,
    config: IndexMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct InventoryReplicaset {
    vars: InventoryVars,
    hosts: IndexMap<String, Value>,
}

impl InventoryReplicaset {
    pub fn extend_failover_priority(&mut self, name: String) {
        match &mut self.vars {
            InventoryVars::ReplicasetInventoryVars {
                failover_priority, ..
            } => {
                failover_priority.insert(name);
                failover_priority.sort();
            }
            _ => unimplemented!(),
        }
    }

    pub fn insert_host(&mut self, host: String, value: Value) {
        self.hosts.insert(host, value);
        self.hosts.sort_keys();
    }

    pub fn extend_hosts(&mut self, hosts: IndexMap<String, Value>) {
        self.hosts.extend(hosts);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum InventoryVars {
    HostInventoryVars(IndexMap<String, Value>),
    ReplicasetInventoryVars {
        replicaset_alias: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        weight: Option<usize>,
        failover_priority: IndexSet<String>,
        roles: Vec<Role>,
    },
}

#[cfg(test)]
mod test;
