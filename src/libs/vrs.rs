use serde::{Serialize, Deserialize};
use serde_yaml::Value;

#[derive(Serialize, Deserialize, Clone)]
/// Inventory vars with hardcoded important fields
/// ```yaml
/// vars:
///     ansible_user: "admin"
///     ansible_password: "'88{bvTp9Gbj<J"m"
///     cartridge_bootstrap_vshard: true
///     cartridge_app_name: "tarantool-cluster"
///     cartridge_cluster_cookie: "tarantool-cluster-cookie"
///     wait_cluster_has_no_issues_retries: 20
///     instance_start_retries: 20
/// ```
pub struct Vars {
    #[serde(default = "change_me")]
    ansible_user: String,
    #[serde(default = "change_me")]
    ansible_password: String,
    #[serde(default = "change_me")]
    cartridge_app_name: String,
    #[serde(default = "change_me")]
    cartridge_cluster_cookie: String,
    #[serde(flatten, skip_serializing_if = "Value::is_null", default)]
    another_fields: Value,
}

impl Default for Vars {
    fn default() -> Self {
        Self {
            ansible_user: "root".into(),
            ansible_password: "change_me".into(),
            cartridge_app_name: "myapp".into(),
            cartridge_cluster_cookie: "myapp-cookie".into(),
            another_fields: Value::Null,
        }
    }
}

pub fn change_me() -> String {
    "CHANGE_ME".into()
}

