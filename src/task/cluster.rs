pub(in crate::task) mod fd;
pub(in crate::task) mod fs;
pub(in crate::task) mod hst;
pub(in crate::task) mod ins;
pub mod name;

use clap::ArgMatches;
use indexmap::IndexMap;
use log::{debug, trace};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::cmp::Ordering;
use std::fmt::Display;
use std::net::IpAddr;

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::hst::v1::Host;
use crate::task::cluster::hst::v2::{HostV2, HostV2Config};
use crate::task::cluster::ins::v2::{InstanceV2, InstanceV2Config, Replicaset};
use crate::task::cluster::ins::{Role, Type};
use crate::task::cluster::name::Name;
use crate::task::flv::Failover;
use crate::task::inventory::{Child, HostVars, Inventory};
use crate::task::vars::Vars;

#[derive(Debug, PartialEq, Eq)]
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
///     hosts: HostV2, //TODO
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
pub(in crate::task) struct Cluster {
    replicasets: Vec<Replicaset>,
    hosts: Vec<HostV2>,
    failover: Failover,
    vars: Vars,
}

impl Default for Cluster {
    /// Host can be Region, Datacenter, Server
    /// ```yaml
    /// hosts:
    ///   - name: selectel
    ///     hosts:
    ///       - name: moscow
    ///         config:
    ///           http_port: 8091
    ///           binary_port: 3031
    ///           distance: 10
    ///         hosts:
    ///           - name: server-1
    ///             config:
    ///               address: 192.168.99.11
    ///           - name: server-2
    ///             config:
    ///               address: 192.168.99.12
    ///     - name: kaukaz
    ///       config:
    ///         distance: 20
    ///       hosts:
    ///         - name: server-3
    ///           config:
    ///             http_port: 8191
    ///             binary_port: 3131
    ///             ip: 10.99.3.100
    /// ```
    fn default() -> Self {
        let replicasets = vec![
            Replicaset {
                name: Name::from("router").with_index(1),
                replicasets_count: Some(1),
                replication_factor: None,
                weight: None,
                failure_domains: Vec::new(),
                roles: vec![Role::router(), Role::failover_coordinator()],
                config: InstanceV2Config::default(),
            },
            Replicaset {
                name: Name::from("storage").with_index(1),
                replicasets_count: Some(2),
                replication_factor: Some(2),
                weight: None,
                failure_domains: Vec::new(),
                roles: vec![Role::storage()],
                config: InstanceV2Config::default(),
            },
            Replicaset {
                name: Name::from("storage").with_index(2),
                replicasets_count: Some(2),
                replication_factor: Some(2),
                weight: None,
                failure_domains: Vec::new(),
                roles: vec![Role::storage()],
                config: InstanceV2Config::default(),
            },
        ];
        let mut host = HostV2::from("cluster")
            .with_hosts(vec![HostV2::from("datacenter-1")
                .with_hosts(vec![
                    HostV2::from("server-1").with_config(
                        HostV2Config::from(IpAddr::from([192, 168, 16, 11]))
                            .with_ports((8081, 3031)),
                    ),
                    HostV2::from("server-2").with_config(
                        HostV2Config::from(IpAddr::from([192, 168, 16, 12]))
                            .with_ports((8081, 3031)),
                    ),
                ])
                .with_config(HostV2Config::from((8081, 3031)))])
            .with_config(HostV2Config::from((8081, 3031)))
            .with_instances(
                replicasets
                    .iter()
                    .flat_map(|replicaset| replicaset.instances())
                    .collect(),
            );
        let failover = Failover::default();
        if let Some(stb) = failover.as_stateboard() {
            host.push_stateboard(stb);
        }
        host.spread();
        Self {
            replicasets,
            hosts: vec![host],
            failover,
            vars: Default::default(),
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        trace!("{:?}", self.hosts);
        write!(f, "{}", &self.hosts.first().unwrap().to_string())
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Cluster {
    type Error = GeninError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        trace!("Сluster file will be constructed based on default values and Genin call arguments");
        let failover = Failover::try_from(args)?;
        Ok(Cluster {
            vars: Vars::from(&failover),
            failover,
            ..Cluster::default()
        })
    }
}

impl<'a> TryFrom<&'a Option<Inventory>> for Cluster {
    type Error = GeninError;

    fn try_from(inventory: &'a Option<Inventory>) -> Result<Self, Self::Error> {
        if let Some(inventory) = inventory {
            Ok(Cluster {
                replicasets: inventory
                    .all
                    .hosts
                    .iter()
                    .filter(|(_, host)| !host.stateboard)
                    .fold(IndexMap::new(), |mut accum, (name, instance)| {
                        trace!(
                            "{} {:?} {:?}",
                            name,
                            name.parent_index_as_usize(),
                            name.last_index_as_usize()
                        );
                        let entry =
                            accum
                                .entry(Name::from(name.get_ancestor()))
                                .or_insert(Replicaset {
                                    name: Name::from(name.get_ancestor()),
                                    replicasets_count: Some(1),
                                    replication_factor: None,
                                    weight: None,
                                    failure_domains: Vec::new(),
                                    roles: inventory
                                        .all
                                        .children
                                        .get(&name.get_parent_name().clone_with_index("replicaset"))
                                        .map(|replicaset| match replicaset {
                                            Child::Replicaset { vars, .. } => vars.roles.clone(),
                                            _ => unreachable!(),
                                        })
                                        .unwrap(),
                                    config: InstanceV2Config::from(&instance.config).clean_ports(),
                                });
                        //TODO: Refactor this in future
                        match name.len() {
                            2 => {
                                if let Some(cnt) = entry.replicasets_count.as_mut() {
                                    *cnt = name.last_index_as_usize().unwrap().max(*cnt);
                                }
                            }
                            3 => {
                                if let Some(cnt) = entry.replicasets_count.as_mut() {
                                    *cnt = name.parent_index_as_usize().unwrap().max(*cnt);
                                }
                                if let Some(cnt) = entry.replication_factor.as_mut() {
                                    *cnt = name.last_index_as_usize().unwrap().max(*cnt);
                                } else {
                                    entry.replication_factor =
                                        Some(name.last_index_as_usize().unwrap());
                                }
                            }
                            _ => {}
                        }
                        accum
                    })
                    .into_values()
                    .collect(),
                hosts: vec![HostV2::from("cluster").with_hosts(
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
                            } => Some(HostV2 {
                                name: name.clone(),
                                config: HostV2Config::from(ansible_host.clone())
                                    .with_additional_config(additional_config.clone())
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
                                instances: inventory
                                    .all
                                    .hosts
                                    .iter()
                                    .filter_map(|(name, instance)| {
                                        let config = HostV2Config::from(&instance.config);
                                        trace!(
                                            "ansible_host: {} instance_address: {}",
                                            ansible_host,
                                            config.address()
                                        );
                                        if ansible_host.eq(&config.address()) {
                                            Some(InstanceV2 {
                                                name: name.clone(),
                                                stateboard: instance.stateboard.then_some(true),
                                                weight: None,
                                                failure_domains: Vec::new(),
                                                roles: Vec::new(),
                                                config: InstanceV2Config::from(&instance.config),
                                            })
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            }),
                            Child::Replicaset { .. } => None,
                        })
                        .collect(),
                )],
                failover: inventory
                    .all
                    .vars
                    .cartridge_failover_params
                    .clone()
                    .ok_or_else(|| {
                        GeninError::new(
                            GeninErrorKind::EmptyField,
                            "inventory vars does not have cartridge_failover_params field",
                        )
                    })?,
                vars: inventory.all.vars.clone(),
            })
        } else {
            Err(GeninError::new(
                GeninErrorKind::EmptyField,
                "the cluster cannot be built from the inventory \
                because the inventory field is empty",
            ))
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
            V1 {
                instances: Vec<TopologyMemberV1>,
                hosts: Vec<Host>,
                #[serde(default)]
                failover: Failover,
                vars: Vars,
            },
            V2 {
                topology: Vec<TopologyMemberV2>,
                hosts: Vec<HostV2Helper>,
                #[serde(default)]
                failover: Failover,
                vars: Vars,
            },
        }

        ClusterHelper::deserialize(deserializer).map(|cluster| match cluster {
            ClusterHelper::V1 {
                instances,
                hosts,
                failover,
                vars,
            } => {
                let mut replicasets: Vec<Replicaset> = instances
                    .into_iter()
                    .flat_map(|member| member.to_replicasets())
                    .collect();
                let mut host = HostV2::from(hosts).with_config(HostV2Config::from((8081, 3031)));
                host.instances = replicasets
                    .iter_mut()
                    .flat_map(|replicaset| replicaset.instances())
                    .collect();
                if let Some(stb) = failover.as_stateboard() {
                    host.push_stateboard(stb);
                }
                host.spread();
                Cluster {
                    replicasets,
                    hosts: vec![host],
                    failover,
                    vars,
                }
            }
            ClusterHelper::V2 {
                mut topology,
                hosts,
                failover,
                vars,
            } => {
                topology.sort();
                let mut replicasets: Vec<Replicaset> = topology
                    .into_iter()
                    .flat_map(|member| member.to_replicasets())
                    .collect();
                let mut host = HostV2::from("cluster")
                    .with_hosts(
                        hosts
                            .into_iter()
                            .map(|host| {
                                let name = Name::from("cluster").with_raw_index(host.name.as_str());
                                host.into_host_v2(name)
                            })
                            .collect(),
                    )
                    .with_config(HostV2Config::from((8081, 3031)));
                host.instances = replicasets
                    .iter_mut()
                    .flat_map(|replicaset| replicaset.instances())
                    .collect();
                if let Some(stb) = failover.as_stateboard() {
                    host.push_stateboard(stb);
                }
                host.spread();
                Cluster {
                    replicasets,
                    hosts: vec![host],
                    failover,
                    vars,
                }
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
        state.serialize_field(
            "topology",
            &TopologyMemberV2::from(self.replicasets.clone()),
        )?;
        state.serialize_field("hosts", &self.hosts.first().unwrap().hosts)?;
        state.serialize_field("failover", &self.failover)?;

        let mut vars = self.vars.clone();
        vars.cartridge_failover_params = None;

        state.serialize_field("vars", &vars)?;
        state.end()
    }
}

#[allow(unused)]
impl Cluster {
    pub fn replicasets(&self) -> &Vec<Replicaset> {
        &self.replicasets
    }

    pub fn hosts(&self) -> &Vec<HostV2> {
        &self.hosts
    }

    pub fn vars(&self) -> &Vars {
        &self.vars
    }

    pub fn failover(&self) -> &Failover {
        &self.failover
    }

    pub fn try_upgrade(mut self, new: &Cluster) -> Result<Self, GeninError> {
        let old_hosts = self
            .hosts()
            .first()
            .ok_or_else(|| {
                GeninError::new(
                    GeninErrorKind::EmptyField,
                    "The top-level array with hosts is empty! \
                All looks like the hosts config is empty.",
                )
            })?
            .lower_level_hosts();
        let new_hosts = new
            .hosts()
            .first()
            .ok_or_else(|| {
                GeninError::new(
                    GeninErrorKind::EmptyField,
                    "The top-level array with hosts is empty! \
                All looks like the hosts config is empty.",
                )
            })?
            .lower_level_hosts();

        let mut diff = new_hosts
            .into_iter()
            .flat_map(|new_host| {
                new_host.instances.iter().filter(|new_instance| {
                    !old_hosts.iter().any(|old_host| {
                        old_host
                            .instances
                            .iter()
                            .any(|old_instance| old_instance.name.eq(&new_instance.name))
                    })
                })
            })
            .collect::<Vec<&InstanceV2>>();

        diff.sort();

        debug!(
            "New instances: {}",
            diff.iter()
                .map(|instance| instance.name.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        self.failover = new.failover.clone();
        self.vars = new.vars.clone();

        if let Some(host) = self.hosts.first_mut() {
            host.delete_stateboard();
            host.merge(new.hosts.first().ok_or_else(|| {
                GeninError::new(
                    GeninErrorKind::EmptyField,
                    "The top-level array with hosts is empty! \
                        All looks like the hosts config is empty.",
                )
            })?);

            host.instances = diff
                .iter()
                .map(
                    |InstanceV2 {
                         name,
                         stateboard,
                         weight,
                         failure_domains,
                         roles,
                         config,
                         ..
                     }| InstanceV2 {
                        name: name.clone(),
                        stateboard: *stateboard,
                        weight: *weight,
                        failure_domains: failure_domains.clone(),
                        roles: roles.clone(),
                        config: InstanceV2Config {
                            http_port: None,
                            binary_port: None,
                            ..config.clone()
                        },
                    },
                )
                .collect();

            if let Some(stateboard) = new.failover.as_stateboard() {
                host.push_stateboard(stateboard);
            }

            host.spread();
        }

        Ok(self)
    }
}

#[derive(Deserialize)]
struct HostV2Helper {
    name: String,
    #[serde(default)]
    config: HostV2Config,
    #[serde(default)]
    hosts: Vec<HostV2Helper>,
}

impl From<HostV2Helper> for HostV2 {
    fn from(helper: HostV2Helper) -> Self {
        let name = Name::from(helper.name.as_str());
        helper.into_host_v2(name)
    }
}

impl HostV2Helper {
    fn into_host_v2(self, name: Name) -> HostV2 {
        if self.hosts.is_empty() {
            return HostV2 {
                name,
                config: self.config,
                hosts: Vec::new(),
                instances: Vec::new(),
            };
        }

        HostV2 {
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
            instances: Vec::new(),
        }
    }
}

#[allow(unused)]
struct TopologyMemberV1 {
    name: Name,
    itype: Type,
    count: usize,
    replicas: usize,
    weight: usize,
    stateboard: bool,
    roles: Vec<Role>,
    config: IndexMap<String, Value>,
}

impl<'de> Deserialize<'de> for TopologyMemberV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            #[serde(default, rename = "type")]
            itype: Type,
            #[serde(default)]
            count: usize,
            #[serde(default)]
            replicas: usize,
            #[serde(default)]
            weight: usize,
            #[serde(default)]
            roles: Vec<Role>,
            #[serde(default)]
            config: IndexMap<String, Value>,
        }

        Helper::deserialize(deserializer).map(
            |Helper {
                 name,
                 mut itype,
                 count,
                 replicas,
                 weight,
                 mut roles,
                 config,
                 ..
             }| {
                if itype == Type::Unknown {
                    itype = Type::from(name.as_str());
                }

                if roles.is_empty() {
                    roles = vec![Role::from(name.as_str())]
                }
                if itype == Type::Storage {
                    TopologyMemberV1 {
                        name: Name::from(name),
                        itype,
                        count,
                        replicas,
                        weight,
                        roles,
                        config,
                        stateboard: false,
                    }
                } else {
                    TopologyMemberV1 {
                        itype: Type::from(name.as_str()),
                        name: Name::from(name),
                        count,
                        replicas: 0,
                        weight,
                        roles,
                        config,
                        stateboard: false,
                    }
                }
            },
        )
    }
}

impl TopologyMemberV1 {
    fn to_replicasets(&self) -> Vec<Replicaset> {
        (1..=self.count)
            .map(|index| Replicaset {
                name: self.name.clone_with_index(index),
                replicasets_count: Some(self.count),
                replication_factor: TopologyMemberV1::as_replication_factor(self.replicas),
                weight: TopologyMemberV1::as_weight(self.weight),
                failure_domains: Vec::new(),
                roles: self.roles.clone(),
                config: InstanceV2Config::from(&self.config),
            })
            .collect()
    }

    fn as_replication_factor(count: usize) -> Option<usize> {
        if count == 0 {
            return None;
        }
        Some(count + 1)
    }

    fn as_weight(weight: usize) -> Option<usize> {
        if weight == 0 {
            return None;
        }
        Some(weight)
    }
}

#[derive(Serialize, Debug, PartialEq, Eq)]
struct TopologyMemberV2 {
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
}

impl<'de> Deserialize<'de> for TopologyMemberV2 {
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
             }| {
                // If type not defined in yaml let's try to infer based on name
                if roles.is_empty() {
                    roles = vec![Role::from(name.as_str())]
                }
                TopologyMemberV2 {
                    name: Name::from(name),
                    replicasets_count,
                    replication_factor,
                    weight,
                    failure_domains,
                    roles,
                    config,
                }
            },
        )
    }
}

impl PartialOrd for TopologyMemberV2 {
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

impl Ord for TopologyMemberV2 {
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

impl TopologyMemberV2 {
    fn from(replicasets: Vec<Replicaset>) -> Vec<TopologyMemberV2> {
        replicasets
            .iter()
            .fold(
                IndexMap::<String, TopologyMemberV2>::new(),
                |mut acc, replicaset| {
                    trace!("Repliacest {}", replicaset.name);
                    acc.entry(replicaset.name.get_ancestor().to_string())
                        .or_insert(TopologyMemberV2 {
                            name: Name::from(replicaset.name.get_ancestor()),
                            replicasets_count: replicaset.replicasets_count,
                            replication_factor: replicaset.replication_factor,
                            weight: replicaset.weight,
                            failure_domains: replicaset.failure_domains.clone(),
                            roles: replicaset.roles.clone(),
                            config: replicaset.config.clone(),
                        });
                    acc
                },
            )
            .into_iter()
            .map(|(_, value)| value)
            .collect()
    }

    fn to_replicasets(&self) -> Vec<Replicaset> {
        (1..=self.replicasets_count.unwrap_or(1))
            .map(|index| Replicaset {
                name: self.name.clone_with_index(index),
                replicasets_count: self.replicasets_count,
                replication_factor: self.replication_factor,
                weight: self.weight,
                failure_domains: self.failure_domains.clone(),
                roles: self.roles.clone(),
                config: self.config.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod test;
