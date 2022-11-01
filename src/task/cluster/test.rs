use clap::{Arg, ArgAction, Command};

use super::*;

#[test]
/// Forward and reverse conversion test between replicasets and topology_members
/// replicasets -> topology_members -> replicasets
fn topology_from_replicasets() {
    let replicasets = vec![
        Replicaset {
            name: Name::from("router").with_index(1),
            replicasets_count: Some(1),
            replication_factor: None,
            weight: None,
            zone: None,
            roles: vec![Role::router(), Role::failover_coordinator()],
            config: Config::default(),
            instances: Vec::new(),
        },
        Replicaset {
            name: Name::from("storage").with_index(1),
            replicasets_count: Some(2),
            replication_factor: Some(1),
            weight: None,
            zone: None,
            roles: vec![Role::storage()],
            config: Config::default(),
            instances: Vec::new(),
        },
        Replicaset {
            name: Name::from("storage").with_index(2),
            replicasets_count: Some(2),
            replication_factor: Some(1),
            weight: None,
            zone: None,
            roles: vec![Role::storage()],
            config: Config::default(),
            instances: Vec::new(),
        },
    ];
    let topology_members_reference = vec![
        TopologyMemberV2 {
            name: "router".into(),
            replicasets_count: Some(1),
            replication_factor: None,
            weight: None,
            zone: None,
            roles: vec![Role::router(), Role::failover_coordinator()],
            config: Config::default(),
        },
        TopologyMemberV2 {
            name: "storage".into(),
            replicasets_count: Some(2),
            replication_factor: Some(1),
            weight: None,
            zone: None,
            roles: vec![Role::storage()],
            config: Config::default(),
        },
    ];

    let topology_members = TopologyMemberV2::from(replicasets.clone());

    assert_eq!(&topology_members, &topology_members_reference);

    let reversed_replicasets: Vec<Replicaset> = topology_members
        .iter()
        .flat_map(|topology_member| topology_member.to_replicasets())
        .collect();

    assert_eq!(&reversed_replicasets, &replicasets);
}

#[test]
/// ClusterV2.topology string -> TopologyMemberV2 -> ClusterV2.topology string
fn topology_member_v2() {
    let topology_member_str: String = r#"---
name: router
replicasets_count: 1
roles:
  - router
  - failover-coordinator
"#
    .into();

    let topology_member_model = TopologyMemberV2 {
        name: Name::from("router"),
        replicasets_count: Some(1),
        replication_factor: None,
        weight: None,
        zone: None,
        roles: vec![Role::router(), Role::failover_coordinator()],
        config: Config::default(),
    };

    let topology_member: TopologyMemberV2 = serde_yaml::from_str(&topology_member_str).unwrap();

    assert_eq!(topology_member, topology_member_model);

    let topology_member_model_str = topology_member_str;

    let topology_member_str = serde_yaml::to_string(&topology_member).unwrap();

    assert_eq!(topology_member_str, topology_member_model_str);
}

#[test]
/// ClusterV2.hosts string -> HostV2 -> ClusterV2.hosts string
fn hosts_v2() {
    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: server-1
    config:
      address: 192.168.16.11
  - name: server-2
    config:
      address: 192.168.16.12
"#
    .into();

    let hosts_v2_model = HostV2::from("cluster")
        .with_hosts(vec![
            HostV2::from("server-1")
                .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 11]))),
            HostV2::from("server-2")
                .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 12]))),
        ])
        .with_config(HostV2Config::from((8081, 3301)));

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into_v2();

    assert_eq!(hosts_v2, hosts_v2_model);

    let hosts_v2_model_str = hosts_v2_str;

    let hosts_v2_str = serde_yaml::to_string(&hosts_v2).unwrap();

    assert_eq!(hosts_v2_str, hosts_v2_model_str);
}

