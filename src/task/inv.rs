use std::ops::{Deref, DerefMut};

use genin::libs::{
    error::{ConfigError, TaskError},
    ins::{is_false, Instance, Role, Type},
};
use indexmap::IndexMap;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use super::cluster::scheme::Scheme;

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct Inventory {
    pub(in crate::task) all: InventoryParts,
}

impl<'a> TryFrom<&'a [u8]> for Inventory {
    type Error = TaskError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice(value).map_err(|err| {
            TaskError::ConfigError(ConfigError::FileFormatError(format!(
                "Deserizalization error {}",
                err
            )))
        })
    }
}

impl TryFrom<Scheme> for Inventory {
    type Error = TaskError;

    fn try_from(mut scheme: Scheme) -> Result<Self, Self::Error> {
        let mut hosts = IndexMap::new();
        let mut children = IndexMap::new();
        let mut replicasets: IndexMap<String, Vec<String>> = IndexMap::new();
        scheme.deref_mut().iter_mut().for_each(|host| {
            debug!("inserting values from {} to final inventory", host.name());
            host.instances.iter_mut().for_each(|instance| {
                hosts.insert(
                    instance.name.clone(),
                    InventoryHost {
                        stateboard: instance.stateboard,
                        config: instance.config.clone(),
                    },
                );
                replicasets
                    .entry(instance.parent.to_string())
                    .or_default()
                    .push(instance.name.to_string());
            });
        });
        replicasets.iter_mut().for_each(|(_, v)| v.sort());

        scheme.deref().iter().for_each(|host| {
            children.extend(
                host.instances
                    .iter()
                    .filter(|instance| {
                        trace!("filtering instance {}", instance.name);
                        !matches!(instance.itype, Type::Replica | Type::Dummy)
                    })
                    .collect::<Vec<&Instance>>()
                    .iter()
                    .map(|instance| {
                        debug!("replicaset keys {:?}", &replicasets.keys());
                        trace!(
                            "replicaset members by key {:?} is: {:?}",
                            &instance.name,
                            replicasets.get(&instance.name)
                        );
                        (
                            format!("{}-replicaset", instance.parent),
                            InventoryReplicaset {
                                hosts: replicasets
                                    .get(&instance.name)
                                    .cloned().unwrap_or_default()
                                    .into_iter()
                                    .map(|member| (member, Value::Null))
                                    .collect(),
                                vars: InventoryVars::ReplicasetInventoryVars {
                                    replicaset_alias: instance.parent.to_string(),
                                    weight: instance.weight,
                                    failover_priority: replicasets
                                        .remove(&instance.name)
                                        .unwrap_or_default(),
                                    roles: instance.roles.clone(),
                                },
                            },
                        )
                    }),
            );
        });

        scheme.deref().iter().for_each(|host| {
            children.insert(
                host.name().into(),
                InventoryReplicaset {
                    vars: InventoryVars::HostInventoryVars(
                        vec![(
                            "ansible_host".to_string(),
                            Value::String(host.ip.to_string()),
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
                vars: scheme.vars.clone(),
                hosts,
                children,
            },
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct InventoryParts {
    vars: IndexMap<String, Value>,
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
