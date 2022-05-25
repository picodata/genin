use crate::{APP_AUTHOR, APP_NAME, APP_VERSION};
use clap::{Arg, ArgMatches, Command};

pub(super) fn read() -> ArgMatches {
    Command::new(APP_NAME)
        .version(APP_VERSION)
        .author(APP_AUTHOR)
        .about("Quick inventory for TDG")
        .subcommand_required(true)
        .dont_collapse_args_in_usage(true)
        .args(&[Arg::new("verbosity")
            .short('v')
            .multiple_occurrences(true)
            .global(true)
            .help("Set logging level based on -v (debug) or -vv (trace)")])
        .subcommands(vec![
            Command::new("build")
                .about("Generate inventory based on cluster.genin.yaml configuration")
                .args(&[
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .takes_value(true)
                        .default_value("cluster.genin.yml")
                        .help(
                            "Absolute or relative path of the file with \
                            the description of the cluster to be generated",
                        ),
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .takes_value(true)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved."
                        ),
                    Arg::new("ansible-user")
                        .long("ansible-user")
                        .takes_value(true)
                        .help("(string, optional): login to perform ansible playbook"),
                    Arg::new("ansible-password")
                        .long("ansible-password")
                        .takes_value(true)
                        .help("(string, optional) :password from ansible user"),
                    Arg::new("cartridge-package-path")
                        .long("catrige-package-path")
                        .takes_value(true)
                        .help("(string, optional): path to application package"),
                    Arg::new("cartridge-cluster-cookie")
                        .long("cartridge-cluster-cookie")
                        .takes_value(true)
                        .help("(string): cluster cookie for all cluster instances"),
                    Arg::new("failover-mode")
                        .long("failover-mode")
                        .takes_value(true)
                        .help("(string): failover mode (statefull, eventual, disabled)"),
                    Arg::new("failover-state-provider")
                        .long("failover-state-provider")
                        .takes_value(true)
                        .help("(string): failover state provider"),
                ]),
            Command::new("init")
                .about("Init genin and create cluster.genin.yaml configuration")
                .args(&[
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .takes_value(true)
                        .help(
                            "The absolute or relative path where the \
                            ready-made cluster inventory will be saved.",
                        ),
                    Arg::new("failover-mode")
                        .long("failover-mode")
                        .short('f')
                        .takes_value(true)
                        .default_value("stateful")
                        .help("(string): failover mode (stateful, eventual, disabled)"),
                    Arg::new("failover-state-provider")
                        .long("failover-state-provider")
                        .short('F')
                        .takes_value(true)
                        .default_value("stateboard")
                        .help("(string): failover state provider (etcd2, stateboard, disabled)"),
                    Arg::new("print")
                        .long("print")
                        .short('p')
                        .takes_value(true)
                        .default_values(&["colorized", "ports"])
                        .multiple_values(true)
                        .help("(list, optional): cluster print output option"),
                ]),
            Command::new("inspect")
                .override_help("Generate and show cluster scheme whithout saving")
                .about(
                    "Read cluster.genin.yaml configuration or inventory.yaml \
                        and display cluster schema. This command is needed \
                        for a quick overview of the cluster distribution.",
                )
                .args(&[
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .takes_value(true)
                        .default_value("cluster.genin.yaml")
                        .help(
                            "Absolute or relative path of the file with \
                            the description of the cluster to be should displayed",
                        ),
                    Arg::new("export-csv")
                        .long("export-csv")
                        .short('e')
                        .takes_value(true)
                        .default_value("cluster.csv")
                        .help("Export resulting schema as csv."),
                ]),
            Command::new("reverse")
                .override_help("Reverse parsing inventory.yaml and save the configuration.")
                .about(
                    "In some cases, you may need to get a cluster.genin.yaml \
                            based on an already prepared inventory. This subcommand \
                            is for that.",
                )
                .args(&[
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .takes_value(true)
                        .default_value("inventory.yaml")
                        .help(
                            "Absolute or relative path of the file with \
                            the ready cluster inventory.",
                        ),
                    Arg::new("output")
                        .long("output")
                        .short('o')
                        .takes_value(true)
                        .help(
                            "The absolute or relative path where the \
                            cluster.genin.yaml will be saved.",
                        ),
                ]),
        ])
        .get_matches()
}
