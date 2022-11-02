pub (in crate::task) mod v1;
pub (in crate::task) mod v2;

use std::{fmt::Display, net::IpAddr};
use serde::{Deserialize, Serialize};

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
            HostType::Datacenter => write!(f, "HostType::Datacenter"),
            HostType::Server => write!(f, "HostType::Server"),
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

    pub fn http_as_option(&self) -> Option<usize> {
        match self {
            PortsVariants::Ports(p) => Some(usize::from(p.http)),
            PortsVariants::None => None,
        }
    }

    pub fn binary_as_option(&self) -> Option<usize> {
        match self {
            PortsVariants::Ports(p) => Some(usize::from(p.binary)),
            PortsVariants::None => None,
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
}

pub fn is_null(u: &usize) -> bool {
    matches!(u, 0)
}

#[cfg(test)]
mod test;
