use crate::task::{cluster::Cluster, inventory::Inventory, state::State};

#[test]
fn build_from_state() {
    let state_str = r#"
{
    "uid": "f5d95f27d5f472aa99b54ef1882317b7839f4b36403a1b4248bec34eb99d43d7",
    "kind": "Upgrade",
    "changes": [],
    "path": "state.tgz",
    "args_str": "genin build -s cluster.genin.yml -f --export-state state.tgz",
    "vars": {
        "ansible_user": "ansible",
        "ansible_password": "ansible",
        "cartridge_app_name": "myapp",
        "cartridge_cluster_cookie": "myapp-cookie",
        "cartridge_package_path": "/tmp/myapp.rpm",
        "cartridge_bootstrap_vshard": true
    },
    "hosts": {
        "name": "cluster",
        "config": {
            "http_port": 8081,
            "binary_port": 3031
        },
        "hosts": [
            {
                "name": "datacenter-1",
                "config": {
                    "http_port": 8081,
                    "binary_port": 3031
                },
                "hosts": [
                    {
                        "name": "server-1",
                        "config": {
                            "http_port": 8081,
                            "binary_port": 3031,
                            "address": "192.168.16.11"
                        },
                        "instances": [
                            {
                                "name": "router-1",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "router",
                                    "failover-coordinator"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8081,
                                    "binary_port": 3031
                                },
                                "vars": {}
                            },
                            {
                                "name": "storage-1-2",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8082,
                                    "binary_port": 3032
                                },
                                "vars": {}
                            },
                            {
                                "name": "storage-2-2",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8083,
                                    "binary_port": 3033
                                },
                                "vars": {}
                            },
                            {
                                "name": "stateboard",
                                "stateboard": true,
                                "weight": null,
                                "failure_domains": [
                                    "server-1"
                                ],
                                "roles": [],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8084,
                                    "binary_port": 3034,
                                    "listen": "192.168.16.11:4401",
                                    "password": "password"
                                },
                                "vars": {}
                            }
                        ]
                    },
                    {
                        "name": "server-2",
                        "config": {
                            "http_port": 8081,
                            "binary_port": 3031,
                            "address": "192.168.16.12"
                        },
                        "instances": [
                            {
                                "name": "storage-1-1",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8081,
                                    "binary_port": 3031
                                },
                                "vars": {}
                            },
                            {
                                "name": "storage-2-1",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8082,
                                    "binary_port": 3032
                                },
                                "vars": {}
                            }
                        ]
                    },
                    {
                        "name": "server-3",
                        "config": {
                            "http_port": 8081,
                            "binary_port": 3031,
                            "address": "192.168.16.13"
                        },
                        "instances": [
                            {
                                "name": "storage-1-3",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8081,
                                    "binary_port": 3031
                                },
                                "vars": {}
                            },
                            {
                                "name": "storage-2-3",
                                "stateboard": null,
                                "weight": null,
                                "failure_domains": [],
                                "roles": [
                                    "storage"
                                ],
                                "cartridge_extra_env": {},
                                "config": {
                                    "http_port": 8082,
                                    "binary_port": 3032
                                },
                                "vars": {}
                            }
                        ]
                    }
                ],
                "instances": []
            }
        ],
        "instances": []
    },
    "failover": {
        "mode": "stateful",
        "state_provider": "stateboard",
        "stateboard_params": {
            "uri": "192.168.16.11:4401",
            "password": "password"
        }
    }
}"#;

    let state: State = serde_json::from_str(state_str).unwrap();

    let cluster = Cluster::from(state);
    let inventory = Inventory::try_from(&cluster).unwrap();

    insta::assert_yaml_snapshot!(inventory);
}
