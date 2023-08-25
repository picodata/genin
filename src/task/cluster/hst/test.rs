use std::net::IpAddr;

use indexmap::IndexMap;
use tabled::Alignment;

use crate::task::{
    cluster::{
        hst::{
            v2::{Address, HostV2, HostV2Config, WithHosts},
            view::{View, FG_BLUE, FG_CYAN, FG_GREEN, FG_WHITE},
        },
        ins::v2::{InstanceV2, InstanceV2Config, Instances},
        name::Name,
        topology::Topology,
        HostV2Helper,
    },
    utils::uncolorize,
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

    let mut hosts_v2: HostV2 = HostV2::from(
        serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str).unwrap(),
    )
    .with_instances(Instances::from(vec![
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(1),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(2),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(3),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(4),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(5),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(6),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(7),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(8),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(9),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage").with_index(1).with_index(10),
            stateboard: None,
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
    ]));

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

fn failure_domain_test_host() -> HostV2 {
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

    host.instances = Instances::from(vec![
        InstanceV2 {
            name: Name::from("router-1"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_WHITE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-1-1"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-1-2"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-1-3"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_BLUE,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-2-1"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_CYAN,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-2-2"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_CYAN,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("storage-2-3"),
            stateboard: Some(false),
            weight: None,
            failure_domains: Default::default(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_CYAN,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("cache-1"),
            stateboard: Some(false),
            weight: None,
            failure_domains: vec!["dc-2".to_string()].into(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_GREEN,
                alignment: Alignment::left(),
            },
        },
        InstanceV2 {
            name: Name::from("cache-2"),
            stateboard: Some(false),
            weight: None,
            failure_domains: vec!["dc-2".to_string(), "server-5".to_string()].into(),
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceV2Config::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_GREEN,
                alignment: Alignment::left(),
            },
        },
    ]);

    host.spread();

    host
}

#[test]
fn hosts_force_failure_domain() {
    let host = failure_domain_test_host();

    assert_eq!(
        host.hosts
            // dc-2
            .last()
            .unwrap()
            .hosts
            // server-4
            .first()
            .unwrap()
            .instances
            .last()
            .unwrap()
            .name
            .to_string(),
        "cache-1".to_string()
    );
    assert_eq!(
        host.hosts
            // dc-2
            .last()
            .unwrap()
            .hosts
            // server-5
            .last()
            .unwrap()
            .instances
            .last()
            .unwrap()
            .name
            .to_string(),
        "cache-2".to_string()
    );
}

#[test]
fn hosts_use_failure_domain_as_zone() {
    fn failure_domain_instance_zone(host: &HostV2, host_index: usize) -> Option<&str> {
        host.hosts
            // dc-2
            .last()
            .unwrap()
            .hosts[host_index]
            // server-1
            .instances
            .last()
            .unwrap()
            .config
            .zone
            .as_deref()
    }

    let mut host = failure_domain_test_host();
    assert_eq!(failure_domain_instance_zone(&host, 0), None);
    assert_eq!(failure_domain_instance_zone(&host, 1), None);

    host.use_failure_domain_as_zone();
    assert_eq!(failure_domain_instance_zone(&host, 0), Some("dc-2"));
    assert_eq!(failure_domain_instance_zone(&host, 1), Some("server-5"));
}

#[test]
fn hosts_v2_spreading() {
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

    let instances = Instances::from(&topology);

    let mut hosts_v2 = HostV2::from("Cluster")
        .with_hosts(vec![
            HostV2::from("Server-1"),
            HostV2::from("Server-2")
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_add_queue(
            instances
                .iter()
                .map(|instance| (instance.name.clone(), instance.clone()))
                .collect(),
        )
        .with_delete_queue(
            instances
                .iter()
                .map(|instance| (instance.name.clone(), instance.clone()))
                .collect(),
        )
        .with_instances(instances)
        .with_config(HostV2Config::from((8081, 3031)))
        .with_address(Address::from([192, 168, 123, 11]));

    hosts_v2.spread();

    println!("{}", &hosts_v2);

    insta::assert_debug_snapshot!(hosts_v2);
}

#[test]
fn hosts_v2_print_table() {
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

    let instances = Instances::from(&topology);

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
        .with_config(HostV2Config::from((8081, 3031)))
        .with_address(Address::from([192, 168, 123, 11]));

    hosts_v2.spread();

    println!("{}", hosts_v2);

    insta::assert_display_snapshot!(uncolorize(hosts_v2));
}

#[test]
fn hosts_v2_spread_stateboard() {
    let mut hosts_v2 = HostV2::from("Cluster")
        .with_hosts(vec![
            HostV2::from("DC1").with_hosts(vec![
                HostV2::from("Rack1").with_hosts(vec![
                    HostV2::from("Server-1")
                        .with_config(HostV2Config::from(IpAddr::from([192, 168, 123, 11]))),
                    HostV2::from("Server-2")
                        .with_config(HostV2Config::from(IpAddr::from([192, 168, 123, 12]))),
                ]),
                HostV2::from("Rack2").with_hosts(vec![
                    HostV2::from("Server-3")
                        .with_config(HostV2Config::from(IpAddr::from([192, 168, 123, 101]))),
                    HostV2::from("Server-4")
                        .with_config(HostV2Config::from(IpAddr::from([192, 168, 123, 102]))),
                ]),
            ]),
            HostV2::from("DC2")
                .with_hosts(vec![
                    HostV2::from("Rack1").with_hosts(vec![
                        HostV2::from("Server-5")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 66, 11]))),
                        HostV2::from("Server-6")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 66, 12]))),
                    ]),
                    HostV2::from("Rack2").with_hosts(vec![
                        HostV2::from("Server-7")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 66, 101]))),
                        HostV2::from("Server-8")
                            .with_config(HostV2Config::from(IpAddr::from([192, 168, 66, 101]))),
                    ]),
                ])
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_config(HostV2Config::from((8081, 3031)));

    let address = Address::Ip("192.168.66.101".parse().unwrap());

    hosts_v2.instances.push(InstanceV2 {
        name: Name::from("stateboard"),
        stateboard: Some(true),
        weight: None,
        failure_domains: vec![hosts_v2.get_name_by_address(&address).unwrap().to_string()].into(),
        roles: Vec::new(),
        cartridge_extra_env: IndexMap::new(),
        config: InstanceV2Config::default(),
        vars: IndexMap::default(),
        view: View::default(),
    });

    hosts_v2.spread();

    assert_eq!(
        hosts_v2
            .hosts
            .last()
            .unwrap()
            .hosts
            .last()
            .unwrap()
            .hosts
            .first()
            .unwrap()
            .instances
            .first()
            .unwrap()
            .name,
        Name::from("stateboard")
    )
}

