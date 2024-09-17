use indexmap::IndexMap;

use crate::task::cluster::{
    ins::{v1::Instance, v2::Instance, Role, Type},
    Cluster,
};

#[test]
fn test_router_deserialization() {
    let yaml = r"
name: router
type: router
count: 1
replicas: 4
weight: 50
roles: [router]
    ";

    let router = Instance {
        name: "router".to_string(),
        parent: Default::default(),
        itype: Type::Router,
        count: 1,
        replicas: 0,
        weight: 0,
        stateboard: false,
        roles: vec![Role::Router("router".to_string())],
        config: Default::default(),
    };

    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_router_deserialization'");

    assert_eq!(router, de_router);

    let yaml = r"
name: router
count: 1
    ";
    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_router_deserialization'");

    assert_eq!(router, de_router);
}

#[test]
fn test_storage_deserialization() {
    let yaml = r"
name: storage
type: storage
count: 2
replicas: 2
weight: 50
roles: [storage]
    ";

    let router = Instance {
        name: "storage".to_string(),
        parent: Default::default(),
        itype: Type::Storage,
        count: 2,
        replicas: 2,
        weight: 50,
        stateboard: false,
        roles: vec![Role::Storage("storage".to_string())],
        config: Default::default(),
    };

    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_storage_deserialization'");

    assert_eq!(router, de_router);

    let yaml = r"
name: storage
count: 10
roles: [api, calculator]
    ";

    let router = Instance {
        name: "storage".to_string(),
        parent: Default::default(),
        itype: Type::Storage,
        count: 10,
        replicas: 1,
        weight: 10,
        stateboard: false,
        roles: vec![
            Role::Api("api".to_string()),
            Role::Custom("calculator".to_string()),
        ],
        config: Default::default(),
    };

    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_storage_deserialization'");

    assert_eq!(router, de_router);
}

#[test]
fn test_custom_deserialization() {
    let yaml = r"
name: calculator
type: custom
count: 3
replicas: 10
weight: 1000
roles: [calculator]
    ";

    let router = Instance {
        name: "calculator".to_string(),
        parent: Default::default(),
        itype: Type::Custom,
        count: 3,
        replicas: 0,
        weight: 0,
        stateboard: false,
        roles: vec![Role::Custom("calculator".to_string())],
        config: Default::default(),
    };

    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_custom_deserialization'");

    assert_eq!(router, de_router);

    let yaml = r"
name: query_predictor
count: 1
    ";

    let router = Instance {
        name: "query_predictor".to_string(),
        parent: Default::default(),
        itype: Type::Custom,
        count: 1,
        replicas: 0,
        weight: 0,
        stateboard: false,
        roles: vec![Role::Custom("query_predictor".to_string())],
        config: Default::default(),
    };

    let de_router: Instance = serde_yaml::from_str(&yaml)
        .expect("Error then deserializing router in test 'test_custom_deserialization'");

    assert_eq!(router, de_router);
}

#[test]
fn default_instances() {
    let instances = vec![
        Instance {
            name: "router".into(),
            parent: "router".into(),
            itype: Type::Router,
            replicasets_count: 1,
            replication_factor: 0,
            weight: 0,
            stateboard: false,
            roles: vec![Role::router(), Role::failover_coordinator()],
            config: IndexMap::new(),
        },
        Instance {
            name: "storage".into(),
            parent: "storage".into(),
            itype: Type::Storage,
            replicasets_count: 2,
            replication_factor: 1,
            weight: 10,
            stateboard: false,
            roles: vec![Role::storage()],
            config: IndexMap::new(),
        },
    ];
    assert_eq!(Cluster::default().topology(), instances);
}
