use indexmap::IndexMap;
use log::trace;
use serde::{Deserialize, Serialize, Serializer};
use serde_yaml::Value;
use std::cmp::Ordering;
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;
use tabled::papergrid::AnsiColor;

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::host::hst::HostConfig;
use crate::task::cluster::host::merge_index_maps;
use crate::task::cluster::host::view::View;
use crate::task::cluster::instance::Role;
use crate::task::cluster::name::Name;
use crate::task::inventory::{InvHostConfig, InventoryHost};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instances(Vec<Instance>);

impl From<Vec<Instance>> for Instances {
    fn from(instances: Vec<Instance>) -> Self {
        Self(instances)
    }
}

impl Instances {
    pub fn iter(&self) -> Iter<Instance> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Instance> {
        self.0.iter_mut()
    }

    #[allow(unused)]
    // used in tests
    pub fn into_iter(self) -> IntoIter<Instance> {
        self.0.into_iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> Option<&Instance> {
        self.0.get(index)
    }

    pub fn reverse(&mut self) {
        self.0.reverse()
    }

    #[allow(unused)]
    // used in tests
    pub fn pop(&mut self) -> Option<Instance> {
        self.0.pop()
    }

    #[allow(unused)]
    // used in tests
    pub fn first(&self) -> Option<&Instance> {
        self.0.first()
    }

    #[allow(unused)]
    // used in tests
    pub fn last(&self) -> Option<&Instance> {
        self.0.last()
    }

    pub fn push(&mut self, instance: Instance) {
        self.0.push(instance)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Instance) -> bool,
    {
        self.0.retain(f)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn extend<I: IntoIterator<Item = Instance>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl IntoIterator for Instances {
    type Item = Instance;

    type IntoIter = std::vec::IntoIter<Instance>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Replicaset TODO: docs, remove public
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct Replicaset {
    pub name: Name,
    pub replicasets_count: Option<usize>,
    pub replication_factor: Option<usize>,
    pub weight: Option<usize>,
    pub failure_domains: Vec<String>,
    pub roles: Vec<Role>,
    pub config: InstanceConfig,
    pub view: View,
}

/// Single view, replicaset member, host in final inventory
///
/// For example, such a topology unit will have two instances:
/// ```yaml
/// - name: "catalogue"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
///
/// This means that any topology eventually turns into a flat list of
/// instances with different names but similar configuration (based on the parent)
/// ```rust
/// let instances = vec![
///     Instance {
///         name: Name::from("catalogue").with_index(1).with_index(1),
///         stateboard: None,
///         weight: Some(10),
///         failure_domains: Default::default(),
///         roles: vec![String::from("catalogue")],
///         cartridge_extra_env: IndexMap::default(),
///         config: InstanceConfig::default(),
///         vars: IndexMap::default(),
///         view: View {
///             alignment: Alignment::left(),
///             color: FG_BLUE,
///         },
///     },
///     Instance {
///         name: Name::from("catalogue").with_index(1).with_index(2),
///         stateboard: None,
///         weight: Some(10),
///         failure_domains: Default::default(),
///         roles: vec![String::from("catalogue")],
///         cartridge_extra_env: IndexMap::default(),
///         config: InstanceConfig::default(),
///         vars: IndexMap::default(),
///         view: View {
///             alignment: Alignment::left(),
///             color: FG_BLUE,
///         },
///     }
/// ]
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instance {
    /// Instance name with replicaset number and the index of the instance in the replicaset
    pub name: Name,
    //TODO: remove stateboard option
    pub stateboard: Option<bool>,
    //TODO: move to config
    pub weight: Option<usize>,
    //TODO: move to config
    pub failure_domains: FailureDomains,
    pub roles: Vec<Role>,
    pub cartridge_extra_env: IndexMap<String, Value>,
    pub config: InstanceConfig,
    pub vars: IndexMap<String, Value>,
    #[serde(skip)]
    pub view: View,
}

impl PartialOrd for Instance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            ord => ord,
        }
    }
}

impl Ord for Instance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl<'a> From<(&'a Name, &'a InventoryHost)> for Instance {
    fn from(inventory_host: (&'a Name, &'a InventoryHost)) -> Self {
        Self {
            name: inventory_host.0.clone(),
            stateboard: inventory_host.1.stateboard.then_some(true),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::default(),
            cartridge_extra_env: inventory_host.1.cartridge_extra_env.clone(),
            config: InstanceConfig::from_inventory_host(&inventory_host.1),
            vars: inventory_host.1.vars.clone(),
            view: View::default(),
        }
    }
}

impl From<Name> for Instance {
    fn from(name: Name) -> Self {
        Self {
            name,
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::default(),
            cartridge_extra_env: IndexMap::default(),
            config: InstanceConfig::default(),
            vars: IndexMap::default(),
            view: View::default(),
        }
    }
}

impl Instance {
    pub fn is_stateboard(&self) -> bool {
        if let Some(stateboard) = self.stateboard {
            stateboard
        } else {
            false
        }
    }

    pub fn with_roles(self, roles: Vec<Role>) -> Self {
        Self { roles, ..self }
    }

    #[allow(unused)]
    // used only in tests
    pub fn with_color(self, color: AnsiColor<'static>) -> Self {
        Self {
            view: View { color, ..self.view },
            ..self
        }
    }
}

