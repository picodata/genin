use serde::{Deserialize, Serialize};
use serde_yaml::Value;

use super::flv::{Failover, FailoverVariants, Mode, StateProvider};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
pub(in crate::task) struct Vars {
    #[serde(default = "change_me")]
    pub(in crate::task) ansible_user: String,
    #[serde(default = "change_me")]
    pub(in crate::task) ansible_password: String,
    #[serde(default = "change_me")]
    pub(in crate::task) cartridge_app_name: String,
    #[serde(default = "change_me")]
    pub(in crate::task) cartridge_cluster_cookie: String,
    #[serde(default = "change_me")]
    pub(in crate::task) cartridge_package_path: String,
    #[serde(default = "default_true")]
    pub(in crate::task) cartridge_bootstrap_vshard: bool,
    #[serde(default)]
    pub(in crate::task) cartridge_failover_params: Failover,
    #[serde(flatten, skip_serializing_if = "Value::is_null", default)]
    pub(in crate::task) another_fields: Value,
}

impl Default for Vars {
    fn default() -> Self {
        Self {
            ansible_user: "root".into(),
            ansible_password: "change_me".into(),
            cartridge_app_name: "myapp".into(),
            cartridge_cluster_cookie: "myapp-cookie".into(),
            cartridge_package_path: "/tmp/myapp.rpm".into(),
            cartridge_bootstrap_vshard: true,
            cartridge_failover_params: Failover {
                mode: Mode::Disabled,
                state_provider: StateProvider::Disabled,
                failover_variants: FailoverVariants::Disabled,
            },
            another_fields: Value::Null,
        }
    }
}

pub fn change_me() -> String {
    "CHANGE_ME".into()
}

pub fn default_true() -> bool {
    true
}

#[cfg(test)]
mod test;
