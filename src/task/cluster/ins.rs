pub(in crate::task) mod v1;
pub(in crate::task) mod v2;

use std::fmt::Display;

use indexmap::IndexMap;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Type {
    Storage,
    Replica,
    Router,
    Custom,
    Dummy,
    Unknown,
}

impl Default for Type {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<'a> From<&'a str> for Type {
    fn from(s: &'a str) -> Self {
        match s.to_lowercase().as_str() {
            "storage" => Type::Storage,
            "router" => Type::Router,
            _ => Type::Custom,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Role {
    Custom(String),
    FailoverCoordinator(String),
    Storage(String),
    Router(String),
    Api(String),
}

struct RoleVisitor;

impl<'de> Visitor<'de> for RoleVisitor {
    type Value = Role;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a tarantool app role")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "failover-coordinator" => Ok(Role::FailoverCoordinator(v.into())),
            "storage" | "app.storage" | "app.role.storage" => Ok(Role::Storage(v.into())),
            "router" | "app.router" | "app.role.router" => Ok(Role::Router(v.into())),
            "api" | "app.api" | "app.role.api" => Ok(Role::Api(v.into())),
            _ => Ok(Role::Custom(v.into())),
        }
    }
}

impl<'de> Deserialize<'de> for Role {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RoleVisitor)
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::FailoverCoordinator(s) => serializer.serialize_str(s),
            Self::Storage(s) => serializer.serialize_str(s),
            Self::Router(s) => serializer.serialize_str(s),
            Self::Api(s) => serializer.serialize_str(s),
            Self::Custom(s) => serializer.serialize_str(s),
        }
    }
}

impl<'a> From<&'a str> for Role {
    fn from(s: &'a str) -> Self {
        match s.to_lowercase().as_str() {
            s if s.contains("storage") => Self::Storage(s.to_string()),
            s if s.contains("router") => Self::Router(s.to_string()),
            s => Self::Custom(s.to_string()),
        }
    }
}

#[allow(unused)]
impl Role {
    #[inline]
    pub fn failover_coordinator() -> Self {
        Self::FailoverCoordinator("failover-coordinator".into())
    }
    #[inline]
    pub fn storage() -> Self {
        Self::Storage("storage".into())
    }
    #[inline]
    pub fn router() -> Self {
        Self::Router("router".into())
    }
    #[inline]
    pub fn api() -> Self {
        Self::Api("api".into())
    }
}

pub fn is_zero(u: &usize) -> bool {
    matches!(u, 0)
}

pub fn is_false(v: &bool) -> bool {
    !*v
}

pub fn default_weight() -> usize {
    10
}

#[allow(unused)]
fn count_one() -> usize {
    1
}

pub trait IntoV2 {
    fn into_v2(&self) -> Vec<v2::Replicaset>;
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    http_port: Option<usize>,
    binary_port: Option<usize>,
    #[serde(flatten)]
    config: IndexMap<String, Value>,
}

impl From<IndexMap<String, Value>> for Config {
    fn from(config: IndexMap<String, Value>) -> Self {
        Config {
            config,
            ..Config::default()
        }
    }
}

impl Config {
    pub fn is_empty(&self) -> bool {
        self.http_port.is_none() && self.binary_port.is_none() && self.config.is_empty()
    }

    pub fn config(&self) -> IndexMap<String, Value> {
        self.config.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name {
    childrens: Vec<String>,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.childrens.last().unwrap())
    }
}

impl<'a> From<&'a str> for Name {
    fn from(s: &'a str) -> Self {
        Self {
            childrens: vec![s.to_string()],
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self { childrens: vec![s] }
    }
}

impl<'a> From<&'a Name> for &'a str {
    fn from(val: &'a Name) -> Self {
        val.childrens.last().map(|s| s.as_str()).unwrap()
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.childrens
            .last()
            .unwrap()
            .partial_cmp(other.childrens.last().unwrap())
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.childrens
            .last()
            .unwrap()
            .cmp(other.childrens.last().unwrap())
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.into())
    }
}

impl Name {
    pub fn with_index<T: Display>(self, index: T) -> Self {
        Self {
            childrens: vec![
                self.childrens.clone(),
                vec![format!("{}-{}", self.childrens.last().unwrap(), index)],
            ]
            .concat(),
        }
    }

    pub fn clone_with_index<T: Display>(&self, index: T) -> Self {
        Self {
            childrens: vec![
                self.childrens.clone(),
                vec![format!("{}-{}", self.childrens.last().unwrap(), index)],
            ]
            .concat(),
        }
    }

    /// Returns the name of the ancestor on the basis of which the
    /// current name is formed.
    ///
    /// * If the Name has no children, then the original name will be
    /// returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let topology_member_name = Name::from("router");
    /// let replicaset_name = topology_member_name.clone_with_index(1);
    /// let instance_name = topology_member_name.clone_with_index(3);
    ///
    /// assert_eq!(instance_name.name(), "router-1-3");
    /// assert_eq!(instance_name.get_ancestor(), "router");
    ///
    /// // Ancestor name of topology_member_name is "router" because he
    /// // does not have childrens.
    /// assert_eq!(topology_member_name.get_ancestor(), "router");
    /// ```
    pub fn get_ancestor(&self) -> &str {
        self.childrens.first().unwrap()
    }

    /// Returns the name of the parent on the basis of which the
    /// current name is formed.
    ///
    /// * If the parent Name has no children, then the original name
    /// will be returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let topology_member_name = Name::from("router");
    /// let replicaset_name = topology_member_name.clone_with_index(1);
    /// let instance_name = topology_member_name.clone_with_index(3);
    ///
    /// assert_eq!(instance_name.name(), "router-1-3");
    /// assert_eq!(instance_name.get_parent(), "router-1");
    /// assert_eq!(replicaset_name.get_parent(), "router");
    ///
    /// // Parent name of topology_member_name is "router" because he
    /// // does not have childrens.
    /// assert_eq!(topology_member_name.get_parent(), "router");
    /// ```
    pub fn get_parent(&self) -> &str {
        self.childrens
            .get(self.childrens.len() - 2)
            .unwrap_or_else(|| self.childrens.first().unwrap())
    }
}

//TODO: fix test
//#[cfg(test)]
//mod test;
