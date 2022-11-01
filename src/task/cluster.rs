pub(in crate::task) mod fd;
pub(in crate::task) mod fs;
pub(in crate::task) mod hst;
pub(in crate::task) mod ins;

use std::fmt::Display;
use std::net::IpAddr;

use crate::error::GeninError;
use crate::task::vrs::Vars;
use clap::ArgMatches;
use indexmap::IndexMap;
use log::trace;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use crate::task::cluster::hst::v1::Host;
use crate::task::flv::Failover;

use crate::task::cluster::hst::v2::{HostV2, HostV2Config};
use crate::task::cluster::ins::{Role, Type};

use self::ins::v2::Replicaset;
use self::ins::{Config, Name};

use super::inv::Inventory;

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
        Self {
            replicasets: vec![
                Replicaset {
                    name: Name::from("router").with_index(1),
                    replicasets_count: Some(1),
                    replication_factor: None,
                    weight: None,
                    zone: None,
                    roles: vec![Role::router(), Role::failover_coordinator()],
                    config: Config::default(),
                    instances: Vec::new(),
                },
                Replicaset {
                    name: Name::from("storage").with_index(1),
                    replicasets_count: Some(2),
                    replication_factor: Some(1),
                    weight: None,
                    zone: None,
                    roles: vec![Role::storage()],
                    config: Config::default(),
                    instances: Vec::new(),
                },
                Replicaset {
                    name: Name::from("storage").with_index(2),
                    replicasets_count: Some(2),
                    replication_factor: Some(1),
                    weight: None,
                    zone: None,
                    roles: vec![Role::storage()],
                    config: Config::default(),
                    instances: Vec::new(),
                },
            ],
            hosts: vec![
                HostV2::from("cluster").with_hosts(vec![HostV2::from("datacenter-1")
                    .with_config(HostV2Config::from((8081, 3301)))
                    .with_hosts(vec![
                        HostV2::from("server-1")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 11]))),
                        HostV2::from("server-2")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 12]))),
                    ])]),
            ],
            failover: Default::default(),
            vars: Default::default(),
        }
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.hosts.first().unwrap())
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Cluster {
    type Error = GeninError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        trace!("Сluster file will be constructed based on default values and Genin call arguments");
        Ok(Cluster {
            failover: Failover::try_from(args)?,
            ..Cluster::default()
        })
    }
}

impl<'a> TryFrom<&'a Option<Inventory>> for Cluster {
    type Error = GeninError;

    fn try_from(value: &'a Option<Inventory>) -> Result<Self, Self::Error> {
        todo!()
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
                let mut host = HostV2::from(hosts);
                host.instances = replicasets
                    .iter_mut()
                    .flat_map(|replicaset| replicaset.instances())
                    .collect();
                host.spread();
                Cluster {
                    replicasets,
                    hosts: vec![host],
                    failover,
                    vars,
                }
            }
            ClusterHelper::V2 {
                topology,
                hosts,
                failover,
                vars,
            } => {
                let mut replicasets: Vec<Replicaset> = topology
                    .into_iter()
                    .flat_map(|member| member.to_replicasets())
                    .collect();
                let mut host = HostV2::from("cluster")
                    .with_hosts(hosts.into_iter().map(|host| host.into_v2()).collect());
                host.instances = replicasets
                    .iter_mut()
                    .flat_map(|replicaset| replicaset.instances())
                    .collect();
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

        #[derive(Serialize)]
        struct ClusterVars<'a> {
            ansible_user: &'a String,
            ansible_password: &'a String,
            cartridge_app_name: &'a String,
            cartridge_cluster_cookie: &'a String,
            cartridge_package_path: &'a String,
            cartridge_bootstrap_vshard: &'a bool,
            #[serde(flatten)]
            another_fields: &'a Value,
        }

        state.serialize_field(
            "vars",
            &ClusterVars {
                ansible_user: &self.vars.ansible_user,
                ansible_password: &self.vars.ansible_password,
                cartridge_app_name: &self.vars.cartridge_app_name,
                cartridge_cluster_cookie: &self.vars.cartridge_cluster_cookie,
                cartridge_package_path: &self.vars.cartridge_package_path,
                cartridge_bootstrap_vshard: &self.vars.cartridge_bootstrap_vshard,
                another_fields: &self.vars.another_fields,
            },
        )?;
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
}

#[derive(Deserialize)]
struct HostV2Helper {
    name: String,
    #[serde(default)]
    config: HostV2Config,
    #[serde(default)]
    hosts: Vec<HostV2Helper>,
}

impl HostV2Helper {
    fn into_v2(self) -> HostV2 {
        if self.hosts.is_empty() {
            return HostV2 {
                name: self.name,
                config: self.config,
                hosts: Vec::new(),
                instances: Vec::new(),
            };
        }

        HostV2 {
            name: self.name,
            config: self.config,
            instances: Vec::new(),
            hosts: self.hosts.into_iter().map(|host| host.into_v2()).collect(),
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
            #[serde(rename = "type")]
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
                        weight: 0,
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
                replication_factor: TopologyMemberV1::as_option(self.replicas),
                weight: TopologyMemberV1::as_option(self.weight),
                zone: None,
                roles: self.roles.clone(),
                config: Config::from(self.config.clone()),
                instances: Vec::new(),
            })
            .collect()
    }

    fn as_option(u: usize) -> Option<usize> {
        if u == 0 {
            return None;
        }
        Some(u)
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
    #[serde(skip_serializing_if = "Option::is_none")]
    zone: Option<String>,
    #[serde(default)]
    roles: Vec<Role>,
    #[serde(skip_serializing_if = "Config::is_empty")]
    config: Config,
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
            zone: Option<String>,
            #[serde(default)]
            roles: Vec<Role>,
            #[serde(default)]
            config: Config,
        }

        Helper::deserialize(deserializer).map(
            |Helper {
                 name,
                 replicasets_count,
                 replication_factor,
                 weight,
                 zone,
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
                    zone,
                    roles,
                    config,
                }
            },
        )
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
                            zone: replicaset.zone.clone(),
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
                zone: self.zone.clone(),
                roles: self.roles.clone(),
                config: self.config.clone(),
                instances: Vec::new(),
            })
            .collect()
    }
}

#[cfg(test)]
mod test;
