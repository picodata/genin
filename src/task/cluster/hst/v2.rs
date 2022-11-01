use log::trace;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell, cmp::Ordering, fmt::Display, net::IpAddr};
use tabled::{builder::Builder, merge::Merge, Alignment, Tabled};

use crate::{
    error::{GeninError, GeninErrorKind},
    task::cluster::ins::v2::InstanceV2,
};

use super::{
    v1::{Host, HostsVariants},
    IP,
};

/// Host can be Region, Datacenter, Server
/// ```yaml
/// hosts:
///     - name: kaukaz
///       distance: 10
///       config:
///         http_port: 8091
///         binary_port: 3031
///       hosts:
///         - name: dc-1
///           hosts:
///             - name: server-1
///               config:
///                 address: 10.20.3.100
///         - name: dc-2
///           hosts:
///             - name: server-1
///               config:
///                 address: 10.20.4.100
///     - name: moscow
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
#[derive(Serialize, Default, Debug, PartialEq, Eq)]
pub struct HostV2 {
    pub name: String, //TODO: remove pub
    #[serde(skip_serializing_if = "HostV2Config::is_none", default)]
    pub config: HostV2Config,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hosts: Vec<HostV2>,
    #[serde(skip)]
    pub instances: Vec<InstanceV2>,
}

impl<'a> From<&'a str> for HostV2 {
    fn from(s: &'a str) -> Self {
        Self {
            name: s.into(),
            ..Self::default()
        }
    }
}

impl From<Vec<Host>> for HostV2 {
    fn from(hosts: Vec<Host>) -> Self {
        HostV2 {
            name: "cluster".into(),
            config: HostV2Config::default(),
            hosts: hosts.into_iter().map(HostV2::into_v2).collect(),
            instances: Vec::new(),
        }
    }
}

impl HostV2 {
    pub fn with_config(self, config: HostV2Config) -> Self {
        Self { config, ..self }
    }

    pub fn with_hosts(self, hosts: Vec<HostV2>) -> Self {
        Self { hosts, ..self }
    }

