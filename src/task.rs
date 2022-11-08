mod args;
pub mod cluster;
mod flv;
pub mod inventory;
pub mod vars;

use log::info;
use std::error::Error;

use crate::error::{GeninError, GeninErrorKind};
use crate::task::cluster::fs::{TryMap, IO};
use crate::task::{
    cluster::fs::{CLUSTER_YAML, INVENTORY_YAML},
    cluster::Cluster,
    inventory::Inventory,
};

/// А function that launches an application and walks it through the state stages.
pub fn run_v2() -> Result<(), Box<dyn Error>> {
    // At first set logging level
    // -v       info
    // -vv      debug
    // -vvv     trace
    let args = args::read();
    std::env::set_var(
        "RUST_LOG",
        match args.get_count("verbosity") {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        },
    );
    env_logger::init();

    info!(
        "Log level {}",
        std::env::var("RUST_LOG").unwrap_or_else(|_| "warn".into())
    );

    // The idea of the first step of creating a task:
    //      - create FsInteration
    //      - map FsInteration as:
    //          - read source from disk
    //          - [map] source deserialized to Data or default Data created (data type depends of
    //          subcomand)
    //          - [map] map data to scheme created from data
    //          - [map] move scheme and data into two closures and return them with fs
    //      - return tupple
    match args.subcommand() {
        Some(("init", args)) => {
            IO::from(args)
                // TODO: make better PathBuf
                .try_into_files(None, Some(CLUSTER_YAML), args.get_flag("force"))?
                .try_map(|IO { output, .. }| {
                    Cluster::try_from(args).map(|cluster| IO {
                        input: Some(cluster),
                        output,
                    })
                })?
                .print_input()
                .serialize_input()?;
        }
        Some(("build", args)) => {
            IO::from(args)
                .try_into_files(
                    Some(CLUSTER_YAML),
                    Some(INVENTORY_YAML),
                    args.get_flag("force"),
                )?
                .deserialize_input::<Cluster>()?
                .print_input()
                .try_map(|IO { input, output }| {
                    Inventory::try_from(&input).map(|inventory| IO {
                        input: Some(inventory),
                        output,
                    })
                })?
                .serialize_input()?;
        }
        Some(("inspect", args)) => {
            let io = IO::from(args)
                .try_into_files(Some(CLUSTER_YAML), None, args.get_flag("force"))?
                .deserialize_input::<Cluster>()?
                .consume_output();
            println!("{}", io);
        }
        Some(("reverse", args)) => {
            IO::from(args)
                .try_into_files(Some(INVENTORY_YAML), None, args.get_flag("force"))?
                .deserialize_input::<Inventory>()?
                .try_map(|IO { input, output }| {
                    Cluster::try_from(&input).map(|cluster| IO {
                        input: Some(cluster),
                        output,
                    })
                })?
                .print_input()
                .serialize_input()?;
        }
        _ => {
            return Err(GeninError::new(GeninErrorKind::ArgsError, "subcommand missing").into());
        }
    }

    Ok(())
}
