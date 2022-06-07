mod args;
mod cluster;
mod inv;
mod flv;

use crate::task::{
    cluster::{
        Context,
        fs::{CLUSTER_YAML, INVENTORY_YAML},
        scheme::Scheme,
    },
    inv::Inventory,
};
use genin::libs::error::{CommandLineError, ConfigError, TaskError};
use log::info;

use self::cluster::{Cluster, fs::FsInteraction};

/// А function that launches an application and walks it through the state stages.
pub fn run() -> Result<(), TaskError> {
    // At first set logging level
    // -v       info
    // -vv      debug
    // -vvv     trace
    let args = args::read();
    std::env::set_var(
        "RUST_LOG",
        match args.occurrences_of("verbosity") {
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
    Task(args)
        .map(|args| match args.subcommand() {
            Some(("init", args)) => FsInteraction::from(args)
                .check(None, Some(CLUSTER_YAML), args.is_present("force"))
                .map_self(|fs| Ok(Context((Cluster::try_from(args)?, fs))))?
                .map(|(data, fs)| Ok((Scheme::try_from(&data)?, data, fs)))?
                .map(|(scheme, data, fs)| {
                    Ok((
                        (Box::new(move || scheme.print())) as Box<dyn FnOnce()>,
                        (Box::new(move || {
                            serde_yaml::to_vec(&data).map_err(|error| {
                                TaskError::ConfigError(ConfigError::FileContentError(format!(
                                    "Error during deserialization {}",
                                    error
                                )))
                            })
                        }))
                            as Box<dyn FnOnce() -> Result<Vec<u8>, TaskError>>,
                        (Box::new(move |bytes: &[u8]| fs.write(bytes)))
                            as Box<dyn FnOnce(&[u8]) -> Result<(), TaskError>>,
                    ))
                }),
            Some(("build", args)) => FsInteraction::from(args)
                .check(Some(CLUSTER_YAML), Some(INVENTORY_YAML), args.is_present("force"))
                .map_self(|fs| Ok(Context((Cluster::try_from(fs.read()?.as_slice())?, fs))))?
                .map(|(data, fs)| Ok((Scheme::try_from(&data)?, data, fs)))?
                .map(|(scheme, _data, fs)| {
                    Ok((
                        (Box::new(move || {})) as Box<dyn FnOnce()>,
                        (Box::new(move || {
                            scheme.print();
                            serde_yaml::to_vec(&Inventory::try_from(scheme)?).map_err(|err| {
                                TaskError::ConfigError(ConfigError::FileContentError(format!(
                                    "serialization error {}",
                                    err
                                )))
                            })
                        }))
                            as Box<dyn FnOnce() -> Result<Vec<u8>, TaskError>>,
                        (Box::new(move |bytes: &[u8]| fs.write(bytes)))
                            as Box<dyn FnOnce(&[u8]) -> Result<(), TaskError>>,
                    ))
                }),
            Some(("inspect", args)) => FsInteraction::from(args)
                .check(Some(CLUSTER_YAML), None, args.is_present("force"))
                .map_self(|fs| Ok(Context((Cluster::try_from(fs.read()?.as_slice())?, fs))))?
                .map(|(data, fs)| Ok((Scheme::try_from(&data)?, data, fs)))?
                .map(|(scheme, _, _)| {
                    Ok((
                        (Box::new(move || scheme.print())) as Box<dyn FnOnce()>,
                        (Box::new(move || Ok(Vec::new())))
                            as Box<dyn FnOnce() -> Result<Vec<u8>, TaskError>>,
                        (Box::new(move |_bytes: &[u8]| Ok(())))
                            as Box<dyn FnOnce(&[u8]) -> Result<(), TaskError>>,
                    ))
                }),
            Some(("reverse", args)) => FsInteraction::from(args)
                .check(Some(INVENTORY_YAML), None, args.is_present("force"))
                .map_self(|fs| Ok(Context((Inventory::try_from(fs.read()?.as_slice())?, fs))))?
                .map(|(data, fs)| Ok((Scheme::try_from(&Cluster::default())?, data, fs)))?
                .map(|(scheme, _data, fs)| {
                    Ok((
                        (Box::new(move || scheme.print())) as Box<dyn FnOnce()>,
                        (Box::new(move || Ok(Vec::new())))
                            as Box<dyn FnOnce() -> Result<Vec<u8>, TaskError>>,
                        (Box::new(move |bytes: &[u8]| fs.write(bytes)))
                            as Box<dyn FnOnce(&[u8]) -> Result<(), TaskError>>,
                    ))
                }),
            _ => Err(TaskError::CommandLineError(
                CommandLineError::SubcommandError("subcommand missing".into()),
            )),
        })?
        .map_self(|Task(Context((scheme_fn, data_fn, fs_fn)))| {
            // Here should happend some magic:
            //      - print scheme
            //      - call closure with serialize into_vec
            //      - write serialized value
            info!("mapping context into final result");
            scheme_fn();
            fs_fn(data_fn()?.as_slice())?;
            Ok(())
        })
}

// Pass self to function and return new type
pub trait MapSelf<S>
where
    Self: Sized,
{
    type Target;
    type Error;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>;
}

// Map Task to new type
pub trait Functor {
    type Unwrapped;
    type Wrapped<U>: Functor;
    type Error;

    fn map<F, U>(self, func: F) -> Result<Self::Wrapped<U>, Self::Error>
    where
        F: FnOnce(Self::Unwrapped) -> Result<U, Self::Error>;
}

/// Task is a main structure whar produce all generation magic,
/// and source configuration yaml serialized in this struct.
/// After all manipulation was done, `Task` will be map_selfed into Inventory.
pub struct Task<T>(
    /// In process of building inventory, or in another scenarios
    /// all data stored inside this Type
    T,
);

impl<T, S> MapSelf<S> for Task<T> {
    type Target = S;
    type Error = TaskError;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>,
    {
        func(self)
    }
}

impl<T> Functor for Task<T> {
    type Unwrapped = T;
    type Wrapped<U> = Task<U>;
    type Error = TaskError;

    fn map<F, U>(self, func: F) -> Result<Self::Wrapped<U>, Self::Error>
    where
        F: FnOnce(Self::Unwrapped) -> Result<U, Self::Error>,
    {
        Ok(Task(func(self.0)?))
    }
}
