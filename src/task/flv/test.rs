use clap::{Arg, ArgAction, Command};

use crate::task::cluster::Cluster;

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
fn test_failover_from_noncomplete_yaml() {
    // test deserialization failover params
    let cluster = Cluster::try_from(
        std::fs::read("test/resources/test-cluster.genin.yaml")
            .unwrap()
            .as_slice(),
    )
    .unwrap();
    let expected_flv = Failover {
        mode: Mode::Stateful,
        state_provider: StateProvider::Stateboard,
        failover_variants: FailoverVariants::StateboardVariant(StateboardParams {
            url: Url {
                ip: "192.168.16.1".parse().unwrap(),
                port: Some(4401),
            },
            password: "some_password".to_string(),
        }),
    };

    assert_eq!(cluster.failover(), &expected_flv);

    // test failover deserialization with uncomplete genin config
    let cluster = Cluster::try_from(
        std::fs::read("test/resources/test-cluster-uncomplete.genin.yaml")
            .unwrap()
            .as_slice(),
    );

    let expected_err = Err(TaskError::ConfigError(ConfigError::FileFormatError(
        "Error then deserializing cluster file data did not match any variant of untagged enum Cluster"
            .to_string(),
    )));

    assert_eq!(cluster, expected_err);
}