#[test]
fn merge_with_increasig_hosts() {
    let hosts_old_str = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.32.101
      - name: server-2
        config:
          address: 192.168.32.102
  - name: dc-2
    hosts:
      - name: server-4
        config:
          address: 192.168.64.101
      - name: server-5
        config:
          address: 192.168.64.102
"#;

    let mut hosts_old = serde_yaml::from_str::<HostV2>(hosts_old_str).unwrap();

    let hosts_new_str = r#"---
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
      - name: server-6
        config:
          address: 192.168.16.16
  - name: dc-3
    hosts:
      - name: server-7
        config:
          address: 192.168.16.17
      - name: server-8
        config:
          address: 192.168.16.18
      - name: server-9
        config:
          address: 192.168.16.19
"#;

    let mut hosts_new = serde_yaml::from_str::<HostV2>(hosts_new_str).unwrap();

    HostV2::merge(&mut hosts_old, &mut hosts_new, true);

    insta::assert_yaml_snapshot!(hosts_old);
}

#[test]
fn merge_with_decreasing_hosts() {
    let hosts_old_str = r#"---
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
      - name: server-6
        config:
          address: 192.168.16.16
  - name: dc-3
    hosts:
      - name: server-7
        config:
          address: 192.168.16.17
      - name: server-8
        config:
          address: 192.168.16.18
      - name: server-9
        config:
          address: 192.168.16.19
"#;

    let mut hosts_old = serde_yaml::from_str::<HostV2>(hosts_old_str).unwrap();

    let hosts_new_str = r#"---
name: cluster
config:
  http_port: 8081
  binary_port: 3301
hosts:
  - name: dc-1
    hosts:
      - name: server-1
        config:
          address: 192.168.32.101
      - name: server-2
        config:
          address: 192.168.32.102
  - name: dc-2
    hosts:
      - name: server-4
        config:
          address: 192.168.64.101
      - name: server-5
        config:
          address: 192.168.64.102
"#;

    let mut hosts_new = serde_yaml::from_str::<HostV2>(hosts_new_str).unwrap();

    HostV2::merge(&mut hosts_old, &mut hosts_new, true);

    insta::assert_yaml_snapshot!(hosts_old);
}
