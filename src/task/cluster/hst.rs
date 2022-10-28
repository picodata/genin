pub (in crate::task) mod v1;
pub (in crate::task) mod v2;

use std::{fmt::Display, net::IpAddr};
use serde::{Deserialize, Serialize};

use crate::error::TaskError;
use crate::task::cluster::ins::v2::InstanceV2;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostType {
    #[serde(rename = "region")]
    Region,
    #[serde(rename = "datacenter")]
    Datacenter,
    #[serde(rename = "server")]
    Server,
}

impl Default for HostType {
    fn default() -> Self {
        Self::Server
    }
}

impl Display for HostType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostType::Region => write!(f, "HostType::Region"),
            HostType::Datacenter => write!(f, "HostType::Region"),
            HostType::Server => write!(f, "HostType::Region"),
        }
    }
}

impl HostType {
    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum PortsVariants {
    Ports(Ports),
    None,
}

impl Default for PortsVariants {
    fn default() -> Self {
        Self::None
    }
}

impl PortsVariants {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn applicate(&self, rhs: &PortsVariants) -> Option<Ports> {
        match (self, rhs) {
            (PortsVariants::Ports(lhs), _) => Some(*lhs),
            (PortsVariants::None, PortsVariants::Ports(rhs)) => Some(*rhs),
            _ => None,
        }
    }

    pub fn up(&mut self) {
        if let PortsVariants::Ports(p) = self {
            p.http += 1;
            p.binary += 1;
        }
    }

    /// if port varinants `None` init them as default
    pub fn or_else(&mut self, ports: Ports) {
        if let PortsVariants::None = self {
            *self = PortsVariants::Ports(ports);
        }
    }

    pub fn http_or_default(&self) -> u16 {
        match self {
            PortsVariants::Ports(p) => p.http,
            PortsVariants::None => Ports::default().http,
        }
    }

    pub fn binary_or_default(&self) -> u16 {
        match self {
            PortsVariants::Ports(p) => p.binary,
            PortsVariants::None => Ports::default().binary,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ports {
    pub http: u16,
    pub binary: u16,
}

impl Default for Ports {
    fn default() -> Self {
        Self {
            http: 8081,
            binary: 3301,
        }
    }
}

#[allow(unused)]
impl Ports {
    pub fn up(&mut self) {
        self.binary += 1;
        self.http += 1;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum IP {
    Server(IpAddr),
    None,
}

impl Default for IP {
    fn default() -> Self {
        Self::None
    }
}

impl ToString for IP {
    fn to_string(&self) -> String {
        match self {
            Self::Server(ip) => ip.to_string(),
            Self::None => String::new(),
        }
    }
}

impl IP {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn applicate(&self, rhs: &IP) -> IP {
        match (self, rhs) {
            (IP::Server(lhs), _) => IP::Server(*lhs),
            (IP::None, IP::Server(rhs)) => IP::Server(*rhs),
            _ => IP::None,
        }
    }
}

pub fn is_null(u: &usize) -> bool {
    matches!(u, 0)
}

#[allow(unused)]
#[derive(Debug)]
pub(in crate::task) struct FlatHost {
    pub(in crate::task) name: String,
    pub(in crate::task) htype: HostType,
    pub(in crate::task) ports: Ports,
    pub(in crate::task) ip: IP,
    pub(in crate::task) deepness: Vec<String>,
    pub(in crate::task) instances: Vec<InstanceV2>,
}

pub(in crate::task) trait TryIntoFlatHosts {
    type Error;

    fn try_into(&self) -> Result<Vec<FlatHost>, Self::Error>;
}

pub(in crate::task) trait Flatten<T> {
    fn flatten(hosts: &Vec<T>, parent: &T) -> Result<Vec<FlatHost>, TaskError>;
}

pub(in crate::task) trait MaxLen {
    fn max_len(&self) -> usize;
}

impl MaxLen for Vec<FlatHost> {
    fn max_len(&self) -> usize {
        self.iter()
            .max_by(|a, b| a.instances.len().cmp(&b.instances.len()))
            .map(|host| host.instances.len())
            .unwrap_or_else(|| self.first().map(|host| host.instances.len()).unwrap_or(0))
    }
}

impl FlatHost {
    pub(in crate::task) fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod test;
