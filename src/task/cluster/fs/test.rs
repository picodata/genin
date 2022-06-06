use clap::{Arg, Command};

use super::*;

#[test]
fn test_fs_interaction_from_args() {
    let fs = FsInteraction::from(
        &Command::new("genin")
            .arg(
                Arg::new("source")
                    .long("source")
                    .takes_value(true)
                    .default_value("default.source.yaml"),
            )
            .arg(
                Arg::new("output")
                    .long("output")
                    .takes_value(true)
                    .default_value("default.output.yaml"),
            )
            .try_get_matches_from(vec!["genin", "--source", "test.yaml"])
            .unwrap(),
    );

    assert_eq!(fs.source, Some(PathBuf::from("test.yaml")));
    assert_eq!(fs.output, Some(PathBuf::from("default.output.yaml")));

    let fs = FsInteraction::from(
        &Command::new("genin")
            .try_get_matches_from(vec!["genin"])
            .unwrap(),
    );

    assert_eq!(fs.source, None);
    assert_eq!(fs.output, None);

    let fs = fs.check(Some("cluster.genin.yaml"), Some("inventory.yaml"), false);

    assert_eq!(fs.source, None);
    assert_eq!(fs.output, Some(PathBuf::from("inventory.yaml")));

    let fs = FsInteraction::from(
        &Command::new("genin")
            .subcommand(Command::new("init"))
            .try_get_matches_from(vec!["genin", "init"])
            .unwrap(),
    )
    .check(None, Some("cluster.genin.yaml"), false);

    assert_eq!(fs.source, None);
    assert_eq!(fs.output, Some(PathBuf::from("cluster.genin.yaml")));
}

#[test]
fn test_fs_interaction_wrong_ext() {
    let fs = FsInteraction::from(
        &Command::new("genin")
            .arg(
                Arg::new("source")
                    .long("source")
                    .takes_value(true)
                    .default_value("default.source.yaml"),
            )
            .try_get_matches_from(vec![
                "genin",
                "--source",
                "test/resources/test-sort-cluster.genin.yml",
            ])
            .unwrap(),
    )
    .check(None, None, false);
    println!("source file state:  {:?}", fs.read());
    assert!(fs.read().is_ok());
}

#[test]
fn test_fs_interaction_errors() {
    let fs = FsInteraction::from(
        &Command::new("genin")
            .arg(
                Arg::new("output")
                    .long("output")
                    .takes_value(true)
                    .default_value("default.output.yaml"),
            )
            .try_get_matches_from(vec!["genin"])
            .unwrap(),
    )
    .check(None, Some("test/output/inventory.yaml"), false);

    assert_eq!(
        fs.read(),
        Err(TaskError::InternalError(InternalError::UndefinedError(
            "Error while trying to read source file. Source file: None".into(),
        )))
    );

    let fs = fs.check(Some("not-exists.yaml"), None, false);

    assert_eq!(
        fs.read(),
        Err(TaskError::InternalError(InternalError::UndefinedError(
            "Error while trying to read source file. Source file: None".to_string()
        )))
    );
}

#[test]
fn test_fs_interaction_file_exists() {
    let fs = FsInteraction::from(
        &Command::new("genin")
            .arg(
                Arg::new("output")
                    .long("output")
                    .takes_value(true)
                    .default_value("test/resources/test-sort-inventory.yaml"),
            )
            .try_get_matches_from(vec!["genin"])
            .unwrap(),
    )
    .check(None, None, false);

    assert_eq!(
        fs.output,
        Some(PathBuf::from("test/resources/test-sort-inventory.copy.yaml"))
    );
}
