use std::cmp::Ordering;

use crate::task::cluster::hst::v2::HostV2Config;
use crate::task::cluster::ins::Name;
use crate::task::cluster::ins::{v1::Instance, IntoV2, Role};

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
    pub failure_domains: Vec<String>,
    pub roles: Vec<Role>,
    pub config: HostV2Config,
}

#[allow(unused)]
impl Replicaset {
    pub fn instances(&self) -> Vec<InstanceV2> {
        (1..=self.replication_factor.unwrap_or(1))
            .map(|index| InstanceV2 {
                name: self.name.clone_with_index(index),
                stateboard: None,
                weight: self.weight,
                zone: self.zone.clone(),
                failure_domains: self.failure_domains.clone(),
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
/// TODO: docs, remove pub
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
    pub weight: Option<usize>,
    pub zone: Option<String>,
    pub failure_domains: Vec<String>,
    pub roles: Vec<Role>,
    pub config: HostV2Config,
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
    fn as_v2_replicaset(&self) -> Vec<Replicaset> {
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
                        failure_domains: Vec::new(),
                        roles: roles.clone(),
                        config: HostV2Config::from(config.clone()),
                    }
                },
            )
            .collect()
    }
}
