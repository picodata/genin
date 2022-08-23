use std::ops::{Deref, DerefMut};

use log::trace;
use crate::error::{ConfigError, TaskError};
use crate::task::hst::{Host, Hosts, HostsVariants, HostType, IP, Ports};
use crate::task::ins::Instance;

#[derive(Debug)]
pub(in crate::task) struct FlatHosts(Vec<FlatHost>);

impl Deref for FlatHosts {
    type Target = Vec<FlatHost>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FlatHosts {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> TryFrom<&'a Hosts> for FlatHosts {
    type Error = TaskError;

    fn try_from(hosts: &'a Hosts) -> Result<Self, Self::Error> {
        Ok(Self(FlatHosts::recursive_from(hosts, &Host::default())?))
    }
}

#[allow(unused)]
#[derive(Debug)]
pub(in crate::task) struct FlatHost {
    pub(in crate::task) name: String,
    pub(in crate::task) htype: HostType,
    pub(in crate::task) ports: Ports,
    pub(in crate::task) ip: IP,
    pub(in crate::task) deepness: Vec<String>,
    pub(in crate::task) instances: Vec<Instance>,
}

#[allow(unused)]
impl FlatHosts {
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
    fn recursive_from(hosts: &Hosts, parent: &Host) -> Result<Vec<FlatHost>, TaskError> {
        hosts
            .deref()
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
                            result.extend(FlatHosts::recursive_from(hosts, host)?);
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

    pub(in crate::task) fn max_len(&self) -> usize {
        self.0
            .iter()
            .max_by(|a, b| a.instances.len().cmp(&b.instances.len()))
            .map(|host| host.instances.len())
            .unwrap_or_else(|| self.0.first().map(|host| host.instances.len()).unwrap_or(0))
    }

    pub(in crate::task) fn downcast(self) -> Vec<FlatHost> {
        let FlatHosts(hosts) = self;
        hosts
    }
}

impl FlatHost {
    pub(in crate::task) fn name(&self) -> &str {
        &self.name
    }
}
