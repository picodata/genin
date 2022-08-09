use std::{fmt::Display, net::IpAddr, ops::Deref};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
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
///         - name: dc-3
///           type: datacenter
///           ports:
///             http: 8091
///             binary: 3031
///           hosts:
///             - name: server-10
///               ip: 10.99.3.100
/// ```
pub struct Hosts(pub Vec<Host>);

impl Default for Hosts {
    /// Host can be Region, Datacenter, Server
    /// ```yaml
    /// hosts:
    ///     - name: datacenter
    ///       type: selectel
    ///       distance: 0
    ///       ports:
    ///         http: 8081
    ///         binary: 3031
    ///       hosts:
    ///         - name: host-1
    ///           ip: 10.20.3.100
    ///         - name: host-2
    ///           ip: 10.20.4.100
    /// ```
    fn default() -> Self {
        Self(vec![Host {
            name: "selectel".into(),
            htype: HostType::Datacenter,
            distance: 0,
            ports: PortsVariants::Ports(Ports::default()),
            ip: IP::None,
            hosts: HostsVariants::Hosts(Hosts(vec![
                Host {
                    name: "host-1".into(),
                    htype: HostType::Server,
                    distance: 0,
                    ports: PortsVariants::None,
                    ip: IP::Server("192.168.16.11".parse().unwrap()),
                    hosts: HostsVariants::None,
                },
                Host {
                    name: "host-2".into(),
                    htype: HostType::Server,
                    distance: 0,
                    ports: PortsVariants::None,
                    ip: IP::Server("192.168.16.12".parse().unwrap()),
                    hosts: HostsVariants::None,
                },
            ])),
        }])
    }
}

impl Deref for Hosts {
    type Target = Vec<Host>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Default)]
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

impl Host {
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Ports {
    pub http: u16,
    pub binary: u16,
}

impl Default for Ports {
    fn default() -> Self {
        Self {
            http: 8081,
            binary: 3031,
        }
    }
}

impl Ports {
    pub fn up(&mut self) {
        self.binary += 1;
        self.http += 1;
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
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

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum HostsVariants {
    Hosts(Hosts),
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

pub fn is_null(u: &usize) -> bool {
    matches!(u, 0)
}
