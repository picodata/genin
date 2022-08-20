use super::*;

use std::{
    fs::{write, File},
    sync::Once,
};

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        match std::path::Path::new("test/outputs").exists() {
            false => {
                std::fs::create_dir("test/outputs").unwrap();
            }
            true => {}
        };
    });
}

#[allow(unused)]
fn test_spreading_to_servers() {
    initialize();
    let _cluster = Cluster::default();

    //assert_eq!(Scheme::try_from(&cluster).unwrap(), scheme);
}

#[test]
fn test_etcd2_failover_from_yaml() {
    initialize();
    let source = "test/resources/test-etcd2-cluster.genin.yaml";
    let output = "test/outputs/test-etcd2-inventory.yaml";

    let cluster = Cluster::try_from(std::fs::read(source).unwrap().as_slice()).unwrap();
    let scheme = Scheme::try_from(&cluster).unwrap();

    std::fs::File::create(output).unwrap_or_else(|_| panic!("File {} creation failed", output));
    let bytes = serde_yaml::to_vec(&scheme.vars).unwrap();
    std::fs::write(output, bytes).unwrap_or_else(|_| panic!("Writing to file {} failed", output));

    //check that vars were written correctly
    let f = std::fs::File::open(output).unwrap();
    let scheme_vars_yml: serde_yaml::Value = serde_yaml::from_reader(f).unwrap();

    let yml_str = r#"
        ansible_user: vagrant
        ansible_password: vagrant
        cartridge_app_name: tdg
        cartridge_cluster_cookie: myapp-cookie
        cartridge_package_path: CHANGE_ME
        cartridge_bootstrap_vshard: true
        cartridge_failover_params:
            mode: stateful
            state_provider: etcd2
            etcd2_params:
                prefix: cartridge/tdg
                lock_delay: 30
                endpoints:
                    - "http://192.168.123.2:2379"
          "#;

    let expected_yml: serde_yaml::Value = serde_yaml::from_str(yml_str).unwrap();

    assert_eq!(expected_yml, scheme_vars_yml);
    std::fs::remove_file(output).unwrap_or_else(|_| panic!("File {} deletion failed", output));
}

#[test]
fn test_stateboard_failover_from_yaml() {
    initialize();
    let source = "test/resources/test-cluster.genin.yaml";
    let output = "test/outputs/test-stateboard-from-yaml-out.yaml";

    let cluster = Cluster::try_from(std::fs::read(source).unwrap().as_slice()).unwrap();
    let scheme = Scheme::try_from(&cluster).unwrap();

    File::create(output).unwrap_or_else(|_| panic!("File {} creation failed", output));
    let bytes = serde_yaml::to_vec(&scheme.vars).unwrap();
    write(output, bytes).unwrap_or_else(|_| panic!("Writing to file {} failed", output));

    //check that vars were written correctly
    let scheme_vars_yml: Value = serde_yaml::from_reader(File::open(output).unwrap()).unwrap();
    let yml_str = r#"
        ansible_user: root
        ansible_password: change_me
        cartridge_app_name: myapp
        cartridge_cluster_cookie: myapp-cookie
        cartridge_package_path: /tmp/myapp.rpm
        cartridge_bootstrap_vshard: true
        cartridge_failover_params:
            mode: stateful
            state_provider: stateboard
            stateboard_params:
                uri: "192.168.16.1:4401"
                password: "some_password"
          "#;
    let expected_yml: Value = serde_yaml::from_str(yml_str).unwrap();

    assert_eq!(scheme_vars_yml, expected_yml);
}
