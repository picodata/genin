use std::ops::{Deref, DerefMut};

use crate::error::TaskError;
use crate::task::flv::DEFAULT_STB_PORT;
use crate::task::vrs::Vars;
use log::{debug, info, trace};
use prettytable::{color, Attr, Cell, Row, Table};
use serde_yaml::Value;

use crate::task::cluster::hst::{FlatHost, MaxLen, Ports, PortsVariants, TryIntoFlatHosts};
use crate::task::cluster::{
    ins::{v2::InstanceV2, IntoV2, Role, Type},
    Cluster,
};

pub(in crate::task) struct Scheme {
    pub(in crate::task) hosts: Vec<FlatHost>,
    pub(in crate::task) vars: Vars,
    pub(in crate::task) ports_vec: Vec<(u16, u16)>,
}

impl<'a> TryFrom<&'a Cluster> for Scheme {
    type Error = TaskError;

    fn try_from(cluster: &'a Cluster) -> Result<Self, Self::Error> {
        // pub struct InstanceV2 {
        //      name: String,
        //      parent: String,
        //      itype: Type,
        //      replicasets_count: usize,
        //      replication_factor: usize,
        //      weight: usize,
        //      roles: Vec<Role>,
        //      config: Value,
        // }
        // - convert vector of instances into vec of reading patterns
        // - multiply all instances to full count
        // - spread routers
        // - spread storages
        // - spread replicas
        // - spread custom

        let mut hosts: Vec<FlatHost> = match cluster {
            Cluster::V1 { hosts, .. } => TryIntoFlatHosts::try_into(hosts)?,
            Cluster::V2 { hosts, .. } => TryIntoFlatHosts::try_into(hosts)?,
        };

        let (instances, failover, vars) = match cluster {
            Cluster::V1 {
                instances,
                failover,
                vars,
                ..
            } => (instances.into_v2(), failover, vars),
            Cluster::V2 {
                topology,
                failover,
                vars,
                ..
            } => (topology.clone(), failover, vars),
        };

        //let mut hosts = FlatHosts::try_from(&cluster.hosts)?;
        let mut ports = PortsVariants::None;
        ports.or_else(hosts[0].ports);

        // Each iteration is Vec with non multiplied instance
        // 1. multiply instance to `count()` and collect it as vector of vectors with Instance
        // 2. spread across hosts
        // 3. represent them as table with empty (dummy) cells
        //replicaset
        instances
            .iter()
            .flat_map(|instance| instance.multiply())
            .rev()
            .fold(
                vec![Vec::new(), Vec::new()],
                |mut acc: Vec<Vec<InstanceV2>>, instances| {
                    trace!(
                        "{:?}",
                        instances
                            .iter()
                            .map(|ins| ins.name.as_str())
                            .collect::<Vec<&str>>()
                    );
                    match instances.last() {
                        Some(InstanceV2 {
                            itype: Type::Router | Type::Storage | Type::Dummy | Type::Replica,
                            ..
                        }) => acc.push(instances),
                        _ => acc[0].extend(instances),
                    }
                    acc
                },
            )
            .into_iter()
            .rev()
            .for_each(|mut instances| {
                trace!(
                    "resulted instances: {:?}",
                    instances
                        .iter()
                        .map(|instance| instance.name.as_str())
                        .collect::<Vec<&str>>()
                );
                debug!("starting port {:?}", &ports);
                // Ports should upped after
                // 1. new instance
                // 2. hosts loop ended
                (0..hosts.len())
                    .cycle()
                    .scan((), |_, index| {
                        trace!("working with host with index {}", index);
                        instances.pop().map(|instance| {
                            instance
                                .is_not_dummy()
                                .then(|| {
                                    trace!(
                                        "pushing {} to host with index {}",
                                        instance.name,
                                        index
                                    );
                                    hosts.deref_mut()[index].instances.push(instance)
                                })
                                .or(None)
                        })
                    })
                    .for_each(|_| {});
            });
        let ports_vec = (1..=hosts.max_len())
            .map(|_| {
                let (http, binary) = (ports.http_or_default(), ports.binary_or_default());
                ports.up();
                (http, binary)
            })
            .collect::<Vec<(u16, u16)>>();

        hosts.iter_mut().for_each(|flhosts| {
            trace!("instances len: {}", flhosts.instances.len());
            let ip = flhosts.ip.to_string();
            flhosts
                .instances
                .iter_mut()
                .enumerate()
                .for_each(|(index, instance)| {
                    trace!("index: {} ports_vec: {:?}", index, ports_vec);
                    instance.config.insert(
                        "advertise_uri".into(),
                        Value::String(format!("{}:{}", ip, ports_vec[index].1)),
                    );
                    instance.config.insert(
                        "http_port".into(),
                        Value::String(ports_vec[index].0.to_string()),
                    );
                });
        });
        // Add stateboard entity
        //
        // cartridge_failover_params:
        //      mode: stateful
        //      state_provider: stateboard
        //      stateboard_params:
        //          uri: 192.168.16.1:3030
        //          password: myapp-password
        failover.failover_variants.with_mut_stateboard(|stb| {
            info!("Failover type: \"Stateboard\" uri: {}", stb.url);
            hosts
                .first_mut()
                .map(|host| {
                    host.instances.push(InstanceV2 {
                        name: "stateboard".into(),
                        parent: "stateboard".into(),
                        itype: Type::Dummy,
                        replicasets_count: 1,
                        replication_factor: 0,
                        weight: 100,
                        stateboard: true,
                        roles: vec![Role::Custom("stateboard".into())],
                        config: vec![
                            (
                                "listen".into(),
                                Value::String(format!(
                                    "0.0.0.0:{}",
                                    stb.url.port.unwrap_or(DEFAULT_STB_PORT)
                                )),
                            ),
                            ("password".into(), Value::String(stb.password.to_string())),
                        ]
                        .into_iter()
                        .collect(),
                    })
                })
                .unwrap_or_else(|| info!("failover type {}", failover.failover_variants));
        });

        hosts.iter().for_each(|host| {
            trace!("Host: {}", host.name());
            host.instances.iter().for_each(|instance| {
                trace!("{}", instance.name);
            });
        });

        Ok(Scheme {
            hosts,
            vars: Vars {
                cartridge_failover_params: failover.clone(),
                ..vars.clone()
            },
            ports_vec,
        })
    }
}

