pub mod hst;
pub mod view;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, hash::Hash, net::IpAddr};

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

pub fn merge_index_maps<A, B>(left: IndexMap<A, B>, right: IndexMap<A, B>) -> IndexMap<A, B>
where
    A: Hash + Eq,
{
    let mut left = left;
    right.into_iter().for_each(|(key, value)| {
        left.entry(key).or_insert(value);
    });
    left
}

#[cfg(test)]
mod test;