#[test]
/// ClusterV1.hosts -> HostV2
/// ClusterV1.hosts == ClusterV2.hosts
/// ClusterV1.hosts -> ClusterV2.hosts HostV2 string
fn hosts_v1_to_hosts_v2() {
    let cluster_v1_str: String = r#"---
instances:
  - name: router
    type: router
    count: 0
hosts:
  - name: selectel
    type: datacenter
    ports:
      http: 8081
      binary: 3031
    hosts:
      - name: server-1
        ip: 192.168.16.11
      - name: server-2
        ip: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: 192.168.16.1:4401
    password: some_password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
"#
    .into();

    let hosts_v2_model = vec![
        HostV2::from("cluster").with_hosts(vec![HostV2::from("selectel")
            .with_hosts(vec![
                HostV2::from("server-1").with_config(
                    HostV2Config::from(IpAddr::from([192, 168, 16, 11])).with_ports((8081, 3031)),
                ),
                HostV2::from("server-2").with_config(
                    HostV2Config::from(IpAddr::from([192, 168, 16, 12])).with_ports((8081, 3031)),
                ),
            ])
            .with_config(HostV2Config::from((8081, 3031)))]),
    ];

    let cluster_v1: Cluster = serde_yaml::from_str(&cluster_v1_str).unwrap();

    assert_eq!(cluster_v1.hosts, hosts_v2_model);

    let cluster_v2_str: String = r#"---
topology:
  - name: router
    replicasets_count: 0
hosts:
  - name: selectel
    config:
      http_port: 8081
      binary_port: 3031
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: "192.168.16.1:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
"#
    .into();

    let cluster_v2: Cluster = serde_yaml::from_str(&cluster_v2_str).unwrap();

    assert_eq!(cluster_v1.hosts, cluster_v2.hosts);
}

#[test]
/// Args(only init) -> Cluster
fn cluster_v2_from_args() {
    let target: String = r#"---
topology:
  - name: router
    replicasets_count: 1
    roles:
      - router
      - failover-coordinator
  - name: storage
    replicasets_count: 2
    replication_factor: 1
    roles:
      - storage
hosts:
  - name: datacenter-1
    config:
      http_port: 8081
      binary_port: 3301
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: "192.168.16.1:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
"#
    .into();

    let cluster = Cluster::try_from(
        &Command::new("genin")
            .arg(
                Arg::new("failover-mode")
                    .long("failover-mode")
                    .action(ArgAction::Set)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .action(ArgAction::Set)
                    .default_value("stateboard"),
            )
            .try_get_matches_from(vec!["genin"])
            .unwrap(),
    )
    .unwrap();

    let cluster_str = serde_yaml::to_string(&cluster).unwrap();

    assert_eq!(cluster_str, target);
}

/// read string ClusterV1 -> serialize string ClusterV2 == ClusterV2 model
#[test]
fn cluster_v1_to_cluster_v2() {
    let cluster_v1_str: String = r#"---
instances:
  - name: router
    type: router
    count: 1
    roles:
      - router
      - api
      - failover-coordinator
  - name: storage
    type: storage
    count: 2
    replicas: 2
    weight: 10
    roles:
      - storage
hosts:
  - name: selectel
    type: datacenter
    ports:
      http: 8081
      binary: 3031
    hosts:
      - name: server-1
        ip: 192.168.16.11
      - name: server-2
        ip: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: 192.168.16.1:4401
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
"#
    .into();
    let cluster_v2_model_str: String = r#"---
topology:
  - name: router
    replicasets_count: 1
    roles:
      - router
      - api
      - failover-coordinator
  - name: storage
    replicasets_count: 2
    replication_factor: 2
    weight: 10
    roles:
      - storage
hosts:
  - name: selectel
    config:
      http_port: 8081
      binary_port: 3031
    hosts:
      - name: server-1
        config:
          http_port: 8081
          binary_port: 3031
          address: 192.168.16.11
      - name: server-2
        config:
          http_port: 8081
          binary_port: 3031
          address: 192.168.16.12
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: "192.168.16.1:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
"#
    .into();

    let cluster_v2: Cluster = serde_yaml::from_str(&cluster_v1_str).unwrap();

    let cluster_v2_str: String = serde_yaml::to_string(&cluster_v2).unwrap();

    assert_eq!(cluster_v2_str, cluster_v2_model_str);
}
