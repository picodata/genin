use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::flv::{Failover, FailoverVariants, Mode, StateProvider};

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

#[cfg(test)]
mod test;