    fn into_v2(host: Host) -> HostV2 {
        match host {
            Host {
                name,
                ports,
                ip,
                hosts: HostsVariants::Hosts(hosts),
                ..
            } => HostV2 {
                name,
                config: HostV2Config {
                    http_port: ports.http_as_option(),
                    binary_port: ports.binary_as_option(),
                    address: Address::from(ip),
                    distance: None,
                },
                hosts: hosts.into_iter().map(HostV2::into_v2).collect(),
                instances: Vec::new(),
            },
            Host {
                name,
                ports,
                ip,
                hosts: HostsVariants::None,
                ..
            } => HostV2 {
                name,
                config: HostV2Config {
                    http_port: ports.http_as_option(),
                    binary_port: ports.binary_as_option(),
                    address: Address::from(ip),
                    distance: None,
                },
                ..HostV2::default()
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct HostV2Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    http_port: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    binary_port: Option<usize>,
    #[serde(default, skip_serializing_if = "Address::is_none")]
    address: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    distance: Option<usize>,
}

impl From<(usize, usize)> for HostV2Config {
    fn from(p: (usize, usize)) -> Self {
        Self {
            http_port: Some(p.0),
            binary_port: Some(p.1),
            ..Self::default()
        }
    }
}

impl From<IpAddr> for HostV2Config {
    fn from(ip: IpAddr) -> Self {
        Self {
            address: Address::Ip(ip),
            ..Self::default()
        }
    }
}

impl From<usize> for HostV2Config {
    fn from(distance: usize) -> Self {
        Self {
            distance: Some(distance),
            ..Self::default()
        }
    }
}

//TODO: check unused
#[allow(unused)]
impl HostV2Config {
    pub fn into_default(self) -> Self {
        Self {
            http_port: Some(8081),
            binary_port: Some(3301),
            address: Address::Ip([127, 0, 0, 1].into()),
            distance: Some(0),
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(
            self,
            Self {
                http_port: None,
                binary_port: None,
                address: Address::None,
                distance: None
            }
        )
    }

    pub fn with_distance(self, distance: usize) -> Self {
        Self {
            distance: Some(distance),
            ..self
        }
    }

    pub fn with_ports(self, ports: (usize, usize)) -> Self {
        Self {
            http_port: Some(ports.0),
            binary_port: Some(ports.1),
            ..self
        }
    }

    pub fn with_http_port(self, http_port: usize) -> Self {
        Self {
            http_port: Some(http_port),
            ..self
        }
    }

    pub fn with_binary_port(self, binary_port: usize) -> Self {
        Self {
            binary_port: Some(binary_port),
            ..self
        }
    }

    pub fn merge(&mut self, other: &HostV2Config) {
        self.address.update(&other.address);
        self.http_port = self.http_port.or(other.http_port);
        self.binary_port = self.binary_port.or(other.binary_port);
        self.distance = self.distance.or(other.distance);
    }

    pub fn address(&self) -> String {
        self.address.to_string()
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub(in crate::task) enum Address {
    Ip(IpAddr),
    IpSubnet(IPSubnet),
    Uri(String),
    #[default]
    None,
}

impl From<IP> for Address {
    fn from(ip: IP) -> Self {
        match ip {
            IP::Server(ip) => Self::Ip(ip),
            _ => Self::None,
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Address::Ip(ip) => write!(f, "{}", ip),
            Address::IpSubnet(_) => unimplemented!(), //TODO
            Address::Uri(uri) => write!(f, "{}", uri),
            Address::None => unimplemented!(), //TODO
        }
    }
}

impl Address {
    pub(in crate::task) fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn update(&mut self, address: &Address) {
        if let Self::None = self {
            *self = address.clone()
        }
    }
}

impl PartialOrd for HostV2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.instances.len().partial_cmp(&other.instances.len()) {
            Some(Ordering::Equal) => self.name.partial_cmp(&other.name),
            ord => ord,
        }
    }
}

impl Ord for HostV2 {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.instances.len().cmp(&other.instances.len()) {
            Ordering::Equal => self.name.cmp(&other.name),
            any => any,
        }
    }
}

impl Display for HostV2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let collector = RefCell::new(vec![Vec::new(); self.depth()]);
        let depth = 0;

        self.form_structure(depth, &collector);

        let mut table = Builder::from_iter(collector.take().into_iter())
            .index()
            .build();
        table.with(Merge::horizontal());
        table.with(Alignment::center());

        write!(f, "{}", table)
    }
}

#[allow(unused)]
impl HostV2 {
    pub fn spread(&mut self) {
        if self.hosts.is_empty() {
            return;
        }

        self.instances.reverse();

        while let Some(instance) = self.instances.pop() {
            self.hosts.sort();
            self.push(instance);
        }

        self.hosts.sort_by(|left, right| left.name.cmp(&right.name));

        self.hosts.iter_mut().for_each(|host| {
            host.config.merge(&self.config);
            host.spread();
        });
    }

    pub fn push(&mut self, instance: InstanceV2) -> Result<(), GeninError> {
        if let Some(host) = self.hosts.first_mut() {
            host.instances.push(instance);
            Ok(())
        } else {
            Err(GeninError::new(
                GeninErrorKind::SpreadingError,
                "failed to get mutable reference to first failure_domain",
            ))
        }
    }

    pub fn with_inner_hosts(mut self, hosts: Vec<HostV2>) -> Self {
        self.hosts = hosts;
        self
    }

    pub fn with_instances(mut self, instances: Vec<InstanceV2>) -> Self {
        self.instances = instances;
        self
    }

    pub fn with_begin_http_port(mut self, port: usize) -> Self {
        self.config.http_port = Some(port);
        self
    }

    pub fn with_begin_binary_port(mut self, port: usize) -> Self {
        self.config.binary_port = Some(port);
        self
    }

    pub fn size(&self) -> usize {
        if self.hosts.is_empty() {
            self.instances.len()
        } else {
            self.hosts.iter().fold(0usize, |acc, fd| acc + fd.size())
        }
    }

