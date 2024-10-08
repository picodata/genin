use std::{collections::VecDeque, net::IpAddr};

use indexmap::IndexMap;
use tabled::Alignment;

use crate::task::{
    cluster::{
        host::{
            hst::{Address, Host, HostConfig, WithHosts},
            view::{View, FG_BLUE, FG_WHITE},
        },
        instance::ins::{FailureDomains, Instance, InstanceConfig, Instances},
        name::Name,
        topology::Topology,
        HostHelper,
    },
    utils::uncolorize,
};
use insta::assert_display_snapshot;

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

    let host: Host = serde_yaml::from_str::<HostHelper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(host.depth(), 2);

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

    let host: Host = serde_yaml::from_str::<HostHelper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(host.depth(), 5);
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

    let host: Host = serde_yaml::from_str::<HostHelper>(&hosts_v2_str)
        .unwrap()
        .into();

    assert_eq!(host.width(), 10);
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

    let mut host: Host = Host::from(serde_yaml::from_str::<HostHelper>(&hosts_v2_str).unwrap())
        .with_instances(Instances::from(vec![
            Instance {
                name: Name::from("storage").with_index(1).with_index(1),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(2),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(3),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(4),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(5),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(6),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(7),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(8),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(9),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
            Instance {
                name: Name::from("storage").with_index(1).with_index(10),
                stateboard: None,
                weight: None,
                failure_domains: Default::default(),
                roles: Vec::new(),
                cartridge_extra_env: IndexMap::new(),
                config: InstanceConfig::default(),
                vars: IndexMap::default(),
                view: View {
                    color: FG_BLUE,
                    alignment: Alignment::left(),
                },
            },
        ]));

    assert_eq!(host.size(), 0);

    host.spread();

    assert_eq!(host.size(), 10);
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

    let host: Host = serde_yaml::from_str::<HostHelper>(&hosts_v2_str)
        .unwrap()
        .into();

    host.lower_level_hosts()
        .iter()
        .for_each(|host| println!("{}", host.name));

    let lower_level_hosts_model = vec![
        Host::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-1"),
        )
        .with_config(HostConfig::from(IpAddr::from([192, 168, 16, 11]))),
        Host::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-2"),
        )
        .with_config(HostConfig::from(IpAddr::from([192, 168, 16, 12]))),
        Host::from(
            Name::from("cluster")
                .with_raw_index("dc-1")
                .with_raw_index("server-3"),
        )
        .with_config(HostConfig::from(IpAddr::from([192, 168, 16, 13]))),
        Host::from(
            Name::from("cluster")
                .with_raw_index("dc-2")
                .with_raw_index("server-1"),
        )
        .with_config(HostConfig::from(IpAddr::from([192, 168, 16, 14]))),
        Host::from(
            Name::from("cluster")
                .with_raw_index("dc-2")
                .with_raw_index("server-2"),
        )
        .with_config(HostConfig::from(IpAddr::from([192, 168, 16, 15]))),
    ];

    assert_eq!(
        host.lower_level_hosts(),
        lower_level_hosts_model.iter().collect::<Vec<&Host>>()
    );
}

fn failure_domain_test_host() -> Host {
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

    let mut host: Host = serde_yaml::from_str::<HostHelper>(&hosts_v2_str)
        .unwrap()
        .into();

    fn new_instance(name: Name, failure_domains: FailureDomains) -> Instance {
        Instance {
            name,
            stateboard: Some(false),
            weight: None,
            failure_domains,
            roles: Vec::new(),
            cartridge_extra_env: IndexMap::new(),
            config: InstanceConfig::default(),
            vars: IndexMap::default(),
            view: View {
                color: FG_WHITE,
                alignment: Alignment::left(),
            },
        }
    }

    host.instances = Instances::from(vec![
        new_instance("router-1".into(), Default::default()),
        // storages replicaset 1
        new_instance("storage-1-1".into(), Default::default()),
        new_instance("storage-1-2".into(), Default::default()),
        new_instance("storage-1-3".into(), Default::default()),
        // storages replicaset 2
        new_instance("storage-2-1".into(), Default::default()),
        new_instance("storage-2-2".into(), Default::default()),
        new_instance("storage-2-3".into(), Default::default()),
        // stateboards for dc-1
        new_instance("stateboard-1-1".into(), vec!["dc-1".into()].into()),
        new_instance("stateboard-1-2".into(), vec!["dc-1".into()].into()),
        new_instance("stateboard-1-3".into(), vec!["dc-1".into()].into()),
        // caches for dc-2
        new_instance("cache-2-1".into(), vec!["dc-2".into()].into()),
        // make sure one of the cashes has more strict requirements(it *must* be placed on server-5)
        new_instance(
            "cache-2-2".into(),
            vec!["dc-2".into(), "server-5".into()].into(),
        ),
        new_instance("cache-2-3".into(), vec!["dc-2".into()].into()),
    ]);

    host.spread();

    host
}

fn find_instance(
    host: &Host,
    mut predicate: impl FnMut(&Instance) -> bool,
) -> Option<(&Host, &Instance)> {
    let mut queue = VecDeque::new();
    queue.push_front(host);

    while !queue.is_empty() {
        let current = queue.pop_back()?;
        let instance = current
            .instances
            .iter()
            .find(|instance| predicate(instance));
        if let Some(instance) = instance {
            return Some((current, instance));
        }
        queue.extend(current.hosts.iter());
    }
    None
}

