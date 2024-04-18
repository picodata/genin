use std::convert::TryFrom;

use clap::{Arg, ArgAction, Command};

use crate::task::{
    cluster::hst::v2::Address, flv::StateboardParams, serde_genin, utils::uncolorize,
};

use super::*;

#[test]
fn default_cluster() {
    let args = Command::new("init")
        .arg(
            Arg::new("failover-mode")
                .long("failover-mode")
                .short('m')
                .action(ArgAction::Set)
                .default_value("stateful")
                .help("(string): failover mode (statefull, eventual, disabled)"),
        )
        .arg(
            Arg::new("failover-state-provider")
                .long("failover-state-provider")
                .short('F')
                .action(ArgAction::Set)
                .default_value("stateboard")
                .help("(string): failover state provider"),
        )
        .try_get_matches_from(vec!["init"])
        .unwrap();

    let cluster = Cluster::try_from(&args)
        .unwrap()
        .clear_instances()
        .as_text_with_comments()
        .unwrap();

    insta::assert_display_snapshot!(cluster);
}

#[test]
/// ClusterV2.hosts string -> HostV2 -> ClusterV2.hosts string
fn cluster_hosts_v2_serde() {
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
            HostV2::from(Name::from("cluster").with_raw_index("server-1"))
                .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 11]))),
            HostV2::from(Name::from("cluster").with_raw_index("server-2"))
                .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 12]))),
        ])
        .with_config(HostV2Config::from((8081, 3301)));

    let hosts_v2: HostV2 = serde_yaml::from_str::<HostV2Helper>(&hosts_v2_str)
        .unwrap()
        .into();

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
    uri: 192.168.16.11:4401
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

    let mut hosts_v2_model = HostV2::from("cluster")
        .with_hosts(vec![HostV2::from(
            Name::from("cluster").with_raw_index("selectel"),
        )
        .with_hosts(vec![
            HostV2::from(
                Name::from("cluster")
                    .with_raw_index("selectel")
                    .with_raw_index("server-1"),
            )
            .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 11]))),
            HostV2::from(
                Name::from("cluster")
                    .with_raw_index("selectel")
                    .with_raw_index("server-2"),
            )
            .with_config(HostV2Config::from(IpAddr::from([192, 168, 16, 12]))),
        ])])
        .with_config(HostV2Config::from((8081, 3031)));

    hosts_v2_model.with_stateboard(&Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: crate::task::flv::FailoverVariants::StateboardVariant(
            StateboardParams {
                uri: crate::task::flv::Uri {
                    address: Address::Ip("192.168.16.11".parse().unwrap()),
                    port: 4401,
                },
                password: "some_password".into(),
            },
        ),
        ..Default::default()
    });

    hosts_v2_model.spread();

    let mut cluster_v1: Cluster = serde_yaml::from_str(&cluster_v1_str).unwrap();
    cluster_v1.hosts.add_queue = IndexMap::default();
    cluster_v1.hosts.delete_queue = IndexMap::default();

    println!(
        "stateboard 1: {:?}",
        hosts_v2_model
            .hosts
            .first()
            .unwrap()
            .hosts
            .first()
            .unwrap()
            .instances
    );
    println!(
        "stateboard 2: {:?}",
        cluster_v1
            .hosts
            .hosts
            .first()
            .unwrap()
            .hosts
            .first()
            .unwrap()
            .instances
    );

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
    uri: "192.168.16.11:4401"
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

    let mut cluster_v2: Cluster = serde_yaml::from_str(&cluster_v2_str).unwrap();
    cluster_v2.hosts.add_queue = IndexMap::default();
    cluster_v2.hosts.delete_queue = IndexMap::default();

    println!(
        "stateboard 3: {:?}",
        cluster_v1
            .hosts
            .hosts
            .first()
            .unwrap()
            .hosts
            .first()
            .unwrap()
            .instances
    );
    println!(
        "stateboard 4: {:?}",
        cluster_v2
            .hosts
            .hosts
            .first()
            .unwrap()
            .hosts
            .first()
            .unwrap()
            .instances
    );

    assert_eq!(cluster_v1.hosts, cluster_v2.hosts);
}

#[test]
/// Args(only init) -> Cluster
fn cluster_v2_from_args() {
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
    .unwrap()
    .clear_instances();

    insta::assert_yaml_snapshot!(cluster);
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
    uri: 192.168.16.11:4401
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

    insta::assert_yaml_snapshot!(cluster_v2);
}

