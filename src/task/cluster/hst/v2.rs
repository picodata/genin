use indexmap::IndexMap;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml::{Number, Value};
use std::fmt;
use std::{borrow::Cow, cell::RefCell, cmp::Ordering, fmt::Display, net::IpAddr};
use tabled::papergrid::AnsiColor;
use tabled::{builder::Builder, merge::Merge, Alignment, Tabled};

use crate::task::cluster::hst::view::BG_BLACK;
use crate::task::cluster::hst::{merge_index_maps, v1::Host, v1::HostsVariants, view::View, IP};
use crate::task::cluster::ins::v2::Instances;
use crate::task::flv::{Failover, FailoverVariants};
use crate::task::{AsError, ErrConfMapping, TypeError, DICT, LIST, NUMBER, STRING};
use crate::{
    error::{GeninError, GeninErrorKind},
    task::{
        cluster::ins::v2::{InstanceV2, InstanceV2Config},
        flv::Uri,
    },
    task::{cluster::name::Name, inventory::InvHostConfig},
};

use super::view::FG_BRIGHT_BLACK;

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
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct HostV2 {
    pub name: Name,
    #[serde(skip_serializing_if = "HostV2Config::is_none", default)]
    pub config: HostV2Config,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hosts: Vec<HostV2>,
    #[serde(skip)]
    pub instances: Instances,
}

impl<'a> From<&'a str> for HostV2 {
    fn from(s: &'a str) -> Self {
        Self {
            name: Name::from(s),
            config: HostV2Config::default(),
            hosts: Vec::default(),
            instances: Instances::default(),
        }
    }
}

impl From<Name> for HostV2 {
    fn from(name: Name) -> Self {
        Self {
            name,
            config: HostV2Config::default(),
            hosts: Vec::default(),
            instances: Instances::default(),
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
            instances: Instances::default(),
        }
    }
}

impl From<Host> for HostV2 {
    fn from(host: Host) -> Self {
        HostV2 {
            name: Name::from(host.name),
            config: HostV2Config {
                http_port: host.ports.http_as_option(),
                binary_port: host.ports.binary_as_option(),
                address: Address::from(host.ip),
                distance: Some(host.distance).and_then(|distance| {
                    if distance.eq(&0) {
                        None
                    } else {
                        Some(distance)
                    }
                }),
                additional_config: IndexMap::new(),
            },
            hosts: match host.hosts {
                HostsVariants::None => Vec::new(),
                HostsVariants::Hosts(hosts) => hosts.into_iter().map(HostV2::from).collect(),
            },
            instances: Instances::default(),
        }
    }
}

pub trait WithHosts<T> {
    fn with_hosts(self, hosts: T) -> Self;
}

impl WithHosts<Vec<HostV2>> for HostV2 {
    fn with_hosts(self, hosts: Vec<HostV2>) -> Self {
        Self { hosts, ..self }
    }
}

