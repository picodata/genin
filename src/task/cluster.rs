pub(in crate::task) mod fs;
pub(in crate::task) mod hst;
pub(in crate::task) mod ins;
pub(in crate::task) mod scheme;

use crate::error::{ConfigError, TaskError};
use crate::task::vrs::Vars;
use clap::ArgMatches;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::task::cluster::hst::v1::Host;
use crate::task::cluster::ins::v1::Instance;
use crate::task::flv::Failover;

use crate::traits::{Functor, MapSelf};

use crate::task::cluster::hst::{
    v2::{HostV2, HostsVariantsV2},
    HostType, Ports, PortsVariants, IP,
};
use crate::task::cluster::ins::{v2::InstanceV2, Role, Type};

use self::ins::IntoV2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
/// Cluster is a `genin` specific configuration file
/// ```rust
/// Cluster {
///     // Array of instances in free order
///     // instances:
///     // - name: "catalogue"
///     //   type: "storage"
///     //   count: 1
///     //   replicas: 2
///     //   weight: 10
///     instance: Instaces
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
///     hosts: Hosts,
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
pub(in crate::task) enum Cluster {
    V1 {
        instances: Vec<Instance>,
        hosts: Vec<Host>,
        #[serde(default)]
        failover: Failover,
        vars: Vars,
    },
    V2 {
        topology: Vec<InstanceV2>,
        hosts: Vec<HostV2>,
        #[serde(default)]
        failover: Failover,
        vars: Vars,
    },
}

impl Default for Cluster {
    /// Host can be Region, Datacenter, Server
    /// ```yaml
    /// hosts:
    ///     - name: kaukaz
    ///       type: region
    ///       distance: 10
    ///       ports:
    ///         http: 8091
    ///         binary: 3031
    ///       hosts:
    ///         - name: dc-1
    ///           type: datacenter
    ///           hosts:
    ///             - name: server-1
    ///               ip: 10.20.3.100
    ///         - name: dc-2
    ///           type: datacenter
    ///           hosts:
    ///             - name: server-1
    ///               ip: 10.20.4.100
    ///     - name: moscow
    ///       type: region
    ///       distance: 20
    ///       hosts:
    ///         - name: dc-3
    ///           type: datacenter
    ///           ports:
    ///             http: 8091
    ///             binary: 3031
    ///           hosts:
    ///             - name: server-10
    ///               ip: 10.99.3.100
    /// ```
    fn default() -> Self {
        Cluster::V2 {
            topology: vec![
                InstanceV2 {
                    name: "router".into(),
                    parent: "router".into(),
                    itype: Type::Router,
                    replicasets_count: 1,
                    replication_factor: 0,
                    weight: 0,
                    stateboard: false,
                    roles: vec![Role::router(), Role::failover_coordinator()],
                    config: IndexMap::new(),
                },
                InstanceV2 {
                    name: "storage".into(),
                    parent: "storage".into(),
                    itype: Type::Storage,
                    replicasets_count: 2,
                    replication_factor: 1,
                    weight: 10,
                    stateboard: false,
                    roles: vec![Role::storage()],
                    config: IndexMap::new(),
                },
            ],
            hosts: vec![HostV2 {
                name: "selectel".into(),
                htype: HostType::Datacenter,
                distance: 0,
                ports: PortsVariants::Ports(Ports::default()),
                ip: IP::None,
                hosts: HostsVariantsV2::Hosts(vec![
                    HostV2 {
                        name: "host-1".into(),
                        htype: HostType::Server,
                        distance: 0,
                        ports: PortsVariants::None,
                        ip: IP::Server(
                            "192.168.16.11"
                                .parse()
                                .expect("Error then parsing default ip address"),
                        ),
                        hosts: HostsVariantsV2::None,
                    },
                    HostV2 {
                        name: "host-2".into(),
                        htype: HostType::Server,
                        distance: 0,
                        ports: PortsVariants::None,
                        ip: IP::Server(
                            "192.168.16.12"
                                .parse()
                                .expect("Error then parsing default ip address"),
                        ),
                        hosts: HostsVariantsV2::None,
                    },
                ]),
            }],
            failover: Default::default(),
            vars: Default::default(),
        }
    }
}

pub(in crate::task) struct Context<T>(pub(in crate::task) T);

impl<T, S> MapSelf<S> for Context<T> {
    type Target = S;
    type Error = TaskError;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>,
    {
        func(self)
    }
}

impl<T> Functor for Context<T> {
    type Unwrapped = T;
    type Wrapped<U> = Context<U>;
    type Error = TaskError;

    fn map<F, U>(self, func: F) -> Result<Self::Wrapped<U>, Self::Error>
    where
        F: FnOnce(Self::Unwrapped) -> Result<U, Self::Error>,
    {
        Ok(Context(func(self.0)?))
    }
}

impl<'a> TryFrom<&'a [u8]> for Cluster {
    type Error = TaskError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice(value)
            .map_err(|error| {
                TaskError::ConfigError(ConfigError::FileFormatError(format!(
                    "Error then deserializing cluster file {}",
                    error
                )))
            })
            .map(|cluster: Cluster| cluster.sort())
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Cluster {
    type Error = TaskError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        if let Cluster::V2 {
            topology,
            hosts,
            vars,
            ..
        } = Cluster::default()
        {
            return Ok(Cluster::V2 {
                topology,
                hosts,
                failover: Failover::try_from(args)?,
                vars,
            }
            .sort());
        }
        Err(TaskError::ConfigError(ConfigError::UndefinedError(
            "Error then creating Cluster from args".into(),
        )))
    }
}

impl Cluster {
    #[inline]
    fn sort(mut self) -> Self {
        match &mut self {
            Cluster::V1 { instances, .. } => instances.sort(),
            Cluster::V2 { topology, .. } => topology.sort(),
        }
        self
    }

    pub(in crate::task) fn failover(&self) -> &Failover {
        match self {
            Cluster::V1 { failover, .. } => failover,
            Cluster::V2 { failover, .. } => failover,
        }
    }

    pub(in crate::task) fn vars_mut(&mut self) -> &mut Vars {
        match self {
            Cluster::V1 { vars, .. } => vars,
            Cluster::V2 { vars, .. } => vars,
        }
    }

    pub(in crate::task) fn topology(&self) -> Vec<InstanceV2> {
        match self {
            Cluster::V1 { instances, .. } => instances.into_v2(),
            Cluster::V2 { topology, .. } => topology.clone(),
        }
    }
}

#[cfg(test)]
mod test;
