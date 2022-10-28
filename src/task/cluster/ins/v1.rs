use std::cmp::Ordering;

use serde::{Serialize, Deserialize, de::Error};
use indexmap::IndexMap;
use serde_yaml::Value;

use crate::task::cluster::ins::{Type, count_one, default_weight, is_zero, Role, is_false};

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

impl PartialOrd for Instance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.itype, other.itype) {
            (Type::Router, Type::Storage)
            | (Type::Router, Type::Custom)
            | (Type::Router, Type::Replica)
            | (Type::Storage, Type::Custom)
            | (Type::Storage, Type::Replica) => Some(Ordering::Less),
            (left, right) if left == right => self.name.partial_cmp(&other.name),
            _ => Some(Ordering::Greater),
        }
    }
}

impl Ord for Instance {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.itype, other.itype) {
            (Type::Router, Type::Storage)
            | (Type::Router, Type::Custom)
            | (Type::Router, Type::Replica)
            | (Type::Storage, Type::Custom)
            | (Type::Storage, Type::Replica) => Ordering::Less,
            (left, right) if left == right => self.name.cmp(&other.name),
            _ => Ordering::Greater,
        }
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

