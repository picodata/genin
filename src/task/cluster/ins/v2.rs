use indexmap::IndexMap;
use serde::{de::Error, Deserialize, Serialize};
use serde_yaml::Value;
use std::cmp::Ordering;

use crate::task::cluster::ins::{
    count_one, default_weight, is_false, is_zero, v1::Instance, IntoV2, Role, Type,
};

#[derive(Serialize, Clone, Debug, Default, PartialEq, Eq)]
/// Some tarantool cartridge instance
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct InstanceV2 {
    pub name: String,
    #[serde(skip)]
    pub parent: String,
    #[serde(rename = "type")]
    pub itype: Type,
    #[serde(default)]
    pub replicasets_count: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub replication_factor: usize,
    #[serde(default = "default_weight", skip_serializing_if = "is_zero")]
    pub weight: usize,
    #[serde(default, skip_serializing_if = "is_false")]
    pub stateboard: bool,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub config: IndexMap<String, Value>,
}

impl PartialOrd for InstanceV2 {
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

impl Ord for InstanceV2 {
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

impl<'de> Deserialize<'de> for InstanceV2 {
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
            pub replicasets_count: usize,
            #[serde(default = "count_one")]
            pub replication_factor: usize,
            #[serde(default = "default_weight")]
            pub weight: usize,
            pub roles: Vec<Role>,
            pub config: IndexMap<String, Value>,
        }

        if let Ok(InstanceHelper {
            mut roles,
            mut itype,
            name,
            replicasets_count,
            replication_factor,
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
                return Ok(InstanceV2 {
                    name,
                    itype,
                    replicasets_count,
                    replication_factor,
                    weight,
                    roles,
                    config,
                    ..Default::default()
                });
            } else {
                return Ok(InstanceV2 {
                    itype: Type::from(name.as_str()),
                    name,
                    replicasets_count: 0,
                    replication_factor: 0,
                    weight: 0,
                    roles,
                    config,
                    ..Default::default()
                });
            }
        }
        Err(Error::custom("Error then deserializing InstanceV2"))
    }
}

#[allow(unused)]
impl InstanceV2 {
    /// Check that instance spreading should be forsed through hosts
    pub fn can_be_same(&self) -> bool {
        matches!(self.itype, Type::Router | Type::Storage | Type::Replica)
    }

    /// Multiply instances to `count`
    /// if instance is a storage and has replicas, multiply them too
    pub fn multiply(&self) -> Vec<Vec<InstanceV2>> {
        let mut result = vec![(1..=self.replicasets_count)
            .map(|master_num| InstanceV2 {
                name: format!("{}-{}", &self.name, master_num),
                parent: format!("{}-{}", &self.name, master_num),
                replicasets_count: 1,
                replication_factor: 0,
                itype: self.itype,
                weight: self.weight,
                stateboard: false,
                roles: self.roles.clone(),
                config: self.config.clone(),
            })
            .rev()
            .collect()];
        result.extend(match self.itype {
            Type::Storage => (1..=self.replicasets_count)
                .map(|master_num| {
                    (1..=self.replication_factor)
                        .map(|replica_num| InstanceV2 {
                            name: format!("{}-{}-replica-{}", &self.name, master_num, replica_num),
                            parent: format!("{}-{}", &self.name, master_num),
                            replicasets_count: 1,
                            replication_factor: 0,
                            itype: Type::Replica,
                            weight: self.weight,
                            stateboard: false,
                            roles: self.roles.clone(),
                            config: self.config.clone(),
                        })
                        .rev()
                        .chain((1..=master_num).map(|num| InstanceV2 {
                            name: format!("dummy-{}", num),
                            parent: format!("dummy-{}", num),
                            replicasets_count: 1,
                            replication_factor: 0,
                            itype: Type::Dummy,
                            weight: self.weight,
                            stateboard: false,
                            roles: self.roles.clone(),
                            config: self.config.clone(),
                        }))
                        .collect::<Vec<InstanceV2>>()
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

impl IntoV2 for Vec<Instance> {
    fn into_v2(&self) -> Vec<InstanceV2> {
        self.iter()
            .map(
                |Instance {
                     name,
                     parent,
                     itype,
                     count,
                     replicas,
                     weight,
                     stateboard,
                     roles,
                     config,
                 }| {
                    InstanceV2 {
                        name: name.clone(),
                        parent: parent.clone(),
                        itype: itype.clone(),
                        replicasets_count: *count,
                        replication_factor: *replicas,
                        weight: *weight,
                        stateboard: *stateboard,
                        roles: roles.clone(),
                        config: config.clone(),
                    }
                },
            )
            .collect()
    }
}
