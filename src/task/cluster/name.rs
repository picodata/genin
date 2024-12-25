use std::{fmt::Display, hash::Hash};

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Name {
    childrens: Vec<String>,
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.childrens.last().unwrap())
    }
}

impl<'a> From<&'a str> for Name {
    fn from(s: &'a str) -> Self {
        Self {
            childrens: vec![s.to_string()],
        }
    }
}

impl From<String> for Name {
    fn from(s: String) -> Self {
        Self { childrens: vec![s] }
    }
}

impl<'a> From<&'a Name> for &'a str {
    fn from(val: &'a Name) -> Self {
        val.childrens.last().map(|s| s.as_str()).unwrap()
    }
}

impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.childrens
            .last()
            .unwrap()
            .cmp(other.childrens.last().unwrap())
    }
}

impl Hash for Name {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl PartialEq for Name {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Name {}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        let instance_regex = Regex::new(
            r"(?x)
^(?P<name>[a-zA-Z_]+) # instance name
[-_]
(?P<replicaset_num>\d) # replicaset number
[-_]
(?P<instance_num>\d)$ # instance number in replicaset",
        )
        .unwrap();
        let replicaset_regex = Regex::new(
            r"(?x)
^(?P<name>[a-zA-Z_]+) # instance name
[-_]
(?P<replicaset_num>\d)$ # replicaset number",
        )
        .unwrap();

        match (
            instance_regex.captures(&name),
            replicaset_regex.captures(&name),
        ) {
            (Some(captures), None) => Ok(Name::from(&captures["name"])
                .with_index(&captures["replicaset_num"])
                .with_index(&captures["instance_num"])),
            (None, Some(captures)) => {
                Ok(Name::from(&captures["name"]).with_index(&captures["replicaset_num"]))
            }
            _ => Ok(Name::from(name)),
        }
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.into())
    }
}

impl Name {
    pub fn with_index<T: Display>(self, index: T) -> Self {
        Self {
            childrens: [
                self.childrens.clone(),
                vec![format!("{}-{}", self.childrens.last().unwrap(), index)],
            ]
            .concat(),
        }
    }

    pub fn clone_with_index<T: Display>(&self, index: T) -> Self {
        Self {
            childrens: [
                self.childrens.clone(),
                vec![format!("{}-{}", self.childrens.last().unwrap(), index)],
            ]
            .concat(),
        }
    }

    pub fn with_raw_index<T: Display>(self, index: T) -> Self {
        Self {
            childrens: self
                .childrens
                .into_iter()
                .chain(vec![index.to_string()])
                .collect(),
        }
    }

    pub fn clone_with_raw_index<T: Display>(&self, index: T) -> Self {
        Self {
            childrens: self
                .childrens
                .clone()
                .into_iter()
                .chain(vec![index.to_string()])
                .collect(),
        }
    }

    /// Returns the name of the ancestor on the basis of which the
    /// current name is formed.
    ///
    /// * If the Name has no children, then the original name will be
    ///   returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let topology_member_name = Name::from("router");
    /// let replicaset_name = topology_member_name.clone_with_index(1);
    /// let instance_name = topology_member_name.clone_with_index(3);
    ///
    /// assert_eq!(instance_name.name(), "router-1-3");
    /// assert_eq!(instance_name.get_ancestor_str(), "router");
    ///
    /// // Ancestor name of topology_member_name is "router" because he
    /// // does not have childrens.
    /// assert_eq!(topology_member_name.get_ancestor_str(), "router");
    /// ```
    pub fn get_ancestor_str(&self) -> &str {
        self.childrens.first().unwrap()
    }

    /// Returns the `&str` with name of the parent on the basis of which the
    /// current name is formed.
    ///
    /// * If the parent Name has no children, then the original name
    ///   will be returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let topology_member_name = Name::from("router");
    /// let replicaset_name = topology_member_name.clone_with_index(1);
    /// let instance_name = topology_member_name.clone_with_index(3);
    ///
    /// assert_eq!(instance_name.name(), "router-1-3");
    /// assert_eq!(instance_name.get_parent_str(), "router-1");
    /// assert_eq!(replicaset_name.get_parent_str(), "router");
    ///
    /// // Parent name of topology_member_name is "router" because he
    /// // does not have childrens.
    /// assert_eq!(topology_member_name.get_parent_str(), "router");
    /// ```
    pub fn get_parent_str(&self) -> &str {
        self.childrens
            .get(self.childrens.len().saturating_sub(2))
            .unwrap_or_else(|| self.childrens.first().unwrap())
    }

    /// Returns the name of the parent on the basis of which the
    /// current name is formed.
    ///
    /// * If the parent Name has no children, then the original name
    ///   will be returned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let topology_member_name = Name::from("router");
    /// let replicaset_name = topology_member_name.clone_with_index(1);
    /// let instance_name = topology_member_name.clone_with_index(3);
    ///
    /// assert_eq!(instance_name.name(), Name::from("router").with_index(1).with_index(3));
    /// assert_eq!(instance_name.get_parent_name(), Name::from("router").with_index(1));
    /// assert_eq!(replicaset_name.get_parent_name(), Name::from("router"));
    ///
    /// // Parent name of topology_member_name is "router" because he
    /// // does not have childrens.
    /// assert_eq!(topology_member_name.get_parent_name(), Name::from("router"));
    /// ```
    pub fn get_parent_name(&self) -> Self {
        let len = self.len();
        if len > 1 {
            Self {
                childrens: self.childrens.clone().drain(0..=len - 2).collect(),
            }
        } else {
            self.clone()
        }
    }

    pub fn get_ancestor_name(&self) -> Self {
        Self {
            childrens: self.childrens.clone().drain(0..1).collect(),
        }
    }

    pub fn as_replicaset_name(&self) -> Self {
        if self.len() == 3 {
            self.get_parent_name().clone_with_index("replicaset")
        } else {
            self.clone_with_index("replicaset")
        }
    }

    pub fn as_replicaset_alias(&self) -> Self {
        if self.len() == 3 {
            self.get_parent_name()
        } else {
            self.clone()
        }
    }

    pub fn len(&self) -> usize {
        self.childrens.len()
    }

    pub fn parent_index_as_usize(&self) -> Option<usize> {
        self.childrens
            .get(self.childrens.len() - 2)
            .unwrap_or_else(|| self.childrens.first().unwrap())
            .split('-')
            .last()
            .map(|index| index.parse::<usize>().unwrap())
    }

    pub fn last_index_as_usize(&self) -> Option<usize> {
        self.childrens
            .last()
            .unwrap_or_else(|| self.childrens.first().unwrap())
            .split('-')
            .last()
            .map(|index| index.parse::<usize>().unwrap())
    }
}

#[cfg(test)]
mod test;
