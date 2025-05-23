use indexmap::IndexMap;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml::{Number, Value};
use std::{borrow::Cow, cell::RefCell, cmp::Ordering, fmt::Display, net::IpAddr};
use std::{fmt, mem};
use tabled::papergrid::AnsiColor;
use tabled::{builder::Builder, merge::Merge, Alignment, Tabled};

use crate::task::cluster::host::view::BG_BLACK;
use crate::task::cluster::host::{merge_index_maps, view::View, IP};
use crate::task::cluster::instance::ins::{FailureDomains, Instances};
use crate::task::flv::{Failover, FailoverVariants};
use crate::task::state::Change;
use crate::task::{AsError, ErrConfMapping, TypeError, DICT, LIST, NUMBER, STRING};
use crate::{
    error::{GeninError, GeninErrorKind},
    task::{
        cluster::instance::ins::{Instance, InstanceConfig},
        flv::Uri,
    },
    task::{cluster::name::Name, inventory::InvHostConfig},
};

use super::view::{FG_BRIGHT_BLACK, FG_WHITE};

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
pub struct Host {
    pub name: Name,
    #[serde(skip_serializing_if = "HostConfig::is_none", default)]
    pub config: HostConfig,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hosts: Vec<Host>,
    #[serde(skip)]
    pub add_queue: IndexMap<Name, Instance>,
    #[serde(skip)]
    pub delete_queue: IndexMap<Name, Instance>,
    #[serde(skip_serializing_if = "Instances::is_empty", default)]
    pub instances: Instances,
}

impl<'a> From<&'a str> for Host {
    fn from(s: &'a str) -> Self {
        Self {
            name: Name::from(s),
            config: HostConfig::default(),
            hosts: Vec::default(),
            add_queue: IndexMap::default(),
            delete_queue: IndexMap::default(),
            instances: Instances::default(),
        }
    }
}

impl From<Name> for Host {
    fn from(name: Name) -> Self {
        Self {
            name,
            config: HostConfig::default(),
            hosts: Vec::default(),
            add_queue: IndexMap::default(),
            delete_queue: IndexMap::default(),
            instances: Instances::default(),
        }
    }
}

pub trait WithHosts<T> {
    fn with_hosts(self, hosts: T) -> Self;
}

impl WithHosts<Vec<Host>> for Host {
    fn with_hosts(self, hosts: Vec<Host>) -> Self {
        Self { hosts, ..self }
    }
}

impl PartialOrd for Host {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Host {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.instances.len().cmp(&other.instances.len()) {
            Ordering::Equal => self.name.cmp(&other.name),
            any => any,
        }
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let collector = RefCell::new(vec![Vec::new(); self.depth()]);
        let depth = 0;

        self.form_structure(depth, &collector);

        let mut table = Builder::from_iter(collector.take()).build();
        table.with(Merge::horizontal());
        table.with(Alignment::center());

        write!(f, "{}", table)
    }
}

impl Host {
    #[cfg(test)]
    pub fn with_config(self, config: HostConfig) -> Self {
        Self { config, ..self }
    }

    pub fn spread(&mut self) {
        self.inner_spread();
    }

