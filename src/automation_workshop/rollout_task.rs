//! Authored rollout checks replay a small history over the supplied virtual fabric.
use super::change_rollout::{Change, Inventory};
use crate::{rest_controller::Spec, switching::Jack};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Desk {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub slot: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub host: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub spec: Spec,
    pub initial: Inventory,
    pub approved: [Desk; 2],
    pub reserved: Desk,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub intended_peer: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub isolated_peer: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Recorded {
    Apply { change: Change },
    VerifyPilot { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Work {
    pub inventory: Inventory,
    pub pilot_verified: bool,
    pub reconciled: bool,
}

impl Task {
    pub fn validate(&self) -> Result<()> {
        let fabric = self.initial.installed(&self.spec)?;
        let mut seen = std::collections::BTreeSet::new();
        for desk in self.approved.iter().chain(std::iter::once(&self.reserved)) {
            let slot = self
                .spec
                .ports
                .get(&desk.slot)
                .ok_or("Include each supplied desk in the editable port map.")?;
            let port = Jack::Switch {
                switch: slot.switch.clone(),
                port: slot.port.clone(),
            };
            let host = Jack::Host {
                host: desk.host.clone(),
            };
            if !seen.insert(&desk.host)
                || !fabric
                    .cables
                    .iter()
                    .any(|c| c.up && ((c.a == port && c.b == host) || (c.b == port && c.a == host)))
            {
                return Err(
                    "Bind distinct desk identities to their actual active cable positions.".into(),
                );
            }
        }
        if !self.initial.local.is_empty()
            || self.intended_peer == self.isolated_peer
            || !fabric.hosts.contains_key(&self.intended_peer)
            || !fabric.hosts.contains_key(&self.isolated_peer)
            || seen.contains(&self.intended_peer)
            || seen.contains(&self.isolated_peer)
        {
            return Err(
                "Provide separate service peers and no pre-applied device overrides.".into(),
            );
        }
        let protected = self
            .initial
            .desired
            .bindings
            .get(&self.intended_peer)
            .ok_or("The intended service needs a known controller profile.")?;
        for desk in &self.approved {
            if self.connects(&self.initial, desk)? {
                return Err("Both approved desks must begin outside the supplied service.".into());
            }
        }
        let prepared = self.approved.clone().map(|desk| Change::Rebind {
            slot: desk.slot,
            profile: protected.profile.clone(),
        });
        let [first, second] = prepared;
        let reference = [
            Recorded::Apply { change: first },
            Recorded::VerifyPilot { index: 0 },
            Recorded::Apply { change: second },
            Recorded::Apply {
                change: Change::Reconcile,
            },
        ];
        let work = self.replay(&reference)?;
        if self.checks(&work)?.iter().any(|(passed, _)| !passed) {
            return Err("The supplied policy must admit a staged repair with all ordinary and isolated paths.".into());
        }
        Ok(())
    }

    fn connects(&self, inventory: &Inventory, d: &Desk) -> Result<bool> {
        inventory.visible(&self.spec, &d.host, &self.intended_peer)
    }

    fn approved_pilot(&self, inventory: &Inventory, index: usize) -> Result<bool> {
        let pilot = self
            .approved
            .get(index)
            .ok_or("Choose an approved pilot desk.")?;
        let other = &self.approved[1 - index];
        let forecast = inventory.preview(&self.spec, &Change::Reconcile)?;
        let mut after = inventory.clone();
        after.apply(&self.spec, forecast)?;
        Ok(self.connects(inventory, pilot)?
            && self.connects(&after, pilot)?
            && !self.connects(inventory, other)?
            && !self.connects(&after, other)?
            && after.settings(&other.slot)? == self.initial.settings(&other.slot)?
            && after.settings(&self.reserved.slot)?
                == self.initial.settings(&self.reserved.slot)?)
    }

    pub fn replay(&self, records: &[Recorded]) -> Result<Work> {
        if records.len() > 24 {
            return Err(
                "Keep up to 24 recorded virtual rollout actions. Reset to try a new sequence."
                    .into(),
            );
        }
        self.initial.installed(&self.spec)?;
        let mut work = Work {
            inventory: self.initial.clone(),
            pilot_verified: false,
            reconciled: false,
        };
        for record in records {
            match record {
                Recorded::Apply { change } => {
                    let preview = work.inventory.preview(&self.spec, change)?;
                    work.inventory.apply(&self.spec, preview)?;
                    work.reconciled = matches!(change, Change::Reconcile) && work.pilot_verified;
                }
                Recorded::VerifyPilot { index } => {
                    work.pilot_verified |= self.approved_pilot(&work.inventory, *index)?;
                }
            }
        }
        Ok(work)
    }

    pub fn checks(&self, work: &Work) -> Result<Vec<(bool, String)>> {
        let reserved_unchanged = work.inventory.settings(&self.reserved.slot)?
            == self.initial.settings(&self.reserved.slot)?;
        let mut rows = vec![
            (
                work.pilot_verified,
                "Verify one approved pilot after a controller change, before its peer is moved."
                    .into(),
            ),
            (
                work.reconciled,
                "Reconcile and confirm the finished rollout persists.".into(),
            ),
            (
                reserved_unchanged,
                "Keep the reserved port in its original state.".into(),
            ),
            (
                !self.connects(&work.inventory, &self.reserved)?,
                "The reserve device stays outside the rollout service.".into(),
            ),
        ];
        for desk in &self.approved {
            rows.push((
                self.connects(&work.inventory, desk)?,
                format!(
                    "{} reaches the dispatch service in both directions.",
                    desk.host
                ),
            ));
            rows.push((
                work.inventory
                    .isolated(&self.spec, &desk.host, &self.isolated_peer)?,
                format!("{} remains isolated from the unrelated service.", desk.host),
            ));
        }
        Ok(rows)
    }
}
