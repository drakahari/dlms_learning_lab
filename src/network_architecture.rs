//! Bounded structural campus-layout analysis, not IP forwarding or convergence.
//! Links represent supplied, already-configured routed adjacencies. No host I/O.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_NODES: usize = 16;
pub const MAX_WIRES: usize = 32;
pub const HELP: &str = "Analyze the supplied campus roles, cable endpoints and stated outages. A structural path assumes the supplied routed links are already configured. It is not a ping, IP/routing-policy verification, Layer-2/STP simulation, convergence-time or throughput measurement. Access devices do not carry transit paths between other access devices in this exercise.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Design {
    TwoTier,
    ThreeTier,
    SpineLeaf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Access,
    CollapsedCore,
    Distribution,
    Core,
    Leaf,
    Spine,
}
impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Self::Access => "Access",
            Self::CollapsedCore => "Distribution + core",
            Self::Distribution => "Distribution",
            Self::Core => "Core",
            Self::Leaf => "Leaf",
            Self::Spine => "Spine",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
}
fn optional_block<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,48>")] String);
    Ok(Option::<Text>::deserialize(d)?.map(|v| v.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub role: Role,
    #[serde(default, deserialize_with = "optional_block")]
    pub block: Option<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Port,8>")]
    pub ports: Vec<Port>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wire {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Jack {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub node: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub port: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub wire: String,
    pub a: Option<Jack>,
    pub b: Option<Jack>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Connection,32>")]
    pub connections: Vec<Connection>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub schema_version: u32,
    pub design: Design,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Node,16>")]
    pub nodes: Vec<Node>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Wire,32>")]
    pub wires: Vec<Wire>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Failure {
    Node {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        id: String,
    },
    Wire {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        id: String,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Failure,4>")]
    pub unavailable: Vec<Failure>,
}
// Observations are derived, never accepted from saved progress as grade authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Path {
    pub nodes: Vec<String>,
    pub wires: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub physical: Option<Path>,
    pub hierarchy: Option<Path>,
    pub layout_issues: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spine_alternatives: Option<Vec<Path>>,
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn label(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 120 && !s.chars().any(char::is_control)
}
type Graph = BTreeMap<String, Vec<(String, String)>>;
impl Facts {
    pub fn node(&self, name: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == name)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || !(3..=MAX_NODES).contains(&self.nodes.len())
            || !(1..=MAX_WIRES).contains(&self.wires.len())
            || self.nodes.iter().map(|n| n.ports.len()).sum::<usize>() > 96
        {
            return Err("Supply version 1 and a bounded campus device/cable inventory.".into());
        }
        let mut nodes = BTreeSet::new();
        for n in &self.nodes {
            let mut ports = BTreeSet::new();
            if !id(&n.id)
                || !label(&n.label)
                || !nodes.insert(&n.id)
                || !(1..=8).contains(&n.ports.len())
                || n.ports.iter().any(|p| !id(&p.id) || !ports.insert(&p.id))
                || match n.role {
                    Role::Access | Role::Distribution | Role::Leaf => {
                        n.block.as_deref().is_none_or(|b| !id(b))
                    }
                    Role::CollapsedCore | Role::Core | Role::Spine => n.block.is_some(),
                }
            {
                return Err("Use distinct device/port IDs, readable labels and a building block for access/distribution devices.".into());
            }
        }
        let mut wires = BTreeSet::new();
        if self
            .wires
            .iter()
            .any(|w| !id(&w.id) || !label(&w.label) || !wires.insert(&w.id))
        {
            return Err("Use distinct cable IDs and readable cable labels.".into());
        }
        let roles_ok = self.nodes.iter().all(|n| match self.design {
            Design::TwoTier => matches!(n.role, Role::Access | Role::CollapsedCore),
            Design::ThreeTier => matches!(n.role, Role::Access | Role::Distribution | Role::Core),
            Design::SpineLeaf => matches!(n.role, Role::Leaf | Role::Spine),
        });
        let count = |role| self.nodes.iter().filter(|n| n.role == role).count();
        if !roles_ok
            || (self.design != Design::SpineLeaf && count(Role::Access) < 2)
            || match self.design {
                Design::TwoTier => {
                    count(Role::CollapsedCore) == 0
                        || count(Role::Core) != 0
                        || count(Role::Distribution) != 0
                }
                Design::ThreeTier => {
                    count(Role::Core) == 0
                        || count(Role::CollapsedCore) != 0
                        || self
                            .nodes
                            .iter()
                            .filter(|n| n.role == Role::Access)
                            .any(|a| {
                                !self
                                    .nodes
                                    .iter()
                                    .any(|d| d.role == Role::Distribution && d.block == a.block)
                            })
                }
                Design::SpineLeaf => count(Role::Leaf) < 2 || count(Role::Spine) == 0,
            }
        {
            return Err(
                "The supplied roles must match the selected campus or canonical leaf/spine design."
                    .into(),
            );
        }
        Ok(())
    }
    /// Allows unplugged ends and physical conflicts in unfinished saved drafts.
    /// Unknown equipment, extra cables and oversized lists are never admitted.
    pub fn validate_draft(&self, c: &Configuration) -> Result<(), String> {
        self.validate()?;
        let supplied: BTreeSet<_> = self.wires.iter().map(|w| &w.id).collect();
        let mut used = BTreeSet::new();
        if c.connections.len() != supplied.len() {
            return Err("Keep exactly the supplied cables, including any unplugged ones.".into());
        }
        for link in &c.connections {
            if !supplied.contains(&link.wire) || !used.insert(&link.wire) {
                return Err("Keep every supplied cable identity exactly once.".into());
            }
            for jack in [&link.a, &link.b].into_iter().flatten() {
                if self
                    .node(&jack.node)
                    .is_none_or(|n| !n.ports.iter().any(|p| p.id == jack.port))
                {
                    return Err(format!(
                        "{} names a device or port outside the supplied inventory.",
                        link.wire
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn validate_experiment(&self, e: &Experiment) -> Result<(), String> {
        self.validate()?;
        if e.unavailable.len() > 4 {
            return Err("Inspect at most four stated failures at once.".into());
        }
        let mut nodes = BTreeSet::new();
        let mut wires = BTreeSet::new();
        for f in &e.unavailable {
            let valid = match f {
                Failure::Node { id } => self.node(id).is_some() && nodes.insert(id),
                Failure::Wire { id } => self.wires.iter().any(|w| w.id == *id) && wires.insert(id),
            };
            if !valid {
                return Err("Use distinct failures from the supplied devices/cables.".into());
            }
        }
        Ok(())
    }
    fn allowed(&self, a: &str, b: &str) -> bool {
        let (Some(a), Some(b)) = (self.node(a), self.node(b)) else {
            return false;
        };
        match (self.design, a.role, b.role) {
            (Design::SpineLeaf, Role::Leaf, Role::Spine)
            | (Design::SpineLeaf, Role::Spine, Role::Leaf) => true,
            (Design::TwoTier, Role::Access, Role::CollapsedCore)
            | (Design::TwoTier, Role::CollapsedCore, Role::Access)
            | (Design::TwoTier, Role::CollapsedCore, Role::CollapsedCore)
            | (Design::ThreeTier, Role::Distribution, Role::Core)
            | (Design::ThreeTier, Role::Core, Role::Distribution)
            | (Design::ThreeTier, Role::Core, Role::Core) => true,
            (Design::ThreeTier, Role::Access, Role::Distribution)
            | (Design::ThreeTier, Role::Distribution, Role::Access)
            | (Design::ThreeTier, Role::Distribution, Role::Distribution) => a.block == b.block,
            _ => false,
        }
    }
    pub fn upstream(&self, c: &Configuration, node: &str) -> Result<BTreeSet<String>, String> {
        // Physical conflicts cannot manufacture two independent uplinks.
        self.inspect(c, &Experiment::default(), node, node)?;
        let n = self.node(node).ok_or("Choose a supplied device.")?;
        let parent = match (self.design, n.role) {
            (Design::SpineLeaf, Role::Leaf) => Role::Spine,
            (Design::TwoTier, Role::Access) => Role::CollapsedCore,
            (Design::ThreeTier, Role::Access) => Role::Distribution,
            (Design::ThreeTier, Role::Distribution) => Role::Core,
            _ => return Err("This role has no upstream tier in the supplied design.".into()),
        };
        let mut result = BTreeSet::new();
        for wire in &c.connections {
            let (Some(a), Some(b)) = (&wire.a, &wire.b) else {
                continue;
            };
            let other = if a.node == node {
                b
            } else if b.node == node {
                a
            } else {
                continue;
            };
            if self.node(&other.node).is_some_and(|p| p.role == parent)
                && self.allowed(node, &other.node)
            {
                result.insert(other.node.clone());
            }
        }
        Ok(result)
    }
    pub fn inspect(
        &self,
        c: &Configuration,
        e: &Experiment,
        from: &str,
        to: &str,
    ) -> Result<Observation, String> {
        self.validate_draft(c)?;
        self.validate_experiment(e)?;
        if self.node(from).is_none() || self.node(to).is_none() {
            return Err("Choose source and destination devices from this campus.".into());
        }
        let down_node = |n: &str| {
            e.unavailable
                .iter()
                .any(|f| matches!(f,Failure::Node{id}if id==n))
        };
        let down_wire = |n: &str| {
            e.unavailable
                .iter()
                .any(|f| matches!(f,Failure::Wire{id}if id==n))
        };
        let mut jacks = BTreeSet::new();
        let mut physical = Graph::new();
        let mut hierarchy = Graph::new();
        let mut issues = vec![];
        for link in &c.connections {
            // Occupancy remains physical even when a device/cable is unavailable.
            for jack in [&link.a, &link.b].into_iter().flatten() {
                if !jacks.insert(jack) {
                    return Err(format!(
                        "Port {}/{} is used by more than one cable. Unplug one end first.",
                        self.node(&jack.node).unwrap().label,
                        jack.port
                    ));
                }
            }
            let (Some(a), Some(b)) = (&link.a, &link.b) else {
                issues.push(format!("{} has an unplugged end.", link.wire));
                continue;
            };
            if a.node == b.node {
                return Err(format!(
                    "{} joins one device to itself; this exercise does not model that loop.",
                    link.wire
                ));
            }
            let allowed = self.allowed(&a.node, &b.node);
            if !allowed {
                issues.push(format!(
                    "{} joins {} to {} outside the supplied tier/building roles.",
                    link.wire,
                    self.node(&a.node).unwrap().label,
                    self.node(&b.node).unwrap().label
                ));
            }
            if down_node(&a.node) || down_node(&b.node) || down_wire(&link.wire) {
                continue;
            }
            for (first, second) in [(&a.node, &b.node), (&b.node, &a.node)] {
                physical
                    .entry(first.clone())
                    .or_default()
                    .push((second.clone(), link.wire.clone()));
                if allowed {
                    hierarchy
                        .entry(first.clone())
                        .or_default()
                        .push((second.clone(), link.wire.clone()));
                }
            }
        }
        for graph in [&mut physical, &mut hierarchy] {
            for neighbors in graph.values_mut() {
                neighbors.sort();
            }
        }
        let paths = if down_node(from) || down_node(to) {
            (None, None)
        } else {
            (
                walk(&physical, from, to, |_| true, None),
                walk(
                    &hierarchy,
                    from,
                    to,
                    |n| {
                        self.node(n)
                            .is_some_and(|n| !matches!(n.role, Role::Access | Role::Leaf))
                    },
                    Some(&|n| match self.node(n).unwrap().role {
                        Role::Access | Role::Leaf => 0,
                        Role::CollapsedCore | Role::Distribution | Role::Spine => 1,
                        Role::Core => 2,
                    }),
                ),
            )
        };
        let spine_alternatives = if self.design == Design::SpineLeaf
            && from != to
            && [from, to]
                .iter()
                .all(|n| self.node(n).is_some_and(|n| n.role == Role::Leaf))
        {
            Some(spine_paths(&hierarchy, from, to))
        } else {
            None
        };
        Ok(Observation {
            spine_alternatives,
            physical: paths.0,
            hierarchy: paths.1,
            layout_issues: issues,
        })
    }
}
/// Distinct internal spines, not cable combinations or a claimed ECMP result.
fn spine_paths(graph: &Graph, from: &str, to: &str) -> Vec<Path> {
    let mut parents = BTreeMap::new();
    for (spine, wire) in graph.get(from).into_iter().flatten() {
        parents.entry(spine).or_insert(wire);
    }
    parents
        .into_iter()
        .filter_map(|(spine, first)| {
            let (_, second) = graph.get(spine)?.iter().find(|(leaf, _)| leaf == to)?;
            Some(Path {
                nodes: vec![from.into(), spine.clone(), to.into()],
                wires: vec![first.clone(), second.clone()],
            })
        })
        .collect()
}
/// A hierarchy path can rise through the tiers, then descend; it cannot use a
/// distribution block as a shortcut between otherwise disconnected cores.
fn walk(
    graph: &Graph,
    from: &str,
    to: &str,
    transit: impl Fn(&str) -> bool,
    rank: Option<&dyn Fn(&str) -> u8>,
) -> Option<Path> {
    type Vertex = (String, bool);
    let start = (from.to_string(), false);
    let mut seen = BTreeSet::from([start.clone()]);
    let mut parents = BTreeMap::<Vertex, (Vertex, String)>::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(at) = queue.pop_front() {
        let (node, descending) = &at;
        if node == to {
            let mut cursor = at;
            let mut nodes = vec![cursor.0.clone()];
            let mut wires = vec![];
            while cursor != start {
                let (prev, wire) = parents.get(&cursor)?;
                nodes.push(prev.0.clone());
                wires.push(wire.clone());
                cursor = prev.clone();
            }
            nodes.reverse();
            wires.reverse();
            return Some(Path { nodes, wires });
        }
        if node != from && !transit(node) {
            continue;
        }
        if let Some(neighbors) = graph.get(node) {
            for (next, wire) in neighbors {
                let mut down = *descending;
                if let Some(rank) = rank {
                    let (a, b) = (rank(node), rank(next));
                    if down && b > a {
                        continue;
                    }
                    down |= b < a;
                }
                let next = (next.clone(), down);
                if seen.insert(next.clone()) {
                    parents.insert(next.clone(), (at.clone(), wire.clone()));
                    queue.push_back(next);
                }
            }
        }
    }
    None
}
