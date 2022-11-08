use serde::{Deserialize, Serialize};

use crate::task::cluster::hst::{is_null, HostType, PortsVariants, IP};

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

#[allow(unused)]
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
