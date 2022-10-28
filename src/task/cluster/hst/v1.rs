use log::trace;
use serde::{Deserialize, Serialize};

use crate::error::{ConfigError, TaskError};
use crate::task::cluster::hst::{
    is_null, FlatHost, Flatten, HostType, PortsVariants, TryIntoFlatHosts, IP,
};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub struct Host {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "HostType::is_server", default)]
    pub htype: HostType,
    #[serde(skip_serializing_if = "is_null", default)]
    pub distance: usize,
    #[serde(skip_serializing_if = "PortsVariants::is_none", default)]
    pub ports: PortsVariants,
    #[serde(skip_serializing_if = "IP::is_none", default)]
    pub ip: IP,
    #[serde(skip_serializing_if = "HostsVariants::is_none", default)]
    pub hosts: HostsVariants,
}

impl Host {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum HostsVariants {
    Hosts(Vec<Host>),
    None,
}

impl Default for HostsVariants {
    fn default() -> Self {
        Self::None
    }
}

impl HostsVariants {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

impl TryIntoFlatHosts for Vec<Host> {
    type Error = TaskError;

    fn try_into(&self) -> Result<Vec<FlatHost>, Self::Error> {
        Ok(Vec::<FlatHost>::flatten(self, &Host::default())?)
    }
}

#[allow(unused)]
impl Flatten<Host> for Vec<FlatHost> {
    /// Recursively iterate over datacentres and inners.
    /// Create list of hosts and return it.
    ///
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
    ///         ...
    /// ```
    /// This yaml will be merged into:
    /// ```rust
    /// vec![
    ///     Host{
    ///         name: "server-1",
    ///         htype: HostType::Server,
    ///         ports: ports.append(&parent_ports),
    ///         ip: ip.append(&parent_ip),
    ///         deepness: vec![vec!["server-1"], parent_vec].concat(),
    ///         instances: Vec::new(),
    ///     }
    ///     Host{...}
    ///     Host{...}
    /// ]
    /// ```
    fn flatten(hosts: &Vec<Host>, parent: &Host) -> Result<Vec<FlatHost>, TaskError> {
        hosts
            .iter()
            .try_fold(Vec::new(), |mut result: Vec<FlatHost>, host| {
                match host.htype {
                    HostType::Server => {
                        result.push(FlatHost {
                            name: host.name().into(),
                            htype: HostType::Server,
                            ports: host.ports.applicate(&parent.ports).unwrap_or_default(),
                            ip: host.ip.applicate(&parent.ip),
                            deepness: vec![host.name.to_string(), parent.name.to_string()],
                            instances: Vec::new(),
                        });
                        trace!("Flattering server: {}", host.name());
                        Ok(result)
                    }
                    _ => match &host.hosts {
                        HostsVariants::Hosts(hosts) => {
                            result.extend(Vec::flatten(hosts, host)?);
                            Ok(result)
                        }
                        HostsVariants::None => Err(TaskError::ConfigError(
                            ConfigError::FileContentError(format!(
                                "{} {} does not contains inner hosts!",
                                host.name(),
                                host.htype
                            )),
                        )),
                    },
                }
            })
    }
}
