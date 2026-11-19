//! Scope a stable broadcast election and two explicit comparison trials.
use super::election::{Participant, Relationship, Roles};
use crate::automation_workshop::workspace::Outcome;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,54>")]
    pub segment: String,
    // The supplied order is primary, standby, observer, and later arrival.
    pub members: [Participant; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub priorities: [String; 3],
}

impl Draft {
    pub fn validate(&self) -> Result<()> {
        if self
            .priorities
            .iter()
            .any(|value| value.len() > 5 || value.chars().any(char::is_control))
        {
            return Err("Use one short value in each priority field.".into());
        }
        Ok(())
    }
}

pub struct Stage {
    pub members: [Participant; 4],
    pub roles: Roles,
}

pub struct Observed {
    pub fresh: Stage,
    pub arrival: Stage,
    pub outage: Stage,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        if self.segment.trim().is_empty()
            || self.segment.chars().any(char::is_control)
            || !self.members[..3].iter().all(|member| member.present)
            || self.members[3].present
            || self.members[3].priority < 3
        {
            return Err(
                "Supply three current routers and one eligible later arrival on the same segment."
                    .into(),
            );
        }
        Roles::fresh(&self.members)?;
        self.observe(&self.start())?;
        Ok(())
    }

    pub fn start(&self) -> Draft {
        Draft {
            priorities: std::array::from_fn(|i| self.members[i].priority.to_string()),
        }
    }

    pub fn observe(&self, draft: &Draft) -> Result<Observed> {
        draft.validate()?;
        let mut fresh = self.members.clone();
        for (member, value) in fresh[..3].iter_mut().zip(&draft.priorities) {
            let text = value.trim();
            let parsed = (!text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit()))
                .then(|| text.parse::<u8>().ok())
                .flatten();
            member.priority = parsed.ok_or_else(|| {
                format!(
                    "{} needs a whole interface priority from 0 to 255.",
                    member.label
                )
            })?;
        }
        let elected = Roles::fresh(&fresh)?;
        let mut arrival = fresh.clone();
        arrival[3].present = true;
        let declared = Roles::settle(&arrival, elected)?;
        let mut outage = arrival.clone();
        outage[0].present = false;
        let recovered = Roles::settle(&outage, declared)?;
        Ok(Observed {
            fresh: Stage {
                members: fresh,
                roles: elected,
            },
            arrival: Stage {
                members: arrival,
                roles: declared,
            },
            outage: Stage {
                members: outage,
                roles: recovered,
            },
        })
    }

    pub fn outcomes(&self, draft: &Draft) -> Result<Vec<Outcome>> {
        let Observed {
            fresh,
            arrival,
            outage,
        } = self.observe(draft)?;
        let [primary, standby, observer, late] = &self.members;
        let early = fresh.roles.designated == Some(0);
        let ready = fresh.roles.backup == Some(1);
        let keeps = arrival.roles
            == (Roles {
                designated: Some(0),
                backup: Some(1),
            });
        let higher = arrival.members[3].priority > arrival.members[0].priority
            && arrival.members[3].priority > arrival.members[1].priority;
        let observer_policy = fresh.members[2].priority == 0
            && arrival.roles.relationship(&arrival.members, 2, 3) == Relationship::RemainTwoWay
            && outage.roles.relationship(&outage.members, 2, 1)
                == Relationship::WouldAttemptAdjacency;
        let recovers = keeps
            && outage.roles
                == (Roles {
                    designated: Some(1),
                    backup: Some(3),
                });
        Ok(vec![
            row(
                "Choose the fresh designated router",
                early,
                format!("{} becomes DR on a fresh segment.", primary.label),
                format!(
                    "For a fresh election, choose a priority for {} that wins among the three present routers. A priority tie uses numerical Router ID.",
                    primary.label
                ),
            ),
            row(
                "Prepare the initial backup",
                ready,
                format!("{} becomes BDR before the newcomer arrives.", standby.label),
                format!(
                    "Give {} a nonzero priority that wins the remaining backup choice after the DR.",
                    standby.label
                ),
            ),
            row(
                "Keep the incumbent roles through the higher-priority join",
                keeps && higher,
                format!(
                    "{} joins with higher priority and both earlier leaders retain their roles.",
                    late.label
                ),
                format!(
                    "In the fresh plan choose the required leaders with priorities below {}. A later higher-priority router does not replace a stable DR or BDR.",
                    late.priority
                ),
            ),
            row(
                "Keep the observer connected and out of the election",
                observer_policy,
                format!(
                    "{} stays a neighbor without taking a leadership role.",
                    observer.label
                ),
                format!(
                    "Set {} to priority 0. It can still be a neighbor and attempt adjacency with the elected roles.",
                    observer.label
                ),
            ),
            row(
                "Promote the standby after the named outage",
                recovers,
                format!(
                    "{} takes over as DR and {} becomes BDR.",
                    standby.label, late.label
                ),
                format!(
                    "Check the original backup. If {} is the original BDR, it takes over when {} goes down.",
                    standby.label, primary.label
                ),
            ),
        ])
    }
}

fn row(name: &str, passed: bool, good: String, retry: String) -> Outcome {
    Outcome {
        name: name.into(),
        passed,
        detail: if passed { good } else { retry },
    }
}
