//! Bounded, isolated learning examples for the existing native subnet/allocation tasks.
use crate::{
    engine::{self, Goal, State},
    pack,
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
/// Only these two native math assignments participate; this is not a general plugin model.
pub fn goal(state: &State) -> Option<Goal> {
    match state {
        State::Subnet { .. } => Some(Goal::Subnet),
        State::Vlsm { .. } => Some(Goal::Vlsm),
        _ => None,
    }
}

/// Explain an explicitly filled, solved practice state; never awards/checks credit.
pub fn worked_summary(state: &State) -> Option<String> {
    if !engine::solved(state, &goal(state)?) {
        return None;
    }
    match state {
        State::Subnet { .. } => {
            let e = crate::ipv4::Exercise::from_state(state)?;
            let [network, broadcast, first, last] = e.expected()?;
            Some(format!(
                "Worked example: /{} leaves {} host bits, giving {} addresses. The first address, {network}, identifies the network; the last, {broadcast}, is broadcast. Usable hosts run from {first} through {last}. {} is one valid choice for workstation 2; it differs from {}. These are ungraded reference values; your edits are checked separately.",
                e.prefix,
                32 - e.prefix,
                e.capacity()?,
                e.second.as_deref()?,
                e.ip
            ))
        }
        State::Vlsm { allocations, .. } => {
            let parts: Option<Vec<String>> = allocations
                .iter()
                .map(|a| {
                    let (network, broadcast, _prefix) = engine::cidr(&a.cidr)?;
                    let total = u64::from(broadcast) - u64::from(network) + 1;
                    Some(format!(
                        "{}: {} gives {total} addresses and {} usable places for {} hosts",
                        a.name,
                        a.cidr,
                        total - 2,
                        a.hosts
                    ))
                })
                .collect();
            Some(format!(
                "Worked example: {}. Each starts on its own boundary, stays inside the pool and avoids overlap. These are ungraded reference allocations; your edits are checked separately.",
                parts?.join("; ")
            ))
        }
        _ => None,
    }
}
/// Answers/allocations may change. The supplied problem must not change on resume/import.
pub fn same_problem(a: &State, b: &State) -> bool {
    match (a, b) {
        (
            State::Subnet {
                ip: a,
                prefix: ap,
                second: as_,
                ..
            },
            State::Subnet {
                ip: b,
                prefix: bp,
                second: bs,
                ..
            },
        ) => a == b && ap == bp && as_.is_some() == bs.is_some(),
        (
            State::Vlsm {
                base: a,
                prefix: ap,
                allocations: aa,
            },
            State::Vlsm {
                base: b,
                prefix: bp,
                allocations: ba,
            },
        ) => {
            a == b
                && ap == bp
                && aa.len() == ba.len()
                && aa
                    .iter()
                    .zip(ba)
                    .all(|(a, b)| a.name == b.name && a.hosts == b.hosts)
        }
        _ => false,
    }
}
/// Original assignments keep supplied values; older subnet saves may lack workstation 2.
pub fn same_assignment(a: &State, b: &State) -> bool {
    match (a, b) {
        (
            State::Subnet {
                ip: a, prefix: ap, ..
            },
            State::Subnet {
                ip: b, prefix: bp, ..
            },
        ) => a == b && ap == bp,
        _ => same_problem(a, b),
    }
}
/// Separation depends on mathematical inputs, not row order or optional metadata.
pub fn distinct_example(a: &State, b: &State) -> bool {
    match (a, b) {
        (State::Subnet { .. }, State::Subnet { .. }) => !same_assignment(a, b),
        (
            State::Vlsm {
                base: a,
                prefix: ap,
                allocations: aa,
            },
            State::Vlsm {
                base: b,
                prefix: bp,
                allocations: ba,
            },
        ) => {
            let mut ah: Vec<_> = aa.iter().map(|a| a.hosts).collect();
            let mut bh: Vec<_> = ba.iter().map(|a| a.hosts).collect();
            ah.sort_unstable();
            bh.sort_unstable();
            a != b || ap != bp || ah != bh
        }
        _ => false,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: State,
    pub example_solution: Vec<String>,
    pub try_change: String,
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        let Some(goal) = goal(&self.fixture) else {
            return Err("Addressing examples require a subnet or allocation problem".into());
        };
        if matches!(self.fixture, State::Subnet { second: None, .. })
            || pack::validate_state(&self.fixture).is_err()
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1600)
            || self.example_solution.is_empty()
            || self.example_solution.len() > 20
            || self
                .example_solution
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 4096)
            || engine::solved(&self.fixture, &goal)
        {
            return Err("Invalid addressing teaching content or already solved example".into());
        }
        let mut candidate = self.fixture.clone();
        for action in &self.example_solution {
            engine::execute(&mut candidate, action)?;
        }
        if !same_problem(&candidate, &self.fixture) || !engine::solved(&candidate, &goal) {
            return Err("Addressing example does not reach its unchanged problem".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub state: State,
    pub checked: Option<State>,
    pub hints: usize,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            state: lesson.fixture.clone(),
            checked: None,
            hints: 0,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.hints <= 3
            && goal(&self.state).is_some()
            && pack::validate_state(&self.state).is_ok()
            && self
                .checked
                .as_ref()
                .is_none_or(|s| same_problem(s, &self.state) && pack::validate_state(s).is_ok())
    }
    /// An explicit fill affects only ungraded example answers, not completion or assignment data.
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !same_problem(&self.state, &lesson.fixture) {
            return Err("Example problem differs from this lesson".into());
        }
        let mut candidate = lesson.fixture.clone();
        for action in &lesson.example_solution {
            engine::execute(&mut candidate, action)?;
        }
        self.state = candidate;
        self.checked = None;
        Ok(())
    }
    pub fn check(&mut self) {
        self.checked = Some(self.state.clone());
    }
    pub fn checks(&self) -> Vec<(bool, String)> {
        if self.checked.as_ref() != Some(&self.state) {
            return Vec::new();
        }
        goal(&self.state).map_or_else(Vec::new, |g| engine::checks(&self.state, &g))
    }
    pub fn solved(&self) -> bool {
        let checks = self.checks();
        !checks.is_empty() && checks.iter().all(|r| r.0)
    }
}
pub fn prepare(a: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if crate::workspace::is_physical_workspace(a) {
        return;
    }
    if crate::workspace::is_file_backup_workspace(a) {
        return;
    }
    if crate::workspace::is_etherchannel_workspace(a) || crate::workspace::is_cron_workspace(a) {
        return;
    }
    if crate::workspace::has_nfs_metadata(a) || crate::workspace::has_cron_metadata(a) {
        return;
    }
    let w = a.workspace.as_mut().expect("workspace prepared");
    if w.address_learning.is_none() {
        let mut l = Learning::new(lesson);
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.address_learning = Some(l);
    }
}
/// A single changing octet, not a claim that every prefix changes only the last octet.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockDiagram {
    pub octet: usize,
    pub value: u8,
    pub start: u16,
    pub end: u16,
    pub block_size: u16,
    pub later_octets: usize,
    pub visible: Vec<(u16, u16)>,
}
pub fn block_diagram(ip: std::net::Ipv4Addr, prefix: u8) -> Option<BlockDiagram> {
    if prefix > 30 {
        return None;
    }
    let octet = (prefix / 8) as usize;
    let block_size = 1u16 << (8 - prefix % 8);
    let value = ip.octets()[octet];
    let start = u16::from(value) / block_size * block_size;
    let total = 256 / block_size;
    let selected = start / block_size;
    let first = selected.saturating_sub(1).min(total.saturating_sub(4));
    let count = total.min(4);
    Some(BlockDiagram {
        octet,
        value,
        start,
        end: start + block_size - 1,
        block_size,
        later_octets: 3 - octet,
        visible: (first..first + count)
            .map(|n| (n * block_size, (n + 1) * block_size - 1))
            .collect(),
    })
}
