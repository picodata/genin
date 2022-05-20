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

    let fs = fs.check(Some("cluster.genin.yaml"), Some("inventory.yaml"));

    assert_eq!(fs.source, Some(PathBuf::from("cluster.genin.yaml")));
    assert_eq!(fs.output, Some(PathBuf::from("inventory.yaml")));
}

#[test]
fn test_fs_interaction_read() {
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
    .check(None, Some("test/output/inventory.yaml"));

    assert_eq!(
        fs.read(),
        Err(TaskError::InternalError(InternalError::Undefined(
            "Error while trying to read source file. Source file: None".into(),
        )))
    );
    let fs = fs.check(Some("not-exists.yaml"), None);

    assert_eq!(
        fs.read(),
        Err(TaskError::ConfigError(ConfigError::FileContentError(
            "Error then opening file not-exists.yaml! \
            Err: No such file or directory (os error 2)"
                .into()
        )))
    );

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
                "test/resources/test-cluster.genin.yaml",
            ])
            .unwrap(),
    );

    assert!(fs.read().is_ok());
}