#[test]
fn cluster_v1_tdg_to_cluster_v2() {
    let cluster_v1_tdg_str: String = r#"---
instances:
  - name: l-n
    type: custom
    count: 2
    replicas: 0
    weight: 10
    roles:
      - logger
      - notifier
  - name: op-t
    type: custom
    count: 5
    replicas: 0
    weight: 10
    roles:
      - output_processor
      - task_runner
  - name: c-ip
    type: custom
    count: 5
    replicas: 0
    weight: 10
    roles:
      - connector
      - input_processor
      - failover-coordinator
  - name: sch
    type: custom
    count: 2
    replicas: 0
    weight: 10
    roles:
      - scheduler
  - name: storage
    type: storage
    count: 3
    replicas: 2
    weight: 10
    roles:
      - storage
hosts:
  - name: vagr_tdg
    type: datacenter
    ports:
      http: 8081
      binary: 3031
    hosts:
      - name: tdg-1
        ip: 192.168.123.2
      - name: tdg-2
        ip: 192.168.123.3
      - name: tdg-3
        ip: 192.168.123.4
failover:
  mode: stateful
  state_provider: etcd2
  etcd2_params:
    prefix: cartridge/tdg
    lock_delay: 30
    endpoints:
      - "http://192.168.123.2:2379"
vars:
  ansible_user: vagrant
  ansible_password: vagrant
  cartridge_app_name: tdg
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: ./tdg-1.7.17-0-g2a5b4bd18.rpm
"#
    .into();

    let cluster_v2: Cluster = serde_yaml::from_str(&cluster_v1_tdg_str).unwrap();

    insta::assert_yaml_snapshot!(cluster_v2);
}

#[test]
fn cluster_v2_from_inventory() {
    let inventory_str: String = r#"---
all:
  vars:
    ansible_user: ansible
    ansible_password: ansible
    cartridge_app_name: myapp
    cartridge_cluster_cookie: myapp-cookie
    cartridge_package_path: /tmp/myapp.rpm
    cartridge_bootstrap_vshard: true
    cartridge_failover_params:
      mode: stateful
      state_provider: stateboard
      stateboard_params:
        uri: "192.168.16.11:4401"
        password: password
  hosts:
    router-1:
      config:
        advertise_uri: "192.168.16.11:3031"
        http_port: 8081
    storage-1-2:
      config:
        advertise_uri: "192.168.16.11:3032"
        http_port: 8082
    storage-2-2:
      config:
        advertise_uri: "192.168.16.11:3033"
        http_port: 8083
    stateboard:
      stateboard: true
      config:
        listen: "192.168.16.11:4401"
        password: password
    storage-1-1:
      config:
        advertise_uri: "192.168.16.12:3031"
        http_port: 8081
    storage-2-1:
      config:
        advertise_uri: "192.168.16.12:3032"
        http_port: 8082
  children:
    router-1-replicaset:
      vars:
        replicaset_alias: router
        failover_priority:
          - router-1
        roles:
          - router
          - failover-coordinator
      hosts:
        router-1: ~
    storage-1-replicaset:
      vars:
        replicaset_alias: storage-1
        failover_priority:
          - storage-1-1
          - storage-1-2
        roles:
          - storage
      hosts:
        storage-1-2: ~
        storage-1-1: ~
    storage-2-replicaset:
      vars:
        replicaset_alias: storage-2
        failover_priority:
          - storage-2-1
          - storage-2-2
        roles:
          - storage
      hosts:
        storage-2-2: ~
        storage-2-1: ~
    server-1:
      vars:
        ansible_host: 192.168.16.11
      hosts:
        router-1: ~
        storage-1-2: ~
        storage-2-2: ~
        stateboard: ~
    server-2:
      vars:
        ansible_host: 192.168.16.12
      hosts:
        storage-1-1: ~
        storage-2-1: ~"#
        .into();

    let inventory: Inventory = serde_yaml::from_str(&inventory_str).unwrap();

    let cluster_v2 = Cluster::try_from(&inventory).unwrap().clear_instances();

    insta::assert_yaml_snapshot!(cluster_v2);
}

#[test]
fn cluster_v2_upgrade() {
    let old_cluster_str: String = r#"---
topology:
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
  - name: api
    failure_domains: [server-2]
    role:
      - api
hosts:
  - name: datacenter-1
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
    uri: "192.168.16.11:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true"#
        .into();

    let new_cluster_str: String = r#"---
topology:
  - name: router
    replicasets_count: 4
    roles:
      - router
      - failover-coordinator
  - name: storage
    replicasets_count: 2
    replication_factor: 2
    roles:
      - storage
  - name: api
    failure_domains: [server-2]
    role:
      - api
hosts:
  - name: datacenter-1
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
    uri: "192.168.16.11:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true"#
        .into();

    let mut old_cluster: Cluster = serde_yaml::from_str(&old_cluster_str).unwrap();
    let mut new_cluster: Cluster = serde_yaml::from_str(&new_cluster_str).unwrap();

    old_cluster.merge(&mut new_cluster, true).unwrap();

    println!("{}", old_cluster);

    insta::assert_display_snapshot!(uncolorize(old_cluster))
}

#[test]
fn mallformed_content_err() {
    let bytes = r"
topology:
      config:
        http_port: -1000000

    - replication_factor: 1
      roles: [value]
"
    .as_bytes();

    let result = format!("{:?}", serde_genin::from_slice::<Cluster>(bytes));

    insta::assert_display_snapshot!(uncolorize(result));
}