impl WithHosts<Vec<Host>> for HostV2 {
    fn with_hosts(self, hosts: Vec<Host>) -> Self {
        Self {
            hosts: hosts.into_iter().map(HostV2::from).collect(),
            ..self
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

impl HostV2 {
    pub fn with_config(self, config: HostV2Config) -> Self {
        Self { config, ..self }
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
                    instances: Instances::default(),
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
                hosts: Vec::default(),
                instances: Instances::default(),
            },
        }
    }

    pub fn spread(mut self) -> Self {
        self.inner_spread();
        self
    }

    pub fn inner_spread(&mut self) {
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
                    debug!(
                        "host: {} instance: {} config: {:?}",
                        self.name, instance.name, instance.config
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
                debug!(
                    "start pushing instance {} with failure domain",
                    instance.name
                );
                self.push_to_failure_domain(instance).unwrap();
            }
        }

        self.hosts.sort_by(|left, right| left.name.cmp(&right.name));
        self.hosts.iter_mut().for_each(|host| {
            host.config = host.config.clone().merge(self.config.clone());
            host.inner_spread();
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
        debug!(
            "trying to find reqested failure_domains inside host {} for instance {}",
            self.name, instance.name,
        );

        let failure_domain_index = instance
            .failure_domains
            .iter()
            .position(|domain| domain.eq(&self.name.to_string()));

        // if we found some name equality between host name and failure domain
        // remove it and push instance
        if let Some(index) = failure_domain_index {
            let domain_name = instance.failure_domains.remove(index);
            debug!(
                "found failure domain {} for {} instance",
                domain_name, instance.name
            );
            if !self.contains_failure_domains(&instance.failure_domains) {
                debug!(
                    "cleaning failure domains for instance {}, as no more needed failure domains can be found",
                    instance.name
                );
                instance.failure_domains = Vec::new();
            }
            // if it is the last failure domain binding(in other words, we find the needed place for instance),
            // keep that failure domain name in the instance and remove others.
            if instance.failure_domains.is_empty() {
                instance.failure_domains = vec![self.name.to_string()];
            }

            debug!(
                "failure domains for {} is: {}",
                instance.name,
                instance.failure_domains.join(" ")
            );

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
            debug!(
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

    #[allow(unused)]
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
            debug!(
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
            debug!(
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
            (0..remainder).for_each(|index| {
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
                                    fg_color: instance.view.color.clone(),
                                    bg_color: BG_BLACK,
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

    pub fn with_instances(self, instances: Instances) -> Self {
        Self { instances, ..self }
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

    pub fn with_stateboard(mut self, failover: &Failover) -> Self {
        if let Failover {
            failover_variants: FailoverVariants::StateboardVariant(stateboard),
            ..
        } = failover
        {
            self.instances.push(InstanceV2 {
                name: Name::from("stateboard"),
                stateboard: Some(true),
                weight: None,
                failure_domains: self
                    .get_name_by_address(&stateboard.uri.address)
                    .map(|name| vec![name.to_string()])
                    .unwrap_or_default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::default(),
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
                vars: IndexMap::default(),
                view: View {
                    alignment: Alignment::center(),
                    color: FG_BRIGHT_BLACK,
                },
            });
        }
        self
    }

    #[allow(unused)]
    // used only in tests
    pub fn with_http_port(self, http_port: u16) -> Self {
        Self {
            config: HostV2Config {
                http_port: Some(http_port),
                ..self.config
            },
            ..self
        }
    }

    #[allow(unused)]
    // used only in tests
    pub fn with_binary_port(self, binary_port: u16) -> Self {
        Self {
            config: HostV2Config {
                binary_port: Some(binary_port),
                ..self.config
            },
            ..self
        }
    }

    #[allow(unused)]
    // used only in tests
    pub fn with_address(self, address: Address) -> Self {
        Self {
            config: HostV2Config {
                address,
                ..self.config
            },
            ..self
        }
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
                        instances: Instances::default(),
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

    /// For every instance that has failure domain available, replace its zone with that domain name.
    pub fn use_failure_domain_as_zone(&mut self) {
        for instance in self.instances.iter_mut() {
            if let Some(failure_domain) = instance.failure_domains.first() {
                instance.config.zone = Some(failure_domain.clone());
            }
        }
        for sub_host in self.hosts.iter_mut() {
            sub_host.use_failure_domain_as_zone()
        }
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

impl HostV2Config {
    pub fn is_none(&self) -> bool {
        self.http_port.is_none()
            && self.binary_port.is_none()
            && self.address.is_none()
            && self.additional_config.is_empty()
    }

    pub fn with_ports(self, ports: (u16, u16)) -> Self {
        Self {
            http_port: Some(ports.0),
            binary_port: Some(ports.1),
            ..self
        }
    }

    pub fn with_additional_config(self, additional_config: IndexMap<String, Value>) -> Self {
        Self {
            additional_config,
            ..self
        }
    }

    pub fn address(&self) -> Address {
        self.address.clone()
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
    Domain(String),
    Instance {
        name: String,
        http_port: u16,
        binary_port: u16,
        fg_color: AnsiColor<'static>,
        bg_color: AnsiColor<'static>,
    },
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
                fg_color,
                ..
            } => Cow::Owned(format!(
                "{prefix}{name}{suffix}\n{bb_prefix}{http_port}/{binary_port}{bb_suffix}",
                prefix = fg_color.get_prefix(),
                suffix = fg_color.get_suffix(),
                name = name,
                http_port = http_port,
                binary_port = binary_port,
                bb_prefix = FG_BRIGHT_BLACK.get_prefix(),
                bb_suffix = FG_BRIGHT_BLACK.get_suffix(),
            )),
            DomainMember::Dummy => Cow::Owned(Default::default()),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq, Eq)]
pub struct IPSubnet(Vec<IpAddr>);

#[derive(Deserialize)]
pub struct InvalidHostV2 {
    #[serde(skip)]
    pub offset: String,
    #[serde(default)]
    name: Value,
    #[serde(default)]
    config: Value,
    #[serde(default)]
    hosts: Value,
}

impl fmt::Debug for InvalidHostV2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // name: String
        match &self.name {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "{}- name: {}",
                    &self.offset,
                    "Missing field 'name'".as_error().as_str()
                ))?;
            }
            Value::String(name) => {
                formatter.write_fmt(format_args!("{}- name: {}", &self.offset, name))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}- name: {}",
                    &self.offset,
                    self.name.type_error(STRING).as_error()
                ))?;
            }
        }

        // config: InvalidHostV2Config
        match &self.config {
            Value::Null => {}
            config @ Value::Mapping(_) => formatter.write_fmt(format_args!(
                "{}  config: {:?}",
                &self.offset,
                serde_yaml::from_value::<InvalidHostV2Config>(config.clone())
                    .map(|mut config| {
                        config.offset = format!("{}    ", &self.offset);
                        config
                    })
                    .unwrap()
            ))?,
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  config: {}",
                    &self.offset,
                    self.config.type_error(DICT).as_error()
                ))?;
            }
        }

        // hosts: InvalidHostV2
        match &self.hosts {
            Value::Null => {}
            Value::Sequence(hosts) => {
                formatter.write_fmt(format_args!("{}  hosts: ", &self.offset))?;
                hosts
                    .iter()
                    .try_for_each(|host| -> Result<(), std::fmt::Error> {
                        formatter.write_fmt(format_args!(
                            "{:?}",
                            serde_yaml::from_value::<InvalidHostV2>(host.clone())
                                .map(|mut host| {
                                    host.offset = format!("{}    ", &self.offset);
                                    host
                                })
                                .unwrap()
                        ))
                    })?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}  hosts: {}",
                    &self.offset,
                    self.hosts.type_error(LIST).as_error()
                ))?;
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct InvalidHostV2Config {
    #[serde(skip)]
    offset: String,
    #[serde(default)]
    pub http_port: Value,
    #[serde(default)]
    pub binary_port: Value,
    #[serde(default)]
    pub address: Value,
    #[serde(default)]
    pub distance: Value,
    #[serde(default)]
    pub additional_config: Value,
}

impl fmt::Debug for InvalidHostV2Config {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // http_port: u16
        match &self.http_port {
            Value::Null => {}
            Value::Number(http_port) => {
                if http_port > &Number::from(0) && http_port < &Number::from(u16::MAX) {
                    formatter.write_fmt(format_args!("{}http_port: {}", self.offset, http_port))?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}http_port: {}",
                        &self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}http_port: {}",
                    &self.offset,
                    self.http_port.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // binary_port: u16
        match &self.binary_port {
            Value::Null => {}
            Value::Number(binary_port) => {
                if binary_port > &Number::from(0) && binary_port < &Number::from(u16::MAX) {
                    formatter
                        .write_fmt(format_args!("{}binary_port: {}", self.offset, binary_port))?;
                } else {
                    formatter.write_fmt(format_args!(
                        "{}binary_port: {}",
                        &self.offset,
                        "Not in range 0..65535".as_error()
                    ))?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}binary_port: {}",
                    &self.offset,
                    self.binary_port.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // address: String
        match &self.address {
            Value::Null => {}
            Value::String(address) => {
                formatter.write_fmt(format_args!("{}address: {}", self.offset, address))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}address: {}",
                    &self.offset,
                    self.address.type_error(STRING).as_error()
                ))?;
            }
        }

        // distance: usize
        match &self.distance {
            Value::Null => {}
            Value::Number(distance) if distance >= &Number::from(0) => {
                formatter.write_fmt(format_args!("{}distance: {}", self.offset, distance))?;
            }
            Value::Number(distance) if distance < &Number::from(0) => {
                formatter.write_fmt(format_args!("{}distance: {}", self.offset, distance))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}distance: {}",
                    &self.offset,
                    self.distance.type_error(NUMBER).as_error()
                ))?;
            }
        }

        // additional_config: IndexMap<String, Value>
        match &self.additional_config {
            Value::Null => {}
            Value::Mapping(additional_config) => {
                formatter.write_fmt(format_args!(
                    "{}additional_config: {:?}",
                    &self.offset,
                    ErrConfMapping {
                        offset: format!("{}  ", &self.offset),
                        value: additional_config,
                    }
                ))?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "{}additional_config: {}",
                    &self.offset,
                    self.additional_config.type_error(DICT).as_error()
                ))?;
            }
        }

        Ok(())
    }
}
