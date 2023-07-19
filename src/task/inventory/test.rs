use std::convert::TryFrom;

use crate::task::cluster::Cluster;

use super::Inventory;

#[test]
fn inventory_from_cluster_v1() {
    let cluster_v1_str: String = r#"---
instances:
  - name: api
    count: 1
    replicas: 0
    weight: 10
    roles:
      - api
  - name: calculator
    count: 1
    replicas: 0
    weight: 10
    roles:
      - calculator
  - name: storage
    type: storage
    count: 2
    replicas: 2
    weight: 10
    roles:
      - storage
    config:
      memtx_memory: 1179869184
      vinyl_memory: 10345452343
  - name: cache
    type: storage
    count: 2
    replicas: 2
    weight: 10
    roles:
      - storage
  - name: router
    type: router
    count: 1
    replicas: 0
    weight: 10
    roles:
      - router
      - failover-coordinator
hosts:
  - name: docker
    type: datacenter
    ports:
      http: 8100
      binary: 5002
    hosts:
      - name: host-1
        ip: 10.99.16.65
      - name: host-2
        ip: 10.99.16.66
failover:
  mode: stateful
  state_provider: stateboard
  stateboard_params:
    uri:
      ip: 10.99.16.66
      port: 5001
    password: genin-app
vars:
  ansible_user: ansible
  ansible_password: ansible
  cartridge_app_name: genin-app
  cartridge_cluster_cookie: genin-app-secret-cookie"#
        .into();

    let inventory_model_str: String = r#"---
all:
  vars:
    ansible_user: ansible
    ansible_password: ansible
    cartridge_app_name: genin-app
    cartridge_cluster_cookie: genin-app-secret-cookie
    cartridge_failover_params:
      mode: stateful
      state_provider: stateboard
      stateboard_params:
        uri: "10.99.16.66:5001"
        password: genin-app
  hosts:
    api-1:
      config:
        advertise_uri: "10.99.16.65:5002"
        http_port: 8100
    storage-1-1:
      config:
        advertise_uri: "10.99.16.65:5003"
        http_port: 8101
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    storage-1-3:
      config:
        advertise_uri: "10.99.16.65:5004"
        http_port: 8102
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    storage-2-2:
      config:
        advertise_uri: "10.99.16.65:5005"
        http_port: 8103
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    cache-1-1:
      config:
        advertise_uri: "10.99.16.65:5006"
        http_port: 8104
    cache-1-3:
      config:
        advertise_uri: "10.99.16.65:5007"
        http_port: 8105
    cache-2-2:
      config:
        advertise_uri: "10.99.16.65:5008"
        http_port: 8106
    router-1:
      config:
        advertise_uri: "10.99.16.65:5009"
        http_port: 8107
    calculator-1:
      config:
        advertise_uri: "10.99.16.66:5002"
        http_port: 8100
    storage-1-2:
      config:
        advertise_uri: "10.99.16.66:5003"
        http_port: 8101
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    storage-2-1:
      config:
        advertise_uri: "10.99.16.66:5004"
        http_port: 8102
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    storage-2-3:
      config:
        advertise_uri: "10.99.16.66:5005"
        http_port: 8103
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
    cache-1-2:
      config:
        advertise_uri: "10.99.16.66:5006"
        http_port: 8104
    cache-2-1:
      config:
        advertise_uri: "10.99.16.66:5007"
        http_port: 8105
    cache-2-3:
      config:
        advertise_uri: "10.99.16.66:5008"
        http_port: 8106
    stateboard:
      stateboard: true
      config:
        listen: "10.99.16.66:5001"
        password: genin-app
  children:
    api-1-replicaset:
      vars:
        replicaset_alias: api-1
        failover_priority:
          - api-1
        roles:
          - api
        weight: 10
      hosts:
        api-1: ~
    storage-1-replicaset:
      vars:
        replicaset_alias: storage-1
        failover_priority:
          - storage-1-1
          - storage-1-2
          - storage-1-3
        roles:
          - storage
        weight: 10
      hosts:
        storage-1-1: ~
        storage-1-3: ~
        storage-1-2: ~
    storage-2-replicaset:
      vars:
        replicaset_alias: storage-2
        failover_priority:
          - storage-2-1
          - storage-2-2
          - storage-2-3
        roles:
          - storage
        weight: 10
      hosts:
        storage-2-2: ~
        storage-2-1: ~
        storage-2-3: ~
    cache-1-replicaset:
      vars:
        replicaset_alias: cache-1
        failover_priority:
          - cache-1-1
          - cache-1-2
          - cache-1-3
        roles:
          - storage
        weight: 10
      hosts:
        cache-1-1: ~
        cache-1-3: ~
        cache-1-2: ~
    cache-2-replicaset:
      vars:
        replicaset_alias: cache-2
        failover_priority:
          - cache-2-1
          - cache-2-2
          - cache-2-3
        roles:
          - storage
        weight: 10
      hosts:
        cache-2-2: ~
        cache-2-1: ~
        cache-2-3: ~
    router-1-replicaset:
      vars:
        replicaset_alias: router-1
        failover_priority:
          - router-1
        roles:
          - router
          - failover-coordinator
        weight: 10
      hosts:
        router-1: ~
    calculator-1-replicaset:
      vars:
        replicaset_alias: calculator-1
        failover_priority:
          - calculator-1
        roles:
          - calculator
        weight: 10
      hosts:
        calculator-1: ~
    host-1:
      vars:
        ansible_host: 10.99.16.65
      hosts:
        api-1: ~
        storage-1-1: ~
        storage-1-3: ~
        storage-2-2: ~
        cache-1-1: ~
        cache-1-3: ~
        cache-2-2: ~
        router-1: ~
    host-2:
      vars:
        ansible_host: 10.99.16.66
      hosts:
        calculator-1: ~
        storage-1-2: ~
        storage-2-1: ~
        storage-2-3: ~
        cache-1-2: ~
        cache-2-1: ~
        cache-2-3: ~
        stateboard: ~
"#
    .into();

    let cluster: Cluster = serde_yaml::from_str(&cluster_v1_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    let inventory_str = serde_yaml::to_string(&inventory).unwrap();

    assert_eq!(inventory_str, inventory_model_str);
}

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

    let inventory_model_str: String = r#"---
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
      vars:
        cartridge_extra_env:
          TARANTOOL_NET_MSG_MAX: 1536
        cartridge_force_leader_control_instance: false
    stateboard:
      stateboard: true
      config:
        listen: "192.168.16.11:4401"
        password: password
  children:
    router-1-replicaset:
      vars:
        replicaset_alias: router-1
        failover_priority:
          - router-1
        roles:
          - router
      hosts:
        router-1: ~
    server-1:
      vars:
        ansible_host: 192.168.16.11
      hosts:
        router-1: ~
        stateboard: ~"#
        .into();

    let inventory_model = serde_yaml::from_str(&inventory_model_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    assert_eq!(inventory, inventory_model);
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

    let inventory_model_str: String = r#"---
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
    storage-1:
      config:
        advertise_uri: "192.168.16.11:3031"
        http_port: 8081
    stateboard:
      stateboard: true
      config:
        listen: "192.168.16.11:4401"
        password: password
  children:
    storage-1-replicaset:
      vars:
        replicaset_alias: storage-1
        failover_priority:
          - storage-1
        roles:
          - storage
        all_rw: true
      hosts:
        storage-1: ~
    server-1:
      vars:
        ansible_host: 192.168.16.11
      hosts:
        storage-1: ~
        stateboard: ~"#
        .into();

    let inventory_model: Inventory = serde_yaml::from_str(&inventory_model_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    assert_eq!(inventory, inventory_model);
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

    let inventory_model_str: String = r#"---
all:
  vars:
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
    storage-1:
      config:
        advertise_uri: "192.168.16.11:3031"
        http_port: 8081
    stateboard:
      stateboard: true
      config:
        listen: "192.168.16.11:4401"
        password: password
  children:
    storage-1-replicaset:
      vars:
        replicaset_alias: storage-1
        failover_priority:
          - storage-1
        roles:
          - storage
        all_rw: true
      hosts:
        storage-1: ~
    server-1:
      vars:
        ansible_host: 192.168.16.11
      hosts:
        storage-1: ~
        stateboard: ~"#
        .into();

    let inventory_model: Inventory = serde_yaml::from_str(&inventory_model_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    assert_eq!(inventory, inventory_model);
}
