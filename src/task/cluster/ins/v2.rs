use std::cmp::Ordering;

use crate::task::cluster::ins::Name;
use crate::task::cluster::ins::{v1::Instance, Config, IntoV2, Role};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Replicaset TODO: docs, remove public
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct Replicaset {
    pub name: Name,
    pub replicasets_count: Option<usize>,
    pub replication_factor: Option<usize>,
    pub weight: Option<usize>,
    pub zone: Option<String>,
    pub roles: Vec<Role>,
    pub config: Config,
    pub instances: Vec<InstanceV2>,
}

#[allow(unused)]
impl Replicaset {
    pub fn instances(&mut self) -> Vec<InstanceV2> {
        (1..=self.replication_factor.unwrap_or(1))
            .map(|index| InstanceV2 {
                name: self.name.clone_with_index(index),
                stateboard: Some(false),
                roles: self.roles.clone(),
                config: self.config.clone(),
            })
            .collect()
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}

impl PartialOrd for Replicaset {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            ord => ord,
        }
    }
}

impl Ord for Replicaset {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// TODO: docs
///
/// ```yaml
/// - name: "catalogue"
///   type: "storage"
///   replicasets_count: 1
///   replication_factor: 2
///   weight: 10
/// ```
pub struct InstanceV2 {
    pub name: Name,
    pub stateboard: Option<bool>,
    pub roles: Vec<Role>,
    pub config: Config,
}

impl PartialOrd for InstanceV2 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(Ordering::Equal) => Some(Ordering::Equal),
            ord => ord,
        }
    }
}

impl Ord for InstanceV2 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl IntoV2 for Vec<Instance> {
    fn into_v2(&self) -> Vec<Replicaset> {
        self.iter()
            .map(
                |Instance {
                     name,
                     count,
                     replicas,
                     weight,
                     roles,
                     config,
                     ..
                 }| {
                    Replicaset {
                        name: Name::from(name.as_str()),
                        replicasets_count: Some(*count),
                        replication_factor: Some(*replicas),
                        weight: Some(*weight),
                        zone: None,
                        roles: roles.clone(),
                        config: Config {
                            config: config.clone(),
                            ..Config::default()
                        },
                        instances: Vec::new(), //TODO
                    }
                },
            )
            .collect()
    }
}
