pub mod fs;
pub mod hst;
pub mod ins;
pub mod name;
pub mod topology;

use clap::ArgMatches;
use indexmap::IndexMap;
use log::{debug, trace};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::fmt::Display;
use std::net::IpAddr;
use tabled::Alignment;

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::hst::v1::Host;
use crate::task::cluster::hst::v2::{HostV2, HostV2Config, WithHosts};
use crate::task::cluster::hst::view::{View, BG_BRIGHT_BLACK};
use crate::task::cluster::ins::v2::{InstanceV2, InstanceV2Config, Instances};
use crate::task::cluster::ins::Role;
use crate::task::cluster::name::Name;
use crate::task::cluster::topology::Topology;
use crate::task::flv::Failover;
use crate::task::flv::{Mode, StateProvider, StateboardParams};
use crate::task::inventory::{Child, HostVars, Inventory};
use crate::task::vars::Vars;
use crate::{DEFAULT_BINARY_PORT, DEFAULT_HTTP_PORT};

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
pub struct Cluster {
    pub topology: Topology,
    pub hosts: HostV2,
    pub failover: Failover,
    pub vars: Vars,
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
            topology: Topology::default(),
            hosts: HostV2::from("cluster")
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
                .with_config(HostV2Config::from((8081, 3031))),
            failover: Failover {
                mode: Mode::Stateful,
                state_provider: StateProvider::Stateboard,
                failover_variants: super::flv::FailoverVariants::StateboardVariant(
                    StateboardParams {
                        uri: super::flv::Uri {
                            address: hst::v2::Address::Ip(IpAddr::from([192, 168, 16, 11])),
                            port: 4401,
                        },
                        password: String::from("password"),
                    },
                ),
            },
            vars: Default::default(),
        }
        .spread()
    }
}

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.hosts)
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
                topology: Topology::from(Instances::from(
                    inventory
                        .all
                        .hosts
                        .iter()
                        .filter(|(_, host)| !host.stateboard)
                        .map(|(name, inventory_host)| {
                            let mut instance = InstanceV2::from((name, inventory_host)).with_roles(
                                inventory
                                    .all
                                    .children
                                    .get(&name.get_parent_name().clone_with_index("replicaset"))
                                    .map(|replicaset| match replicaset {
                                        Child::Replicaset { vars, .. } => vars.roles.clone(),
                                        _ => unreachable!(),
                                    })
                                    .unwrap(),
                            );

                            instance.config.http_port = None;
                            instance.config.binary_port = None;

                            instance
                        })
                        .collect::<Vec<InstanceV2>>(),
                )),
                hosts: HostV2::from("cluster").with_hosts(
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
                                instances: Instances::from(
                                    inventory
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
                                                    config: InstanceV2Config::from(
                                                        &instance.config,
                                                    ),
                                                    vars: instance.vars.clone(),
                                                    view: View::default(),
                                                })
                                            } else {
                                                None
                                            }
                                        })
                                        .collect::<Vec<InstanceV2>>(),
                                ),
                            }),
                            Child::Replicaset { .. } => None,
                        })
                        .collect::<Vec<HostV2>>(),
                ),
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
                topology: Topology,
                hosts: Vec<HostV2>,
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
            } => Cluster {
                hosts: HostV2::from("cluster")
                    .with_hosts(hosts)
                    .with_http_port(DEFAULT_HTTP_PORT)
                    .with_binary_port(DEFAULT_BINARY_PORT),
                topology: Topology::from(instances),
                failover,
                vars,
            }
            .spread(),
            ClusterHelper::V2 {
                topology,
                hosts,
                failover,
                vars,
            } => Cluster {
                hosts: HostV2::from("cluster")
                    .with_hosts(hosts)
                    .with_http_port(DEFAULT_HTTP_PORT)
                    .with_binary_port(DEFAULT_BINARY_PORT),
                topology,
                failover,
                vars,
            }
            .spread(),
        })
    }
}

impl Serialize for Cluster {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Cluster", 4)?;
        state.serialize_field("topology", &self.topology)?;
        state.serialize_field("hosts", &self.hosts.hosts)?;
        state.serialize_field("failover", &self.failover)?;

        let mut vars = self.vars.clone();
        vars.cartridge_failover_params = None;

        state.serialize_field("vars", &vars)?;
        state.end()
    }
}

impl Cluster {
    pub fn spread(self) -> Self {
        Self {
            hosts: self
                .hosts
                .with_instances(Instances::from(&self.topology))
                .spread()
                .with_stateboard(&self.failover)
                .spread(),
            topology: self.topology,
            failover: self.failover,
            vars: self.vars,
        }
    }

    pub fn try_upgrade(mut self, new: &Cluster) -> Result<Self, GeninError> {
        let old_hosts = self.hosts.lower_level_hosts();
        let new_hosts = new.hosts.lower_level_hosts();

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

        self.hosts.delete_stateboard();
        self.hosts.merge(&new.hosts);

        self.hosts.instances = Instances::from(
            diff.iter()
                .map(
                    |InstanceV2 {
                         name,
                         stateboard,
                         weight,
                         failure_domains,
                         roles,
                         config,
                         vars,
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
                        vars: vars.clone(),
                        view: View {
                            alignment: Alignment::left(),
                            color: BG_BRIGHT_BLACK,
                        },
                    },
                )
                .collect::<Vec<InstanceV2>>(),
        );

        self.hosts = self.hosts.with_stateboard(&self.failover);
        self.hosts = self.hosts.spread();

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
                hosts: Vec::default(),
                instances: Instances::default(),
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
            instances: Instances::default(),
        }
    }
}

#[allow(unused)]
struct TopologyMemberV1 {
    name: Name,
    count: usize,
    replicas: usize,
    weight: usize,
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
                 count,
                 replicas,
                 weight,
                 mut roles,
                 config,
                 ..
             }| {
                if roles.is_empty() {
                    roles = vec![Role::from(name.as_str())]
                }
                TopologyMemberV1 {
                    name: Name::from(name),
                    count,
                    replicas,
                    weight,
                    roles,
                    config,
                }
            },
        )
    }
}

#[cfg(test)]
mod test;