#[test]
fn hosts_force_failure_domain() {
    let host = failure_domain_test_host();
    assert_display_snapshot!("host_with_failure_domains", uncolorize(&host));
    let dc2 = host.hosts.last().unwrap();

    let cache1_name = "cache-2-1".to_string();
    let (_, cache1) =
        find_instance(dc2, |instance| instance.name.to_string() == cache1_name).unwrap();
    assert_eq!(cache1.name.to_string(), "cache-2-1");

    let cache2_name = "cache-2-2".to_string();
    let (cache2_parent, cache2) =
        find_instance(dc2, |instance| instance.name.to_string() == cache2_name).unwrap();
    assert_eq!(cache2_parent.name.to_string(), "server-5");
    assert_eq!(cache2.name.to_string(), "cache-2-2");
}

#[test]
fn hosts_use_failure_domain_as_zone() {
    fn failure_domain_instance_zone<'a>(host: &'a Host, instance_name: &str) -> Option<&'a str> {
        let (_, instance) =
            find_instance(host, |instance| instance.name.to_string() == instance_name).unwrap();
        instance.config.zone.as_deref()
    }

    let mut host = failure_domain_test_host();
    assert_eq!(failure_domain_instance_zone(&host, "cache-2-1"), None);
    assert_eq!(failure_domain_instance_zone(&host, "cache-2-2"), None);

    host.use_failure_domain_as_zone(1);
    assert_eq!(
        failure_domain_instance_zone(&host, "cache-2-1"),
        Some("dc-2")
    );
    assert_eq!(
        failure_domain_instance_zone(&host, "cache-2-2"),
        Some("dc-2")
    );
    assert_eq!(
        failure_domain_instance_zone(&host, "stateboard-1-1"),
        Some("dc-1")
    );
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

    let mut host = Host::from("Cluster")
        .with_hosts(vec![
            Host::from("Server-1"),
            Host::from("Server-2")
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
        .with_config(HostConfig::from((8081, 3031)))
        .with_address(Address::from([192, 168, 123, 11]));

    host.spread();

    println!("{}", &host);

    insta::assert_debug_snapshot!(host);
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

    let mut host = Host::from("Cluster")
        .with_hosts(vec![
            Host::from("DC1").with_hosts(vec![
                Host::from("Rack1")
                    .with_hosts(vec![Host::from("Server-1"), Host::from("Server-2")]),
                Host::from("Rack2")
                    .with_hosts(vec![Host::from("Server-3"), Host::from("Server-4")]),
            ]),
            Host::from("DC2")
                .with_hosts(vec![
                    Host::from("Rack1")
                        .with_hosts(vec![Host::from("Server-5"), Host::from("Server-6")]),
                    Host::from("Rack2")
                        .with_hosts(vec![Host::from("Server-7"), Host::from("Server-8")]),
                ])
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_instances(instances)
        .with_config(HostConfig::from((8081, 3031)))
        .with_address(Address::from([192, 168, 123, 11]));

    host.spread();

    println!("{}", host);

    insta::assert_display_snapshot!(uncolorize(host));
}

#[test]
fn hosts_v2_spread_stateboard() {
    let mut host = Host::from("Cluster")
        .with_hosts(vec![
            Host::from("DC1").with_hosts(vec![
                Host::from("Rack1").with_hosts(vec![
                    Host::from("Server-1")
                        .with_config(HostConfig::from(IpAddr::from([192, 168, 123, 11]))),
                    Host::from("Server-2")
                        .with_config(HostConfig::from(IpAddr::from([192, 168, 123, 12]))),
                ]),
                Host::from("Rack2").with_hosts(vec![
                    Host::from("Server-3")
                        .with_config(HostConfig::from(IpAddr::from([192, 168, 123, 101]))),
                    Host::from("Server-4")
                        .with_config(HostConfig::from(IpAddr::from([192, 168, 123, 102]))),
                ]),
            ]),
            Host::from("DC2")
                .with_hosts(vec![
                    Host::from("Rack1").with_hosts(vec![
                        Host::from("Server-5")
                            .with_config(HostConfig::from(IpAddr::from([192, 168, 66, 11]))),
                        Host::from("Server-6")
                            .with_config(HostConfig::from(IpAddr::from([192, 168, 66, 12]))),
                    ]),
                    Host::from("Rack2").with_hosts(vec![
                        Host::from("Server-7")
                            .with_config(HostConfig::from(IpAddr::from([192, 168, 66, 101]))),
                        Host::from("Server-8")
                            .with_config(HostConfig::from(IpAddr::from([192, 168, 66, 101]))),
                    ]),
                ])
                .with_http_port(25000)
                .with_binary_port(26000),
        ])
        .with_config(HostConfig::from((8081, 3031)));

    let address = Address::Ip("192.168.66.101".parse().unwrap());

    host.instances.push(Instance {
        name: Name::from("stateboard"),
        stateboard: Some(true),
        weight: None,
        failure_domains: vec![host.get_name_by_address(&address).unwrap().to_string()].into(),
        roles: Vec::new(),
        cartridge_extra_env: IndexMap::new(),
        config: InstanceConfig::default(),
        vars: IndexMap::default(),
        view: View::default(),
    });

    host.spread();

    assert_eq!(
        host.hosts
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

    let mut hosts_old = serde_yaml::from_str::<Host>(hosts_old_str).unwrap();

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

    let mut hosts_new = serde_yaml::from_str::<Host>(hosts_new_str).unwrap();

    Host::merge(&mut hosts_old, &mut hosts_new, true);

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

    let mut hosts_old = serde_yaml::from_str::<Host>(hosts_old_str).unwrap();

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

    let mut hosts_new = serde_yaml::from_str::<Host>(hosts_new_str).unwrap();

    Host::merge(&mut hosts_old, &mut hosts_new, true);

    insta::assert_yaml_snapshot!(hosts_old);
}
