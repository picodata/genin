use std::net::IpAddr;

use serde::Deserialize;

use crate::task::cluster::{
    hst::v2::{Address, HostV2, HostV2Config},
    ins::{
        v2::{InstanceV2, Replicaset},
        Name, Role,
    },
    HostV2Helper, TopologyMemberV2,
};

#[test]
fn hosts_v2_deepth() {
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

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(hosts_v2.depth(), 2);

    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: room-5
        hosts:
          - name: rack-1
            hosts:
              - name: server-1
                config:
                  address: 192.168.16.12
"#
    .into();

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(hosts_v2.depth(), 5);
}

#[test]
fn hosts_v2_width() {
    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
  - name: dc-2
    hosts:
      - name: server-1
        config:
          address: 192.168.16.13
      - name: server-2
        config:
          address: 192.168.16.14
  - name: dc-3
    hosts:
      - name: server-1
        config:
          address: 192.168.16.15
      - name: server-2
        config:
          address: 192.168.16.16
  - name: dc-4
    hosts:
      - name: server-1
        config:
          address: 192.168.16.17
      - name: server-2
        config:
          address: 192.168.16.18
  - name: dc-5
    hosts:
      - name: server-1
        config:
          address: 192.168.16.19
      - name: server-2
        config:
          address: 192.168.16.20
"#
    .into();

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(hosts_v2.width(), 10);
}

#[test]
fn hosts_v2_size() {
    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
      - name: server-3
        config:
          address: 192.168.16.13
  - name: dc-2
    hosts:
      - name: server-1
        config:
          address: 192.168.16.14
      - name: server-2
        config:
          address: 192.168.16.15
"#
    .into();

    let mut hosts_v2: HostV2 =
        HostV2::from(serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str).unwrap()).with_instances(
            Replicaset {
                name: Name::from("storage"),
                replicasets_count: Some(1),
                replication_factor: Some(10),
                weight: None,
                failure_domains: Vec::new(),
                zone: None,
                roles: Vec::new(),
                config: HostV2Config::default(),
            }
            .instances(),
        );

    assert_eq!(hosts_v2.size(), 0);

    hosts_v2.spread();

    assert_eq!(hosts_v2.size(), 10);
}

#[test]
fn hosts_v2_lower_level_hosts() {
    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
      - name: server-3
        config:
          address: 192.168.16.13
  - name: dc-2
    hosts:
      - name: server-1
        config:
          address: 192.168.16.14
      - name: server-2
        config:
          address: 192.168.16.15
"#
    .into();

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

    hosts_v2
        .lower_level_hosts()
        .iter()
        .for_each(|host| println!("{}", host.name));

    let lower_level_hosts_model = vec![
        HostV2::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-1"),
        )
        .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 11]))),
        HostV2::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-2"),
        )
        .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 12]))),
        HostV2::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-3"),
        )
        .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 13]))),
        HostV2::from(
            Name::from("cluster")
                .with_raw_index("dc-2")
                .with_raw_index("server-1"),
        )
        .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 14]))),
        HostV2::from(
            Name::from("cluster")
                .with_raw_index("dc-2")
                .with_raw_index("server-2"),
        )
        .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 15]))),
    ];

    assert_eq!(
        hosts_v2.lower_level_hosts(),
        lower_level_hosts_model.iter().collect::<Vec<&HostV2>>()
    );
}

