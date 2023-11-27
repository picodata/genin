use std::fmt;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::{
    flv::{Failover, FailoverVariants, Mode, StateProvider},
    AsError, TypeError, BOOL, DICT, STRING,
};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
/// Inventory vars with hardcoded important fields
/// ```yaml
/// vars:
///     ansible_user: "admin"
///     ansible_password: "'88{bvTp9Gbj<J"m"
///     cartridge_bootstrap_vshard: true
///     cartridge_app_name: "tarantool-cluster"
///     cartridge_cluster_cookie: "tarantool-cluster-cookie"
///     cartridge_package_path: "/tmp/tarantool-cluster.rpm"
///     cartridge_bootstrap_vshard: true
///     wait_cluster_has_no_issues_retries: 20
///     instance_start_retries: 20
/// ```
pub struct Vars {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ansible_user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ansible_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cartridge_app_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cartridge_cluster_cookie: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cartridge_package_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cartridge_bootstrap_vshard: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cartridge_failover_params: Option<Failover>,
    #[serde(default, flatten, skip_serializing_if = "IndexMap::is_empty")]
    pub another_fields: IndexMap<String, Value>,
}

impl Default for Vars {
    fn default() -> Self {
        Self {
            ansible_user: Some("ansible".into()),
            ansible_password: Some("ansible".into()),
            cartridge_app_name: Some("myapp".into()),
            cartridge_cluster_cookie: Some("myapp-cookie".into()),
            cartridge_package_path: Some("/tmp/myapp.rpm".into()),
            cartridge_bootstrap_vshard: Some(true),
            cartridge_failover_params: Some(Failover {
                mode: Mode::Disabled,
                state_provider: StateProvider::Disabled,
                failover_variants: FailoverVariants::Disabled,
                ..Default::default()
            }),
            another_fields: IndexMap::new(),
        }
    }
}

impl<'a> From<&'a Failover> for Vars {
    fn from(failover: &'a Failover) -> Self {
        Self {
            cartridge_failover_params: Some(failover.clone()),
            ..Vars::default()
        }
    }
}

impl Vars {
    pub fn with_failover(self, failover: Failover) -> Self {
        Self {
            cartridge_failover_params: Some(failover),
            ..self
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub struct InvalidVars {
    pub ansible_user: Value,
    pub ansible_password: Value,
    pub cartridge_app_name: Value,
    pub cartridge_cluster_cookie: Value,
    pub cartridge_package_path: Value,
    pub cartridge_bootstrap_vshard: Value,
    pub cartridge_failover_params: Value,
    #[serde(default, flatten, skip_serializing_if = "IndexMap::is_empty")]
    pub another_fields: Value,
}

impl fmt::Debug for InvalidVars {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.ansible_user {
            Value::Null => {}
            Value::String(ansible_user) => {
                formatter.write_str("\n  ansible_user: ")?;
                formatter.write_str(ansible_user.as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  ansible_user: {}",
                    self.ansible_user.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.ansible_password {
            Value::Null => {}
            Value::String(ansible_password) => {
                formatter.write_str("\n  ansible_password: ")?;
                formatter.write_str(ansible_password.as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  ansible_password: {}",
                    self.ansible_password.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.cartridge_cluster_cookie {
            Value::Null => {}
            Value::String(cartridge_cluster_cookie) => {
                formatter.write_str("\n  cartridge_cluster_cookie: ")?;
                formatter.write_str(cartridge_cluster_cookie.as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  cartridge_cluster_cookie: {}",
                    self.cartridge_cluster_cookie.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.cartridge_package_path {
            Value::Null => {}
            Value::String(cartridge_package_path) => {
                formatter.write_str("\n  cartridge_package_path: ")?;
                formatter.write_str(cartridge_package_path.as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  cartridge_package_path: {}",
                    self.cartridge_package_path.type_error(STRING).as_error()
                ))?;
            }
        }

        match &self.cartridge_bootstrap_vshard {
            Value::Null => {}
            Value::Bool(cartridge_bootstrap_vshard) => {
                formatter.write_str("\n  cartridge_bootstrap_vshard: ")?;
                formatter.write_str(cartridge_bootstrap_vshard.to_string().as_str())?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  cartridge_bootstrap_vshard: {}",
                    self.cartridge_bootstrap_vshard.type_error(BOOL).as_error()
                ))?;
            }
        }

        match &self.cartridge_failover_params {
            Value::Null => {}
            Value::Mapping(cartridge_failover_params) => {
                formatter.write_str("\n  cartridge_failover_params:")?;
                for (key, item) in cartridge_failover_params {
                    formatter
                        .write_fmt(format_args!("\n  {}: ", key.as_str().unwrap_or_default()))?;
                    print_value_recursive(formatter, "  ", item)?;
                }
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  cartridge_failover_params: {}",
                    self.cartridge_failover_params.type_error(DICT).as_error()
                ))?;
            }
        }

        match &self.another_fields {
            Value::Null => {}
            another_fields @ Value::Mapping(_) => {
                print_value_recursive(formatter, "\n    ", another_fields)?;
            }
            _ => {
                formatter.write_fmt(format_args!(
                    "\n  another_fields: {}",
                    self.another_fields.type_error(DICT).as_error()
                ))?;
            }
        }

        Ok(())
    }
}

pub fn print_value_recursive(
    formatter: &mut fmt::Formatter<'_>,
    offset: &str,
    value: &Value,
) -> fmt::Result {
    match value {
        Value::Null => {}
        Value::Sequence(seq) => {
            let new_offset = format!("{offset}  ");
            for item in seq {
                formatter.write_fmt(format_args!("{offset}- "))?;
                print_value_recursive(formatter, &new_offset, item)?;
            }
        }
        Value::Mapping(mapping) => {
            let new_offset = format!("{offset}  ");
            for (key, item) in mapping {
                formatter.write_fmt(format_args!(
                    "{}{}: ",
                    offset,
                    key.as_str().unwrap_or_default()
                ))?;
                print_value_recursive(formatter, &new_offset, item)?;
            }
        }
        Value::Bool(b) => {
            formatter.write_str(b.to_string().as_str())?;
        }
        Value::Number(n) => {
            formatter.write_str(n.to_string().as_str())?;
        }
        Value::String(s) => {
            formatter.write_str(s)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test;
