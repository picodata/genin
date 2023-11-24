use clap::{Arg, ArgAction, Command};

use crate::task::utils::uncolorize;

use super::*;

#[test]
fn test_failover_disabled() {
    let failover = Failover::try_from(
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
            .try_get_matches_from(vec![
                "genin",
                "--failover-mode",
                "disabled",
                "--failover-state-provider",
                "stateboard",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.mode, Mode::Disabled);
    assert_eq!(failover.state_provider, StateProvider::Disabled);

    let failover = Failover::try_from(
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
            .try_get_matches_from(vec![
                "genin",
                "--failover-mode",
                "disabled",
                "--failover-state-provider",
                "etcd2",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.mode, Mode::Disabled);
    assert_eq!(failover.state_provider, StateProvider::Disabled);

    let failover = Failover::try_from(
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
            .try_get_matches_from(vec!["genin", "--failover-state-provider", "disabled"])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.mode, Mode::Disabled);
    assert_eq!(failover.state_provider, StateProvider::Disabled);
}

#[test]
fn test_failover_stateboard() {
    // by default
    let failover = Failover::try_from(
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
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::Stateboard);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::StateboardVariant(StateboardParams::default())
    );

    // from args
    let failover = Failover::try_from(
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
            .try_get_matches_from(vec![
                "genin",
                "--failover-mode",
                "stateful",
                "--failover-state-provider",
                "stateboard",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::Stateboard);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::StateboardVariant(StateboardParams::default())
    );

    // override defined in file (eventual)
    let failover = Failover::try_from(
        &Command::new("genin")
            .arg(Arg::new("source").long("source").action(ArgAction::Set))
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
            .try_get_matches_from(vec![
                "genin",
                "--source",
                "test/resources/test-eventual-cluster.genin.yaml",
                "--failover-mode",
                "stateful",
                "--failover-state-provider",
                "stateboard",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::Stateboard);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::StateboardVariant(StateboardParams::default())
    );

    // override defined in file (etcd2)
    let failover = Failover::try_from(
        &Command::new("genin")
            .arg(Arg::new("source").long("source").action(ArgAction::Set))
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
            .try_get_matches_from(vec![
                "genin",
                "--source",
                "test/resources/test-etcd2-cluster.genin.yaml",
                "--failover-mode",
                "stateful",
                "--failover-state-provider",
                "stateboard",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::Stateboard);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::StateboardVariant(StateboardParams::default())
    );
}

#[test]
fn test_failover_etcd() {
    // default etcd2 from args
    let failover = Failover::try_from(
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
            .try_get_matches_from(vec!["genin", "--failover-state-provider", "etcd2"])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::ETCD2);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::ETCD2Variant(ETCD2Params::default())
    );

    // override failover mode from configuration file
    let failover = Failover::try_from(
        &Command::new("genin")
            .arg(Arg::new("source").long("source").action(ArgAction::Set))
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
            .try_get_matches_from(vec![
                "genin",
                "--source",
                "test/resources/test-etcd2-cluster.genin.yaml",
                "--failover-state-provider",
                "etcd2",
            ])
            .unwrap(),
    )
    .unwrap();

    assert_eq!(failover.state_provider, StateProvider::ETCD2);
    assert_eq!(
        failover.failover_variants,
        FailoverVariants::ETCD2Variant(ETCD2Params::default())
    );
}

#[test]
fn failover_from_uri() {
    let failover_str: String = r#"---
mode: stateful
state_provider: stateboard
stateboard_params:
  uri: 192.168.16.11:4401
  password: some_password
"#
    .into();

    let failover: Failover = serde_yaml::from_str(&failover_str).unwrap();

    let failover_model = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            uri: Uri {
                address: Address::Ip("192.168.16.11".parse().unwrap()),
                port: DEFAULT_STATEBOARD_PORT,
            },
            password: "some_password".to_string(),
        }),
        ..Default::default()
    };

    assert_eq!(failover, failover_model);
}

#[test]
fn failover_from_ip_and_port() {
    let failover_str: String = r#"---
mode: stateful
state_provider: stateboard
stateboard_params:
  uri:
    ip: 192.168.16.11
    port: 4401
  password: some_password
"#
    .into();

    let failover: Failover = serde_yaml::from_str(&failover_str).unwrap();

    let failover_model = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            uri: Uri {
                address: Address::Ip("192.168.16.11".parse().unwrap()),
                port: DEFAULT_STATEBOARD_PORT,
            },
            password: "some_password".to_string(),
        }),
        ..Default::default()
    };

    assert_eq!(failover, failover_model);
}

