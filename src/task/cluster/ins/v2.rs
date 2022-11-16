use indexmap::IndexMap;
use log::trace;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::cmp::Ordering;

use crate::task::cluster::hst::merge_index_maps;
use crate::task::cluster::hst::v2::HostV2Config;
use crate::task::cluster::ins::{v1::Instance, AsV2Replicaset, Role};
use crate::task::cluster::name::Name;
use crate::task::inventory::InvHostConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Replicaset TODO: docs, remove public
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct Replicaset {
    pub name: Name,
    pub replicasets_count: Option<usize>,
    pub replication_factor: Option<usize>,
    pub weight: Option<usize>,
    pub failure_domains: Vec<String>,
    pub roles: Vec<Role>,
    pub config: InstanceV2Config,
}

#[allow(unused)]
impl Replicaset {
    pub fn instances(&self) -> Vec<InstanceV2> {
        if self.replication_factor.is_none() {
            return vec![InstanceV2 {
                name: self.name.clone(),
                stateboard: None,
                weight: self.weight,
                failure_domains: self.failure_domains.clone(),
                roles: self.roles.clone(),
                config: self.config.clone(),
            }];
        }
        (1..=self.replication_factor.unwrap_or(1))
            .map(|index| InstanceV2 {
                name: self.name.clone_with_index(index),
                stateboard: None,
                weight: self.weight,
                failure_domains: self.failure_domains.clone(),
                roles: self.roles.clone(),
                config: self.config.clone(),
            })
            .collect()
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

impl PartialOrd for Replicaset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            ord => ord,
        }
    }
}

impl Ord for Replicaset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// TODO: docs, remove pub
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct InstanceV2 {
    pub name: Name,
    pub stateboard: Option<bool>,
    pub weight: Option<usize>,
    pub failure_domains: Vec<String>,
    pub roles: Vec<Role>,
    pub config: InstanceV2Config,
}

impl PartialOrd for InstanceV2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            ord => ord,
        }
    }
}

impl Ord for InstanceV2 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl AsV2Replicaset for Vec<Instance> {
    fn as_v2_replicaset(&self) -> Vec<Replicaset> {
        self.iter()
            .map(
                |Instance {
                     name,
                     count,
                     replicas,
                     weight,
                     roles,
                     config,
                     ..
                 }| {
                    Replicaset {
                        name: Name::from(name.as_str()),
                        replicasets_count: Some(*count),
                        replication_factor: Some(*replicas),
                        weight: Some(*weight),
                        failure_domains: Vec::new(),
                        roles: roles.clone(),
                        config: InstanceV2Config::from(config),
                    }
                },
            )
            .collect()
    }
}

impl InstanceV2 {
    pub fn is_stateboard(&self) -> bool {
        if let Some(stateboard) = self.stateboard {
            stateboard
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceV2Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_rw: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vshard_group: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_config: IndexMap<String, Value>,
}

impl<'a> From<&'a InvHostConfig> for InstanceV2Config {
    fn from(config: &'a InvHostConfig) -> Self {
        match config {
            InvHostConfig::Instance {
                advertise_uri,
                http_port,
                zone,
                additional_config,
            } => Self {
                http_port: Some(*http_port),
                binary_port: Some(advertise_uri.port),
                all_rw: None,
                zone: zone.clone(),
                vshard_group: None,
                additional_config: additional_config.clone(),
            },
            InvHostConfig::Stateboard(additional_config) => Self {
                http_port: None,
                binary_port: None,
                all_rw: None,
                zone: None,
                vshard_group: None,
                additional_config: additional_config.clone(),
            },
        }
    }
}

impl<'a> From<&'a IndexMap<String, Value>> for InstanceV2Config {
    fn from(config: &'a IndexMap<String, Value>) -> Self {
        Self {
            http_port: config
                .get("http_port")
                .map(|http_port| http_port.as_str().unwrap().parse().unwrap()),
            binary_port: config.get("advertise_uri").map(|advertise_uri| {
                serde_yaml::from_str(advertise_uri.as_str().unwrap()).unwrap()
            }),
            all_rw: config.get("all_rw").map(|all_rw| all_rw.as_bool().unwrap()),
            zone: config
                .get("zone")
                .map(|zone| zone.as_str().unwrap().to_string()),
            vshard_group: config
                .get("vshard_group")
                .map(|vshard_group| vshard_group.as_str().unwrap().to_string()),
            additional_config: config
                .into_iter()
                .filter_map(|(name, value)| match name.as_str() {
                    "all_rw" | "zone" | "vshard_group" | "http_port" | "advertise_uri" => None,
                    _ => Some((name.to_string(), value.clone())),
                })
                .collect(),
        }
    }
}

#[allow(unused)]
impl InstanceV2Config {
    pub fn merge_and_up_ports(self, other: HostV2Config, index: u16) -> Self {
        trace!("Config before merge: {:?}", &self);
        Self {
            http_port: self
                .http_port
                .or_else(|| other.http_port.map(|port| port + index)),
            binary_port: self
                .binary_port
                .or_else(|| other.binary_port.map(|port| port + index)),
            all_rw: self.all_rw,
            zone: self.zone,
            vshard_group: self.vshard_group,
            additional_config: merge_index_maps(self.additional_config, other.additional_config),
        }
    }

    pub fn merge_with_host_v2_config(self, other: HostV2Config) -> Self {
        Self {
            http_port: self.http_port.or(other.http_port),
            binary_port: self.binary_port.or(other.binary_port),
            all_rw: self.all_rw,
            zone: self.zone,
            vshard_group: self.vshard_group,
            additional_config: merge_index_maps(self.additional_config, other.additional_config),
        }
    }

    pub fn is_none(&self) -> bool {
        self.http_port.is_none()
            && self.binary_port.is_none()
            && self.all_rw.is_none()
            && self.zone.is_none()
            && self.vshard_group.is_none()
            && self.additional_config.is_empty()
    }

    pub fn clean_ports(self) -> Self {
        Self {
            http_port: None,
            binary_port: None,
            ..self
        }
    }
}
