//! Settled designated roles on one compatible OSPF broadcast segment.
//!
//! Explicit events retain consistent incumbents. This is not the distributed
//! election state machine, neighbor synchronization, or packet exchange.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

pub const MAX_MEMBERS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub label: String,
    pub router_id: Ipv4Addr,
    pub priority: u8,
    pub present: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roles {
    pub designated: Option<usize>,
    pub backup: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Designated,
    Backup,
    Other,
    Absent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relationship {
    WouldAttemptAdjacency,
    RemainTwoWay,
    NoCurrentNeighbor,
}

fn eligible(members: &[Participant], index: usize) -> bool {
    members
        .get(index)
        .is_some_and(|member| member.present && member.priority != 0)
}

fn valid_members(members: &[Participant]) -> bool {
    if !(2..=MAX_MEMBERS).contains(&members.len()) {
        return false;
    }
    let ids: BTreeSet<_> = members.iter().map(|member| member.router_id).collect();
    let labels: BTreeSet<_> = members.iter().map(|member| member.label.as_str()).collect();
    ids.len() == members.len()
        && labels.len() == members.len()
        && members.iter().all(|member| {
            !member.router_id.is_unspecified()
                && !member.router_id.is_multicast()
                && !member.label.trim().is_empty()
                && member.label.len() <= 32
                && !member.label.chars().any(char::is_control)
        })
}

fn rank(members: &[Participant], except: Option<usize>) -> Option<usize> {
    members
        .iter()
        .enumerate()
        .filter(|(index, member)| member.present && member.priority != 0 && Some(*index) != except)
        .max_by_key(|(_, member)| (member.priority, u32::from(member.router_id)))
        .map(|(index, _)| index)
}

impl Roles {
    /// All present members enter a fresh settled election together. Roles
    /// are derived from priority and numeric Router ID inside this fixture.
    pub fn fresh(members: &[Participant]) -> Result<Self, String> {
        Self::settle(
            members,
            Self {
                designated: None,
                backup: None,
            },
        )
    }

    /// Settled result after a supplied join, loss, or priority change.
    /// The previous declarations must come from the same ordered segment.
    pub fn settle(members: &[Participant], previous: Self) -> Result<Self, String> {
        if !valid_members(members)
            || previous
                .designated
                .is_some_and(|index| index >= members.len())
            || previous.backup.is_some_and(|index| index >= members.len())
            || previous.designated.is_some() && previous.designated == previous.backup
        {
            return Err(
                "Use distinct known routers and valid prior roles on the supplied segment.".into(),
            );
        }
        let continuing = previous
            .designated
            .filter(|&index| eligible(members, index));
        let available_backup = previous.backup.filter(|&index| eligible(members, index));
        let designated = continuing
            .or(available_backup)
            .or_else(|| rank(members, None));
        let backup = available_backup
            .filter(|index| Some(*index) != designated)
            .or_else(|| rank(members, designated));
        Ok(Self { designated, backup })
    }

    pub fn member_role(self, members: &[Participant], index: usize) -> Role {
        match members.get(index) {
            Some(member) if member.present => {
                if Some(index) == self.designated {
                    Role::Designated
                } else if Some(index) == self.backup {
                    Role::Backup
                } else {
                    Role::Other
                }
            }
            _ => Role::Absent,
        }
    }

    pub fn relationship(self, members: &[Participant], left: usize, right: usize) -> Relationship {
        let left_role = self.member_role(members, left);
        let right_role = self.member_role(members, right);
        if left == right || [left_role, right_role].contains(&Role::Absent) {
            Relationship::NoCurrentNeighbor
        } else if left_role == Role::Other && right_role == Role::Other {
            Relationship::RemainTwoWay
        } else {
            Relationship::WouldAttemptAdjacency
        }
    }
}
