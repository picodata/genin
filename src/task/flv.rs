use clap::ArgMatches;
use core::fmt;
use log::{debug, error, warn};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use serde_yaml::Value;
use std::{fmt::Display, net::SocketAddr};
use thiserror::Error;

use crate::{
    error::{GeninError, GeninErrorKind},
    DEFAULT_STATEBOARD_PORT,
};

use super::{cluster::hst::v2::Address, AsError, TypeError, LIST, NUMBER, STRING};

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
/// Failover enum
/// ```yaml
/// failover:
///     mode: stateful
///     state_provider: stateboard
///     stateboard_params:
///         uri: "10.99.3.100:4001"
///         password: "vG?-GG!4sxV8q5:f"
/// ```
pub struct Failover {
    pub mode: Mode,
    #[serde(skip_serializing_if = "StateProvider::is_disabled")]
    pub state_provider: StateProvider,
    #[serde(skip_serializing_if = "FailoverVariants::is_disabled", flatten)]
    pub failover_variants: FailoverVariants,
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
    type Error = FailoverError;

    fn try_from(args: &ArgMatches) -> Result<Self, Self::Error> {
        match (
            args.get_one::<String>("failover-mode").map(|s| s.as_str()),
            args.get_one::<String>("failover-state-provider")
                .map(|s| s.as_str()),
        ) {
            (Some("disabled"), _) => Ok(Self {
                mode: Mode::Disabled,
                state_provider: StateProvider::Disabled,
                failover_variants: FailoverVariants::Disabled,
            }),
            (_, Some("disabled")) => {
                warn!(
                    "`failover-state-provider` passed as `disabled`, but `failover-mode` \
                    has incorect value. please use `failover-mode` `disabled` intead of \
                    disabling vi `failover-state-provider`"
                );
                Ok(Self {
                    mode: Mode::Disabled,
                    state_provider: StateProvider::Disabled,
                    failover_variants: FailoverVariants::Disabled,
                })
            }
            (Some("eventual"), _) => Ok(Self {
                mode: Mode::Eventual,
                state_provider: StateProvider::Disabled,
                failover_variants: FailoverVariants::Disabled,
            }),
            (Some("stateful"), Some(arg)) => Ok(Self {
                mode: Mode::Stateful,
                state_provider: StateProvider::try_from(arg)?,
                failover_variants: FailoverVariants::try_from(arg)?,
            }),
            _ => Err(FailoverError::InvalidParams(
                "Unknown failover options".into(),
            )),
        }
    }
}

#[derive(Error, Debug)]
pub enum FailoverError {
    #[error("invalid parameters {0}")]
    InvalidParams(String),
}

impl From<String> for FailoverError {
    fn from(value: String) -> Self {
        Self::InvalidParams(value)
    }
}

impl<'de> Deserialize<'de> for Failover {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum FailoverHelper {
            Enabled {
                mode: Mode,
                state_provider: StateProvider,
                #[serde(flatten)]
                failover_variants: FailoverVariants,
            },
            Disabled {
                mode: Mode,
            },
        }

        match FailoverHelper::deserialize(deserializer) {
            Ok(FailoverHelper::Enabled {
                mode,
                state_provider,
                failover_variants,
            }) => Ok(Self {
                mode,
                state_provider,
                failover_variants,
            }),
            Ok(FailoverHelper::Disabled { mode }) => Ok(Self {
                mode,
                state_provider: StateProvider::Disabled,
                failover_variants: FailoverVariants::Disabled,
            }),
            Err(e) => {
                error!("Failover looks like {:?}", e);
                Err(e)
            }
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    #[serde(rename = "stateful")]
    Stateful,
    #[serde(rename = "eventual")]
    Eventual,
    #[serde(rename = "disabled")]
    Disabled,
}

struct ModeVisitor;

impl<'de> Visitor<'de> for ModeVisitor {
    type Value = Mode;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "expecting one of 'mode' variant")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v.to_lowercase().as_str() {
            "stateful" => Ok(Mode::Stateful),
            "eventual" => Ok(Mode::Eventual),
            "disabled" => Ok(Mode::Disabled),
            _ => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Other(v),
                &self,
            )),
        }
    }
}

impl<'de> Deserialize<'de> for Mode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ModeVisitor)
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Stateful
    }
}

