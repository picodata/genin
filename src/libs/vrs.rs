use serde::{Serialize, Deserialize};
use serde_yaml::Value;
use std::collections::HashMap;

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

enum VarsField {
    AnsibleUser,
    AnsiblePassword,
    CartridgeAppName,
    CartridgeClusterCookie,
    AnotherFields
}

impl VarsField {
    fn as_str(&self) -> String {
        match self {
            VarsField::AnsibleUser => "ansible_user".to_string(),
            VarsField::AnsiblePassword => "ansible_password".to_string(),
            VarsField::CartridgeAppName => "cartridge_app_name".to_string(),
            VarsField::CartridgeClusterCookie => "cartridge_cluster_cookie".to_string(),
            VarsField::AnotherFields => "another_fields".to_string(),
        }
    }
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

impl Vars {
    pub fn get_hashmap(&self) -> HashMap<String, Value> {
        let mut vars: HashMap<String, Value> = HashMap::from([
            (VarsField::AnsibleUser.as_str(), Value::String(self.ansible_user.clone())),
            (VarsField::AnsiblePassword.as_str(), Value::String(self.ansible_password.clone())),
            (VarsField::CartridgeAppName.as_str(), Value::String(self.cartridge_app_name.clone())),
            (VarsField::CartridgeClusterCookie.as_str(), Value::String(self.cartridge_cluster_cookie.clone()))
        ]);
        
        match self.another_fields.clone().as_mapping() {
            Some(fields_mapping) => {
                fields_mapping.into_iter()
                    .for_each( |var| {
                        vars.insert(var.0.as_str().unwrap().to_string(), var.1.clone());
                    });
            },
            None => {}
        }
        
        vars
    }
}

pub fn change_me() -> String {
    "CHANGE_ME".into()
}