impl Deref for Scheme {
    type Target = Vec<FlatHost>;

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
            vec![Cell::new("ports").with_style(Attr::Bold)]
                .into_iter()
                .chain(
                    self.hosts
                        .deref()
                        .iter()
                        .map(|host| Cell::new(host.name()).with_style(Attr::Bold)),
                )
                .collect::<Vec<Cell>>(),
        ));

        self.ports_vec
            .iter()
            .enumerate()
            .for_each(|(pos, (http, binary))| {
                table.add_row(Row::new(
                    vec![Cell::new(format!("{}/{}", http, binary).as_str())]
                        .into_iter()
                        .chain(self.hosts.iter().map(|host| {
                            host.instances
                                .get(pos)
                                .map(|instance| match instance.itype {
                                    Type::Router => Cell::new(&instance.name)
                                        .with_style(Attr::ForegroundColor(color::BLUE)),
                                    Type::Storage => Cell::new(&instance.name)
                                        .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
                                    Type::Replica => Cell::new(&instance.name)
                                        .with_style(Attr::ForegroundColor(color::GREEN)),
                                    _ => Cell::new(&instance.name)
                                        .with_style(Attr::ForegroundColor(color::CYAN)),
                                })
                                .unwrap_or_else(Cell::default)
                        }))
                        .collect::<Vec<Cell>>(),
                ));
            });
        table.printstd();
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
mod test;