impl<'s> TryFrom<&'s str> for Mode {
    type Error = GeninError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        debug!("failover mode: {}", s);
        match s.to_lowercase().as_str() {
            "stateful" => Ok(Self::Stateful),
            "eventual" => Ok(Self::Eventual),
            "disabled" => Ok(Self::Disabled),
            _ => Err(GeninError::new(
                GeninErrorKind::ArgsError,
                format!("Unknown failover-mode argument {}", s).as_str(),
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum StateProvider {
    #[serde(rename = "stateboard")]
    Stateboard,
    #[serde(rename = "etcd2")]
    ETCD2,
    #[serde(rename = "disabled")]
    Disabled,
}

impl Default for StateProvider {
    fn default() -> Self {
        Self::Disabled
    }
}

impl<'s> TryFrom<&'s str> for StateProvider {
    type Error = String;

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "etcd2" => Ok(Self::ETCD2),
            "stateboard" => Ok(Self::Stateboard),
            invalid => Err(format!(
                "Unknown failover-state-provider argument {}",
                invalid
            )),
        }
    }
}

impl StateProvider {
    fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum FailoverVariants {
    #[serde(rename = "stateboard_params")]
    StateboardVariant(StateboardParams),
    #[serde(rename = "etcd2_params")]
    ETCD2Variant(ETCD2Params),
    Disabled,
}

impl Default for FailoverVariants {
    fn default() -> Self {
        Self::Disabled
    }
}

impl<'a> TryFrom<&'a str> for FailoverVariants {
    type Error = String;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "stateboard" => Ok(FailoverVariants::StateboardVariant(
                StateboardParams::default(),
            )),
            "etcd2" => Ok(FailoverVariants::ETCD2Variant(ETCD2Params::default())),
            invalid => Err(format!(
                "invalid value `failover-state-provider` `{}`",
                invalid
            )),
        }
    }
}

impl Display for FailoverVariants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailoverVariants::ETCD2Variant(_) => write!(f, "etcd2"),
            FailoverVariants::StateboardVariant(_) => write!(f, "stateboard"),
            _ => write!(f, "disabled"),
        }
    }
}

#[allow(unused)]
impl FailoverVariants {
    pub fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub fn is_stateboard(&self) -> bool {
        matches!(self, Self::StateboardVariant(_))
    }

    pub fn is_etcd2(&self) -> bool {
        matches!(self, Self::ETCD2Variant(_))
    }

