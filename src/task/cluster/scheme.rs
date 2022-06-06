use std::ops::{Deref, DerefMut};

use genin::libs::{
    error::TaskError,
    hst::{Ports, PortsVariants},
    ins::{Instance, Role, Type},
    vrs::Vars,
};
use log::{debug, info, trace};
use prettytable::{color, Attr, Cell, Row, Table};
use serde_yaml::Value;

use crate::task::cluster::hosts::FlatHosts;

use super::{hosts::FlatHost, Cluster};

pub(in crate::task) struct Scheme {
    pub(in crate::task) hosts: FlatHosts,
    pub(in crate::task) vars: Vars,
}

impl<'a> TryFrom<&'a Cluster> for Scheme {
    type Error = TaskError;

    fn try_from(cluster: &'a Cluster) -> Result<Self, Self::Error> {
        // pub struct Instance {
        //      name: String,
        //      parent: String,
        //      itype: Type,
        //      count: usize,
        //      replicas: usize,
        //      weight: usize,
        //      roles: Vec<Role>,
        //      config: Value,
        // }
        // - convert vector of instances into vec of sreading patterns
        // - multiply all instances to full count
        // - spread routers
        // - spread storages
        // - spread replicas
        // - spread custom

        let mut hosts = FlatHosts::try_from(&cluster.hosts)?;
        let mut replicasets = Vec::new();
        let mut another_replicasets = Vec::new();

        cluster.instances.iter().for_each(|instance| {
            instance
                .can_be_same()
                .then(|| {
                    replicasets
                        .is_empty()
                        .then(|| replicasets.push(vec![instance]))
                        .unwrap_or_else(|| {
                            replicasets.iter_mut().for_each(|repl| {
                                repl.last()
                                    .unwrap()
                                    .count
                                    .eq(&instance.count)
                                    .then(|| repl.push(instance))
                                    .unwrap_or_else(|| another_replicasets.push(instance))
                            })
                        })
                })
                .unwrap_or_else(|| another_replicasets.push(instance))
        });

        let mut ports = PortsVariants::None;

        replicasets.push(another_replicasets);
        replicasets.into_iter().for_each(|replicaset| {
            // Each iteration is Vec with non multiplied instances
            // 1. multiply instance to `count()` and collect it as vector
            // 2. add replicase for each instance if hi is replicated
            replicaset
                .into_iter()
                .fold(Vec::new(), |mut acc: Vec<Vec<Instance>>, instance| {
                    acc.extend(instance.multiply());
                    acc
                })
                .into_iter()
                .for_each(|mut multiplied_instances| {
                    // if port already some add 1
                    ports.up();
                    debug!(
                        "mutliplied: {:?}",
                        &multiplied_instances
                            .iter()
                            .map(|ins| ins.name.as_str())
                            .collect::<Vec<&str>>()
                    );
                    debug!("starting port {:?}", &ports);
                    (0..hosts.len())
                        .cycle()
                        .scan((), |_, index| {
                            trace!("working with host with index {}", index);
                            multiplied_instances.pop().map(|mut instance| {
                                instance
                                    .is_not_dummy()
                                    .then(|| {
                                        ports.or_else(hosts[index].ports);
                                        trace!(
                                            "pushing {} to host with index {}",
                                            instance.name,
                                            index
                                        );
                                        instance.config.insert(
                                            "advertise_uri".into(),
                                            Value::String(format!(
                                                "{}:{}",
                                                hosts[index].ip.to_string(),
                                                ports.binary_or_default()
                                            )),
                                        );
                                        instance.config.insert(
                                            "http_port".into(),
                                            Value::String(ports.http_or_default().to_string()),
                                        );
                                        hosts.deref_mut()[index].instances.push(instance)
                                    })
                                    .or(None)
                            })
                        })
                        .for_each(|_| {});
                });
        });

        (1..hosts.len()).for_each(|iteration| {
            let (left, right) = hosts.split_at_mut(iteration);
            left.last_mut()
                .map(|left| {
                    right.iter_mut().rev().for_each(|right| {
                        left.instances
                            .last()
                            .map(|llast| {
                                llast.itype.eq(&Type::Custom)
                                    && right
                                        .instances
                                        .last()
                                        .map(|rlast| !rlast.parent.eq(&llast.parent))
                                        .unwrap_or_else(|| false)
                                    && left.instances.len() > right.instances.len()
                                    && left.instances.len() - right.instances.len() >= 2
                            })
                            .unwrap_or_else(|| false)
                            .then(|| {
                                trace!("moving instance from {} to {}", left.name(), right.name());
                                left.instances
                                    .pop()
                                    .map(|instance| right.instances.push(instance))
                                    .unwrap_or_else(|| {});
                            })
                            .map(|_| {
                                debug!("instance moved from {} to {}", left.name(), right.name())
                            })
                            .unwrap_or_else(|| {});
                    });
                })
                .unwrap_or_else(|| {});
        });

        // Add stateboard entity
        //
        // cartridge_failover_params:
        //      mode: stateful
        //      state_provider: stateboard
        //      stateboard_params:
        //          uri: 192.168.16.1:3030
        //          password: myapp-password
        cluster
            .failover
            .failover_variants
            .with_mut_stateboard(|stb| {
                info!("Failover type: \"Stateboard\" uri: {}", stb.uri);
                hosts
                    .first_mut()
                    .map(|host| {
                        host.instances.push(Instance {
                            name: "stateboard".into(),
                            parent: "stateboard".into(),
                            itype: Type::Dummy,
                            count: 1,
                            replicas: 0,
                            weight: 100,
                            stateboard: true,
                            roles: vec![Role::Custom("stateboard".into())],
                            config: vec![
                                (
                                    "listen".into(),
                                    Value::String(format!("0.0.0.0:{}", stb.uri.port)),
                                ),
                                ("password".into(), Value::String(stb.password.to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        })
                    })
                    .unwrap_or_else(|| {
                        info!("failover type {}", cluster.failover.failover_variants)
                    });
            });

        hosts.iter().for_each(|host| {
            trace!("Host: {}", host.name());
            host.instances.iter().for_each(|instance| {
                trace!("{}", instance.name);
            });
        });

        Ok(Scheme {
            hosts,
            vars: cluster.vars.clone(),
        })
    }
}

impl Deref for Scheme {
    type Target = FlatHosts;

    fn deref(&self) -> &Self::Target {
        &self.hosts
    }
}

impl DerefMut for Scheme {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hosts
    }
}

#[allow(unused)]
impl Scheme {
    pub(in crate::task) fn print(&self) {
        let mut table = Table::new();

        table.set_titles(Row::new(
            vec![Cell::new("")]
                .into_iter()
                .chain(
                    self.hosts
                        .deref()
                        .iter()
                        .map(|host| Cell::new(host.name()).with_style(Attr::Bold)),
                )
                .collect::<Vec<Cell>>(),
        ));
        (0..self.hosts.max_len()).for_each(|pos| {
            table.add_row(Row::new(
                vec![Cell::new(format!("{}", pos).as_str())]
                    .into_iter()
                    .chain(self.hosts.iter().map(|host| {
                        host.instances
                            .get(pos)
                            .map(|instance| match instance.itype {
                                Type::Router => Cell::new(&Self::instance_table(
                                    &instance.name,
                                    &Ports::default(),
                                ))
                                .with_style(Attr::ForegroundColor(color::BLUE)),
                                Type::Storage => Cell::new(&Self::instance_table(
                                    &instance.name,
                                    &Ports::default(),
                                ))
                                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
                                Type::Replica => Cell::new(&Self::instance_table(
                                    &instance.name,
                                    &Ports::default(),
                                ))
                                .with_style(Attr::ForegroundColor(color::GREEN)),
                                _ => Cell::new(&instance.name)
                                    .with_style(Attr::ForegroundColor(color::CYAN)),
                            })
                            .unwrap_or_else(|| Cell::new(" "))
                    }))
                    .collect::<Vec<Cell>>(),
            ));
        });
        table.printstd();
    }

    pub(in crate::task) fn downcast(self) -> Vec<FlatHost> {
        let Scheme { hosts, .. } = self;
        hosts.downcast()
    }

    fn instance_table(s: &str, p: &Ports) -> String {
        let mut table = Table::new();
        table.set_titles(Row::new(vec![Cell::new(s)]));
        table.add_row(Row::new(vec![
            Cell::new(&p.http.to_string()),
            Cell::new(&p.binary.to_string()),
        ]));
        table.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_spreading_to_servers() {
        let _cluster = Cluster::default();

        //assert_eq!(Scheme::try_from(&cluster).unwrap(), scheme);
    }
}
