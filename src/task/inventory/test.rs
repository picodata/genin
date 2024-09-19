use std::convert::TryFrom;

use crate::task::cluster::Cluster;

use super::Inventory;

#[test]
fn inventory_per_instance_vars() {
    let cluster_v2_str: String = r#"---
topology:
  - name: router
    replicasets_count: 1
    roles:
      - router
    vars:
      cartridge_extra_env:
        TARANTOOL_NET_MSG_MAX: 1536
      cartridge_force_leader_control_instance: false
hosts:
  - name: datacenter-1
    hosts:
      - name: server-1
        config:
          http_port: 8081
          binary_port: 3031
          address: 192.168.16.11
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

    let cluster = serde_yaml::from_str(&cluster_v2_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    insta::assert_yaml_snapshot!(inventory);
}

#[test]
fn all_rw_only_in_replicaset() {
    let cluster_v2_str: String = r#"---
topology:
  - name: storage
    replicasets_count: 1
    roles:
      - storage
    all_rw: true
hosts:
  - name: datacenter-1
    hosts:
      - name: server-1
        config:
          http_port: 8081
          binary_port: 3031
          address: 192.168.16.11
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

    let cluster: Cluster = serde_yaml::from_str(&cluster_v2_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    insta::assert_yaml_snapshot!(inventory);
}

#[test]
fn test_build_without_ansible_password() {
    let cluster_v2_str: String = r#"---
topology:
  - name: storage
    replicasets_count: 1
    roles:
      - storage
    all_rw: true
hosts:
  - name: datacenter-1
    hosts:
      - name: server-1
        config:
          http_port: 8081
          binary_port: 3031
          address: 192.168.16.11
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri: "192.168.16.11:4401"
    password: password
vars:
  cartridge_app_name: myapp
  cartridge_cluster_cookie: myapp-cookie
  cartridge_package_path: /tmp/myapp.rpm
  cartridge_bootstrap_vshard: true"#
        .into();

    let cluster: Cluster = serde_yaml::from_str(&cluster_v2_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    insta::assert_yaml_snapshot!(inventory);
}
