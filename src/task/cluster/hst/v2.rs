use indexmap::IndexMap;
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{borrow::Cow, cell::RefCell, cmp::Ordering, fmt::Display, net::IpAddr};
use tabled::{builder::Builder, merge::Merge, Alignment, Tabled};

use crate::{
    error::{GeninError, GeninErrorKind},
    task::{
        cluster::ins::v2::{InstanceV2, InstanceV2Config},
        flv::{StateboardParams, Uri},
    },
    task::{cluster::name::Name, inventory::InvHostConfig},
};

use super::{
    merge_index_maps,
    v1::{Host, HostsVariants},
    IP,
};

/// Host can be Region, Datacenter, Server
/// ```yaml
/// hosts:
///     - name: kaukaz
///       config:
///         http_port: 8091
///         binary_port: 3031
///         distance: 10
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
///       hosts:
///         - name: dc-3
///           type: datacenter
///           ports:
///             http_port: 8091
///             binary_port: 3031
///             distance: 20
///           hosts:
///             - name: server-10
///               ip: 10.99.3.100
/// ```
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct HostV2 {
    pub name: Name,
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
            name: Name::from(s),
            config: HostV2Config::default(),
            hosts: Vec::new(),
            instances: Vec::new(),
        }
    }
}

impl From<Name> for HostV2 {
    fn from(name: Name) -> Self {
        Self {
            name,
            config: HostV2Config::default(),
            hosts: Vec::new(),
            instances: Vec::new(),
        }
    }
}

impl From<Vec<Host>> for HostV2 {
    fn from(hosts: Vec<Host>) -> Self {
        HostV2 {
            name: Name::from("cluster"),
            config: HostV2Config::default(),
            hosts: hosts
                .into_iter()
                .map(|host| HostV2::into_host_v2(Name::from("cluster"), host))
                .collect(),
            instances: Vec::new(),
        }
    }
}

#[allow(unused)]
impl HostV2 {
    pub fn with_config(self, config: HostV2Config) -> Self {
        Self { config, ..self }
    }

    pub fn with_hosts(self, hosts: Vec<HostV2>) -> Self {
        Self { hosts, ..self }
    }

    fn into_host_v2(parent_name: Name, host: Host) -> HostV2 {
        match host {
            Host {
                name,
                ports,
                ip,
                hosts: HostsVariants::Hosts(hosts),
                ..
            } => {
                let name = parent_name.with_raw_index(name);
                HostV2 {
                    name: name.clone(),
                    config: HostV2Config {
                        http_port: ports.http_as_option(),
                        binary_port: ports.binary_as_option(),
                        address: Address::from(ip),
                        distance: None,
                        additional_config: IndexMap::new(),
                    },
                    hosts: hosts
                        .into_iter()
                        .map(|host| HostV2::into_host_v2(name.clone(), host))
                        .collect(),
                    instances: Vec::new(),
                }
            }
            Host {
                name,
                ports,
                ip,
                hosts: HostsVariants::None,
                ..
            } => HostV2 {
                name: parent_name.with_raw_index(name),
                config: HostV2Config {
                    http_port: ports.http_as_option(),
                    binary_port: ports.binary_as_option(),
                    address: Address::from(ip),
                    distance: None,
                    additional_config: IndexMap::new(),
                },
                hosts: Vec::new(),
                instances: Vec::new(),
            },
        }
    }