#[test]
fn upgrade_with_same_name() {
    let old_cluster_str = r#"---
topology:
  - name: router
    replicasets_count: 2
    roles:
      - router
      - failover-coordinator
  - name: cfgfetcher
    replicasets_count: 2
    roles:
      - cfgfetcher
hosts:
  - name: datacenter-1
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
    uri: "192.168.16.11:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true"#;

    let new_cluster_str = r#"---
topology:
  - name: router
    replicasets_count: 2
    roles:
      - router
      - failover-coordinator
  - name: cfgfetcher
    replicasets_count: 2
    replication_factor: 2
    roles:
      - cfgfetcher
hosts:
  - name: datacenter-1
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
    uri: "192.168.16.11:4401"
    password: password
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true"#;

    let mut cluster_old: Cluster = serde_yaml::from_str(old_cluster_str).unwrap();
    let mut cluster_new: Cluster = serde_yaml::from_str(new_cluster_str).unwrap();

    cluster_old.merge(&mut cluster_new, false).unwrap();
    cluster_old.hosts.spread();

    let mut upgrade_with_same_name = uncolorize(&cluster_old);

    let inventory = Inventory::try_from(&cluster_old).unwrap();

    upgrade_with_same_name = format!(
        "{upgrade_with_same_name}\n{}",
        serde_yaml::to_string(&inventory).unwrap()
    );

    insta::assert_display_snapshot!("upgrade_with_same_name", upgrade_with_same_name);
}

#[test]
fn missing_main_fields_err() {
    let bytes = r#"---
topology:
complex:
"#
    .as_bytes();

    let result = format!("{:?}", serde_genin::from_slice::<Cluster>(bytes));

    insta::assert_display_snapshot!(uncolorize(result));
}

#[test]
fn missing_toplogy_set_fields() {
    let bytes = r"
topology:
    - replicasets_count: 3
      replication_factor: 10
      cartridge_extra_env:
        FOO: bar
        BIZ:
          - 10
          - two
      roles: [calculator]
      vars:
        ENV_1: 1000
        ENV_2: 600
      config:
        http_port: -1000000

    - replication_factor: 1
      roles: [value]
"
    .as_bytes();

    let result = format!("{:?}", serde_genin::from_slice::<Cluster>(bytes));

    insta::assert_display_snapshot!(uncolorize(result));
}

#[test]
fn missing_host_fields() {
    let bytes = r"
hosts:
  - name: server-1
    config:
      - some: value
  - config:
      distance: 1000
    hosts:
      - name: server-10
      - name: server-11
"
    .as_bytes();

    let result = format!("{:?}", serde_genin::from_slice::<Cluster>(bytes));

    insta::assert_display_snapshot!(uncolorize(result));
}

#[test]
fn placeholders_in_config() {
    let bytes = r"
---
topology:
  - name: router
    replicasets_count: <<replicasets_count>> # указываем здесь количество репликасетов хранилищ из сайзинга
    roles:
      - router
      - failover-coordinator
  - name: storage
    replicasets_count: <<replicasets_count>>
    replication_factor: <<replication_factor>>  #для инсталляции в одном ЦОД это число должно быть 2, для инсталяции в двух ЦОД - 4
    roles:
      - storage
".as_bytes();

    let result = format!("{:?}", serde_genin::from_slice::<Cluster>(bytes));

    insta::assert_display_snapshot!(uncolorize(result));
}

#[test]
fn specify_ansible_host() {
    let cluster = r#"---
topology:
  - name: router
    replicasets_count: 2
    roles:
      - router
      - failover-coordinator
hosts:
  - name: datacenter-1
    config:
      http_port: 8081
      binary_port: 3031
    hosts:
      - name: server-1
        config:
          address: 192.168.16.11
      - name: server-2
        config:
          ansible_host: 192.168.16.14
          address: 192.168.16.12
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true
    "#;

    let mut cluster: Cluster = serde_yaml::from_str(cluster).unwrap();
    cluster.hosts.spread();

    let inventory = Inventory::try_from(&cluster).unwrap();
    let ansible_host = inventory.all.children.values().last().unwrap();
    let ansible_host = match ansible_host {
        Child::Replicaset { .. } => panic!("unexpected"),
        Child::Host { vars, .. } => &vars.ansible_host,
    };
    assert_eq!(ansible_host, &Address::from("192.168.16.14"));

    let with_default_ansible_host = inventory
        .all
        .children
        .values()
        .filter(|child| {
            let ansible_host = match child {
                Child::Replicaset { .. } => return false,
                Child::Host { vars, .. } => &vars.ansible_host,
            };
            ansible_host == &Address::from("192.168.16.11")
        })
        .count();

    assert_eq!(with_default_ansible_host, 1);
}
