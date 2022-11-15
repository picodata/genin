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
        http_port: "8100"
    storage-1-1:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.65:5003"
        http_port: "8101"
    storage-1-3:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.65:5004"
        http_port: "8102"
    storage-2-2:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.65:5005"
        http_port: "8103"
    cache-1-1:
      config:
        advertise_uri: "10.99.16.65:5006"
        http_port: "8104"
    cache-1-3:
      config:
        advertise_uri: "10.99.16.65:5007"
        http_port: "8105"
    cache-2-2:
      config:
        advertise_uri: "10.99.16.65:5008"
        http_port: "8106"
    router-1:
      config:
        advertise_uri: "10.99.16.65:5009"
        http_port: "8107"
    calculator-1:
      config:
        advertise_uri: "10.99.16.66:5002"
        http_port: "8100"
    storage-1-2:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.66:5003"
        http_port: "8101"
    storage-2-1:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.66:5004"
        http_port: "8102"
    storage-2-3:
      config:
        memtx_memory: 1179869184
        vinyl_memory: 10345452343
        advertise_uri: "10.99.16.66:5005"
        http_port: "8103"
    cache-1-2:
      config:
        advertise_uri: "10.99.16.66:5006"
        http_port: "8104"
    cache-2-1:
      config:
        advertise_uri: "10.99.16.66:5007"
        http_port: "8105"
    cache-2-3:
      config:
        advertise_uri: "10.99.16.66:5008"
        http_port: "8106"
  children:
    api-replicaset:
      vars:
        replicaset_alias: api
        weight: 10
        failover_priority:
          - api-1
        roles:
          - api
      hosts:
        api-1: ~
    storage-1-replicaset:
      vars:
        replicaset_alias: storage-1
        weight: 10
        failover_priority:
          - storage-1-1
          - storage-1-2
          - storage-1-3
        roles:
          - storage
      hosts:
        storage-1-1: ~
        storage-1-2: ~
        storage-1-3: ~
    storage-2-replicaset:
      vars:
        replicaset_alias: storage-2
        weight: 10
        failover_priority:
          - storage-2-1
          - storage-2-2
          - storage-2-3
        roles:
          - storage
      hosts:
        storage-2-1: ~
        storage-2-2: ~
        storage-2-3: ~
    cache-1-replicaset:
      vars:
        replicaset_alias: cache-1
        weight: 10
        failover_priority:
          - cache-1-1
          - cache-1-2
          - cache-1-3
        roles:
          - storage
      hosts:
        cache-1-1: ~
        cache-1-2: ~
        cache-1-3: ~
    cache-2-replicaset:
      vars:
        replicaset_alias: cache-2
        weight: 10
        failover_priority:
          - cache-2-1
          - cache-2-2
          - cache-2-3
        roles:
          - storage
      hosts:
        cache-2-1: ~
        cache-2-2: ~
        cache-2-3: ~
    router-replicaset:
      vars:
        replicaset_alias: router
        weight: 10
        failover_priority:
          - router-1
        roles:
          - router
          - failover-coordinator
      hosts:
        router-1: ~
    calculator-replicaset:
      vars:
        replicaset_alias: calculator
        weight: 10
        failover_priority:
          - calculator-1
        roles:
          - calculator
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
"#
    .into();

    let cluster: Cluster = serde_yaml::from_str(&cluster_v1_str).unwrap();

    let inventory = Inventory::try_from(&Some(cluster)).unwrap();

    let inventory_str = serde_yaml::to_string(&inventory).unwrap();

    assert_eq!(inventory_str, inventory_model_str);
}
