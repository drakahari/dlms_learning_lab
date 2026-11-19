//! Supplied site, carrier and hosted-service path planning with stated outages.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Location {
    Worksite,
    Branch,
    ServiceProvider,
    HostedSite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Workstation,
    LocalService,
    Gateway,
    ProviderTransit,
    HostedService,
}

impl Role {
    fn transit(self) -> bool {
        matches!(self, Self::Gateway | Self::ProviderTransit)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub label: String,
    pub role: Role,
    pub location: Location,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    pub ends: [usize; 2],
    pub circuit_group: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outage {
    None,
    Node { index: usize },
    Link { index: usize },
    CircuitGroup { id: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Topology {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Node,12>")]
    pub nodes: Vec<Node>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Link,24>")]
    pub links: Vec<Link>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path {
    pub nodes: Vec<usize>,
    pub links: Vec<usize>,
}

impl Topology {
    pub fn validate(&self) -> Result<(), String> {
        let mut nodes = BTreeSet::new();
        let mut links = BTreeSet::new();
        if !(3..=12).contains(&self.nodes.len())
            || !(2..=24).contains(&self.links.len())
            || self.nodes.iter().any(|node| {
                node.label.trim().is_empty()
                    || node.label.chars().any(char::is_control)
                    || !nodes.insert(&node.label)
            })
            || self.links.iter().any(|link| {
                let [a, b] = link.ends;
                a == b
                    || a >= self.nodes.len()
                    || b >= self.nodes.len()
                    || link.label.trim().is_empty()
                    || link.label.chars().any(char::is_control)
                    || !links.insert(&link.label)
            })
        {
            return Err(
                "Supply a small network of distinct named devices and circuit options.".into(),
            );
        }
        Ok(())
    }

    pub fn path(
        &self,
        active: u32,
        start: usize,
        end: usize,
        outage: Outage,
    ) -> Result<Option<Path>, String> {
        self.validate()?;
        if start == end
            || start >= self.nodes.len()
            || end >= self.nodes.len()
            || active >> self.links.len() != 0
        {
            return Err("Select supplied circuit options and two distinct endpoints.".into());
        }
        match outage {
            Outage::Node { index } if index >= self.nodes.len() => {
                return Err("Unknown unavailable node.".into());
            }
            Outage::Link { index } if index >= self.links.len() => {
                return Err("Unknown unavailable circuit.".into());
            }
            Outage::CircuitGroup { id }
                if !self.links.iter().any(|link| link.circuit_group == Some(id)) =>
            {
                return Err("Unknown supplied circuit group.".into());
            }
            _ => {}
        }
        let offline = |node| matches!(outage,Outage::Node{index} if node==index);
        if offline(start) || offline(end) {
            return Ok(None);
        }
        let mut previous = vec![None; self.nodes.len()];
        let mut visited = vec![false; self.nodes.len()];
        visited[start] = true;
        let mut queue = VecDeque::from([start]);
        while let Some(here) = queue.pop_front() {
            if here == end {
                let mut nodes = vec![end];
                let mut links = vec![];
                let mut walk = end;
                while let Some((parent, edge)) = previous[walk] {
                    links.push(edge);
                    nodes.push(parent);
                    walk = parent;
                }
                nodes.reverse();
                links.reverse();
                return Ok(Some(Path { nodes, links }));
            }
            if here != start && !self.nodes[here].role.transit() {
                continue;
            }
            for (index, link) in self.links.iter().enumerate() {
                if active & (1 << index) == 0
                    || matches!(outage,Outage::Link{index:down}if down==index)
                    || matches!(outage,Outage::CircuitGroup{id}if link.circuit_group==Some(id))
                {
                    continue;
                }
                let [a, b] = link.ends;
                let peer = if a == here {
                    b
                } else if b == here {
                    a
                } else {
                    continue;
                };
                if !visited[peer] && !offline(peer) {
                    visited[peer] = true;
                    previous[peer] = Some((here, index));
                    queue.push_back(peer);
                }
            }
        }
        Ok(None)
    }
}