#[test]
fn hosts_force_failure_domain() {
    let hosts_v2_str: String = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          address: 192.168.16.12
      - name: server-3
        config:
          address: 192.168.16.13
  - name: dc-2
    hosts:
      - name: server-4
        config:
          address: 192.168.16.14
      - name: server-5
        config:
          address: 192.168.16.15
"#
    .into();

    let mut host: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

    host.instances = vec![
        InstanceV2 {
            name: Name::from("storage-1-1"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("storage-1-2"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("storage-1-3"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("storage-2-1"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("storage-2-2"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("storage-2-3"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: Vec::new(),
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("cache-1"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: vec!["dc-2".to_string()],
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
        InstanceV2 {
            name: Name::from("cache-2"),
            stateboard: Some(false),
            weight: None,
            zone: None,
            failure_domains: vec!["dc-2".to_string()],
            roles: Vec::new(),
            config: HostV2Config::default(),
        },
    ];

    host.spread();

    assert_eq!(
        host.hosts
            // dc-2
            .last()
            .unwrap()
            .hosts
            // server-1
            .first()
            .unwrap()
            .instances
            .last()
            .unwrap()
            .name
            .to_string(),
        "cache-2".to_string()
    );
    assert_eq!(
        host.hosts
            // dc-2
            .last()
            .unwrap()
            .hosts
            // server-2
            .last()
            .unwrap()
            .instances
            .last()
            .unwrap()
            .name
            .to_string(),
        "cache-1".to_string()
    );
}

#[test]
fn hosts_v2_spreading() {
    #[derive(Deserialize)]
    struct Topology(Vec<TopologyMemberV2>);

    let topology_str: String = r#"---
- name: router
  replicasets_count: 1
  roles:
    - router
    - failover-coordinator
- name: storage
  replicasets_count: 2
  replication_factor: 2
  roles:
    - storage
"#
    .into();

    let topology: Topology = serde_yaml::from_str(&topology_str).unwrap();

    let instances = topology
        .0
        .into_iter()
        .flat_map(|topology_member| {
            topology_member
                .to_replicasets()
                .into_iter()
                .flat_map(|replicaset| replicaset.instances())
        })
        .collect::<Vec<InstanceV2>>();

    let mut hosts_v2 = HostV2::from("Cluster")
        .with_hosts(vec![
            HostV2::from("Server-1"),
            HostV2::from("Server-2")
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_instances(instances)
        .with_config(
            HostV2Config::from((8081, 3031)).with_address(Address::from([192, 168, 123, 11])),
        );

    hosts_v2.spread();

    println!("{}", hosts_v2);

    let mut hosts_v2_model = HostV2::from("Cluster")
        .with_hosts(vec![
            HostV2::from("Server-1").with_instances(vec![
                InstanceV2 {
                    name: Name::from("router").with_index(1),
                    stateboard: None,
                    weight: None,
                    roles: vec![Role::router(), Role::failover_coordinator()],
                    failure_domains: Vec::new(),
                    zone: None,
                    config: HostV2Config::from((8081, 3031))
                        .with_address(Address::from([192, 168, 123, 11])),
                },
                InstanceV2 {
                    name: Name::from("storage").with_index(1).with_index(2),
                    stateboard: None,
                    weight: None,
                    roles: vec![Role::storage()],
                    failure_domains: Vec::new(),
                    zone: None,
                    config: HostV2Config::from((8082, 3032))
                        .with_address(Address::from([192, 168, 123, 11])),
                },
                InstanceV2 {
                    name: Name::from("storage").with_index(2).with_index(2),
                    stateboard: None,
                    weight: None,
                    roles: vec![Role::storage()],
                    failure_domains: Vec::new(),
                    zone: None,
                    config: HostV2Config::from((8083, 3033))
                        .with_address(Address::from([192, 168, 123, 11])),
                },
            ]),
            HostV2::from("Server-2")
                .with_http_port(25000)
                .with_binary_port(26000)
                .with_instances(vec![
                    InstanceV2 {
                        name: Name::from("storage").with_index(1).with_index(1),
                        stateboard: None,
                        weight: None,
                        roles: vec![Role::storage()],
                        failure_domains: Vec::new(),
                        zone: None,
                        config: HostV2Config::default(),
                    },
                    InstanceV2 {
                        name: Name::from("storage").with_index(2).with_index(1),
                        stateboard: None,
                        weight: None,
                        roles: vec![Role::storage()],
                        failure_domains: Vec::new(),
                        zone: None,
                        config: HostV2Config::default(),
                    },
                ]),
        ])
        .with_config(
            HostV2Config::from((8081, 3031)).with_address(Address::from([192, 168, 123, 11])),
        );

    hosts_v2_model.spread();

    assert_eq!(hosts_v2, hosts_v2_model);
}

#[test]
fn hosts_v2_print_table() {
    #[derive(Deserialize)]
    struct Topology(Vec<TopologyMemberV2>);

    let topology_str: String = r#"---
- name: router
  replicasets_count: 8
  roles:
    - router
    - failover-coordinator
- name: storage
  replicasets_count: 3
  replication_factor: 12
  roles:
    - storage
"#
    .into();

    let topology: Topology = serde_yaml::from_str(&topology_str).unwrap();

    let instances = topology
        .0
        .into_iter()
        .flat_map(|topology_member| {
            topology_member
                .to_replicasets()
                .into_iter()
                .flat_map(|replicaset| replicaset.instances())
        })
        .collect::<Vec<InstanceV2>>();

    let mut hosts_v2 = HostV2::from("Cluster")
        .with_hosts(vec![
            HostV2::from("DC1").with_hosts(vec![
                HostV2::from("Rack1")
                    .with_hosts(vec![HostV2::from("Server-1"), HostV2::from("Server-2")]),
                HostV2::from("Rack2")
                    .with_hosts(vec![HostV2::from("Server-3"), HostV2::from("Server-4")]),
            ]),
            HostV2::from("DC2")
                .with_hosts(vec![
                    HostV2::from("Rack1")
                        .with_hosts(vec![HostV2::from("Server-5"), HostV2::from("Server-6")]),
                    HostV2::from("Rack2")
                        .with_hosts(vec![HostV2::from("Server-7"), HostV2::from("Server-8")]),
                ])
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_instances(instances)
        .with_config(
            HostV2Config::from((8081, 3031)).with_address(Address::from([192, 168, 123, 11])),
        );

    hosts_v2.spread();

    println!("{}", hosts_v2);

    let table = String::from(
"+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
|                                                       Cluster                                                       |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
|                           DC1                           |                            DC2                            |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
|           Rack1           |            Rack2            |            Rack1            |            Rack2            |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
|  Server-1   |  Server-2   |   Server-3   |   Server-4   |   Server-5   |   Server-6   |   Server-7   |   Server-8   |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
|  router-1   |  router-5   |  router-3    |  router-7    | router-2     | router-6     | router-4     | router-8     |
|  8081 3031  |  8081 3031  |  8081 3031   |  8081 3031   | 25000 26000  | 25000 26000  | 25000 26000  | 25000 26000  |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
| storage-1-1 | storage-1-5 | storage-1-3  | storage-1-7  | storage-1-2  | storage-1-6  | storage-1-4  | storage-1-8  |
| 8082 3032   | 8082 3032   | 8082 3032    | 8082 3032    | 25001 26001  | 25001 26001  | 25001 26001  | 25001 26001  |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
| storage-1-9 | storage-2-1 | storage-1-11 | storage-2-3  | storage-1-10 | storage-2-2  | storage-1-12 | storage-2-4  |
| 8083 3033   | 8083 3033   | 8083 3033    | 8083 3033    | 25002 26002  | 25002 26002  | 25002 26002  | 25002 26002  |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
| storage-2-5 | storage-2-9 | storage-2-7  | storage-2-11 | storage-2-6  | storage-2-10 | storage-2-8  | storage-2-12 |
| 8084 3034   | 8084 3034   | 8084 3034    | 8084 3034    | 25003 26003  | 25003 26003  | 25003 26003  | 25003 26003  |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
| storage-3-1 | storage-3-5 | storage-3-3  | storage-3-7  | storage-3-2  | storage-3-6  | storage-3-4  | storage-3-8  |
| 8085 3035   | 8085 3035   | 8085 3035    | 8085 3035    | 25004 26004  | 25004 26004  | 25004 26004  | 25004 26004  |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+
| storage-3-9 |             | storage-3-11 |              | storage-3-10 |              | storage-3-12 |              |
| 8086 3036   |             | 8086 3036    |              | 25005 26005  |              | 25005 26005  |              |
+-------------+-------------+--------------+--------------+--------------+--------------+--------------+--------------+");

    assert_eq!(hosts_v2.to_string(), table);
}
