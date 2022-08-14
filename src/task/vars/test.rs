use indexmap::IndexMap;

use crate::{
    task::{
        cluster::hst::v2::Address,
        flv::{Failover, FailoverVariants, Mode, StateProvider, StateboardParams, Uri},
        vars::Vars,
    },
    DEFAULT_STATEBOARD_PORT,
};

#[test]
fn vars_serialization() {
    let vars = Vars::default();

    let vars_model_str: String = r#"---
ansible_user: ansible
ansible_password: ansible
cartridge_app_name: myapp
cartridge_cluster_cookie: myapp-cookie
cartridge_package_path: /tmp/myapp.rpm
cartridge_bootstrap_vshard: true
cartridge_failover_params:
  mode: disabled
"#
    .into();

    let vars_str = serde_yaml::to_string(&vars).unwrap();

    assert_eq!(vars_str, vars_model_str);
}

#[test]
fn vars_from_uncomplete_str() {
    let vars_str: String = r#"---
ansible_user: ansible
ansible_password: ansible
cartridge_app_name: genin-app
cartridge_cluster_cookie: genin-app-secret-cookie"#
        .into();

    let vars: Vars = serde_yaml::from_str(&vars_str).unwrap();

    let vars_model = Vars {
        ansible_user: Some("ansible".into()),
        ansible_password: Some("ansible".into()),
        cartridge_app_name: Some("genin-app".into()),
        cartridge_cluster_cookie: Some("genin-app-secret-cookie".into()),
        cartridge_package_path: None,
        cartridge_bootstrap_vshard: None,
        cartridge_failover_params: None,
        another_fields: IndexMap::new(),
    };

    assert_eq!(vars, vars_model);
}

#[test]
fn vars_failover() {
    let stateboard_failover = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            uri: Uri {
                address: Address::Ip("192.168.16.11".parse().unwrap()),
                port: DEFAULT_STATEBOARD_PORT,
            },
            password: "some_password".to_string(),
        }),
    };

    let vars = Vars::from(&stateboard_failover);

    let vars_model = Vars {
        cartridge_failover_params: Some(Failover {
            mode: Mode::Stateful,
            state_provider: StateProvider::Stateboard,
            failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
                uri: Uri {
                    address: Address::Ip("192.168.16.11".parse().unwrap()),
                    port: DEFAULT_STATEBOARD_PORT,
                },
                password: "some_password".to_string(),
            }),
        }),
        ..Vars::default()
    };

    assert_eq!(vars, vars_model);
}