    pub fn width(&self) -> usize {
        self.hosts.iter().fold(0usize, |acc, fd| {
            if fd.hosts.is_empty() {
                acc + 1
            } else {
                acc + fd.width()
            }
        })
    }

    pub fn depth(&self) -> usize {
        let depth = if self.hosts.is_empty() {
            self.instances.len()
        } else {
            self.hosts.iter().fold(
                0usize,
                |acc, fd| {
                    if fd.depth() > acc {
                        fd.depth()
                    } else {
                        acc
                    }
                },
            )
        };
        depth + 1
    }

    fn form_structure(&self, mut depth: usize, collector: &RefCell<Vec<Vec<DomainMember>>>) {
        if self.instances.is_empty() {
            trace!(
                "Spreading instances for {} skipped. Width {}. Current level {} vector lenght {}",
                self.name,
                self.width(),
                depth,
                collector.borrow().get(depth).unwrap().len()
            );

            self.hosts
                .iter()
                .for_each(|host| host.form_structure(depth + 1, collector));
        } else {
            trace!(
                "Spreading instances for {} -> {:?}",
                self.name,
                self.instances
                    .iter()
                    .map(|instance| instance.name.to_string())
                    .collect::<Vec<String>>()
            );
            collector
                .borrow_mut()
                .get_mut(depth)
                .map(|level| level.push(DomainMember::from(self.name.as_str())))
                .unwrap();
            let remainder = collector.borrow().len() - depth - 1;
            (0..remainder).into_iter().for_each(|index| {
                collector
                    .borrow_mut()
                    .get_mut(depth + index + 1)
                    .map(|level| {
                        if let Some(instance) = self.instances.get(index) {
                            level.push(DomainMember::Instance {
                                name: instance.name.to_string(),
                                http_port: self.config.http_port.unwrap_or(8080) + index,
                                binary_port: self.config.binary_port.unwrap_or(3030) + index,
                            });
                        } else {
                            level.push(DomainMember::Dummy);
                        }
                    })
                    .unwrap();
            });
        }
    }

    pub fn bottom_level(&self) -> Vec<&HostV2> {
        self.hosts
            .iter()
            .flat_map(|host| {
                if !host.hosts.is_empty() {
                    host.bottom_level()
                } else {
                    Vec::new()
                }
            })
            .collect()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Params {
    begin_binary_port: Option<usize>,
    begin_http_port: Option<usize>,
}

//TODO: check in future
#[allow(unused)]
impl Params {
    pub fn update_from(&mut self, rhs: Params) {
        if self.begin_http_port.is_none() && rhs.begin_http_port.is_some() {
            self.begin_http_port = rhs.begin_http_port;
        }
        if self.begin_binary_port.is_none() && rhs.begin_binary_port.is_some() {
            self.begin_binary_port = rhs.begin_binary_port;
        }
    }
}

#[derive(Clone, Tabled, Debug)]
pub enum DomainMember {
    #[tabled(display_with("Self::display_domain", args))]
    Domain(String),
    #[tabled(display_with("Self::display_instance", args))]
    Instance {
        #[tabled(inline)]
        name: String,
        #[tabled(inline)]
        http_port: usize,
        #[tabled(inline)]
        binary_port: usize,
    },
    #[tabled(display_with("Self::display_valid", args))]
    Dummy,
}

impl<'a> From<&'a str> for DomainMember {
    fn from(s: &'a str) -> Self {
        Self::Domain(s.to_string())
    }
}

impl<'a> From<DomainMember> for Cow<'a, str> {
    fn from(val: DomainMember) -> Self {
        match val {
            DomainMember::Domain(name) => Cow::Owned(name),
            DomainMember::Instance {
                name,
                http_port,
                binary_port,
            } => Cow::Owned(format!("{}\n{} {}", name, http_port, binary_port)),
            DomainMember::Dummy => Cow::Owned(Default::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub(in crate::task) struct IPSubnet(Vec<IpAddr>);
