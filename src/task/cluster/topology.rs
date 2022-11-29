use std::cmp::Ordering;

use indexmap::IndexMap;
use log::trace;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use tabled::Alignment;

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

impl From<Instances> for Topology {
    fn from(instances: Instances) -> Self {
        Self(
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
                         config,
                         vars,
                         ..
                     }| {
                        trace!(
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
                            config,
                            vars,
                            ..
                        },
                    )| {
                        trace!(
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
        )
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
            config: InstanceV2Config,
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
                    config,
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

#[cfg(test)]
mod test;
