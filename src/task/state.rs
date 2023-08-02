use std::{
    fmt::Display,
    fs::{create_dir_all, File},
    io,
    path::PathBuf,
};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use sha256::{digest, try_digest, TrySha256Digest};
use thiserror::Error;

use crate::task::cluster::hst::view::{FG_GREEN, FG_RED};
use crate::task::{cluster::hst::v2::HostV2, flv::Failover, vars::Vars};

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    uid: String,
    kind: StateKind,
    #[serde(default)]
    instances_changes: Vec<Change>,
    #[serde(default)]
    hosts_changes: Vec<Change>,
    pub vars: Vars,
    pub hosts: HostV2,
    pub failover: Failover,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StateKind {
    Build,
    Upgrade,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl<'a> TryFrom<&'a PathBuf> for State {
    type Error = StateError;

    fn try_from(path: &'a PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(file)?)
    }
}

impl State {
    pub fn builder() -> StateBuilder {
        StateBuilder {
            uid: None,
            kind: None,
            hosts_changes: None,
            instances_changes: None,
            hosts: None,
            vars: None,
            failover: None,
        }
    }

    pub fn dump_by_path(&self, path: &str) -> Result<(), io::Error> {
        if let Some(parent) = PathBuf::from(path).parent() {
            match create_dir_all(parent) {
                Err(err) if err.kind() != io::ErrorKind::AlreadyExists => {
                    return Err(err);
                }
                _ => {}
            }
        }

        serde_json::to_writer(File::create(path)?, self).unwrap();

        Ok(())
    }

    pub fn dump_by_uid(&self, state_dir: &str) -> Result<(), io::Error> {
        self.dump_by_path(&format!("{state_dir}/{}.json", &self.uid))
    }

    pub fn from_latest(args: &ArgMatches) -> Result<Self, StateError> {
        let file = File::open(format!(
            "{}/latest.json",
            args.get_one::<String>("state-dir")
                .cloned()
                .unwrap_or(".geninstate".into())
        ))?;

        serde_json::from_reader(file).map_err(StateError::from)
    }
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("io error")]
    Io(#[from] io::Error),
    #[error("serde error")]
    Serde(#[from] serde_json::Error),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Change {
    Added(String),
    Removed(String),
}

impl Display for Change {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Change::Added(name) => write!(
                f,
                "  {}+ {name}{}",
                FG_GREEN.get_prefix(),
                FG_GREEN.get_suffix()
            ),
            Change::Removed(name) => write!(
                f,
                "  {}- {name}{}",
                FG_RED.get_prefix(),
                FG_RED.get_suffix()
            ),
        }
    }
}

pub struct StateBuilder {
    uid: Option<String>,
    kind: Option<StateKind>,
    instances_changes: Option<Vec<Change>>,
    hosts_changes: Option<Vec<Change>>,
    hosts: Option<HostV2>,
    vars: Option<Vars>,
    failover: Option<Failover>,
}

#[allow(unused)]
impl StateBuilder {
    pub fn uid<D: TrySha256Digest<Error = io::Error>>(
        self,
        parts: Vec<D>,
    ) -> Result<Self, io::Error> {
        let uid: Result<String, io::Error> =
            parts.into_iter().try_fold(String::new(), |uid, part| {
                Ok(digest(format!("{uid}{}", try_digest(part)?)))
            });

        Ok(Self {
            uid: Some(uid?),
            ..self
        })
    }

    pub fn make_build_state(self) -> Self {
        Self {
            kind: Some(StateKind::Build),
            ..self
        }
    }

    pub fn make_upgrade_state(self) -> Self {
        Self {
            kind: Some(StateKind::Upgrade),
            ..self
        }
    }

    pub fn hosts_changes(self, changes: Vec<Change>) -> Self {
        Self {
            hosts_changes: Some(changes),
            ..self
        }
    }

    pub fn instances_changes(self, changes: Vec<Change>) -> Self {
        Self {
            instances_changes: Some(changes),
            ..self
        }
    }

    pub fn hosts(self, hosts: &HostV2) -> Self {
        Self {
            hosts: Some(hosts.clone()),
            ..self
        }
    }

    pub fn vars(self, vars: &Vars) -> Self {
        Self {
            vars: Some(vars.clone()),
            ..self
        }
    }

    pub fn failover(self, failover: &Failover) -> Self {
        Self {
            failover: Some(failover.clone()),
            ..self
        }
    }

    pub fn build(self) -> Result<State, String> {
        Ok(State {
            uid: self.uid.ok_or::<String>("uid is not set".into())?,
            kind: self.kind.ok_or::<String>("kind is not set".into())?,
            instances_changes: self.instances_changes.unwrap_or_default(),
            hosts_changes: self.hosts_changes.unwrap_or_default(),
            hosts: self.hosts.ok_or::<String>("hosts is not set".into())?,
            vars: self.vars.ok_or::<String>("vars is not set".into())?,
            failover: self
                .failover
                .ok_or::<String>("failover is not set".into())?,
        })
    }
}

#[cfg(test)]
mod test;
