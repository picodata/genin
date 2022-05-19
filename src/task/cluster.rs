mod flv;
pub(in crate::task) mod fs;
pub(in crate::task) mod hosts;
pub(in crate::task) mod scheme;

use clap::ArgMatches;
use genin::libs::{
    error::{ConfigError, TaskError},
    hst::Hosts,
    ins::Instances,
    vrs::Vars,
};
use serde::{Deserialize, Serialize};

use flv::Failover;

use super::{Functor, MapSelf};

#[derive(Default, Serialize, Deserialize)]
pub(in crate::task) struct Cluster {
    /// Array of instances in free order
    /// ```yaml
    /// instances:
    ///     - name: "catalogue"
    ///       type: "storage"
    ///       count: 1
    ///       replicas: 2
    ///       weight: 10
    /// ```
    pub(in crate::task) instances: Instances,
    /// Array or arrays with hosts parameters
    /// ```yaml
    /// hosts:
    ///     - name: kavkaz
    ///       type: region
    ///       distance: 10
    ///       ports:
    ///         http: 8091
    ///         binary: 3031
    ///       hosts:
    ///         - name: dc-1
    ///           type: datacenter
    ///           hosts:
    ///             - name: server-1
    ///               ip: 10.20.3.100
    ///         - name: dc-2
    ///           type: datacenter
    ///           hosts:
    ///             - name: server-1
    ///               ip: 10.20.4.100
    ///     - name: moscow
    ///       type: region
    ///       distance: 20
    ///       hosts:
    ///         - name: dc-3
    ///           type: datacenter
    ///           ports:
    ///             http: 8091
    ///             binary: 3031
    ///           hosts:
    ///             - name: server-10
    ///               ip: 10.99.3.100
    /// ```
    pub(in crate::task) hosts: Hosts,
    #[serde(default)]
    /// Failover coordinator struct.
    /// If cluster should be without failover (`failover_mode: "disabled"`)
    /// this field will be skipped
    /// ```yaml
    /// failover:
    ///     mode: stateful
    ///     state_provider: stateboard
    ///     stateboard_params:
    ///         uri: "10.99.3.100:4001"
    ///         password: "vG?-GG!4sxV8q5:f"
    /// ```
    pub(in crate::task) failover: Failover,
    /// Ansible cartridge vars in freedom format
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
    /// Although declaring wars does not allow declaring all parameters,
    /// the most important ones will still be added during inventory generation
    pub(in crate::task) vars: Vars,
}

pub(in crate::task) struct Context<T>(pub(in crate::task) T);

impl<T, S> MapSelf<S> for Context<T> {
    type Target = S;
    type Error = TaskError;

    fn map_self<F>(self, func: F) -> Result<Self::Target, Self::Error>
    where
        F: FnOnce(Self) -> Result<Self::Target, Self::Error>,
    {
        func(self)
    }
}

impl<T> Functor for Context<T> {
    type Unwrapped = T;
    type Wrapped<U> = Context<U>;
    type Error = TaskError;

    fn map<F, U>(self, func: F) -> Result<Self::Wrapped<U>, Self::Error>
    where
        F: FnOnce(Self::Unwrapped) -> Result<U, Self::Error>,
    {
        Ok(Context(func(self.0)?))
    }
}

impl<'a> TryFrom<&'a [u8]> for Cluster {
    type Error = TaskError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        serde_yaml::from_slice(value)
            .map_err(|error| {
                TaskError::ConfigError(ConfigError::FileFormatError(format!(
                    "Error then deserializing cluster file {}",
                    error
                )))
            })
            .map(|cluster: Cluster| cluster.sort())
    }
}

impl<'a> TryFrom<&'a ArgMatches> for Cluster {
    type Error = TaskError;

    fn try_from(args: &'a ArgMatches) -> Result<Self, Self::Error> {
        Ok(Cluster {
            failover: Failover::try_from(args)?,
            ..Cluster::default()
        }
        .sort())
    }
}

impl Cluster {
    #[inline]
    fn sort(self) -> Self {
        Self {
            instances: self.instances.sort(),
            ..self
        }
    }
}

#[cfg(test)]
mod test {
    use genin::libs::ins::Type;

    use super::*;

    #[test]
    fn test_cluster_sorted_after_from() {
        let cluster = Cluster::try_from(
            std::fs::read("../../test/resources/test-cluster-sort.genin.yaml")
                .unwrap()
                .as_slice(),
        )
        .unwrap();

        println!(
            "test_cluster_sorted_after_from: {:?}",
            cluster
                .instances
                .iter()
                .map(|instance| &instance.itype)
                .collect::<Vec<&Type>>()
        );

        assert_eq!(&cluster.instances[0].itype, &Type::Router);
        assert_eq!(&cluster.instances[1].itype, &Type::Storage);
        assert_eq!(&cluster.instances[2].itype, &Type::Storage);
        assert_eq!(&cluster.instances[3].itype, &Type::Custom);
        assert_eq!(&cluster.instances[4].itype, &Type::Custom);
    }
}