    pub fn inner_spread(&mut self) {
        self.instances.reverse();

        debug!(
            "host: {} has spreading queue: {} ",
            self.name,
            self.instances
                .iter()
                .map(|instance| instance.name.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        //TODO: error propagation
        let mut instances = mem::take(&mut self.instances);

        while let Some(instance) = instances.pop() {
            // We sort here, as default hosts sort order is from least instances filled to most filled.
            // And of course we'd like to push to least filled firstly.
            self.hosts.sort();
            if instance.failure_domains.in_progress() {
                debug!(
                    "start pushing instance {} with failure domain",
                    instance.name
                );
                self.push_to_failure_domain(instance).unwrap();
            } else {
                debug!("instance {} is either finished its failure domains processing, or doesn't have one", instance.name);
                self.push(instance).unwrap();
            }
        }

        self.hosts.sort_by(|left, right| left.name.cmp(&right.name));
        self.hosts.iter_mut().for_each(|host| {
            host.config = host.config.clone().merge(self.config.clone());
            host.inner_spread();
        });

        self.finish_host_spread();
    }

    fn finish_host_spread(&mut self) {
        if !self.hosts.is_empty() {
            debug!(
                "host {} has children, therefore it is not a target for host spread finishing",
                self.name
            );
            return;
        }
        debug!(
            "finishing spread for {}, spread instances: {:#?}",
            self.name,
            self.instances
                .iter()
                .map(|instance| (instance.name.to_string(), instance.failure_domains.clone()))
                .collect::<Vec<_>>()
        );

        let mut instances = mem::take(&mut self.instances);

        for (index, instance) in instances.iter_mut().enumerate() {
            instance.config = instance
                .config
                .clone()
                .merge_and_up_ports(self.config.clone(), index as u16);
            debug!(
                "host: {} instance: {} config: {:?}",
                self.name, instance.name, instance.config
            );
        }

        self.instances = instances
    }

    fn push(&mut self, instance: Instance) -> Result<(), GeninError> {
        let host = if let Some(host) = self.hosts.first_mut() {
            host
        } else {
            self
        };
        host.instances.push(instance.clone());
        host.add_queue
            .insert(instance.name.clone(), instance.clone());
        host.delete_queue.insert(instance.name.clone(), instance);
        Ok(())
    }

    fn push_to_failure_domain(&mut self, mut instance: Instance) -> Result<(), GeninError> {
        debug!(
            "trying to find reqested failure_domains inside host {} for instance {}",
            self.name, instance.name,
        );

        self.advertise_as_failure_domain(&mut instance)?;

        // Nothing to do if self became the final destination for instance failure domains trip.
        if !instance.failure_domains.in_progress() {
            debug!(
                "host {} is final failure domain for instance {}",
                self.name, instance.name
            );
            return self.push(instance);
        }

        let failure_domains = instance.failure_domains.try_get_queue()?;

        // retain only hosts that contains one of failure domain members
        // failure_domains: ["dc-1"] -> vec!["dc-1"]
        let mut failure_domain_hosts: Vec<&mut Host> = self
            .hosts
            .iter_mut()
            .filter_map(|host| {
                host.contains_failure_domains(failure_domains)
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
                failure_domains.join(" "),
            );
            failure_domain_hosts.sort();
            if let Some(host) = failure_domain_hosts.first_mut() {
                host.instances.push(instance.clone());
                host.add_queue
                    .insert(instance.name.clone(), instance.clone());
                host.add_queue.insert(instance.name.clone(), instance);
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
                failure_domains.join(" "),
            ),
        ))
    }

    fn advertise_as_failure_domain(&mut self, instance: &mut Instance) -> Result<(), GeninError> {
        let failure_domains = instance.failure_domains.try_get_queue()?;
        let failure_domain_index = failure_domains
            .iter()
            .position(|domain| domain.eq(&self.name.to_string()));

        // if we found some name equality between host name and failure domain
        // remove it and push instance
        if let Some(index) = failure_domain_index {
            let domain_name = failure_domains.remove(index);
            debug!(
                "found failure domain {} for {} instance",
                domain_name, instance.name
            );
            if !self.contains_failure_domains(failure_domains) {
                debug!(
                    "cleaning failure domains for instance {}, as no more needed failure domains can be found",
                    instance.name
                );
                *failure_domains = Vec::new();
            }
            // if it is the last failure domain binding(in other words, we find the needed place for instance),
            // finalize failure domain binding.
            if failure_domains.is_empty() {
                instance.failure_domains = FailureDomains::Finished(self.name.to_string());
            }
        };
        Ok(())
    }

    #[allow(unused)]
    /// Count number for instances spreaded in Host on all levels
    ///
    /// * If top level Host has 10 instances and instances not spreaded `size() = 0`
    /// * If 20 instances already spreaded accross Host childrens  `size() = 20`
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
                                debug!("Inserting instance {}", instance.name);
                                level.push(DomainMember::Instance {
                                    name: instance.name.to_string(),
                                    http_port: instance.config.http_port.unwrap_or(8081),
                                    binary_port: instance.config.binary_port.unwrap_or(3031),
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

    pub fn lower_level_hosts(&self) -> Vec<&Host> {
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

    pub fn with_add_queue(self, add_queue: IndexMap<Name, Instance>) -> Self {
        Self { add_queue, ..self }
    }

    pub fn with_delete_queue(self, delete_queue: IndexMap<Name, Instance>) -> Self {
        Self {
            delete_queue,
            ..self
        }
    }

    pub fn delete_stateboard(&mut self) {
        if self.hosts.is_empty() {
            self.instances.retain(|instance| !instance.is_stateboard())
        } else {
            self.hosts
                .iter_mut()
                .for_each(|host| host.delete_stateboard())
        }
    }

    pub fn with_stateboard(&mut self, failover: &Failover) {
        if let Failover {
            failover_variants: FailoverVariants::StateboardVariant(stateboard),
            ..
        } = failover
        {
            self.instances.push(Instance {
                name: Name::from("stateboard"),
                stateboard: Some(true),
                weight: None,
                failure_domains: self
                    .get_name_by_address(&stateboard.uri.address)
                    .map(|name| vec![name.to_string()])
                    .unwrap_or_default()
                    .into(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::default(),
                config: InstanceConfig {
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
                    ..InstanceConfig::default()
                },
                vars: IndexMap::default(),
                view: View {
                    alignment: Alignment::center(),
                    color: FG_BRIGHT_BLACK,
                },
            });
        }
    }

    #[allow(unused)]
    // used only in tests
    pub fn with_http_port(self, http_port: u16) -> Self {
        Self {
            config: HostConfig {
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
            config: HostConfig {
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
            config: HostConfig {
                address,
                ..self.config
            },
            ..self
        }
    }

    /// Merge two already sorted hosts tree
    ///
    /// left [server-1, server-2, server-3]
    /// right [server-2, server-4, server-5, server-6]
    /// -> left [server-2, server-4, server-5, server-6]
    /// left [server-1, server-2, server-3]
    /// right [server-1, server-2]
    /// -> left [server-1, server-2]
    pub fn merge(left: &mut Host, right: &mut Host, idiomatic: bool) -> Vec<Change> {
        std::mem::swap(&mut left.config.distance, &mut right.config.distance);
        std::mem::swap(
            &mut left.config.additional_config,
            &mut right.config.additional_config,
        );

        right.instances.iter_mut().for_each(|right| {
            if let Some(left) = left
                .instances
                .iter_mut()
                .find(|left| left.name == right.name)
            {
                std::mem::swap(&mut left.weight, &mut right.weight);
                std::mem::swap(&mut left.roles, &mut right.roles);
                std::mem::swap(
                    &mut left.cartridge_extra_env,
                    &mut right.cartridge_extra_env,
                );
                std::mem::swap(&mut left.config.zone, &mut right.config.zone);
                std::mem::swap(
                    &mut left.config.vshard_group,
                    &mut right.config.vshard_group,
                );
                std::mem::swap(&mut left.config.all_rw, &mut right.config.all_rw);
                std::mem::swap(
                    &mut left.config.additional_config,
                    &mut right.config.additional_config,
                );
                std::mem::swap(&mut left.vars, &mut right.vars);
            }
        });

        let mut hosts_diff = Vec::new();

        if left.hosts.len() > right.hosts.len() {
            left.hosts.retain(|left_host| {
                let contains = right
                    .hosts
                    .iter()
                    .any(|right_host| right_host.name.eq(&left_host.name));

                if !contains {
                    hosts_diff.push(Change::Removed(left_host.name.to_string()));
                }
                contains
            });
        }

        let mut similar_instances = left.add_queue.clone();
        similar_instances
            .retain(|name, _| right.add_queue.contains_key(&name.clone().with_index(1)));

        right.add_queue.retain(|name, _| {
            if !idiomatic && name.len() == 3 && name.get_parent_name().with_index(1).eq(name) {
                !left.add_queue.contains_key(name)
                    && !left.add_queue.contains_key(&name.get_parent_name())
            } else {
                !left.add_queue.contains_key(name)
            }
        });

        std::mem::swap(&mut left.add_queue, &mut right.add_queue);

        left.delete_queue.retain(|name, _| {
            !right.delete_queue.contains_key(name) && !similar_instances.contains_key(name)
        });

        right.hosts.iter_mut().for_each(|right_host| {
            if let Some(left_host) = left
                .hosts
                .iter_mut()
                .find(|left_host| left_host.name.eq(&right_host.name))
            {
                hosts_diff.extend(Host::merge(left_host, right_host, idiomatic));
            } else {
                right_host.clear_instances();
                left.hosts.push(right_host.clone());
                hosts_diff.push(Change::Added(right_host.name.to_string()));
            }
        });

        hosts_diff
    }

    /// For every instance that has finalized failure domain, replace its zone with that domain name.
    pub fn use_failure_domain_as_zone(&mut self, dc_lvl: u8) {
        let lvl: u8 = 0;
        let zone: Option<String> = None;
        self.set_zone(lvl, dc_lvl, zone)
    }

    pub fn set_zone(&mut self, mut lvl: u8, dc_lvl: u8, mut zone: Option<String>) {
        if dc_lvl == lvl {
            zone = Some(self.name.to_string());
        }

        for instance in self.instances.iter_mut() {
            if let FailureDomains::Finished(failure_domain) = &instance.failure_domains {
                instance.config.zone = zone.clone().or(Some(failure_domain.clone()));
            }
        }

        lvl += 1;
        for sub_host in self.hosts.iter_mut() {
            sub_host.set_zone(lvl, dc_lvl, zone.clone())
        }
    }

    /// used only when restoring from state
    pub fn finalize_failure_domains(&mut self) {
        for instance in self.instances.iter_mut() {
            if instance.failure_domains.in_progress() {
                instance.failure_domains = FailureDomains::Finished(self.name.to_string())
            }
        }
        for sub_host in self.hosts.iter_mut() {
            sub_host.finalize_failure_domains()
        }
    }

    pub fn clear_instances(&mut self) {
        self.instances.clear();
        if !self.hosts.is_empty() {
            self.hosts
                .iter_mut()
                .for_each(|host| host.clear_instances());
        }
    }

    pub fn add_diff(&mut self) {
        let mut instances_for_spreading = self
            .add_queue
            .iter()
            .map(|(_, instance)| instance.clone())
            .collect::<Vec<Instance>>();
        instances_for_spreading.sort();
        self.instances = Instances::from(instances_for_spreading);
    }

    pub fn remove_diff(&mut self) {
        self.instances
            .retain(|instance| !self.delete_queue.contains_key(&instance.name));
        self.hosts.iter_mut().for_each(|host| {
            host.delete_queue = self.delete_queue.clone();
            host.remove_diff()
        })
    }

    pub fn collect_instances(&mut self) -> Instances {
        let mut instances = self.instances.clone();
        if !self.hosts.is_empty() {
            self.hosts
                .iter_mut()
                .for_each(|host| instances.extend(host.collect_instances()));
        }
        instances
    }

    pub fn clear_view(&mut self) {
        self.instances.iter_mut().for_each(|instance| {
            instance.view.color = FG_WHITE;
        });

        self.hosts.iter_mut().for_each(|host| host.clear_view());
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct HostConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Address::is_none")]
    pub address: Address,
    #[serde(default, skip_serializing_if = "Address::is_none")]
    pub ansible_host: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distance: Option<usize>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_config: IndexMap<String, Value>,
}

impl From<(u16, u16)> for HostConfig {
    fn from(p: (u16, u16)) -> Self {
        Self {
            http_port: Some(p.0),
            binary_port: Some(p.1),
            ..Self::default()
        }
    }
}

impl From<IpAddr> for HostConfig {
    fn from(ip: IpAddr) -> Self {
        Self {
            address: Address::Ip(ip),
            ..Self::default()
        }
    }
}

impl From<Address> for HostConfig {
    fn from(address: Address) -> Self {
        Self {
            address,
            ..Self::default()
        }
    }
}

impl From<usize> for HostConfig {
    fn from(distance: usize) -> Self {
        Self {
            distance: Some(distance),
            ..Self::default()
        }
    }
}

impl<'a> From<&'a InvHostConfig> for HostConfig {
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
                ansible_host: Default::default(),
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
                ansible_host: Default::default(),
                distance: None,
                additional_config: additional_config.clone(),
            },
        }
    }
}

impl From<IndexMap<String, Value>> for HostConfig {
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

impl HostConfig {
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

    pub fn with_ansible_host(self, ansible_host: Address) -> Self {
        Self {
            ansible_host,
            ..self
        }
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }

    /// Wraps ansible_host behavior - if it isn't supplied, address would be used as a replacement.
    pub fn ansible_host(&self) -> Address {
        if !self.ansible_host.is_none() {
            return self.ansible_host.clone();
        }
        self.address.clone()
    }

    pub fn merge(self, other: HostConfig) -> Self {
        Self {
            http_port: self.http_port.or(other.http_port),
            binary_port: self.binary_port.or(other.binary_port),
            address: self.address.or(other.address),
            ansible_host: self.ansible_host.or(other.ansible_host),
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

impl From<DomainMember> for Cow<'_, str> {
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

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidHost {
    pub offset: String,
    name: Value,
    config: Value,
    hosts: Value,
}

impl fmt::Debug for InvalidHost {
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

        // config: InvalidHostConfig
        match &self.config {
            Value::Null => {}
            config @ Value::Mapping(_) => formatter.write_fmt(format_args!(
                "{}  config: {:?}",
                &self.offset,
                serde_yaml::from_value::<InvalidHostConfig>(config.clone())
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

        // hosts: InvalidHost
        match &self.hosts {
            Value::Null => {}
            Value::Sequence(hosts) => {
                formatter.write_fmt(format_args!("{}  hosts: ", &self.offset))?;
                hosts
                    .iter()
                    .try_for_each(|host| -> Result<(), std::fmt::Error> {
                        formatter.write_fmt(format_args!(
                            "{:?}",
                            serde_yaml::from_value::<InvalidHost>(host.clone())
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

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidHostConfig {
    #[serde(skip)]
    offset: String,
    pub http_port: Value,
    pub binary_port: Value,
    pub address: Value,
    pub distance: Value,
    pub additional_config: Value,
}

impl fmt::Debug for InvalidHostConfig {
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
