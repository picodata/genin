use std::{cmp::Ordering, collections::HashSet};

use indexmap::IndexMap;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_yaml::{Number, Value};
use tabled::Alignment;

use crate::task::{AsError, ErrConfMapping, TypeError, BOOL, DICT, LIST, NUMBER, STRING};

use super::{
    hst::view::{TableColors, View},
    ins::{
        v2::{InstanceV2, InstanceV2Config, Instances},
        Role,
    },
    name::Name,
    TopologyMemberV1,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
                     InstanceV2 {
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
                                failure_domains: failure_domains.clone(),
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
                                        .map(|instance_num| InstanceV2 {
                                            name: name
                                                .clone_with_index(repliaset_num)
                                                .clone_with_index(instance_num),
                                            stateboard: None,
                                            weight: *weight,
                                            failure_domains: failure_domains.clone(),
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
                                        .collect::<Vec<InstanceV2>>()
                                } else {
                                    vec![InstanceV2 {
                                        name: name.clone_with_index(repliaset_num),
                                        stateboard: None,
                                        weight: *weight,
                                        failure_domains: failure_domains.clone(),
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
                            .collect::<Vec<InstanceV2>>()
                    },
                )
                .collect::<Vec<InstanceV2>>(),
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

impl From<Vec<TopologyMemberV1>> for Topology {
    fn from(members: Vec<TopologyMemberV1>) -> Self {
        Self(
            members
                .into_iter()
                .map(
                    |TopologyMemberV1 {
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
                            failure_domains: Vec::new(),
                            roles,
                            cartridge_extra_env: IndexMap::default(),
                            config: InstanceV2Config {
                                additional_config: config,
                                ..InstanceV2Config::default()
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
                failure_domains: Vec::new(),
                roles: vec![Role::router(), Role::failover_coordinator()],
                cartridge_extra_env: IndexMap::default(),
                config: InstanceV2Config::default(),
                vars: IndexMap::default(),
            },
            TopologySet {
                name: Name::from("storage"),
                replicasets_count: Some(2),
                replication_factor: Some(2),
                weight: None,
                failure_domains: Vec::new(),
                roles: vec![Role::storage()],
                cartridge_extra_env: IndexMap::default(),
                config: InstanceV2Config::default(),
                vars: IndexMap::default(),
            },
        ])
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
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
    #[serde(skip_serializing_if = "InstanceV2Config::is_none")]
    config: InstanceV2Config,
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
            config: Option<InstanceV2Config>,
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
                 mut roles,
                 all_rw,
                 cartridge_extra_env,
                 config,
                 vars,
             }| {
                // If type not defined in yaml let's try to infer based on name
                if roles.is_empty() {
                    roles = vec![Role::from(name.as_str())]
                }
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
        match (
            &self.failure_domains.is_empty(),
            &other.failure_domains.is_empty(),
        ) {
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            _ => Some(Ordering::Equal),
        }
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

#[derive(Deserialize)]
pub struct InvalidTopologySet {
    #[serde(skip)]
    pub offset: String,
    #[serde(default)]
    name: Value,
    #[serde(default)]
    replicasets_count: Value,
    #[serde(default)]
    replication_factor: Value,
    #[serde(default)]
    weight: Value,
    #[serde(default)]
    failure_domains: Value,
    #[serde(default)]
    roles: Value,
    #[serde(default)]
    all_rw: Value,
    #[serde(default)]
    cartridge_extra_env: Value,
    #[serde(default)]
    config: Value,
    #[serde(default)]
    vars: Value,
}

impl std::fmt::Debug for InvalidTopologySet {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("\n")?;
        // name: String
        match &self.name {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "{}- name: {}",
                    self.offset,
                    "Missing field 'name'".as_error()
                ))?;
                formatter.write_str("\n")?;
            }
            Value::String(name) => {
                formatter.write_fmt(format_args!("{}- name: {}", self.offset, name))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}- name: {}",
                    self.offset,
                    self.name.type_error(STRING).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // replicasets_count: usize
        match &self.replicasets_count {
            Value::Null => {}
            Value::Number(replicasets_count) if replicasets_count.is_u64() => {
                formatter.write_fmt(format_args!(
                    "{}  replicasets_count: {}",
                    self.offset, replicasets_count
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  replicasets_count: {}",
                    self.offset,
                    self.replicasets_count.type_error(NUMBER).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // replication_factor: usize
        match &self.replication_factor {
            Value::Null => {}
            Value::Number(replication_factor) if replication_factor.is_u64() => {
                formatter.write_fmt(format_args!(
                    "{}  replication_factor: {}",
                    self.offset, replication_factor
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  replication_factor: {}",
                    self.offset,
                    self.replication_factor.type_error(NUMBER).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // replication_factor: usize
        match &self.weight {
            Value::Null => {}
            Value::Number(weight) if weight.is_u64() => {
                formatter.write_fmt(format_args!("{}  weight: {}", self.offset, weight))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  weight: {}",
                    self.offset,
                    self.weight.type_error(NUMBER).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // failure_domains: Vec<String>
        match &self.failure_domains {
            Value::Null => {}
            Value::Sequence(failure_domains) => {
                formatter.write_fmt(format_args!(
                    "{}  failure_domains: {:?}",
                    self.offset,
                    ErrFailureDomains {
                        offset: format!("{}    ", &self.offset),
                        value: failure_domains
                    }
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  failure_domains: {}",
                    self.offset,
                    self.failure_domains.type_error(LIST).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // roles: Vec<Role>
        match &self.roles {
            Value::Null => {}
            Value::Sequence(roles) => {
                formatter.write_fmt(format_args!(
                    "{}  roles: {:?}",
                    self.offset,
                    ErrRoles {
                        offset: format!("{}    ", &self.offset),
                        value: roles
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  roles: {}",
                    self.offset,
                    self.roles.type_error(LIST).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // all_rw: bool
        match &self.all_rw {
            Value::Null => {}
            Value::Bool(all_rw) => {
                formatter.write_fmt(format_args!("{}  all_rw: {}", self.offset, all_rw))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  all_rw: {}",
                    self.offset,
                    self.all_rw.type_error(BOOL).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // cartridge_extra_env: IndexMap<String, Value>
        match &self.cartridge_extra_env {
            Value::Null => {}
            Value::Mapping(cartridge_extra_env) => {
                formatter.write_fmt(format_args!(
                    "{}  cartridge_extra_env: {:?}",
                    self.offset,
                    ErrConfMapping {
                        offset: format!("{}  ", self.offset),
                        value: cartridge_extra_env,
                    }
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  cartridge_extra_env: {}",
                    self.offset,
                    self.cartridge_extra_env.type_error(DICT).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        match &self.config {
            Value::Null => {}
            config @ Value::Mapping(_) => formatter.write_fmt(format_args!(
                "{}  config: {:?}",
                &self.offset,
                serde_yaml::from_value::<ErrInstanceV2Config>(config.clone())
                    .map(|mut config| {
                        config.offset = format!("{}  ", &self.offset);
                        config
                    })
                    .unwrap()
            ))?,
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  config: {}",
                    self.offset,
                    self.config.type_error(DICT).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // vars: IndexMap<String, Value>
        match &self.vars {
            Value::Null => {}
            Value::Mapping(vars) => {
                formatter.write_fmt(format_args!(
                    "{}  vars: {:?}",
                    self.offset,
                    ErrConfMapping {
                        offset: format!("{}  ", self.offset),
                        value: vars,
                    }
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  vars: {}",
                    self.offset,
                    self.vars.type_error(DICT).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        Ok(())
    }
}

struct ErrFailureDomains<'a> {
    offset: String,
    value: &'a Vec<Value>,
}

impl<'a> std::fmt::Debug for ErrFailureDomains<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("\n")?;
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

struct ErrRoles<'a> {
    offset: String,
    value: &'a Vec<Value>,
}

impl<'a> std::fmt::Debug for ErrRoles<'a> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("\n")?;
        self.value.iter().try_for_each(|value| match value {
            Value::String(role) => {
                formatter.write_fmt(format_args!("{}- {}", &self.offset, role))?;
                formatter.write_str("\n")
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}- {}",
                    &self.offset,
                    value.type_error(STRING)
                ))?;
                formatter.write_str("\n")
            }
        })?;

        Ok(())
    }
}

#[derive(Deserialize, Default)]
pub struct ErrInstanceV2Config {
    #[serde(skip)]
    offset: String,
    #[serde(default)]
    http_port: Value,
    #[serde(default)]
    binary_port: Value,
    #[serde(default)]
    all_rw: Value,
    #[serde(default)]
    zone: Value,
    #[serde(default)]
    vshard_group: Value,
    #[serde(default)]
    additional_config: Value,
}

impl std::fmt::Debug for ErrInstanceV2Config {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("\n")?;

        // http_port: u16
        match &self.http_port {
            Value::Null => {}
            Value::Number(http_port) => {
                if http_port > &Number::from(0) && http_port < &Number::from(u16::MAX) {
                    formatter
                        .write_fmt(format_args!("{}  http_port: {}", self.offset, http_port))?;
                    formatter.write_str("\n")?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}  http_port: {}",
                        self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                    formatter.write_str("\n")?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  http_port: {}",
                    self.offset,
                    self.http_port.type_error(NUMBER).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // binary_port: u16
        match &self.binary_port {
            Value::Null => {}
            Value::Number(binary_port) => {
                if binary_port > &Number::from(0) && binary_port < &Number::from(u16::MAX) {
                    formatter
                        .write_fmt(format_args!("{}  http_port: {}", self.offset, binary_port))?;
                    formatter.write_str("\n")?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}  binary_port: {}",
                        self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                    formatter.write_str("\n")?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  binary_port: {}",
                    self.offset,
                    self.binary_port.type_error(NUMBER).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // all_rw: bool
        match &self.all_rw {
            Value::Null => {}
            Value::Bool(all_rw) => {
                formatter.write_fmt(format_args!("{}  all_rw: {}", self.offset, all_rw))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  all_rw: {}",
                    self.offset,
                    self.all_rw.type_error(BOOL).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // zone: String
        match &self.zone {
            Value::Null => {}
            Value::String(zone) => {
                formatter.write_fmt(format_args!("{}  zone: {}", self.offset, zone))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  zone: {}",
                    self.offset,
                    self.zone.type_error(STRING).as_error()
                ))?;
                formatter.write_str("\n")?;
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
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  vshard_group: {}",
                    self.offset,
                    self.vshard_group.type_error(STRING).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        // additional_config: IndexMap<String, Value>
        match &self.additional_config {
            Value::Null => {}
            Value::Mapping(additional_config) => {
                formatter.write_fmt(format_args!(
                    "{}  additional_config: {:?}",
                    self.offset,
                    ErrConfMapping {
                        offset: format!("{}  ", self.offset),
                        value: additional_config,
                    }
                ))?;
                formatter.write_str("\n")?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  additional_config: {}",
                    self.offset,
                    self.additional_config.type_error(DICT).as_error()
                ))?;
                formatter.write_str("\n")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
