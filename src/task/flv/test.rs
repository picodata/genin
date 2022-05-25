use clap::{Arg, Command};

use super::*;

#[test]
fn test_failover_disabled() {
    let failover = Failover::try_from(
        &Command::new("genin")
            .arg(
                Arg::new("failover-mode")
                    .long("failover-mode")
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
            .arg(Arg::new("source").long("source").takes_value(true))
            .arg(
                Arg::new("failover-mode")
                    .long("failover-mode")
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
            .arg(Arg::new("source").long("source").takes_value(true))
            .arg(
                Arg::new("failover-mode")
                    .long("failover-mode")
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
            .arg(Arg::new("source").long("source").takes_value(true))
            .arg(
                Arg::new("failover-mode")
                    .long("failover-mode")
                    .takes_value(true)
                    .default_value("stateful"),
            )
            .arg(
                Arg::new("failover-state-provider")
                    .long("failover-state-provider")
                    .takes_value(true)
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
