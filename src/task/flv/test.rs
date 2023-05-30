use clap::{Arg, ArgAction, Command};

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
    };

    assert_eq!(failover, failover_model);
}

#[test]
fn failover_to_str() {
    let failover = Failover {
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

    let failover_model_str: String = r#"---
mode: stateful
state_provider: stateboard
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
    };

    assert_eq!(flv, flv_model);
}