    pub fn spread(&mut self) {
        if self.hosts.is_empty() {
            self.instances
                .iter_mut()
                .enumerate()
                .for_each(|(index, instance)| {
                    *instance = InstanceV2 {
                        config: instance
                            .config
                            .clone()
                            .merge_and_up_ports(self.config.clone(), index as u16),
                        ..instance.clone()
                    };
                    trace!(
                        "host: {} instance: {} config: {:?}",
                        self.name,
                        instance.name,
                        instance.config
                    );
                });
            return;
        }

        self.instances.reverse();

        debug!(
            "instances spreading queue: {} ",
            self.instances
                .iter()
                .map(|instance| instance.name.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        //TODO: error propagation
        while let Some(instance) = self.instances.pop() {
            if instance.failure_domains.is_empty() {
                self.hosts.sort();
                self.push(instance).unwrap();
            } else {
                trace!(
                    "start pushing instance {} with failure domain",
                    instance.name
                );
                self.push_to_failure_domain(instance).unwrap();
            }
        }

        self.hosts.sort_by(|left, right| left.name.cmp(&right.name));
        self.hosts.iter_mut().for_each(|host| {
            host.config = host.config.clone().merge(self.config.clone());
            host.spread();
        });
    }

    fn push(&mut self, instance: InstanceV2) -> Result<(), GeninError> {
        if let Some(host) = self.hosts.first_mut() {
            host.instances.push(instance);
            Ok(())
        } else {
            Err(GeninError::new(
                GeninErrorKind::SpreadingError,
                format!(
                    "failed to get mutable reference to first host in hosts: [{}]",
                    self.hosts
                        .iter()
                        .map(|host| host.name.to_string())
                        .collect::<Vec<String>>()
                        .join(" ")
                ),
            ))
        }
    }

    fn push_to_failure_domain(&mut self, mut instance: InstanceV2) -> Result<(), GeninError> {
        trace!(
            "trying to find reqested failure_domains inside host {} for instance {}",
            self.name,
            instance.name,
        );

        let failure_domain_index = instance
            .failure_domains
            .iter()
            .position(|domain| domain.eq(&self.name.to_string()));

        // if we found some name equality between host name and failure domain
        // remove it and push instance
        if let Some(index) = failure_domain_index {
            trace!(
                "removing {} failure domain from bindings in {}",
                instance.failure_domains.remove(index),
                instance.name
            );
            if !self.contains_failure_domains(&instance.failure_domains) {
                trace!(
                    "removing all failure domains from bindings in {}",
                    instance.name
                );
                instance.failure_domains = Vec::new();
            }
            self.hosts.sort();
            return self.push(instance);
        };

        // retain only hosts that contains one of failure domain members
        // failure_domains: ["dc-1"] -> vec!["dc-1"]
        let mut failure_domain_hosts: Vec<&mut HostV2> = self
            .hosts
            .iter_mut()
            .filter_map(|host| {
                (instance.failure_domains.contains(&self.name.to_string())
                    || host.contains_failure_domains(&instance.failure_domains))
                .then_some(host)
            })
            .collect();
        if !failure_domain_hosts.is_empty() {
            trace!(
                "following hosts [{}] contains one or more of this failure domains [{}]",
                failure_domain_hosts
                    .iter()
                    .map(|host| host.name.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                instance.failure_domains.join(" "),
            );
            failure_domain_hosts.sort();
            if let Some(host) = failure_domain_hosts.first_mut() {
                host.instances.push(instance);
                return Ok(());
            };
        }
        Err(GeninError::new(
            GeninErrorKind::UnknownFailureDomain,
            format!(
                "none of the hosts [{} {}] are eligible for the failure domain [{}]",
                self.name,
                self.hosts
                    .iter()
                    .map(|host| host.name.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
                instance.failure_domains.join(" "),
            ),
        ))
    }

    pub fn with_inner_hosts(mut self, hosts: Vec<HostV2>) -> Self {
        self.hosts = hosts;
        self
    }

    pub fn with_instances(mut self, instances: Vec<InstanceV2>) -> Self {
        self.instances = instances;
        self
    }

    pub fn with_http_port(mut self, port: u16) -> Self {
        self.config.http_port = Some(port);
        self
    }

    pub fn with_binary_port(mut self, port: u16) -> Self {
        self.config.binary_port = Some(port);
        self
    }

    /// Count number for instances spreaded in HostV2 on all levels
    ///
    /// * If top level HostV2 has 10 instances and instances not spreaded `size() = 0`
    /// * If 20 instances already spreaded accross HostV2 childrens  `size() = 20`
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

    fn form_structure(&self, depth: usize, collector: &RefCell<Vec<Vec<DomainMember>>>) {
        collector
            .borrow_mut()
            .get_mut(depth)
            .map(|level| {
                level.extend(vec![
                    DomainMember::from(self.name.to_string());
                    self.width()
                ])
            })
            .unwrap();

        if self.instances.is_empty() {
            trace!(
                "Spreading instances for {} skipped. Width {}. Current level {} vector lenght {}",
                self.name,
                self.width(),
                depth,
                collector.borrow().get(depth).unwrap().len()
            );

            debug!("Row depth {} header {}", depth, self.name);

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
                .map(|level| level.push(DomainMember::from(self.name.to_string())))
                .unwrap();
            let remainder = collector.borrow().len() - depth - 1;
            (0..remainder).into_iter().for_each(|index| {
                collector
                    .borrow_mut()
                    .get_mut(depth + index + 1)
                    .map(|level| {
                        if let Some(instance) = self.instances.get(index) {
                            if instance.is_stateboard() {
                                level.push(DomainMember::Domain(instance.name.to_string()));
                            } else {
                                level.push(DomainMember::Instance {
                                    name: instance.name.to_string(),
                                    http_port: instance.config.http_port.unwrap(),
                                    binary_port: instance.config.binary_port.unwrap(),
                                });
                            }
                        } else {
                            level.push(DomainMember::Dummy);
                        }
                    })
                    .unwrap();
            });
        }
    }

    pub fn lower_level_hosts(&self) -> Vec<&HostV2> {
        if self.hosts.is_empty() {
            vec![self]
        } else {
            self.hosts
                .iter()
                .flat_map(|host| host.lower_level_hosts())
                .collect()
        }
    }

    fn contains_failure_domains(&self, failure_domais: &Vec<String>) -> bool {
        if failure_domais.contains(&self.name.to_string()) {
            return true;
        } else if !self.hosts.is_empty() {
            for host in &self.hosts {
                if host.contains_failure_domains(failure_domais) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_name_by_address(&self, address: &Address) -> Option<&Name> {
        if self.config.address.eq(address) {
            Some(&self.name)
        } else {
            self.hosts.iter().fold(None, |accum, host| {
                accum.or_else(|| host.get_name_by_address(address))
            })
        }
    }

    pub fn prune_instances(&mut self) {
        self.instances = Vec::new();
        if !self.hosts.is_empty() {
            self.hosts
                .iter_mut()
                .for_each(|host| host.prune_instances())
        }
    }

    pub fn delete_stateboard(&mut self) {
        if self.hosts.is_empty() {
            self.instances
                .retain(|instance| instance.name != Name::from("stateboard"))
        } else {
            self.hosts
                .iter_mut()
                .for_each(|host| host.delete_stateboard())
        }
    }

    pub fn push_stateboard(&mut self, stateboard: &StateboardParams) {
        self.instances.push(InstanceV2 {
            name: Name::from("stateboard"),
            stateboard: Some(true),
            weight: None,
            failure_domains: self
                .get_name_by_address(&stateboard.uri.address)
                .map(|name| vec![name.to_string()])
                .unwrap_or_default(),
            roles: Vec::new(),
            config: InstanceV2Config {
                additional_config: vec![
                    (
                        String::from("listen"),
                        Value::String(stateboard.uri.to_string()),
                    ),
                    (
                        String::from("password"),
                        Value::String(stateboard.password.clone()),
                    ),
                ]
                .into_iter()
                .collect(),
                ..InstanceV2Config::default()
            },
        });
    }

    pub fn merge(&mut self, rhs: &HostV2) {
        if !rhs.hosts.is_empty() {
            rhs.hosts.iter().for_each(|rhs_host| {
                if let Some(self_host) = self
                    .hosts
                    .iter_mut()
                    .find(|self_host| self_host.name.eq(&rhs_host.name))
                {
                    self_host.merge(rhs_host);
                } else {
                    self.hosts.push(HostV2 {
                        instances: Vec::new(),
                        ..rhs_host.clone()
                    });
                }
            })
        } else {
            self.hosts = rhs.hosts.clone();
            self.config = HostV2Config {
                http_port: self.config.http_port,
                binary_port: self.config.binary_port,
                ..rhs.config.clone()
            };
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

        let mut table = Builder::from_iter(collector.take().into_iter()).build();
        table.with(Merge::horizontal());
        table.with(Alignment::center());

        write!(f, "{}", table)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct HostV2Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Address::is_none")]
    pub address: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<usize>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_config: IndexMap<String, Value>,
}

impl From<(u16, u16)> for HostV2Config {
    fn from(p: (u16, u16)) -> Self {
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

impl From<Address> for HostV2Config {
    fn from(address: Address) -> Self {
        Self {
            address,
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

impl<'a> From<&'a InvHostConfig> for HostV2Config {
    fn from(config: &'a InvHostConfig) -> Self {
        match config {
            InvHostConfig::Instance {
                advertise_uri,
                http_port,
                additional_config,
                ..
            } => Self {
                http_port: Some(*http_port),
                binary_port: Some(advertise_uri.port),
                address: advertise_uri.address.clone(),
                distance: None,
                additional_config: additional_config.clone(),
            },
            InvHostConfig::Stateboard(additional_config) => Self {
                http_port: None,
                binary_port: None,
                address: additional_config
                    .get("listen")
                    .map(|value| {
                        serde_yaml::from_str::<Uri>(value.as_str().unwrap())
                            .unwrap()
                            .address
                    })
                    .unwrap(),
                distance: None,
                additional_config: additional_config.clone(),
            },
        }
    }
}

impl From<IndexMap<String, Value>> for HostV2Config {
    fn from(additional_config: IndexMap<String, Value>) -> Self {
        let uri: Uri = additional_config
            .get("advertise_uri")
            .map(|value| serde_yaml::from_value(value.clone()).unwrap())
            .unwrap();
        Self {
            http_port: additional_config
                .get("http_port")
                .map(|value| value.as_str().unwrap().parse::<u16>().unwrap()),
            binary_port: Some(uri.port),
            address: uri.address,
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
            additional_config: IndexMap::new(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.http_port.is_none()
            && self.binary_port.is_none()
            && self.address.is_none()
            && self.additional_config.is_empty()
    }

    pub fn with_distance(self, distance: usize) -> Self {
        Self {
            distance: Some(distance),
            ..self
        }
    }

    pub fn with_ports(self, ports: (u16, u16)) -> Self {
        Self {
            http_port: Some(ports.0),
            binary_port: Some(ports.1),
            ..self
        }
    }

    pub fn with_address(self, address: Address) -> Self {
        Self { address, ..self }
    }

    pub fn with_http_port(self, http_port: u16) -> Self {
        Self {
            http_port: Some(http_port),
            ..self
        }
    }

    pub fn with_binary_port(self, binary_port: u16) -> Self {
        Self {
            binary_port: Some(binary_port),
            ..self
        }
    }

    pub fn with_additional_config(self, additional_config: IndexMap<String, Value>) -> Self {
        Self {
            additional_config,
            ..self
        }
    }

    pub fn address_to_string(&self) -> String {
        self.address.to_string()
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }

    pub fn binary_port(&self) -> Option<u16> {
        self.binary_port
    }

    pub fn http_port(&self) -> Option<u16> {
        self.http_port
    }

    pub fn additional_config(&self) -> IndexMap<String, Value> {
        self.additional_config.clone()
    }

    pub fn merge(self, other: HostV2Config) -> Self {
        Self {
            http_port: self.http_port.or(other.http_port),
            binary_port: self.binary_port.or(other.binary_port),
            address: self.address.or(other.address),
            distance: self.distance.or(other.distance),
            additional_config: merge_index_maps(self.additional_config, other.additional_config),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Address {
    Ip(IpAddr),
    IpSubnet(Vec<IpAddr>),
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

impl From<[u8; 4]> for Address {
    fn from(array: [u8; 4]) -> Self {
        Self::Ip(array.into())
    }
}

impl<'a> From<&'a str> for Address {
    fn from(s: &'a str) -> Self {
        if let Ok(ip) = s.parse::<IpAddr>() {
            Self::Ip(ip)
        } else {
            Self::Uri(s.to_string())
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

#[allow(unused)]
impl Address {
    pub(in crate::task) fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn or_else<F: FnOnce() -> Self>(self, function: F) -> Self {
        if let Self::None = self {
            function()
        } else {
            self
        }
    }

    pub fn or(self, rhs: Self) -> Self {
        if let Self::None = self {
            rhs
        } else {
            self
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Params {
    begin_binary_port: Option<usize>,
    begin_http_port: Option<usize>,
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
        http_port: u16,
        #[tabled(inline)]
        binary_port: u16,
    },
    #[tabled(display_with("Self::display_valid", args))]
    Dummy,
}

impl<'a> From<&'a str> for DomainMember {
    fn from(s: &'a str) -> Self {
        Self::Domain(s.to_string())
    }
}

impl From<String> for DomainMember {
    fn from(s: String) -> Self {
        Self::Domain(s)
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
pub struct IPSubnet(Vec<IpAddr>);
