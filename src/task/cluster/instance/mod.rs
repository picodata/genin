pub mod ins;

use indexmap::IndexMap;
use serde::{de::Visitor, Deserialize, Serialize};
use serde_yaml::Value;

use super::host::hst::Address;

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

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
pub struct Config {
    pub http_port: Option<usize>,
    pub binary_port: Option<usize>,
    pub address: Address,
    #[serde(flatten)]
    pub config: IndexMap<String, Value>,
}

impl From<IndexMap<String, Value>> for Config {
    fn from(config: IndexMap<String, Value>) -> Self {
        Config {
            config,
            ..Config::default()
        }
    }
}

//TODO: fix test
//#[cfg(test)]
//mod test;
