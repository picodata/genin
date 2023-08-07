use std::{fs::File, io::Read, path::PathBuf};

use clap::{Arg, ArgAction, Command};

use crate::{
    error::{GeninError, GeninErrorKind},
    task::{cluster::fs::INVENTORY_YAML, insert_comments},
};

use super::{
    cluster::{
        fs::{TryMap, CLUSTER_YAML, IO},
        Cluster,
    },
    inventory::Inventory,
};

#[test]
fn default_cluster() {
    let args = Command::new("init")
        .arg(
            Arg::new("failover-mode")
                .long("failover-mode")
                .short('m')
                .action(ArgAction::Set)
                .default_value("stateful")
                .help("(string): failover mode (statefull, eventual, disabled)"),
        )
        .arg(
            Arg::new("failover-state-provider")
                .long("failover-state-provider")
                .short('F')
                .action(ArgAction::Set)
                .default_value("stateboard")
                .help("(string): failover state provider"),
        )
        .try_get_matches_from(vec!["init"])
        .unwrap();

    let cluster = IO::from(&args)
        .try_into_files(None, Some(INVENTORY_YAML), false)
        .unwrap()
        .try_map(|_| {
            Ok(IO {
                input: Some(()),
                output: Some(
                    Cluster::try_from(&args)
                        .map(|cluster| insert_comments(&cluster))
                        .unwrap()
                        .unwrap(),
                ),
            })
        })
        .unwrap()
        .output
        .unwrap();

    insta::assert_display_snapshot!(cluster);
}

#[test]
fn build_consistency_100_times() {
    let args = Command::new("genin")
        .args(&[
            Arg::new("source")
                .long("source")
                .short('s')
                .action(ArgAction::Set),
            Arg::new("force")
                .long("force")
                .short('f')
                .action(ArgAction::SetTrue),
        ])
        .try_get_matches_from(vec![
            "genin",
            "--source",
            "tests/resources/test-cluster.genin.yaml",
        ])
        .unwrap();

    let build_inventory = || -> String {
        let bytes = IO::from(&args)
            .try_into_files(Some(CLUSTER_YAML), None, args.get_flag("force"))
            .unwrap()
            .deserialize_input::<Cluster>()
            .unwrap()
            .try_map(|IO { input, .. }| {
                Inventory::try_from(&input).map(|inventory| IO {
                    input: Some(inventory),
                    output: Some(Vec::new()),
                })
            })
            .unwrap()
            .serialize_input()
            .unwrap()
            .output
            .unwrap();

        String::from_utf8(bytes).unwrap()
    };

    let first = build_inventory();

    for _ in 0..100 {
        assert_eq!(build_inventory(), first);
    }
}

#[test]
fn upgrade_consistency_100_times() {
    let args = Command::new("genin")
        .args(&[
            Arg::new("old").long("old").action(ArgAction::Set),
            Arg::new("new").long("new").action(ArgAction::Set),
        ])
        .try_get_matches_from(vec![
            "genin",
            "--old",
            "tests/resources/test-cluster.genin.yaml",
            "--new",
            "tests/resources/test-upgrade-cluster.genin.yaml",
        ])
        .unwrap();

    let upgrade_inventory = || -> String {
        let bytes = IO {
            input: args
                .try_get_one::<String>("old")
                .transpose()
                .and_then(|r| r.map_or(None, |s| Some(PathBuf::from(s.as_str())))),
            output: None,
        }
        .try_into_files(Some(CLUSTER_YAML), None, false)
        .unwrap()
        .deserialize_input::<Cluster>()
        .unwrap()
        .try_map(|IO { input, output }| {
            // 1. read source cluster yaml file what should be upgraded
            // 2. read cluster yaml which should contains information about upgrade
            File::open(args.get_one::<String>("new").unwrap())
                .map_err(|err| GeninError::new(GeninErrorKind::IO, err))
                .and_then(|mut file| {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)
                        .map_err(|err| GeninError::new(GeninErrorKind::IO, err))?;
                    Ok(buffer)
                })
                .and_then(|buffer| {
                    serde_yaml::from_slice::<Cluster>(&buffer)
                        .map_err(|err| GeninError::new(GeninErrorKind::Deserialization, err))
                })
                .and_then(|new| {
                    input
                        .ok_or_else(|| {
                            GeninError::new(GeninErrorKind::EmptyField, "input file is empty")
                        })
                        .and_then(|input_cluster| input_cluster.try_upgrade(&new))
                })
                .map(|upgraded| IO {
                    input: Some(upgraded),
                    output,
                })
        })
        .unwrap()
        .try_map(|IO { input, .. }| {
            Inventory::try_from(&input).map(|inventory| IO {
                input: Some(inventory),
                output: Some(Vec::new()),
            })
        })
        .unwrap()
        .serialize_input()
        .unwrap()
        .output
        .unwrap();

        String::from_utf8(bytes).unwrap()
    };

    let first = upgrade_inventory();

    for _ in 0..100 {
        assert_eq!(upgrade_inventory(), first);
    }
}
