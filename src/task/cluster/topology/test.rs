use indexmap::IndexMap;

use crate::task::cluster::{
    hst::view::{FG_BLUE, FG_CYAN, FG_WHITE},
    ins::{
        v2::{InstanceV2, InstanceV2Config, Instances},
        Role,
    },
    name::Name,
    topology::{Topology, TopologySet},
};

#[test]
/// Forward and reverse conversion test between instances and topology
/// instances -> topology -> instances
fn topology_from_instances() {
    let instances_model = Instances::from(vec![
        InstanceV2::from(Name::from("router").with_index(1))
            .with_roles(vec![Role::router(), Role::failover_coordinator()])
            .with_color(FG_WHITE),
        InstanceV2::from(Name::from("storage").with_index(1).with_index(1))
            .with_roles(vec![Role::storage()])
            .with_color(FG_BLUE),
        InstanceV2::from(Name::from("storage").with_index(1).with_index(2))
            .with_roles(vec![Role::storage()])
            .with_color(FG_BLUE),
        InstanceV2::from(Name::from("storage").with_index(2).with_index(1))
            .with_roles(vec![Role::storage()])
            .with_color(FG_CYAN),
        InstanceV2::from(Name::from("storage").with_index(2).with_index(2))
            .with_roles(vec![Role::storage()])
            .with_color(FG_CYAN),
    ]);
    let topology_model = Topology(vec![
        TopologySet {
            name: "router".into(),
            replicasets_count: Some(1),
            replication_factor: None,
            weight: None,
            failure_domains: Default::default(),
            roles: vec![Role::router(), Role::failover_coordinator()],
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
        },
        TopologySet {
            name: "storage".into(),
            replicasets_count: Some(2),
            replication_factor: Some(2),
            weight: None,
            failure_domains: Default::default(),
            roles: vec![Role::storage()],
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
        },
    ]);

    let topology = Topology::try_from(instances_model.clone()).unwrap();

    assert_eq!(&topology, &topology_model);

    let instances = Instances::from(&topology);

    assert_eq!(&instances, &instances_model);
}

#[test]
/// ClusterV2.topology string -> Topology -> ClusterV2.topology string
fn topology_member_v2() {
    let topology_member_str: String = r#"---
name: router
replicasets_count: 1
roles:
  - router
  - failover-coordinator
"#
    .into();

    let topology_member_model = TopologySet {
        name: Name::from("router"),
        replicasets_count: Some(1),
        replication_factor: None,
        weight: None,
        failure_domains: Default::default(),
        roles: vec![Role::router(), Role::failover_coordinator()],
        cartridge_extra_env: IndexMap::new(),
        config: InstanceV2Config::default(),
        vars: IndexMap::default(),
    };

    let topology_member: TopologySet = serde_yaml::from_str(&topology_member_str).unwrap();

    assert_eq!(topology_member, topology_member_model);

    let topology_member_model_str = topology_member_str;

    let topology_member_str = serde_yaml::to_string(&topology_member).unwrap();

    assert_eq!(topology_member_str, topology_member_model_str);
}

#[test]
fn non_unique_replicaset_names() {
    let instances = Instances::from(vec![
        InstanceV2::from(Name::from("router").with_index(1))
            .with_roles(vec![Role::router(), Role::failover_coordinator()])
            .with_color(FG_WHITE),
        InstanceV2::from(Name::from("storage").with_index(1).with_index(1))
            .with_roles(vec![Role::storage()])
            .with_color(FG_BLUE),
        InstanceV2::from(Name::from("storage").with_index(1).with_index(1))
            .with_roles(vec![Role::storage()])
            .with_color(FG_BLUE),
        InstanceV2::from(Name::from("storage").with_index(2).with_index(1))
            .with_roles(vec![Role::storage()])
            .with_color(FG_CYAN),
        InstanceV2::from(Name::from("storage").with_index(2).with_index(2))
            .with_roles(vec![Role::storage()])
            .with_color(FG_CYAN),
    ]);

    assert_eq!(
        Topology::try_from(instances),
        Err("Replicaset names must be unique".into())
    );

    let topology = Topology(vec![
        TopologySet {
            name: "router".into(),
            replicasets_count: Some(1),
            replication_factor: None,
            weight: None,
            failure_domains: Default::default(),
            roles: vec![Role::router(), Role::failover_coordinator()],
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
        },
        TopologySet {
            name: "router".into(),
            replicasets_count: Some(2),
            replication_factor: Some(2),
            weight: None,
            failure_domains: Default::default(),
            roles: vec![Role::storage()],
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
        },
    ]);

    assert_eq!(
        topology.check_unique(),
        Err("Replicaset names must be unique".into())
    )
}
