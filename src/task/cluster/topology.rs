use std::{cmp::Ordering, collections::HashSet};

use indexmap::IndexMap;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_yaml::{Number, Value};
use tabled::Alignment;

use crate::task::{
    vars::print_value_recursive, AsError, ErrConfMapping, TypeError, BOOL, DICT, LIST, NUMBER,
    STRING,
};

use super::{
    host::view::{TableColors, View},
    instance::{
        ins::{Instance, InstanceConfig, Instances},
        Role,
    },
    name::Name,
    TopologyMember,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Topology(Vec<TopologySet>);

impl TryFrom<Instances> for Topology {
    type Error = String;

    fn try_from(instances: Instances) -> Result<Self, Self::Error> {
        let unique = instances
            .iter()
            .map(|ins| &ins.name)
            .collect::<HashSet<&Name>>();

        if unique.len() < instances.len() {
            return Err("Replicaset names must be unique".into());
        }

        Ok(Self(
            instances
                .iter()
                .fold(
                    IndexMap::<Name, TopologySet>::new(),
                    |mut replicasets,
                     Instance {
                         name,
                         weight,
                         failure_domains,
                         roles,
                         cartridge_extra_env,
                         config,
                         vars,
                         ..
                     }| {
                        debug!(
                            "Instance {} will be mapped to {}",
                            name,
                            name.get_parent_str()
                        );
                        // iteration over all instances and fold it into replicasets like:
                        // storage-1-1, storage-1-2, storage-2-1, storage-2-2 -> storage-1, storage-2
                        if let Some(replication_factor) = replicasets
                            .entry(name.get_parent_name())
                            .or_insert(TopologySet {
                                name: name.get_parent_name(),
                                replicasets_count: None,
                                replication_factor: Some(0),
                                weight: *weight,
                                failure_domains: failure_domains.clone().into(),
                                roles: roles.clone(),
                                cartridge_extra_env: cartridge_extra_env.clone(),
                                config: config.clone(),
                                vars: vars.clone(),
                            })
                            .replication_factor
                            .as_mut()
                        {
                            *replication_factor += 1;
                        }

                        replicasets
                    },
                )
                .into_iter()
                .fold(
                    IndexMap::new(),
                    |mut topology_set,
                     (
                        _,
                        TopologySet {
                            name,
                            replication_factor,
                            weight,
                            failure_domains,
                            roles,
                            cartridge_extra_env,
                            config,
                            vars,
                            ..
                        },
                    )| {
                        debug!(
                            "Replicaset {} will be mapped to {}",
                            name,
                            name.get_ancestor_str()
                        );
                        // iteration over all instances and fold it into replicasets like:
                        // storage-1-1, storage-1-2, storage-2-1, storage-2-2 -> storage-1, storage-2
                        if let Some(replicasets_count) = topology_set
                            .entry(name.get_ancestor_name())
                            .or_insert(TopologySet {
                                name: name.get_ancestor_name(),
                                replicasets_count: Some(0),
                                replication_factor,
                                weight,
                                failure_domains,
                                roles,
                                cartridge_extra_env,
                                config,
                                vars,
                            })
                            .replicasets_count
                            .as_mut()
                        {
                            *replicasets_count += 1;
                        }
                        topology_set
                    },
                )
                .into_iter()
                .map(|(_, mut topology_set)| {
                    trace!("TopologySet {:?}", &topology_set);
                    topology_set.replication_factor =
                        topology_set
                            .replication_factor
                            .and_then(|replication_factor| {
                                if matches!(replication_factor, 0 | 1) {
                                    None
                                } else {
                                    Some(replication_factor)
                                }
                            });
                    topology_set
                })
                .collect(),
        ))
    }
}

impl<'a> From<&'a Topology> for Instances {
    fn from(topology: &'a Topology) -> Self {
        let mut table_colors = TableColors::new();
        Self::from(
            topology
                .0
                .iter()
                .flat_map(
                    |TopologySet {
                         name,
                         replicasets_count,
                         replication_factor,
                         weight,
                         failure_domains,
                         roles,
                         cartridge_extra_env,
                         config,
                         vars,
                     }| {
                        (1..=replicasets_count.unwrap_or(1))
                            .flat_map(|repliaset_num| {
                                if !replication_factor.is_none() {
                                    (1..=replication_factor.unwrap())
                                        .map(|instance_num| Instance {
                                            name: name
                                                .clone_with_index(repliaset_num)
                                                .clone_with_index(instance_num),
                                            stateboard: None,
                                            weight: *weight,
                                            failure_domains: failure_domains.clone().into(),
                                            roles: roles.clone(),
                                            config: config.clone(),
                                            cartridge_extra_env: cartridge_extra_env.clone(),
                                            vars: vars.clone(),
                                            view: View {
                                                alignment: Alignment::left(),
                                                color: table_colors.next_color(
                                                    name.clone_with_index(repliaset_num),
                                                ),
                                            },
                                        })
                                        .collect::<Vec<Instance>>()
                                } else {
                                    vec![Instance {
                                        name: name.clone_with_index(repliaset_num),
                                        stateboard: None,
                                        weight: *weight,
                                        failure_domains: failure_domains.clone().into(),
                                        roles: roles.clone(),
                                        cartridge_extra_env: cartridge_extra_env.clone(),
                                        config: config.clone(),
                                        vars: vars.clone(),
                                        view: View {
                                            alignment: Alignment::left(),
                                            color: table_colors.next_color(name.clone()),
                                        },
                                    }]
                                }
                            })
                            .collect::<Vec<Instance>>()
                    },
                )
                .collect::<Vec<Instance>>(),
        )
    }
}

impl Topology {
    pub fn check_unique(self) -> Result<Self, String> {
        let unique = self
            .0
            .iter()
            .map(|topology_set| &topology_set.name)
            .collect::<HashSet<&Name>>();

        if unique.len() < self.0.len() {
            return Err("Replicaset names must be unique".into());
        }

        Ok(self)
    }
}

impl From<Vec<TopologyMember>> for Topology {
    fn from(members: Vec<TopologyMember>) -> Self {
        Self(
            members
                .into_iter()
                .map(
                    |TopologyMember {
                         name,
                         count,
                         replicas,
                         weight,
                         roles,
                         config,
                     }| {
                        TopologySet {
                            name,
                            replicasets_count: Some(count),
                            replication_factor: Some(replicas).and_then(|replication_factor| {
                                if replication_factor.eq(&0) {
                                    None
                                } else {
                                    Some(replication_factor + 1)
                                }
                            }),
                            weight: Some(weight).and_then(|weight| {
                                if weight.eq(&0) {
                                    None
                                } else {
                                    Some(weight)
                                }
                            }),
                            failure_domains: Default::default(),
                            roles,
                            cartridge_extra_env: IndexMap::default(),
                            config: InstanceConfig {
                                additional_config: config,
                                ..InstanceConfig::default()
                            },
                            vars: IndexMap::default(),
                        }
                    },
                )
                .collect(),
        )
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self(vec![
            TopologySet {
                name: Name::from("router"),
                replicasets_count: Some(1),
                replication_factor: None,
                weight: None,
                failure_domains: Default::default(),
                roles: vec![Role::router(), Role::failover_coordinator()],
                cartridge_extra_env: IndexMap::default(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
            },
            TopologySet {
                name: Name::from("storage"),
                replicasets_count: Some(2),
                replication_factor: Some(2),
                weight: None,
                failure_domains: Default::default(),
                roles: vec![Role::storage()],
                cartridge_extra_env: IndexMap::default(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
            },
        ])
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
struct TopologySet {
    name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    replicasets_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replication_factor: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    weight: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failure_domains: Vec<String>,
    #[serde(default)]
    roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    cartridge_extra_env: IndexMap<String, Value>,
    #[serde(skip_serializing_if = "InstanceConfig::is_none")]
    config: InstanceConfig,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    vars: IndexMap<String, Value>,
}

impl<'de> Deserialize<'de> for TopologySet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            #[serde(default)]
            replicasets_count: Option<usize>,
            #[serde(default)]
            replication_factor: Option<usize>,
            #[serde(default)]
            weight: Option<usize>,
            #[serde(default)]
            failure_domains: Vec<String>,
            #[serde(default)]
            roles: Vec<Role>,
            #[serde(default)]
            all_rw: Option<bool>,
            #[serde(default)]
            cartridge_extra_env: IndexMap<String, Value>,
            #[serde(default)]
            config: Option<InstanceConfig>,
            #[serde(default)]
            vars: IndexMap<String, Value>,
        }

        Helper::deserialize(deserializer).map(
            |Helper {
                 name,
                 replicasets_count,
                 replication_factor,
                 weight,
                 failure_domains,
                 roles,
                 all_rw,
                 cartridge_extra_env,
                 config,
                 vars,
             }| {
                // If type not defined in yaml let's try to infer based on name
                TopologySet {
                    name: Name::from(name),
                    replicasets_count,
                    replication_factor: replication_factor.and_then(|replication_factor| {
                        if replication_factor.eq(&1) || replication_factor.eq(&0) {
                            None
                        } else {
                            Some(replication_factor)
                        }
                    }),
                    weight,
                    failure_domains,
                    roles,
                    cartridge_extra_env,
                    config: config.unwrap_or_default().with_all_rw(all_rw),
                    vars,
                }
            },
        )
    }
}

impl PartialOrd for TopologySet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TopologySet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (
            &self.failure_domains.is_empty(),
            &other.failure_domains.is_empty(),
        ) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => Ordering::Equal,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidTopologySet {
    name: Value,
    replicasets_count: Value,
    replication_factor: Value,
    weight: Value,
    failure_domains: Value,
    roles: Value,
    all_rw: Value,
    cartridge_extra_env: Value,
    config: Value,
    vars: Value,
}

impl std::fmt::Debug for InvalidTopologySet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // name: String
        match &self.name {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "\n  - name: {}",
                    "Missing field 'name'".as_error()
                ))?;
            }
            Value::String(name) => {
                formatter.write_fmt(format_args!("\n  - name: {}", name))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  - name: {}",
                    self.name.type_error(STRING).as_error()
                ))?;
            }
        }

        // replicasets_count: usize
        match &self.replicasets_count {
            Value::Null => {}
            Value::Number(replicasets_count) if replicasets_count.is_u64() => {
                formatter.write_fmt(format_args!(
                    "\n    replicasets_count: {}",
                    replicasets_count
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    replicasets_count: {}",
                    self.replicasets_count.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // replication_factor: usize
        match &self.replication_factor {
            Value::Null => {}
            Value::Number(replication_factor) if replication_factor.is_u64() => {
                formatter.write_fmt(format_args!(
                    "\n    replication_factor: {}",
                    replication_factor
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    replication_factor: {}",
                    self.replication_factor.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // replication_factor: usize
        match &self.weight {
            Value::Null => {}
            Value::Number(weight) if weight.is_u64() => {
                formatter.write_fmt(format_args!("\n    weight: {}", weight))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "    weight: {}",
                    self.weight.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // failure_domains: Vec<String>
        match &self.failure_domains {
            Value::Null => {}
            Value::Sequence(failure_domains) => {
                formatter.write_fmt(format_args!(
                    "\n    failure_domains: {:?}",
                    InvalidFailureDomains {
                        offset: "\n      ".into(),
                        value: failure_domains
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  failure_domains: {}",
                    self.failure_domains.type_error(LIST).as_error()
                ))?;
            }
        }

        // roles: Vec<Role>
        match &self.roles {
            Value::Null => {}
            Value::Sequence(roles) => {
                formatter.write_fmt(format_args!(
                    "\n    roles: {:?}",
                    InvalidRoles {
                        offset: "\n      ".into(),
                        value: roles
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    roles: {}",
                    self.roles.type_error(LIST).as_error()
                ))?;
            }
        }

        // all_rw: bool
        match &self.all_rw {
            Value::Null => {}
            Value::Bool(all_rw) => {
                formatter.write_fmt(format_args!("\n    all_rw: {}", all_rw))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    all_rw: {}",
                    self.all_rw.type_error(BOOL).as_error()
                ))?;
            }
        }

        // cartridge_extra_env: IndexMap<String, Value>
        match &self.cartridge_extra_env {
            Value::Null => {}
            Value::Mapping(cartridge_extra_env) => {
                formatter.write_fmt(format_args!(
                    "\n    cartridge_extra_env: {:?}",
                    ErrConfMapping {
                        offset: "    ".into(),
                        value: cartridge_extra_env,
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    cartridge_extra_env: {}",
                    self.cartridge_extra_env.type_error(DICT).as_error()
                ))?;
            }
        }

        match &self.config {
            Value::Null => {}
            config @ Value::Mapping(_) => formatter.write_fmt(format_args!(
                "{:?}",
                serde_yaml::from_value::<ErrInstanceConfig>(config.clone())
                    .map(|mut config| {
                        config.offset = "\n    ".into();
                        config
                    })
                    .unwrap()
            ))?,
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    config: {}",
                    self.config.type_error(DICT).as_error()
                ))?;
            }
        }

        // vars: IndexMap<String, Value>
        match &self.vars {
            Value::Null => {}
            Value::Mapping(vars) => {
                formatter.write_fmt(format_args!(
                    "\n    vars: {:?}",
                    ErrConfMapping {
                        offset: "\n    ".into(),
                        value: vars,
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    vars: {}",
                    self.vars.type_error(DICT).as_error()
                ))?;
            }
        }

        Ok(())
    }
}

struct InvalidFailureDomains<'a> {
    offset: String,
    value: &'a Vec<Value>,
}

impl std::fmt::Debug for InvalidFailureDomains<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.iter().try_for_each(|value| match value {
            Value::String(domain) => {
                formatter.write_fmt(format_args!("{}- {}", &self.offset, domain))
            }
            _ => formatter.write_fmt(format_args!(
                "{}- {}",
                &self.offset,
                value.type_error(STRING)
            )),
        })?;

        Ok(())
    }
}

struct InvalidRoles<'a> {
    offset: String,
    value: &'a Vec<Value>,
}

impl std::fmt::Debug for InvalidRoles<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for role in self.value {
            match role {
                Value::String(role) => {
                    formatter.write_fmt(format_args!("{}- {}", &self.offset, role))?;
                }
                _ => {
                    formatter.write_fmt(format_args!(
                        "{}- {}",
                        &self.offset,
                        role.type_error(STRING)
                    ))?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct ErrInstanceConfig {
    #[serde(skip)]
    offset: String,
    http_port: Value,
    binary_port: Value,
    all_rw: Value,
    zone: Value,
    vshard_group: Value,
    #[serde(flatten)]
    additional_config: Value,
}

impl std::fmt::Debug for ErrInstanceConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_fmt(format_args!("{}config: ", self.offset))?;
        // http_port: u16
        match &self.http_port {
            Value::Null => {}
            Value::Number(http_port) => {
                if http_port > &Number::from(0) && http_port < &Number::from(u16::MAX) {
                    formatter
                        .write_fmt(format_args!("{}  http_port: {}", self.offset, http_port))?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}  http_port: {}",
                        self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  http_port: {}",
                    self.offset,
                    self.http_port.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // binary_port: u16
        match &self.binary_port {
            Value::Null => {}
            Value::Number(binary_port) => {
                if binary_port > &Number::from(0) && binary_port < &Number::from(u16::MAX) {
                    formatter
                        .write_fmt(format_args!("{}  http_port: {}", self.offset, binary_port))?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}  binary_port: {}",
                        self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  binary_port: {}",
                    self.offset,
                    self.binary_port.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // all_rw: bool
        match &self.all_rw {
            Value::Null => {}
            Value::Bool(all_rw) => {
                formatter.write_fmt(format_args!("{}  all_rw: {}", self.offset, all_rw))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  all_rw: {}",
                    self.offset,
                    self.all_rw.type_error(BOOL).as_error()
                ))?;
            }
        }

        // zone: String
        match &self.zone {
            Value::Null => {}
            Value::String(zone) => {
                formatter.write_fmt(format_args!("{}  zone: {}", self.offset, zone))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  zone: {}",
                    self.offset,
                    self.zone.type_error(STRING).as_error()
                ))?;
            }
        }

        // vshard_group: String
        match &self.vshard_group {
            Value::Null => {}
            Value::String(vshard_group) => {
                formatter.write_fmt(format_args!(
                    "{}  vshard_group: {}",
                    self.offset, vshard_group
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  vshard_group: {}",
                    self.offset,
                    self.vshard_group.type_error(STRING).as_error()
                ))?;
            }
        }

        // additional_config: IndexMap<String, Value>
        match &self.additional_config {
            Value::Null => {}
            Value::Mapping(additional_config) => {
                for (key, item) in additional_config {
                    formatter.write_fmt(format_args!(
                        "{}  {}: ",
                        &self.offset,
                        key.as_str()
                            .unwrap_or("Eror then printing key".as_error().as_str())
                    ))?;
                    print_value_recursive(formatter, &self.offset, item)?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  additional_config: {}",
                    self.offset,
                    self.additional_config.type_error(DICT).as_error()
                ))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
