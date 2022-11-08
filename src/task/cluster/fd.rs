use std::{borrow::Cow, cell::RefCell, cmp::Ordering};

use log::trace;
use tabled::{builder::Builder, merge::Merge, Alignment, Tabled};


#[allow(unused)]
#[derive(Debug, PartialEq, Eq)]
pub enum Instance {
    Router { name: String },
    Storage { name: String },
    Custom { name: String },
}

impl Instance {
    pub fn name(&self) -> &str {
        match self {
            Instance::Router { name } => name,
            Instance::Storage { name } => name,
            Instance::Custom { name } => name,
        }
    }
}

#[allow(unused)]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FailureDomain {
    name: String,
    params: Params,
    instances: Vec<Instance>,
    failure_domains: Vec<FailureDomain>,
}

impl<'a> From<&'a str> for FailureDomain {
    fn from(s: &'a str) -> Self {
        FailureDomain {
            name: s.into(),
            params: Params {
                begin_http_port: None,
                begin_binary_port: None,
            },
            instances: Vec::new(),
            failure_domains: Vec::new(),
        }
    }
}

impl PartialOrd for FailureDomain {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.instances.len().partial_cmp(&other.instances.len()) {
            Some(Ordering::Equal) => self.name.partial_cmp(&other.name),
            ord => ord,
        }
    }
}

impl Ord for FailureDomain {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.instances.len().cmp(&other.instances.len()) {
            Ordering::Equal => self.name.cmp(&other.name),
            any => any,
        }
    }
}

impl ToString for FailureDomain {
    fn to_string(&self) -> String {
        let mut table = self.as_table_builder().index().build();
        table.with(Merge::horizontal());
        table.with(Alignment::center());
        table.to_string()
    }
}

#[allow(unused)]
impl FailureDomain {
    pub fn spread(&mut self) {
        if self.failure_domains.is_empty() {
            return;
        }

        self.instances.reverse();

        while let Some(instance) = self.instances.pop() {
            self.failure_domains.sort();
            self.push(instance);
        }

        self.failure_domains
            .sort_by(|left, right| left.name.cmp(&right.name));

        self.failure_domains.iter_mut().for_each(|failure_domain| {
            failure_domain.params.update_from(self.params.clone());
            failure_domain.spread();
        });
    }

    pub fn push(&mut self, instance: Instance) {
        if let Some(failure_domain) = self.failure_domains.first_mut() {
            failure_domain.instances.push(instance);
        } else {
            panic!("failed to get mutable reference to first failure_domain")
        }
    }

    pub fn with_inner_domains(mut self, failure_domains: Vec<FailureDomain>) -> Self {
        self.failure_domains = failure_domains;
        self
    }

    pub fn with_instances(mut self, instances: Vec<Instance>) -> Self {
        self.instances = instances;
        self
    }

    pub fn with_begin_http_port(mut self, port: usize) -> Self {
        self.params.begin_http_port = Some(port);
        self
    }

    pub fn with_begin_binary_port(mut self, port: usize) -> Self {
        self.params.begin_binary_port = Some(port);
        self
    }

    pub fn size(&self) -> usize {
        if self.failure_domains.is_empty() {
            self.instances.len()
        } else {
            self.failure_domains
                .iter()
                .fold(0usize, |acc, fd| acc + fd.size())
        }
    }

    pub fn width(&self) -> usize {
        self.failure_domains.iter().fold(0usize, |acc, fd| {
            if fd.failure_domains.is_empty() {
                acc + 1
            } else {
                acc + fd.width()
            }
        })
    }

    pub fn depth(&self) -> usize {
        let depth = if self.failure_domains.is_empty() {
            self.instances.len()
        } else {
            self.failure_domains.iter().fold(0usize, |acc, fd| {
                if fd.depth() > acc {
                    fd.depth()
                } else {
                    acc
                }
            })
        };
        depth + 1
    }

    // !failure_domains.is_empty() -> (Row(vec!["Cluster", "Cluster", "Cluster"]))
    // failure_domains.is_empty() -> (Row(vec!["Storage-1-1", "Storage-1-2", "Storage-1-3"]))
    pub fn print_table(&self) {
        println!("{}", self.to_string());
    }

    fn as_table_builder(&self) -> Builder {
        let collector = RefCell::new(vec![Vec::new(); self.depth()]);
        self.form_structure(0, &collector);
        Builder::from_iter(collector.take().into_iter())
    }

    fn form_structure(&self, mut depth: usize, collector: &RefCell<Vec<Vec<DomainMember>>>) {
        collector
            .borrow_mut()
            .get_mut(depth)
            .map(|level| level.extend(vec![DomainMember::from(self.name.as_str()); self.width()]))
            .unwrap();

        if self.instances.is_empty() {
            trace!(
                "Spreading instances for {} skipped. Width {}. Current level {} vector lenght {}",
                self.name,
                self.width(),
                depth,
                collector.borrow().get(depth).unwrap().len()
            );
            self.failure_domains
                .iter()
                .for_each(|domain| domain.form_structure(depth + 1, collector));
        } else {
            trace!(
                "Spreading instances for {} -> {:?}",
                self.name,
                self.instances
                    .iter()
                    .map(|instance| instance.name())
                    .collect::<Vec<&str>>()
            );
            collector
                .borrow_mut()
                .get_mut(depth)
                .map(|level| level.push(DomainMember::from(self.name.as_str())))
                .unwrap();
            let remainder = collector.borrow().len() - depth - 1;
            (0..remainder).into_iter().for_each(|index| {
                collector
                    .borrow_mut()
                    .get_mut(depth + index + 1)
                    .map(|level| {
                        if let Some(instance) = self.instances.get(index) {
                            level.push(DomainMember::Instance {
                                name: instance.name().to_string(),
                                http_port: self.params.begin_http_port.unwrap_or(8080) + index,
                                binary_port: self.params.begin_binary_port.unwrap_or(3030) + index,
                            });
                        } else {
                            level.push(DomainMember::Dummy);
                        }
                    })
                    .unwrap();
            });
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Params {
    begin_binary_port: Option<usize>,
    begin_http_port: Option<usize>,
}

impl Params {
    pub fn update_from(&mut self, rhs: Params) {
        if self.begin_http_port.is_none() && rhs.begin_http_port.is_some() {
            self.begin_http_port = rhs.begin_http_port;
        }
        if self.begin_binary_port.is_none() && rhs.begin_binary_port.is_some() {
            self.begin_binary_port = rhs.begin_binary_port;
        }
    }
}

#[derive(Clone, Tabled, Debug)]
pub enum DomainMember {
    #[tabled(display_with("Self::display_domain", args))]
    Domain(String),
    #[tabled(display_with("Self::display_instance", args))]
    Instance {
        #[tabled(inline)]
        name: String,
        #[tabled(inline)]
        http_port: usize,
        #[tabled(inline)]
        binary_port: usize,
    },
    #[tabled(display_with("Self::display_valid", args))]
    Dummy,
}

impl<'a> From<&'a str> for DomainMember {
    fn from(s: &'a str) -> Self {
        Self::Domain(s.to_string())
    }
}

impl<'a> From<DomainMember> for Cow<'a, str> {
    fn from(val: DomainMember) -> Self {
        match val {
            DomainMember::Domain(name) => Cow::Owned(name),
            DomainMember::Instance {
                name,
                http_port,
                binary_port,
            } => Cow::Owned(format!("{}\n{} {}", name, http_port, binary_port)),
            DomainMember::Dummy => Cow::Owned(Default::default()),
        }
    }
}

#[cfg(test)]
mod test;