#[test]
fn failover_to_str() {
    let failover = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_timeout: Some(666),
        fencing_enabled: Some(true),
        fencing_pause: Some(42),
        fencing_timeout: Some(24),
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            uri: Uri {
                address: Address::Ip("192.168.16.11".parse().unwrap()),
                port: DEFAULT_STATEBOARD_PORT,
            },
            password: "some_password".to_string(),
        }),
    };

    let failover_model_str: String = r#"---
mode: stateful
state_provider: stateboard
failover_timeout: 666
fencing_enabled: true
fencing_timeout: 24
fencing_pause: 42
stateboard_params:
  uri: "192.168.16.11:4401"
  password: some_password
"#
    .into();

    let failover_str = serde_yaml::to_string(&failover).unwrap();

    assert_eq!(failover_str, failover_model_str);
}

#[test]
fn failover_wrong_stateboard() {
    let flv_str = r#"
mode: stateful
state_provider: stateboard
stateboard_params:
  uri: 'foo-baz-bar.dpc.fpc.gachi.ru:4401'
  password: 'sosiska-123'
"#
    .to_string();

    let flv: Failover = serde_yaml::from_str(&flv_str).unwrap();

    let flv_model = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            uri: Uri {
                address: Address::Uri("foo-baz-bar.dpc.fpc.gachi.ru".into()),
                port: 4401,
            },
            password: "sosiska-123".into(),
        }),
        ..Default::default()
    };

    assert_eq!(flv, flv_model);
}

#[test]
fn invalid_failover() {
    let err1_str = r#"
state_provider: etcd2
etcd2_params:
  prefix: /cartridge
  lock_delay: 30
  endpoints:
    - "http://172.20.73.12:2379"
    - "http://172.20.73.13:2379"
    - "http://172.20.73.14:2379"
  username: 111
  password: 111
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err1_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err2_str = r#"
mode: disabled
state_provider: etcd2
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err2_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err3_str = r#"
mode: eventual
state_provider: etcd2
etcd2_params:
  prefix: /cartridge
  lock_delay: 30
  endpoints:
    - "http://172.20.73.12:2379"
    - "http://172.20.73.13:2379"
    - "http://172.20.73.14:2379"
  username: 111
  password: 111
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err3_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err4_str = r#"
mode: picomod
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err4_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err5_str = r#"
mode: stateful
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err5_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err6_str = r#"
mode: stateful
state_provider: etcd2
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err6_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err7_str = r#"
mode: stateful
state_provider: etcd2
etcd2_params:
  prefix: /cartridge
  lock_delay: 30
  endpoints:
    - "http://172.20.73.12:2379"
    - "http://172.20.73.13:2379"
    - "http://172.20.73.14:2379"
  username: 111
  password: 111
stateboard_params:
  uri: "192.168.16.11:4401"
  password: password
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err7_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err8_str = r#"
mode: stateful
state_provider: etcd2
etcd2_params:
  prefix: /cartridge
  lock_delay: hudred
  endpoints:
    - "http://172.20.73.12:2379"
    - 100000
  username: 111
  password: 111
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err8_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));

    let err9_str = r#"
mode: stateful
state_provider: stateboard
stateboard_params:
  uri: "192.168.16.11:4401"
  password: 123
"#;

    let invalid_v1 = format!(
        "{:?}",
        serde_yaml::from_str::<InvalidFailover>(err9_str).unwrap()
    );

    insta::assert_display_snapshot!(uncolorize(invalid_v1));
}
