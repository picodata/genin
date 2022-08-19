use std::{cmp::Ordering, ops::Deref};

use indexmap::IndexMap;
use serde::{
    de::{Error, Visitor},
    Deserialize, Serialize,
};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Instances(Vec<Instance>);

#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
/// Some tarantool cartridge instance
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   count: 1
///   replicas: 2
///   weight: 10
/// ```
pub struct Instance {
    pub name: String,
    #[serde(skip)]
    pub parent: String,
    #[serde(rename = "type")]
    pub itype: Type,
    #[serde(default)]
    pub count: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub replicas: usize,
    #[serde(default = "default_weight", skip_serializing_if = "is_zero")]
    pub weight: usize,
    #[serde(default, skip_serializing_if = "is_false")]
    pub stateboard: bool,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub config: IndexMap<String, Value>,
}

impl Default for Instances {
    fn default() -> Self {
        Self(vec![
            Instance {
                name: "router".into(),
                parent: "router".into(),
                itype: Type::Router,
                count: 1,
                replicas: 0,
                weight: 0,
                stateboard: false,
                roles: vec![Role::router(), Role::failover_coordinator()],
                config: IndexMap::new(),
            },
            Instance {
                name: "storage".into(),
                parent: "storage".into(),
                itype: Type::Storage,
                count: 2,
                replicas: 1,
                weight: 10,
                stateboard: false,
                roles: vec![Role::storage()],
                config: IndexMap::new(),
            },
        ])
    }
}

impl Deref for Instances {
    type Target = [Instance];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

impl Instances {
    /// Sort instances by spreading priority
    pub fn sort(mut self) -> Self {
        self.0.sort_by(|a, b| match (a.itype, b.itype) {
            (Type::Router, Type::Storage)
            | (Type::Router, Type::Custom)
            | (Type::Router, Type::Replica)
            | (Type::Storage, Type::Custom)
            | (Type::Storage, Type::Replica) => Ordering::Less,
            (left, right) if left == right => a.name.cmp(&b.name),
            _ => Ordering::Greater,
        });
        self
    }
}

impl<'de> Deserialize<'de> for Instance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct InstanceHelper {
            pub name: String,
            #[serde(rename = "type")]
            pub itype: Type,
            pub count: usize,
            #[serde(default = "count_one")]
            pub replicas: usize,
            #[serde(default = "default_weight")]
            pub weight: usize,
            pub roles: Vec<Role>,
            pub config: IndexMap<String, Value>,
        }

        if let Ok(InstanceHelper {
            mut roles,
            mut itype,
            name,
            count,
            replicas,
            weight,
            config,
            ..
        }) = InstanceHelper::deserialize(deserializer)
        {
            // If type not defined in yaml let's try to infer based on name
            if itype == Type::Unknown {
                itype = Type::from(name.as_str());
            }

            if roles.is_empty() {
                roles = vec![Role::from(name.as_str())]
            }
            if itype == Type::Storage {
                return Ok(Instance {
                    name,
                    itype,
                    count,
                    replicas,
                    weight,
                    roles,
                    config,
                    ..Default::default()
                });
            } else {
                return Ok(Instance {
                    itype: Type::from(name.as_str()),
                    name,
                    count,
                    replicas: 0,
                    weight: 0,
                    roles,
                    config,
                    ..Default::default()
                });
            }
        }
        Err(Error::custom("Error then deserializing Instance"))
    }
}

#[allow(unused)]
impl Instance {
    /// Check that instance spreading should be forsed through hosts
    pub fn can_be_same(&self) -> bool {
        matches!(self.itype, Type::Router | Type::Storage | Type::Replica)
    }

    /// Multiply instances to `count`
    /// if instance is a storage and has replicas, multiply them too
    pub fn multiply(&self) -> Vec<Vec<Instance>> {
        let mut result = vec![(1..=self.count)
            .map(|master_num| Instance {
                name: format!("{}-{}", &self.name, master_num),
                parent: format!("{}-{}", &self.name, master_num),
                count: 1,
                replicas: 0,
                itype: self.itype,
                weight: self.weight,
                stateboard: false,
                roles: self.roles.clone(),
                config: self.config.clone(),
            })
            .rev()
            .collect()];
        result.extend(match self.itype {
            Type::Storage => (1..=self.count)
                .map(|master_num| {
                    (1..=self.replicas)
                        .map(|replica_num| Instance {
                            name: format!("{}-{}-replica-{}", &self.name, master_num, replica_num),
                            parent: format!("{}-{}", &self.name, master_num),
                            count: 1,
                            replicas: 0,
                            itype: Type::Replica,
                            weight: self.weight,
                            stateboard: false,
                            roles: self.roles.clone(),
                            config: self.config.clone(),
                        })
                        .rev()
                        .chain((1..=master_num).map(|num| Instance {
                            name: format!("dummy-{}", num),
                            parent: format!("dummy-{}", num),
                            count: 1,
                            replicas: 0,
                            itype: Type::Dummy,
                            weight: self.weight,
                            stateboard: false,
                            roles: self.roles.clone(),
                            config: self.config.clone(),
                        }))
                        .collect::<Vec<Instance>>()
                })
                .collect(),

            _ => Vec::new(),
        });
        result
    }

    pub fn is_not_dummy(&self) -> bool {
        !matches!(self.itype, Type::Dummy)
    }
}

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

#[cfg(test)]
mod test;