pub type FailureDomain = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FailureDomains {
    /// In this state, failure domains were not even provided for the instance.
    NotProvided([FailureDomain; 0]),
    /// In this state, failure domains were provided, but not yet fully processed for the instance.
    InProgress(Vec<FailureDomain>),
    /// In this state, the exact failure domain is found for the instance.
    #[serde(
        skip_deserializing,
        serialize_with = "serialize_finished_failure_domain"
    )]
    Finished(FailureDomain),
}

/// We want to serialize single found failure domain as an array of domains.
fn serialize_finished_failure_domain<S: Serializer>(
    domain: &FailureDomain,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    [domain].serialize(serializer)
}

impl FailureDomains {
    /// Determines, whether some actions are still needed to be done with these failure domains.
    pub fn in_progress(&self) -> bool {
        matches!(self, FailureDomains::InProgress(_))
    }

    /// Try getting mutable ref to inner domains queue, if it is in progress currently.
    pub fn try_get_queue(&mut self) -> Result<&mut Vec<FailureDomain>, GeninError> {
        match self {
            FailureDomains::InProgress(queue) => Ok(queue),
            _ => Err(GeninError::new(
                GeninErrorKind::SpreadingError,
                format!("failure domains are not in progress: {:?}", self,),
            )),
        }
    }
}

impl From<Vec<String>> for FailureDomains {
    fn from(failure_domains: Vec<String>) -> Self {
        if failure_domains.is_empty() {
            Self::NotProvided([])
        } else {
            Self::InProgress(failure_domains)
        }
    }
}

impl From<FailureDomains> for Vec<String> {
    fn from(domains: FailureDomains) -> Self {
        match domains {
            FailureDomains::NotProvided([]) => vec![],
            FailureDomains::InProgress(domains) => domains,
            FailureDomains::Finished(domain) => vec![domain],
        }
    }
}

impl Default for FailureDomains {
    fn default() -> Self {
        Self::NotProvided([])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub all_rw: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vshard_group: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "IndexMap::is_empty")]
    pub additional_config: IndexMap<String, Value>,
}

impl InstanceConfig {
    pub fn from_inventory_host(host: &InventoryHost) -> Self {
        match &host.config {
            InvHostConfig::Instance {
                advertise_uri,
                http_port,
                additional_config,
            } => Self {
                http_port: Some(*http_port),
                binary_port: Some(advertise_uri.port),
                all_rw: None,
                zone: host.zone.clone(),
                vshard_group: None,
                additional_config: additional_config.clone(),
            },
            InvHostConfig::Stateboard(additional_config) => Self {
                http_port: None,
                binary_port: None,
                all_rw: None,
                zone: None,
                vshard_group: None,
                additional_config: additional_config.clone(),
            },
        }
    }
}

impl<'a> From<&'a IndexMap<String, Value>> for InstanceConfig {
    fn from(config: &'a IndexMap<String, Value>) -> Self {
        Self {
            http_port: config
                .get("http_port")
                .map(|http_port| http_port.as_str().unwrap().parse().unwrap()),
            binary_port: config.get("advertise_uri").map(|advertise_uri| {
                serde_yaml::from_str(advertise_uri.as_str().unwrap()).unwrap()
            }),
            all_rw: config.get("all_rw").map(|all_rw| all_rw.as_bool().unwrap()),
            zone: config
                .get("zone")
                .map(|zone| zone.as_str().unwrap().to_string()),
            vshard_group: config
                .get("vshard_group")
                .map(|vshard_group| vshard_group.as_str().unwrap().to_string()),
            additional_config: config
                .into_iter()
                .filter_map(|(name, value)| match name.as_str() {
                    "all_rw" | "zone" | "vshard_group" | "http_port" | "advertise_uri" => None,
                    _ => Some((name.to_string(), value.clone())),
                })
                .collect(),
        }
    }
}

#[allow(unused)]
impl InstanceConfig {
    pub fn merge_and_up_ports(self, other: HostConfig, index: u16) -> Self {
        trace!("Config before merge: {:?}", &self);
        Self {
            http_port: self
                .http_port
                .or_else(|| other.http_port.map(|port| port + index)),
            binary_port: self
                .binary_port
                .or_else(|| other.binary_port.map(|port| port + index)),
            all_rw: self.all_rw,
            zone: self.zone,
            vshard_group: self.vshard_group,
            additional_config: merge_index_maps(self.additional_config, other.additional_config),
        }
    }

    pub fn merge_with_host_v2_config(self, other: HostConfig) -> Self {
        Self {
            http_port: self.http_port.or(other.http_port),
            binary_port: self.binary_port.or(other.binary_port),
            all_rw: self.all_rw,
            zone: self.zone,
            vshard_group: self.vshard_group,
            additional_config: merge_index_maps(self.additional_config, other.additional_config),
        }
    }

    pub fn is_none(&self) -> bool {
        self.http_port.is_none()
            && self.binary_port.is_none()
            && self.all_rw.is_none()
            && self.zone.is_none()
            && self.vshard_group.is_none()
            && self.additional_config.is_empty()
    }

    pub fn clean_ports(self) -> Self {
        Self {
            http_port: None,
            binary_port: None,
            ..self
        }
    }

    pub fn with_all_rw(self, all_rw: Option<bool>) -> Self {
        Self { all_rw, ..self }
    }
}
