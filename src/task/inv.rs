use std::fmt::Display;

use indexmap::IndexMap;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::task::vrs::Vars;
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
        let (cl_hosts, cl_vars) = if let Some(cluster) = cluster {
            (cluster.hosts(), cluster.vars())
        } else {
            return Err(GeninError::new(
                GeninErrorKind::EmptyField,
                "Failed to create inventory from cluster. Cluster is empty.",
            ));
        };

        let mut hosts = IndexMap::new();
        let mut children = IndexMap::new();
        let mut replicasets: IndexMap<String, Vec<String>> = IndexMap::new();
        cl_hosts
            .first()
            .unwrap()
            .bottom_level()
            .into_iter()
            .for_each(|host| {
                debug!(
                    "inserting values from {} to final inventory",
                    host.name.as_str()
                );
                host.instances.iter().for_each(|instance| {
                    hosts.insert(
                        instance.name.to_string(),
                        InventoryHost {
                            // zone: TODO: zone
                            stateboard: instance.stateboard.unwrap_or(false),
                            config: instance.config.config(), //TODO
                        },
                    );
                    replicasets
                        .entry(instance.name.get_parent().to_string())
                        .or_default()
                        .push(instance.name.to_string());
                });
            });
        replicasets.iter_mut().for_each(|(_, v)| v.sort());

        cl_hosts
            .first()
            .unwrap()
            .bottom_level()
            .into_iter()
            .for_each(|host| {
                children.extend(
                    host.instances
                        .iter()
                        //.filter(|instance| {
                        //    trace!("filtering instance {}", instance.name);
                        //    !matches!(instance.itype, Type::Replica | Type::Dummy)
                        //})
                        .map(|instance| {
                            debug!("replicaset keys {:?}", &replicasets.keys());
                            trace!(
                                "replicaset members by key {:?} is: {:?}",
                                &instance.name,
                                replicasets.get(&instance.name.to_string())
                            );
                            (
                                format!("{}-replicaset", instance.name.get_parent(),),
                                InventoryReplicaset {
                                    hosts: replicasets
                                        .get(&instance.name.to_string())
                                        .cloned()
                                        .unwrap_or_default()
                                        .into_iter()
                                        .map(|member| (member, Value::Null))
                                        .collect(),
                                    vars: InventoryVars::ReplicasetInventoryVars {
                                        replicaset_alias: instance.name.get_parent().to_string(),
                                        weight: 0, //TODO
                                        failover_priority: replicasets
                                            .remove(&instance.name.to_string())
                                            .unwrap_or_default(),
                                        roles: instance.roles.clone(),
                                    },
                                },
                            )
                        }),
                );
            });

        cl_hosts
            .first()
            .unwrap()
            .bottom_level()
            .into_iter()
            .for_each(|host| {
                children.insert(
                    host.name.clone(),
                    InventoryReplicaset {
                        vars: InventoryVars::HostInventoryVars(
                            vec![(
                                "ansible_host".to_string(),
                                Value::String(host.config.address()),
                            )]
                            .into_iter()
                            .collect(),
                        ),
                        hosts: host
                            .instances
                            .iter()
                            .map(|instance| (instance.name.to_string(), Value::Null))
                            .collect(),
                    },
                );
            });

        Ok(Self {
            all: InventoryParts {
                vars: cl_vars.clone(),
                hosts,
                children,
            },
        })
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
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
    config: IndexMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
pub struct InventoryReplicaset {
    vars: InventoryVars,
    hosts: IndexMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum InventoryVars {
    HostInventoryVars(IndexMap<String, Value>),
    ReplicasetInventoryVars {
        replicaset_alias: String,
        weight: usize,
        failover_priority: Vec<String>,
        roles: Vec<Role>,
    },
}

#[cfg(test)]
mod test;
