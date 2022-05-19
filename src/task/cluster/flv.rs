use std::{fmt::Display, net::IpAddr};

use clap::ArgMatches;
use serde::{Deserialize, Serialize};

use genin::libs::error::InternalError;

#[derive(Serialize, Deserialize)]
/// Failover enum
/// ```yaml
/// failover:
///     mode: stateful
///     state_provider: stateboard
///     stateboard_params:
///         uri: "10.99.3.100:4001"
///         password: "vG?-GG!4sxV8q5:f"
/// ```
pub(in crate::task) struct Failover {
    pub(in crate::task) mode: Mode,
    #[serde(skip_serializing_if = "StateProvider::is_disabled")]
    pub(in crate::task) state_provider: StateProvider,
    #[serde(skip_serializing_if = "FailoverVariants::is_disabled", flatten)]
    pub(in crate::task) failover_variants: FailoverVariants,
}

impl Default for Failover {
    fn default() -> Self {
        Self {
            mode: Mode::Disabled,
            state_provider: StateProvider::Disabled,
            failover_variants: FailoverVariants::Disabled,
        }
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Failover {
    type Error = InternalError;
    fn try_from(args: &ArgMatches) -> Result<Self, Self::Error> {
        let args @ (mode, _) = (
            args.value_of("failover-mode").unwrap(),
            args.value_of("failover-state-provider").unwrap(),
        );
        Ok(Self {
            mode: Mode::try_from(mode)?,
            state_provider: StateProvider::try_from(args)?,
            failover_variants: FailoverVariants::from(args),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) enum Mode {
    #[serde(rename = "stateful")]
    Stateful,
    #[serde(rename = "eventual")]
    Eventual,
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Stateful
    }
}

impl<'s> TryFrom<&'s str> for Mode {
    type Error = InternalError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        println!("mode {}", s);
        match s {
            "stateful" => Ok(Self::Stateful),
            "eventual" => Ok(Self::Eventual),
            "disabled" => Ok(Self::Disabled),
            _ => Err(InternalError::Undefined(format!(
                "Unknown failover-mode argument {}",
                s
            ))),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) enum StateProvider {
    #[serde(rename = "stateboard")]
    Stateboard,
    #[serde(rename = "etcd2")]
    ETCD2,
    Disabled,
}

impl<'s> TryFrom<(&'s str, &'s str)> for StateProvider {
    type Error = InternalError;

    fn try_from(s: (&'s str, &'s str)) -> Result<Self, Self::Error> {
        println!("state-mode {:?}", s);
        match s {
            ("stateful", "etcd2") => Ok(Self::ETCD2),
            ("stateful", "stateboard") => Ok(Self::Stateboard),
            ("disabled", _) => Ok(Self::Disabled),
            (a, "etcd2") | (a, "stateboard") => Err(InternalError::Undefined(format!(
                "failover-state-provider: 'etcd' and 'stateboard' \
                        uncompatible with state-mode '{}'",
                a
            ))),
            (_, a) => Err(InternalError::Undefined(format!(
                "Unknown failover-state-provider argument {}",
                a,
            ))),
        }
    }
}

impl StateProvider {
    fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) enum FailoverVariants {
    #[serde(rename = "stateboard_params")]
    StateboardVariant(StateboardParams),
    #[serde(rename = "ectd2")]
    ETCD2Variant, //TODO
    Disabled,
}

impl<'s> From<(&'s str, &'s str)> for FailoverVariants {
    fn from(s: (&'s str, &'s str)) -> Self {
        match s {
            ("stateful", "etcd2") => Self::ETCD2Variant,
            ("stateful", "stateboard") => Self::StateboardVariant(StateboardParams {
                uri: Uri {
                    ip: "192.168.16.1".parse().unwrap(),
                    port: 4401,
                },
                password: "change_me".into(),
            }),
            _ => Self::Disabled,
        }
    }
}

impl Display for FailoverVariants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailoverVariants::ETCD2Variant => write!(f, "ETCD2"),
            FailoverVariants::StateboardVariant(_) => write!(f, "stateboard"),
            _ => write!(f, "disabled"),
        }
    }
}

#[allow(unused)]
impl FailoverVariants {
    pub(in crate::task) fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub(in crate::task) fn is_stateboard(&self) -> bool {
        matches!(self, Self::ETCD2Variant)
    }

    pub(in crate::task) fn with_mut_stateboard<F: FnMut(&StateboardParams)>(&self, mut func: F) {
        if let FailoverVariants::StateboardVariant(stb) = self {
            func(stb);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct StateboardParams {
    pub(in crate::task) uri: Uri,
    pub(in crate::task) password: String,
}

#[derive(Serialize, Deserialize)]
pub(in crate::task) struct Uri {
    pub(in crate::task) ip: IpAddr,
    pub(in crate::task) port: u16,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}
