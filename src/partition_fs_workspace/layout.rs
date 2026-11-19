//! Optional relational GPT requirements. Old plans and their wire data omit this.
use super::{Target, core, matches};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Rule {
    AdjacentFree {
        #[serde(deserialize_with = "crate::cron::text::<_,36>")]
        target: String,
        minimum_bytes: u64,
    },
    SeparateDisks {
        #[serde(deserialize_with = "crate::cron::text::<_,36>")]
        first: String,
        #[serde(deserialize_with = "crate::cron::text::<_,36>")]
        second: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,4>")]
    pub rules: Vec<Rule>,
}
impl Plan {
    pub fn validate(&self, targets: &[Target]) -> Result<(), String> {
        if self.version != 1 || !(1..=4).contains(&self.rules.len()) {
            return Err("Supply one to four supported layout requirements.".into());
        }
        let names: BTreeSet<_> = targets.iter().map(|t| t.name.as_str()).collect();
        let mut keys = BTreeSet::new();
        for rule in &self.rules {
            let key = match rule {
                Rule::AdjacentFree {
                    target,
                    minimum_bytes,
                } => {
                    if !names.contains(target.as_str())
                        || *minimum_bytes == 0
                        || !minimum_bytes.is_multiple_of(core::SECTOR_BYTES)
                        || *minimum_bytes > core::MAX_SECTORS * core::SECTOR_BYTES
                    {
                        return Err("A growth window needs a supplied target and a bounded positive whole-sector size.".into());
                    }
                    format!("adjacent:{target}")
                }
                Rule::SeparateDisks { first, second } => {
                    if first == second
                        || !names.contains(first.as_str())
                        || !names.contains(second.as_str())
                    {
                        return Err(
                            "Disk separation needs two distinct supplied partition roles.".into(),
                        );
                    }
                    let mut pair = [first.as_str(), second.as_str()];
                    pair.sort();
                    format!("separate:{}:{}", pair[0], pair[1])
                }
            };
            if !keys.insert(key) {
                return Err("Keep each layout requirement once.".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, state: &core::State, targets: &[Target]) -> Vec<(bool, String)> {
        if state.validate().is_err()
            || state.mbr_profile.is_some()
            || self.validate(targets).is_err()
        {
            return vec![(
                false,
                "The supplied GPT layout or its requirements are invalid.".into(),
            )];
        }
        self.rules.iter().map(|rule|match rule {
            Rule::AdjacentFree{target,minimum_bytes}=>{
                let Some((disk,part))=located(state,targets,target)else{return (false,format!("{target}: create exactly one matching partition before checking its growth window."));};
                let device=&state.disks[disk];
                // Core GPT last usable sector is sectors-34, inclusive. The
                // next boundary is sectors-33; reserved metadata is never free.
                let next=device.partitions().filter(|p|p.start>part.end).map(|p|p.start).min().unwrap_or(device.sectors-33);
                let bytes=next.saturating_sub(part.end+1)*core::SECTOR_BYTES;
                (bytes>=*minimum_bytes,format!("{target} on {disk}{}: {bytes} bytes unused directly after it; need at least {minimum_bytes}. Space elsewhere cannot extend this partition in place.",part.number))
            },
            Rule::SeparateDisks{first,second}=>{
                let(Some((a,_)),Some((b,_)))=(located(state,targets,first),located(state,targets,second))else{return (false,format!("{first} / {second}: create exactly one of each role before checking disk separation."));};
                (a!=b,format!("{first} uses {a}; {second} uses {b}. The task needs these roles on different supplied disks."))
            },
        }).collect()
    }
}

fn located<'a>(
    state: &'a core::State,
    targets: &[Target],
    name: &str,
) -> Option<(&'a str, &'a core::Partition)> {
    let target = targets.iter().find(|t| t.name == name)?;
    let found = matches(state, target);
    (found.len() == 1).then(|| found[0])
}