    pub fn with_mut_stateboard<F: FnMut(&StateboardParams)>(&self, mut func: F) {
        if let FailoverVariants::StateboardVariant(stb) = self {
            func(stb);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct StateboardParams {
    pub uri: Uri,
    pub password: String,
}

impl Default for StateboardParams {
    fn default() -> Self {
        StateboardParams {
            uri: Uri {
                address: Address::Ip("192.168.16.11".parse().unwrap()),
                port: DEFAULT_STATEBOARD_PORT,
            },
            password: "password".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri {
    pub address: Address,
    pub port: u16,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum UriHelper {
            Uri(String),
            IpPort { ip: Address, port: u16 },
        }

        UriHelper::deserialize(deserializer).map(|uri_helper| match uri_helper {
            UriHelper::Uri(uri_str) => {
                if let Ok(socket_addr) = uri_str.parse::<SocketAddr>() {
                    debug!("uri {} looks like socket address", socket_addr);
                    return Ok(Uri {
                        address: Address::Ip(socket_addr.ip()),
                        port: socket_addr.port(),
                    });
                }
                debug!("uri {} looks like string host:port", uri_str);
                let parts = uri_str
                    .rsplit_once(':')
                    .ok_or(serde::de::Error::custom(format!(
                        "uri {} does not match the pattern [host][:port]",
                        uri_str
                    )))?;
                Ok(Uri {
                    address: Address::Uri(parts.0.into()),
                    port: parts.1.parse::<u16>().map_err(|err| {
                        serde::de::Error::custom(format!(
                            "failed to parse port {}: {}",
                            parts.1, err
                        ))
                    })?,
                })
            }
            UriHelper::IpPort { ip: address, port } => Ok(Uri { address, port }),
        })?
    }
}

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(format!("{}:{}", self.address, self.port).as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Cluster failover variant for etcd2 statefull failover mode
/// etcd2_params:
///     prefix: cartridge/myapp
///     lock_delay: 30
///     endpoints: [ "http://192.168.16.11:5699", "http://192.168.16.1::5699" ]
///     username: ansible
///     password: ansible
pub struct ETCD2Params {
    pub prefix: String,
    #[serde(default)]
    pub lock_delay: usize,
    pub endpoints: Vec<UriWithProtocol>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub username: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub password: String,
}

impl Default for ETCD2Params {
    fn default() -> Self {
        Self {
            prefix: "cartridge/myapp".into(),
            lock_delay: 30,
            endpoints: vec![
                UriWithProtocol::try_from("http://192.168.16.11:5699").unwrap(),
                UriWithProtocol::try_from("http://192.168.16.12:5699").unwrap(),
            ],
            username: "username".into(),
            password: "password".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UriWithProtocol {
    protocol: Protocol,
    url: Uri,
}

impl<'a> TryFrom<&'a str> for UriWithProtocol {
    type Error = GeninError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let splitted = value.split("://").collect::<Vec<&str>>();

        match (splitted.first(), splitted.last()) {
            (Some(&"http"), Some(&url)) => Ok(Self {
                protocol: Protocol::Http,
                url: serde_yaml::from_str(url).map_err(|error| {
                    //TODO: replace whith rich types
                    GeninError::new(GeninErrorKind::Deserialization, error)
                })?,
            }),
            (Some(&"https"), Some(&url)) => Ok(Self {
                protocol: Protocol::Https,
                url: serde_yaml::from_str(url)
                    .map_err(|error| GeninError::new(GeninErrorKind::Deserialization, error))?,
            }),
            _ => Err(GeninError::new(
                GeninErrorKind::Deserialization,
                "Error while parsing ETCD2 url",
            )),
        }
    }
}

impl Display for UriWithProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", self.protocol, self.url)
    }
}

struct UriWithProtocolVisior;

impl<'de> Visitor<'de> for UriWithProtocolVisior {
    type Value = UriWithProtocol;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "expecting `url started with protocol` like `http://localhost:8080`"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        UriWithProtocol::try_from(s).map_err(|err| {
            serde::de::Error::unknown_field(err.to_string().as_str(), &["endpoints"])
        })
    }
}

impl<'de> Deserialize<'de> for UriWithProtocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UriWithProtocolVisior)
    }
}

impl Serialize for UriWithProtocol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(format!("{}://{}", self.protocol, self.url).as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Protocol {
    Http,
    Https,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::Https => write!(f, "https"),
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidFailover {
    mode: Value,
    state_provider: Value,
    etcd2_params: Value,
    stateboard_params: Value,
}

impl fmt::Debug for InvalidFailover {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\n  mode: ")?;
        match self {
            InvalidFailover {
                mode: Value::Null, ..
            } => {
                formatter.write_str("Missing field 'mode'".as_error().as_str())?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider,
                etcd2_params,
                stateboard_params,
            } if mode.eq("disabled") || mode.eq("eventual") => {
                formatter.write_str(mode)?;
                if !state_provider.is_null() {
                    formatter.write_str("\n  state_provider: ")?;
                    formatter.write_str(
                        format!("The value cannot be set because the mode is '{mode}'")
                            .as_error()
                            .as_str(),
                    )?;
                }

                if !etcd2_params.is_null() {
                    formatter.write_str("\n  etcd2_params: ")?;
                    formatter.write_str(
                        format!("The value cannot be set because the mode is '{mode}'")
                            .as_error()
                            .as_str(),
                    )?;
                }

                if !stateboard_params.is_null() {
                    formatter.write_str("\n  stateboard_params: ")?;
                    formatter.write_str(
                        format!("The value cannot be set because the mode is '{mode}'")
                            .as_error()
                            .as_str(),
                    )?;
                }
            }
            InvalidFailover {
                mode: Value::String(mode),
                ..
            } if !mode.eq("stateful") => {
                formatter.write_str(
                    format!("Unknown failover mode '{}'", mode)
                        .as_error()
                        .as_str(),
                )?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::Null,
                ..
            } => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: ")?;
                formatter.write_str(
                    "The value cannot be null because mode is stateful"
                        .as_error()
                        .as_str(),
                )?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                etcd2_params: Value::Null,
                stateboard_params: Value::Null,
                ..
            } => {
                formatter.write_str(
                    format!(
                        "The Mode cannot be '{mode}' because no \
                    parameters are set for any of the failover providers"
                    )
                    .as_error()
                    .as_str(),
                )?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                etcd2_params,
                stateboard_params,
            } if !etcd2_params.is_null() && !stateboard_params.is_null() => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: ")?;
                formatter.write_str(state_provider)?;
                formatter.write_str("\n  etcd2_params: ")?;
                formatter.write_str(
                    "It is forbidden to specify both types of stateful failovers at the same time"
                        .as_error()
                        .as_str(),
                )?;
                formatter.write_str("\n  stateboard_params: ")?;
                formatter.write_str(
                    "It is forbidden to specify both types of stateful failovers at the same time"
                        .as_error()
                        .as_str(),
                )?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                etcd2_params: Value::Null,
                stateboard_params,
            } if state_provider.eq("stateboard") => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: stateboard")?;
                formatter.write_str("\n  stateboard_params: ")?;
                formatter.write_fmt(format_args!(
                    "{:?}",
                    serde_yaml::from_value::<InvalidStateboardParams>(stateboard_params.clone())
                        .unwrap()
                ))?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                ..
            } if state_provider.eq("stateboard") => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: stateboard")?;
                formatter.write_str("\n  stateboard_params: ")?;
                formatter.write_str("Missing field 'stateboard_params'".as_error().as_str())?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                etcd2_params,
                stateboard_params: Value::Null,
            } if state_provider.eq("etcd2") => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: etcd2")?;
                formatter.write_str("\n  etcd2_params: ")?;
                formatter.write_fmt(format_args!(
                    "{:?}",
                    serde_yaml::from_value::<InvalidETCD2>(etcd2_params.clone()).unwrap()
                ))?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                ..
            } if state_provider.eq("etcd2") => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: stateboard")?;
                formatter.write_str("\n  etcd2_params: ")?;
                formatter.write_str("Missing field 'etcd2_params'".as_error().as_str())?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                state_provider: Value::String(state_provider),
                ..
            } => {
                formatter.write_str(mode)?;
                formatter.write_str("\n state_provider: ")?;
                formatter.write_str(
                    format!("Unknown state provider '{}'", state_provider)
                        .as_error()
                        .as_str(),
                )?;
            }
            InvalidFailover {
                mode: Value::String(mode),
                ..
            } => {
                formatter.write_str(mode)?;
                formatter.write_str("\n  state_provider: ")?;
                formatter.write_str("Value must be a String".as_error().as_str())?;
            }
            _ => {
                unreachable!()
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidETCD2 {
    prefix: Value,
    lock_delay: Value,
    endpoints: Value,
    username: Value,
    password: Value,
}

impl fmt::Debug for InvalidETCD2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.prefix {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "\n    prefix: {}",
                    "Missing field 'prefix'".as_error()
                ))?;
            }
            Value::String(prefix) => {
                formatter.write_str("\n    prefix: ")?;
                formatter.write_str(prefix)?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    prefix: {}",
                    self.prefix.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.lock_delay {
            Value::Number(lock_delay) => {
                formatter.write_str("\n    lock_delay: ")?;
                formatter.write_str(lock_delay.to_string().as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    lock_delay: {}",
                    self.lock_delay.type_error(NUMBER).as_error()
                ))?;
            }
        }

        match &self.endpoints {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "\n    endpoints: {}",
                    "Missing field 'endpoints'".as_error().as_str()
                ))?;
            }
            Value::Sequence(seq) => {
                formatter.write_str("\n    endpoints:")?;
                for endpoint in seq {
                    if let Ok(uri) = serde_yaml::from_value::<UriWithProtocol>(endpoint.clone()) {
                        formatter.write_str(format!("\n      - {uri}").as_str())?;
                    } else {
                        formatter
                            .write_str(format!("\n      - {}", "InvalidUri".as_error()).as_str())?;
                    }
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    endpoints: {}",
                    self.endpoints.type_error(LIST).as_error()
                ))?;
            }
        }

        match &self.username {
            Value::String(username) => {
                formatter.write_str("\n    username: ")?;
                formatter.write_str(username)?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    username: {}",
                    self.username.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.username {
            Value::String(password) => {
                formatter.write_str("\n    password: ")?;
                formatter.write_str(password)?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n    password: {}",
                    self.username.type_error(STRING).as_error()
                ))?;
            }
        }
        Ok(())
    }
}

#[derive(Deserialize, Default)]
pub struct InvalidStateboardParams {
    uri: Value,
    password: Value,
}

impl fmt::Debug for InvalidStateboardParams {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\n      uri: ")?;
        if let Ok(uri) = serde_yaml::from_value::<Uri>(self.uri.clone()) {
            formatter.write_str(uri.to_string().as_str())?;
        } else {
            formatter.write_str("Invalid Uri".as_error().as_str())?;
        }

        match &self.password {
            Value::Null => {
                formatter.write_fmt(format_args!(
                    "\n      password: {}",
                    "Missing field 'password'".as_error()
                ))?;
            }
            Value::String(password) => {
                formatter.write_str("\n      password: ")?;
                formatter.write_str(password)?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n      password: {}",
                    self.password.type_error(STRING).as_error()
                ))?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
