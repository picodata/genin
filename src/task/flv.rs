use std::{fmt::Display, net::IpAddr};

use clap::ArgMatches;
use log::{error, trace, warn};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::error::{CommandLineError, ConfigError, InternalError, TaskError};

pub(in crate::task) const DEFAULT_STB_PORT: u16 = 4401;

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
    type Error = TaskError;
    fn try_from(args: &ArgMatches) -> Result<Self, Self::Error> {
        match (
            args.value_of("failover-mode"),
            args.value_of("failover-state-provider"),
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
            _ => Err(TaskError::CommandLineError(CommandLineError::OptionError(
                "unknown failover options".into(),
            ))),
        }
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
pub(in crate::task) enum Mode {
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
    type Error = InternalError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        trace!("failover mode: {}", s);
        match s.to_lowercase().as_str() {
            "stateful" => Ok(Self::Stateful),
            "eventual" => Ok(Self::Eventual),
            "disabled" => Ok(Self::Disabled),
            _ => Err(InternalError::FieldDeserializationError(format!(
                "Unknown failover-mode argument {}",
                s
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(in crate::task) enum StateProvider {
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
    type Error = InternalError;

    fn try_from(s: &'s str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "etcd2" => Ok(Self::ETCD2),
            "stateboard" => Ok(Self::Stateboard),
            invalid => Err(InternalError::FieldDeserializationError(format!(
                "Unknown failover-state-provider argument {}",
                invalid,
            ))),
        }
    }
}

impl StateProvider {
    fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(in crate::task) enum FailoverVariants {
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
    type Error = TaskError;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "stateboard" => Ok(FailoverVariants::StateboardVariant(
                StateboardParams::default(),
            )),
            "etcd2" => Ok(FailoverVariants::ETCD2Variant(ETCD2Params::default())),
            invalid => Err(TaskError::CommandLineError(
                CommandLineError::SubcommandError(format!(
                    "invalid value `failover-state-provider` `{}`",
                    invalid
                )),
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
    pub(in crate::task) fn is_disabled(&self) -> bool {
        matches!(self, Self::Disabled)
    }

    pub(in crate::task) fn is_stateboard(&self) -> bool {
        matches!(self, Self::StateboardVariant(_))
    }

    pub(in crate::task) fn is_etcd2(&self) -> bool {
        matches!(self, Self::ETCD2Variant(_))
    }

    pub(in crate::task) fn with_mut_stateboard<F: FnMut(&StateboardParams)>(&self, mut func: F) {
        if let FailoverVariants::StateboardVariant(stb) = self {
            func(stb);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub(in crate::task) struct StateboardParams {
    #[serde(rename = "uri")]
    pub(in crate::task) url: Url,
    pub(in crate::task) password: String,
}

impl Default for StateboardParams {
    fn default() -> Self {
        StateboardParams {
            url: Url {
                ip: "192.168.16.1".parse().unwrap(),
                port: Some(DEFAULT_STB_PORT),
            },
            password: "change_me".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::task) struct Url {
    pub(in crate::task) ip: IpAddr,
    pub(in crate::task) port: Option<u16>,
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}:{}", self.ip, port)
        } else {
            write!(f, "{}", self.ip)
        }
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct UrlHelper(String);

        impl serde::de::Expected for UrlHelper {
            fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "expected ip like:\n  uri:    \nip: 0.0.0.0:4401")
            }
        }

        if let Ok(UrlHelper(raw_ip)) = UrlHelper::deserialize(deserializer) {
            let ip_port_urn = raw_ip.split(':').collect::<Vec<&str>>();

            if let (Some(ip), Some(port)) = (ip_port_urn.first(), ip_port_urn.last()) {
                return Ok(Url {
                    ip: ip.parse().map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Other(ip),
                            &UrlHelper("0.0.0.0:4401".to_string()),
                        )
                    })?,
                    port: Some(port.parse::<u16>().map_err(|_| {
                        serde::de::Error::invalid_value(
                            serde::de::Unexpected::Other(port),
                            &UrlHelper("0.0.0.0:4401".to_string()),
                        )
                    })?),
                });
            }
        }
        Err(serde::de::Error::custom(
            "Error then deserializing uri field in stateboard_params".to_string(),
        ))
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(port) = self.port {
            serializer.serialize_str(format!("{}:{}", self.ip, port).as_str())
        } else {
            serializer.serialize_str(format!("{}:{}", self.ip, DEFAULT_STB_PORT).as_str())
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub(in crate::task) struct Uri {
    pub(in crate::task) ip: IpAddr,
    #[serde(default = "default_stb_port", skip_serializing_if = "Option::is_none")]
    pub(in crate::task) port: Option<u16>,
    pub(in crate::task) urn: String,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(port) = self.port {
            write!(f, "{}:{}/{}", self.ip, port, self.urn)
        } else {
            write!(f, "{}/{}", self.ip, self.urn)
        }
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
pub(in crate::task) struct ETCD2Params {
    pub(in crate::task) prefix: String,
    #[serde(default)]
    pub(in crate::task) lock_delay: usize,
    pub(in crate::task) endpoints: Vec<UrlWithProtocol>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(in crate::task) username: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(in crate::task) password: String,
}

impl Default for ETCD2Params {
    fn default() -> Self {
        Self {
            prefix: "cartridge/myapp".into(),
            lock_delay: 30,
            endpoints: vec![
                UrlWithProtocol::try_from("http://192.168.16.11:5699").unwrap(),
                UrlWithProtocol::try_from("http://192.168.16.12:5699").unwrap(),
            ],
            username: "change_me".into(),
            password: "change_me".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::task) struct UrlWithProtocol {
    protocol: Protocol,
    url: Url,
}

impl<'a> TryFrom<&'a str> for UrlWithProtocol {
    type Error = TaskError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let splitted = value.split("://").collect::<Vec<&str>>();

        match (splitted.first(), splitted.last()) {
            (Some(&"http"), Some(&url)) => Ok(Self {
                protocol: Protocol::Http,
                url: serde_yaml::from_str(url).map_err(|e| {
                    TaskError::InternalError(InternalError::StructDeserializationError(
                        e.to_string(),
                    ))
                })?,
            }),
            (Some(&"https"), Some(&url)) => Ok(Self {
                protocol: Protocol::Https,
                url: serde_yaml::from_str(url).map_err(|e| {
                    TaskError::InternalError(InternalError::StructDeserializationError(
                        e.to_string(),
                    ))
                })?,
            }),
            _ => Err(TaskError::ConfigError(ConfigError::FileContentError(
                "Error while parsing ETCD2 url".to_string(),
            ))),
        }
    }
}

struct UrlWithProtocolVisior;

impl<'de> Visitor<'de> for UrlWithProtocolVisior {
    type Value = UrlWithProtocol;

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
        UrlWithProtocol::try_from(s).map_err(|err| {
            serde::de::Error::unknown_field(err.to_string().as_str(), &["endpoints"])
        })
    }
}

impl<'de> Deserialize<'de> for UrlWithProtocol {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UrlWithProtocolVisior)
    }
}

impl Serialize for UrlWithProtocol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(format!("{}://{}", self.protocol, self.url).as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::task) enum Protocol {
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

pub(in crate::task) fn default_stb_port() -> Option<u16> {
    Some(DEFAULT_STB_PORT)
}

#[cfg(test)]
mod test;
