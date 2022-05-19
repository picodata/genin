pub mod libs;

//mod def;
//mod flv;
//mod lib.ins;

//use clap::ArgMatches;
//use linked_hash_map::LinkedHashMap;
//use log::{debug, trace, warn};
//use regex::Regex;
//use serde::{Deserialize, Deserializer, Serialize, Serializer};
//use serde_yaml::Value;
//use std::fmt::Display;
//use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};
//use std::{cmp::Ordering, net::IpAddr, num::IntErrorKind, str::FromStr};
//
//// For more compliant use
//pub use def::*;
//pub use flv::*;
//pub use lib.ins::*;
//
//#[derive(Debug, Default, Serialize, Deserialize)]
//#[serde(default)]
///// Inventory `Vars` wrapper
/////
///// # Examples
/////
///// ```rust
///// use genin::{Vars, Name, CartridgeFailoverParams, InstanceConfig}
/////
///// let vars = Vars {
/////     rpm_url: ,
/////     rpm_checksum: ,
/////     ansible_user: vagrant,
/////     ansible_password: vagrant,
/////     cartridge_app_name: example,
/////     cartridge_package_path: /tmp/example.rpm,
/////     cartridge_cluster_cookie: example-cookie,
/////     cartridge_failover_params: CartridgeFailoverParams::default(),
/////     cartridge_defaults: InstanceConfig::default(),
/////     // this value can by and another type (mapping, sequence, string, ..etc)
/////     value: Value::Null,
///// }
///// ```
//pub struct Vars {
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub rpm_url: String,
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub rpm_checksum: String,
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub ansible_user: String,
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub ansible_password: String,
//    #[serde(skip_serializing_if = "Name::is_empty")]
//    pub cartridge_app_name: Name,
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub cartridge_package_path: String,
//    #[serde(skip_serializing_if = "String::is_empty")]
//    pub cartridge_cluster_cookie: String,
//    //#[serde(skip_serializing_if = "CartridgeFailoverParams::is_empty")]
//    //pub cartridge_failover_params: CartridgeFailoverParams,
//    #[serde(skip_serializing_if = "InstanceConfig::is_empty")]
//    pub cartridge_defaults: InstanceConfig,
//    #[serde(flatten, default, skip_serializing_if = "Value::is_null")]
//    pub value: Value,
//}
//
//#[allow(unused)]
//#[derive(Clone, Debug)]
///// Standart `Uri` with port or without
/////
///// # Examples
/////
///// ```rust
///// use genin::Uri;
/////
///// let uri = Uri::from(String::from("localhost:5630"));
///// let uri = "localhost:5630".into()
///// ```
//pub struct Uri(IpAddr, Port);
//
//#[allow(unused)]
//#[derive(Debug, Clone, Serialize, Deserialize)]
///// Wrapper around http and binary port value
/////
///// # Examples
/////
///// ```rust
///// use genin::Ports;
/////
///// let ports = Ports{
/////     begin_http: 8080.into(),
/////     begin_binary: "3030".into(),
///// }
///// ```
//pub struct Ports {
//    pub begin_http: Port,
//    pub begin_binary: Port,
//}
//
//#[derive(Default, Debug, Clone, Serialize, Deserialize)]
///// TCP/UDP Port u16
//pub struct Port(u16);
//
//#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
///// Abstract name without forbidden symbols
/////
///// # Examples
/////
///// ```rust
///// use genin::Name;
/////
///// let name = Name::from(String::from("example-app"));
///// let name = "example-app".into();
///// ```
//pub struct Name(pub String);
//
//#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
///// Type for simple parsing some count like numbers of replicas
/////
///// # Examples
/////
///// ```rust
///// use genin::Count;
/////
///// assert_eq!(Count::from(1), Count(1))
///// assert_eq!(*Count(10), 10);
///// // value restricted 8 bits (0-255)
///// assert_err!(Count(600));
///// ```
//pub struct Count(u8);
//
//#[derive(Debug, Serialize, Deserialize)]
//#[serde(untagged)]
///// Hosts enum repesentation
/////
///// # Examples
/////
///// ```rust
///// use genin::Hosts;
/////
///// let yaml = "ports:/n  begin_http: 8080/n  begin_binary:8080/nvagrant:/n  \
/////     host1: 100.0.2.1\n  host2: 100.0.3.1"
///// let hosts: Hosts = serde_yaml::from_str(yaml).expect("deserialization error");
///// match hosts {
/////     Hosts::Datacenter(_) => println!("Ok!"),
/////     _ => panic!().
///// }
///// ```
//pub enum Hosts {
//    Datacenter(LinkedHashMap<Name, ConfiguredHosts>),
//    Hosts(LinkedHashMap<Name, IpAddr>),
//}
//
//#[derive(Debug, Serialize, Deserialize)]
///// Wrapper for inner dc/rack/region `ConfiguredHosts`
//pub struct ConfiguredHosts {
//    #[serde(default = "Ports::config_default")]
//    pub ports: Ports,
//    #[serde(flatten)]
//    pub hosts: Hosts,
//}
//
//#[derive(Debug, Default, Clone, Hash)]
//pub struct Replicaset {
//    pub name: Name,
//    pub members: Option<Vec<Name>>,
//}
//
//#[derive(Clone, Debug, PartialEq, Serialize)]
//pub enum RelationshipType {
//    Master,
//    Replica,
//}
//
//#[derive(Debug, Clone, PartialEq)]
///// Invariant with two states
//pub enum IpAddrPool {
//    Pool(Vec<IpAddr>),
//    Single(IpAddr),
//}
//
//pub trait IsEmpty {
//    fn is_empty(&self) -> bool;
//}
//
//// ---
//// Count implementation
//// ---
//impl Count {
//    /// Return true if Count is empty
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::Count;
//    ///
//    /// let cnt = Count::default();
//    /// assert_eq!(cnt.is_empty(), true);
//    /// ```
//    pub fn is_null(&self) -> bool {
//        self.0 == 0
//    }
//
//    /// Creates new Count with value 1
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::Count;
//    ///
//    /// let cnt = Count::one();
//    /// assert_eq!(cnt, Count(1));
//    /// ````
//    pub fn one() -> Self {
//        Self(1)
//    }
//}
//
//impl Deref for Count {
//    type Target = u8;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//
//impl Add for Count {
//    type Output = Self;
//
//    fn add(self, rhs: Self) -> Self::Output {
//        Self(self.0 + *rhs)
//    }
//}
//
//impl AddAssign for Count {
//    fn add_assign(&mut self, rhs: Self) {
//        self.0 += *rhs;
//    }
//}
//
//impl Sub for Count {
//    type Output = Self;
//
//    fn sub(self, rhs: Self) -> Self::Output {
//        Self(self.0 - *rhs)
//    }
//}
//
//impl SubAssign for Count {
//    fn sub_assign(&mut self, rhs: Self) {
//        self.0 -= *rhs;
//    }
//}
//
//impl From<u8> for Count {
//    fn from(u: u8) -> Self {
//        Self(u)
//    }
//}
//
//// ---
//// Name implementation
//// ---
//impl<'a> From<&'a str> for Name {
//    fn from(s: &'a str) -> Self {
//        let reg = Regex::new("^[a-zA-Z0-9-_]+$").unwrap();
//        match (s.len(), reg.is_match(s)) {
//            (3..=40, true) => Self(String::from(s)),
//            _ => panic!("Invalid instance name! {}", s),
//        }
//    }
//}
//
//impl From<String> for Name {
//    fn from(s: String) -> Self {
//        Self::from(s.as_str())
//    }
//}
//
//impl IsEmpty for Name {
//    fn is_empty(&self) -> bool {
//        self.0.is_empty()
//    }
//}
//
//impl Display for Name {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        write!(f, "{}", &self.0)
//    }
//}
//
//impl Deref for Name {
//    type Target = String;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//
//// ---
//// Port implementation
//// ---
//impl Port {
//    /// Increase the value of inside the `Port` by 1
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::Port;
//    ///
//    /// let port = Port::from(8080);
//    /// assert_eq!(port.increase(), Port(8081));
//    ///
//    /// let port = Port::from(u16::MAX);
//    /// assert_err!(port::increase());
//    /// ```
//    pub fn increase(&mut self) {
//        if self.0 != u16::MAX {
//            self.0 += 1;
//        } else {
//            panic!("Port overflow error! {}", self.0);
//        }
//    }
//}
//
//impl From<u16> for Port {
//    fn from(num: u16) -> Self {
//        Self(num)
//    }
//}
//
//impl<'a> TryFrom<&'a str> for Port {
//    type Error = PortConversionError;
//
//    fn try_from(s: &'a str) -> Result<Port, Self::Error> {
//        match s.parse::<u16>() {
//            Ok(u) => Ok(Self(u)),
//            Err(e) => match e.kind() {
//                IntErrorKind::InvalidDigit => Err(PortConversionError::InvalidCharacters(format!(
//                    "Error then parsing {}! Invalid characters!",
//                    s
//                ))),
//                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
//                    Err(PortConversionError::InvalidRange(format!(
//                        "Error then parsing {}! Port value out of range",
//                        s,
//                    )))
//                }
//                _ => Err(PortConversionError::UnknownError(format!(
//                    "Error then parsing {}! Unknown error!",
//                    s
//                ))),
//            },
//        }
//    }
//}
//
//impl ToString for Port {
//    fn to_string(&self) -> String {
//        self.0.to_string()
//    }
//}
//
//impl Deref for Port {
//    type Target = u16;
//
//    fn deref(&self) -> &Self::Target {
//        &self.0
//    }
//}
//
//impl IsEmpty for Port {
//    fn is_empty(&self) -> bool {
//        matches!(self.0, 0)
//    }
//}
//
//// ---
//// PortConversionError implementation
//// ---
//#[derive(Debug)]
//pub enum PortConversionError {
//    InvalidRange(String),
//    InvalidCharacters(String),
//    UnknownError(String),
//}
//
//impl ToString for PortConversionError {
//    fn to_string(&self) -> String {
//        match self {
//            Self::InvalidRange(s) => s.to_string(),
//            Self::InvalidCharacters(s) => s.to_string(),
//            Self::UnknownError(s) => s.to_string(),
//        }
//    }
//}
//
//// ---
//// Ports implementation
//// ---
//impl Ports {
//    pub fn config_default() -> Self {
//        Self {
//            begin_http: Port::from(8081),
//            begin_binary: Port::from(3031),
//        }
//    }
//}
//
//impl Default for Ports {
//    fn default() -> Self {
//        Self {
//            begin_http: 8080u16.into(),
//            begin_binary: 3030u16.into(),
//        }
//    }
//}
//
//// ---
//// Uri implementation
//// ---
//impl Uri {
//    /// Get from struct `Uri` enum `IpAddr`
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::{Uri, Port};
//    /// use std::net::IpAddr;
//    ///
//    /// let ip_addr = IpAddr::from("192.168.0.1:8080");
//    /// assert_eq!(ip_addr.get_ip(), &IpAddr::from([192, 168, 0, 1]));
//    /// ```
//    pub fn get_ip(&self) -> &IpAddr {
//        &self.0
//    }
//
//    /// Get from struct `Uri` enum `IpAddr`
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::{Uri, Port};
//    /// use std::net::IpAddr;
//    ///
//    /// let ip_addr = IpAddr::from("192.168.0.1:8080");
//    /// assert_eq!(ip_addr.get_port(), &Port::from(8080));
//    /// ```
//    pub fn get_port(&self) -> &Port {
//        &self.1
//    }
//
//    /// Get from struct `Uri` all parts `(IpAddr, Port)`
//    ///
//    /// # Examples
//    ///
//    /// ```rust
//    /// use genin::{Uri, Port};
//    /// use std::net::IpAddr;
//    ///
//    /// let uri = Uri::from("192.168.0.1:8080")
//    /// assert_eq!(
//    ///     uri.get_all(),
//    ///     (&IpAddr::from([192.168.0.1]), Port::from(8080))
//    /// );
//    /// ```
//    pub fn get_all(&self) -> (&IpAddr, &Port) {
//        (&self.0, &self.1)
//    }
//}
//
//impl Default for Uri {
//    fn default() -> Self {
//        Self([0, 0, 0, 0].into(), 0u16.into())
//    }
//}
//
//impl<'a> From<&'a str> for Uri {
//    fn from(s: &'a str) -> Self {
//        match (
//            s.contains('/'),
//            (s.starts_with("http") || s.starts_with("https")),
//        ) {
//            (false, false) => {
//                let uri = s.split(':').collect::<Vec<&str>>();
//                Self(
//                    IpAddr::from_str(*uri.first().unwrap()).unwrap(),
//                    Port::try_from(*uri.last().unwrap()).unwrap(),
//                )
//            }
//            _ => panic!("Invalid uri {}", s),
//        }
//    }
//}
//
//impl From<String> for Uri {
//    fn from(s: String) -> Self {
//        Self::from(s.as_str())
//    }
//}
//
//impl ToString for Uri {
//    fn to_string(&self) -> String {
//        format!("{}:{}", self.0, &*self.1)
//    }
//}
//
//impl<'a> From<(&'a IpAddr, &'a Port)> for Uri {
//    fn from(t: (&'a IpAddr, &'a Port)) -> Self {
//        Self(*t.0, t.1.clone())
//    }
//}
//
//impl IsEmpty for Uri {
//    fn is_empty(&self) -> bool {
//        self.0.is_unspecified() && self.1.is_empty()
//    }
//}
//
//impl<'de> Deserialize<'de> for Uri {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//    where
//        D: Deserializer<'de>,
//    {
//        #[allow(clippy::redundant_closure)]
//        Deserialize::deserialize(deserializer).map(|s: String| Uri::from(s))
//    }
//}
//
//impl Serialize for Uri {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        serializer.serialize_str(&self.to_string())
//    }
//}
//
//// ---
//// Hosts implementation
//// ---
//impl Hosts {
//    pub fn deep_len(&self) -> Count {
//        match self {
//            Self::Datacenter(datacentres) => Count({
//                let mut len = Count(0);
//                datacentres
//                    .iter()
//                    .for_each(|(_, ConfiguredHosts { hosts, .. })| len += hosts.deep_len());
//                *len
//            }),
//            Self::Hosts(hosts) => Count({
//                match hosts.len().try_into() {
//                    Ok(u) => u,
//                    Err(e) => panic!("Error while counting hosts lengh {}", e),
//                }
//            }),
//        }
//    }
//
//    pub fn get_addresses(&self) -> Vec<IpAddr> {
//        match self {
//            Self::Datacenter(datacentres) => {
//                let mut addresses = Vec::new();
//                datacentres
//                    .iter()
//                    .for_each(|(_, ConfiguredHosts { hosts, .. })| {
//                        addresses.extend(hosts.get_addresses())
//                    });
//                addresses
//            }
//            Self::Hosts(hosts) => hosts
//                .iter()
//                .map(|(_, address)| *address)
//                .collect::<Vec<IpAddr>>(),
//        }
//    }
//}
//
//impl Default for Hosts {
//    fn default() -> Self {
//        Self::Hosts({
//            let mut hosts = LinkedHashMap::new();
//            hosts.insert(Name::from("localhost"), IpAddr::from([0, 0, 0, 0]));
//            hosts
//        })
//    }
//}
//
//// ---
//// ConfiguredHosts implementation
//// ---
//impl Default for ConfiguredHosts {
//    fn default() -> Self {
//        Self {
//            ports: Ports::config_default(),
//            hosts: Hosts::default(),
//        }
//    }
//}
//
//// ---
//// RelationshipType implementation
//// ---
//impl Default for RelationshipType {
//    fn default() -> Self {
//        Self::Master
//    }
//}
//
//// ---
//// IpAddrPool implementation
//// ---
//impl IpAddrPool {
//    pub fn extend(&mut self, ip_addr_pool: IpAddrPool) {
//        let extend = |pool: &mut Vec<IpAddr>| match ip_addr_pool {
//            IpAddrPool::Pool(ex_pool) => pool.extend(&ex_pool),
//            IpAddrPool::Single(ex_single) => pool.push(ex_single),
//        };
//        match self {
//            Self::Pool(pool) => extend(pool),
//            Self::Single(single) => {
//                let mut new_pool = vec![*single];
//                extend(&mut new_pool);
//                *self = IpAddrPool::Pool(new_pool);
//            }
//        }
//    }
//}
//
//impl ToString for IpAddrPool {
//    fn to_string(&self) -> String {
//        match self {
//            Self::Pool(pool) => pool
//                .iter()
//                .map(|ip| ip.to_string())
//                .collect::<Vec<String>>()
//                .join(", "),
//            Self::Single(single) => single.to_string(),
//        }
//    }
//}

