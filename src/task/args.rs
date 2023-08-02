use crate::{APP_AUTHOR, APP_NAME, APP_VERSION};
use clap::{Arg, ArgAction, ArgMatches, Command};

pub(super) fn read() -> ArgMatches {
    Command::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHOR)
        .about("Quick inventory generation for tarantool apps")
        .subcommand_required(true)
        .dont_collapse_args_in_usage(true)
        .args(&[Arg::new("verbosity")
            .short('v')
            .action(ArgAction::Count)
            .global(true)
            .help("Set logging level based on -v (debug) or -vv (trace)")])
        .subcommands(vec![
            Command::new("build")
                .about("Generate inventory based on cluster.genin.yaml configuration")
                .args(&[
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .action(ArgAction::Set)
                        .help(
                            "Absolute or relative path of the file with \
                            the description of the cluster to be generated",
                        ),
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .action(ArgAction::Set)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved.",
                        ),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to overwrite the output file, whether \
                            or not it exists.",
                        ),
                    Arg::new("ansible-user")
                        .long("ansible-user")
                        .action(ArgAction::Set)
                        .help("(string, optional): login to perform ansible playbook"),
                    Arg::new("ansible-password")
                        .long("ansible-password")
                        .action(ArgAction::Set)
                        .help("(string, optional) :password from ansible user"),
                    Arg::new("cartridge-package-path")
                        .long("catrige-package-path")
                        .action(ArgAction::Set)
                        .help("(string, optional): path to application package"),
                    Arg::new("cartridge-cluster-cookie")
                        .long("cartridge-cluster-cookie")
                        .action(ArgAction::Set)
                        .help("(string): cluster cookie for all cluster instances"),
                    Arg::new("failover-mode")
                        .long("failover-mode")
                        .short('m')
                        .action(ArgAction::Set)
                        .default_value("stateful")
                        .help("(string): failover mode (statefull, eventual, disabled)"),
                    Arg::new("failover-state-provider")
                        .long("failover-state-provider")
                        .short('F')
                        .action(ArgAction::Set)
                        .default_value("stateboard")
                        .help("(string): failover state provider"),
                    Arg::new("fd-as-zone")
                        .long("fd-as-zone")
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to insert 'failure_domain' field's value of instances in their 'zone' field.",
                        ),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("do not print table and cluster yaml"),
                    Arg::new("export-state")
                        .long("export-state")
                        .action(ArgAction::Set)
                        .help("export the build state"),
                ]),
            Command::new("init")
                .about("Init genin and create cluster.genin.yaml configuration")
                .args(&[
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .action(ArgAction::Set)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved.",
                        ),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to overwrite the output file, whether \
                            or not it exists.",
                        ),
                    Arg::new("failover-mode")
                        .long("failover-mode")
                        .short('m')
                        .action(ArgAction::Set)
                        .default_value("stateful")
                        .help("(string): failover mode (stateful, eventual, disabled)"),
                    Arg::new("failover-state-provider")
                        .long("failover-state-provider")
                        .short('F')
                        .action(ArgAction::Set)
                        .default_value("stateboard")
                        .help("(string): failover state provider (etcd2, stateboard, disabled)"),
                    Arg::new("print")
                        .long("print")
                        .short('p')
                        .action(ArgAction::Set)
                        .default_values(["colorized", "ports"])
                        .num_args(1..=3)
                        .help("(list, optional): cluster print output option"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("do not print table and cluster yaml"),
                ]),
            Command::new("inspect")
                .about(
                    "Read cluster.genin.yaml configuration or inventory.yaml \
                        and display cluster schema. This command is needed \
                        for a quick overview of the cluster distribution.",
                )
                .args(&[
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .action(ArgAction::Set)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved.",
                        ),
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .action(ArgAction::Set)
                        .help(
                            "Absolute or relative path of the file with \
                            the description of the cluster to be should displayed",
                        ),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to overwrite the output file, whether \
                            or not it exists.",
                        ),
                    Arg::new("export-csv")
                        .long("export-csv")
                        .short('e')
                        .action(ArgAction::Set)
                        .default_value("cluster.csv")
                        .help("Export resulting schema as csv."),
                ]),
            Command::new("reverse")
                .about(
                    "In some cases, you may need to get a cluster.genin.yaml \
                    based on an already prepared inventory. This subcommand \
                    is for that.",
                )
                .args(&[
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .action(ArgAction::Set)
                        .help(
                            "Absolute or relative path of the file with \
                            the ready cluster inventory.",
                        ),
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .action(ArgAction::Set)
                        .help(
                            "The absolute or relative path where the \
                            cluster.genin.yaml will be saved.",
                        ),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to overwrite the output file, whether \
                            or not it exists.",
                        ),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("do not print table and cluster yaml"),
                ]),
            Command::new("upgrade")
                .about(
                    "Using the genin configuration and the inventory to be \
                    modified creates a new inventory",
                )
                .args(&[
                    Arg::new("old")
                        .long("old")
                        .action(ArgAction::Set)
                        .help(
                            "Absolute or relative path of the file with \
                            the description of the cluster to be generated",
                        ),
                    Arg::new("new")
                        .long("new")
                        .action(ArgAction::Set)
                        .required(true)
                        .help(
                            "New cluster config based on which the upgrade \
                            will be generated",
                        ),
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .action(ArgAction::Set)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved.",
                        ),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(ArgAction::SetTrue)
                        .help(
                            "Used to overwrite the output file, whether \
                            or not it exists.",
                        ),
                    Arg::new("ansible-user")
                        .long("ansible-user")
                        .action(ArgAction::Set)
                        .help("(string, optional): login to perform ansible playbook"),
                    Arg::new("ansible-password")
                        .long("ansible-password")
                        .action(ArgAction::Set)
                        .help("(string, optional) :password from ansible user"),
                    Arg::new("cartridge-package-path")
                        .long("catrige-package-path")
                        .action(ArgAction::Set)
                        .help("(string, optional): path to application package"),
                    Arg::new("cartridge-cluster-cookie")
                        .long("cartridge-cluster-cookie")
                        .action(ArgAction::Set)
                        .help("(string): cluster cookie for all cluster instances"),
                    Arg::new("failover-mode")
                        .long("failover-mode")
                        .short('m')
                        .action(ArgAction::Set)
                        .default_value("stateful")
                        .help("(string): failover mode (statefull, eventual, disabled)"),
                    Arg::new("failover-state-provider")
                        .long("failover-state-provider")
                        .short('F')
                        .action(ArgAction::Set)
                        .default_value("stateboard")
                        .help("(string): failover state provider"),
                    Arg::new("quiet")
                        .long("quiet")
                        .short('q')
                        .action(ArgAction::SetTrue)
                        .help("do not print table and cluster yaml"),
                    Arg::new("export-state")
                        .long("export-state")
                        .action(ArgAction::Set)
                        .help("export the upgrade state with all distribution features"),
                    Arg::new("state-dir")
                        .long("state-dir")
                        .env("GENIN_STATE_DIR")
                        .action(ArgAction::Set)
                        .help("override .geninstate directory location"),
                    Arg::new("from-latest-state")
                        .long("from-latest-state")
                        .action(ArgAction::SetTrue)
                        .help("make upgrade from latest instead of a config file"),
                ]),
        ])
        .get_matches()
}
