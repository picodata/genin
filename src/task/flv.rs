use std::{collections::HashMap, fmt::Display, net::IpAddr};

use clap::ArgMatches;
use log::{error, trace, warn};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use serde_yaml::Value;

use genin::libs::error::{CommandLineError, ConfigError, InternalError, TaskError};


#[derive(Serialize, Clone, Debug)]
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
            Test(HashMap<String, Mode>),
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
            Ok(FailoverHelper::Test(value)) => {
                error!("{:?}", value);
                Err(serde::de::Error::missing_field("all"))
            }
            Err(e) => {
                error!("Failover looks like {:?}", e);
                Err(e)
            }
        }
    }
}

#[derive(Serialize, PartialEq, Debug, Clone)]
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
        match v {
            "stateful" | "Stateful" | "STATEFUL" => Ok(Mode::Stateful),
            "eventual" | "Eventual" | "EVENTUAL" => Ok(Mode::Eventual),
            "disabled" | "Disabled" | "DISABLED" => Ok(Mode::Disabled),
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
        match s {
            "stateful" | "Stateful" | "STATEFUL" => Ok(Self::Stateful),
            "eventual" | "Eventual" | "EVENTUAL" => Ok(Self::Eventual),
            "disabled" | "Disabled" | "DISABLED" => Ok(Self::Disabled),
            _ => Err(InternalError::FieldDeserializationError(format!(
                "Unknown failover-mode argument {}",
                s
            ))),
        }
    }
}
impl Failover {
    pub fn to_mapping(&self) -> Value {
        match self.mode {
            Mode::Disabled => {
                let mut flv = serde_yaml::Mapping::new();
                flv.insert(
                    Value::String("mode".to_string()),
                    Value::String(format!("{:?}", self.mode.clone()).to_lowercase()),
                );
                Value::Mapping(flv)
            },
            _ => {
                let mut flv = serde_yaml::Mapping::new();
                if self.failover_variants.is_disabled() {
                    flv.insert(
                        Value::String("mode".to_string()),
                        Value::String(format!("{:?}", Mode::Disabled).to_lowercase())
                    );
                } else {
                    flv.insert(
                        Value::String("mode".to_string()),
                        Value::String(format!("{:?}", self.mode.clone()).to_lowercase())
                    );
                    
                    flv.insert(
                        Value::String("state_provider".to_string()),
                        Value::String(format!("{:?}", self.state_provider.clone()).to_lowercase())
                    );
                    
                    //get tuple("stateboard_params" or "etcd2_params", Value::Mapping)
                    let params = self.failover_variants.clone().to_mapping();
                    flv.insert(params.0, params.1);
                }

                Value::Mapping(flv)
            }
       }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
        match s {
            "etcd2" | "ETCD2" | "Etcd2" => Ok(Self::ETCD2),
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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
 
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "stateboard" | "Stateboard" | "STATEBOARD" => Ok(FailoverVariants::StateboardVariant(
                StateboardParams::default(),
            )),
            "etcd2" | "Etcd2" | "ETCD2" => {
                Ok(FailoverVariants::ETCD2Variant(ETCD2Params::default()))
            }
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

    pub fn to_mapping(self) -> (Value, Value) {
        match self {
            Self::StateboardVariant(params) => {
                let mut map_params = serde_yaml::Mapping::new();
                map_params.insert(
                    Value::String("uri".to_string()), 
                    Value::String(format!("{}", params.uri)),
                );
                map_params.insert(
                    Value::String("password".to_string()), 
                    Value::String(params.password)
                );
            
                (Value::String("stateboard_params".to_string()), Value::Mapping(map_params))
            },
            Self::ETCD2Variant(params) => {
                let mut seq_endpoints = serde_yaml::Sequence::new();
                params
                    .endpoints
                    .into_iter()
                    .for_each(|endp| {
                        let mut temp = serde_yaml::Mapping::new();
                        temp.insert(
                            Value::String("url".to_string()),
                            Value::String(format!("{}://{}", endp.protocol, endp.url)),
                        );
                        seq_endpoints.push(Value::String(format!("{}://{}", endp.protocol, endp.url)));
                    });

                let mut map_params = serde_yaml::Mapping::new();
                map_params.insert(
                    Value::String("prefix".to_string()), 
                    Value::String(params.prefix)
                );
                map_params.insert(
                    Value::String("lock_delay".to_string()), 
                    Value::Number(serde_yaml::Number::from(params.lock_delay))
                );
                map_params.insert(
                    Value::String("endpoints".to_string()), 
                    Value::Sequence(seq_endpoints)
                );
                
                if !params.username.is_empty() {
                    map_params.insert(Value::String("username".to_string()), Value::String(params.username));
                }
                if !params.password.is_empty() {
                    map_params.insert(Value::String("password".to_string()), Value::String(params.password));
                }

                (Value::String("etcd2_params".to_string()), Value::Mapping(map_params))
            },
            _ => (Value::Null, Value::Null)
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(in crate::task) struct StateboardParams {
    pub(in crate::task) uri: Uri,
    pub(in crate::task) password: String,
}

impl Default for StateboardParams {
    fn default() -> Self {
        StateboardParams {
            uri: Uri {
                ip: "192.168.16.11".parse().unwrap(),
                port: 4401,
            },
            password: "change_me".into(),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone)]
pub(in crate::task) struct Uri {
    pub(in crate::task) ip: IpAddr,
    pub(in crate::task) port: u16,
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {   
        write!(f, "{}:{}", self.ip, self.port)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(in crate::task) struct Url {
    pub(in crate::task) ip: IpAddr,
    pub(in crate::task) port: u16,
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}

struct UrlVisitor;

impl<'de> Visitor<'de> for UrlVisitor {
    type Value = Url;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(formatter, "expecting `url` like 192.168.16.11:3030")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let splitted = s.split(':').collect::<Vec<&str>>();
        if let (Some(ip), Some(port)) = (splitted.get(0), splitted.get(1)) {
            return Ok(Url {
                ip: ip.parse().map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Other(s), &self)
                })?,
                port: port.parse::<u16>().map_err(|_| {
                    serde::de::Error::invalid_value(serde::de::Unexpected::Other(s), &self)
                })?,
            });
        }
        Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Other(s),
            &self,
        ))
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(UrlVisitor)
    }
}

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(format!("{}:{}", self.ip, self.port).as_str())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

#[derive(PartialEq, Debug, Clone)]
pub(in crate::task) struct UrlWithProtocol {
    protocol: Protocol,
    url: Url,
}

impl<'a> TryFrom<&'a str> for UrlWithProtocol {
    type Error = TaskError;
    
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let splitted = value.split("://").collect::<Vec<&str>>();
        match (splitted.get(0), splitted.get(1)) {
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

#[derive(PartialEq, Debug, Clone)]
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

#[cfg(test)]
mod test;
