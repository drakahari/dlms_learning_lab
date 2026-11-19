//! Explicit source opt-in and realized outcomes; choices themselves never grade.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationTarget {
    #[serde(deserialize_with = "string::<_,160>")]
    pub path: String,
    pub decision: core::ConffileDecision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationUpgradeGoal {
    #[serde(deserialize_with = "string::<_,64>")]
    pub package: String,
    #[serde(deserialize_with = "string::<_,64>")]
    pub incoming_archive: String,
    #[serde(deserialize_with = "list::<_,ConfigurationTarget,4>")]
    pub choices: Vec<ConfigurationTarget>,
}
impl ConfigurationUpgradeGoal {
    pub fn valid(&self, initial: &State) -> bool {
        if !bounded_raw(&self.package, 64)
            || !bounded_raw(&self.incoming_archive, 64)
            || self.choices.len() > 4
            || self.choices.iter().any(|r| !bounded_raw(&r.path, 160))
        {
            return false;
        }
        let c = initial.catalog();
        let Some(record) = initial
            .runtime()
            .records
            .get(&self.package)
            .filter(|r| r.status == Status::Installed)
        else {
            return false;
        };
        let Some(old) = c.archives.get(&record.archive) else {
            return false;
        };
        let Some(new) = c
            .archives
            .get(&self.incoming_archive)
            .filter(|a| a.name == self.package && a.version.compare(&old.version).is_gt())
        else {
            return false;
        };
        if c.publications[&c.published].candidates.get(&self.package)
            != Some(&self.incoming_archive)
        {
            return false;
        }
        let before: BTreeSet<_> = old
            .files
            .iter()
            .filter(|(_, p)| p.conffile)
            .map(|(p, _)| p.as_str())
            .collect();
        let after: BTreeSet<_> = new
            .files
            .iter()
            .filter(|(_, p)| p.conffile)
            .map(|(p, _)| p.as_str())
            .collect();
        if before != after || !(1..=4).contains(&before.len()) {
            return false;
        }
        let targets: BTreeMap<_, _> = self
            .choices
            .iter()
            .map(|r| (r.path.as_str(), r.decision))
            .collect();
        if targets.len() != self.choices.len() {
            return false;
        }
        let mut conflicts = BTreeSet::new();
        for p in &before {
            let Some(local) = initial.runtime().files.get(*p) else {
                return false;
            };
            if core::conffile_disposition(&old.files[*p].contents, local, &new.files[*p].contents)
                == core::ConffileDisposition::Conflict
            {
                conflicts.insert(*p);
            }
            for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
                if p.len() + suffix.len() > 160 {
                    return false;
                }
                let derived = format!("{p}{suffix}");
                if initial.runtime().files.contains_key(&derived)
                    || c.archives.values().any(|a| a.files.contains_key(&derived))
                {
                    return false;
                }
                if matches!(suffix, ".dpkg-old" | ".dpkg-dist") && !c.extra_paths.contains(&derived)
                {
                    return false;
                }
            }
        }
        conflicts == targets.keys().copied().collect()
    }
    pub fn check(&self, initial: &State, current: &State) -> (bool, String) {
        if current.catalog() != initial.catalog() {
            return (
                false,
                "Configuration result belongs to a different supplied catalog".into(),
            );
        }
        if !self.valid(initial) {
            return (
                false,
                "Invalid configuration-upgrade source requirements".into(),
            );
        }
        if !current
            .runtime()
            .records
            .get(&self.package)
            .is_some_and(|r| r.status == Status::Installed && r.archive == self.incoming_archive)
        {
            return (
                false,
                format!(
                    "{}: install the required incoming package before verifying configuration results",
                    self.package
                ),
            );
        }
        let old_record = &initial.runtime().records[&self.package];
        let old = &initial.catalog().archives[&old_record.archive];
        let new = &initial.catalog().archives[&self.incoming_archive];
        let choices: BTreeMap<_, _> = self
            .choices
            .iter()
            .map(|r| (r.path.as_str(), r.decision))
            .collect();
        for (p, default) in old.files.iter().filter(|(_, v)| v.conffile) {
            let local = &initial.runtime().files[p];
            let incoming = &new.files[p].contents;
            let disposition = core::conffile_disposition(&default.contents, local, incoming);
            let expected = match disposition {
                core::ConffileDisposition::UpdateUnmodified => incoming,
                core::ConffileDisposition::Conflict
                    if choices[p.as_str()] == core::ConffileDecision::InstallNew =>
                {
                    incoming
                }
                _ => local,
            };
            if current.runtime().files.get(p) != Some(expected) {
                return (
                    false,
                    format!(
                        "{p}: live configuration differs from the required handover; compare the supplied local and incoming text"
                    ),
                );
            }
            for (suffix, decision, text) in [
                (".dpkg-dist", core::ConffileDecision::KeepLocal, incoming),
                (".dpkg-old", core::ConffileDecision::InstallNew, local),
            ] {
                let key = format!("{p}{suffix}");
                let expected = if disposition == core::ConffileDisposition::Conflict
                    && choices[p.as_str()] == decision
                {
                    Some(text)
                } else {
                    None
                };
                if current.runtime().files.get(&key) != expected {
                    return (
                        false,
                        format!(
                            "{key}: required review/recovery copy differs; a version change alone is insufficient"
                        ),
                    );
                }
            }
            for suffix in [".dpkg-new", ".dpkg-tmp"] {
                if current
                    .runtime()
                    .files
                    .contains_key(&format!("{p}{suffix}"))
                {
                    return (
                        false,
                        format!("{p}: unfinished configuration staging file remains"),
                    );
                }
            }
        }
        (
            true,
            format!(
                "{}: incoming package and every required live/review/recovery configuration match",
                self.package
            ),
        )
    }
}
pub(super) fn goal<'a>(d: &'a Definition, package: &str) -> Option<&'a ConfigurationUpgradeGoal> {
    d.requirements.iter().find_map(|r| {
        if let Requirement::ConfigurationUpgrade { outcome } = r {
            (outcome.package == package).then_some(outcome)
        } else {
            None
        }
    })
}
pub(super) fn admit(d: &Definition, s: &State, a: &Action) -> Result<(), String> {
    match a {
        Action::UpgradeConffiles { upgrade } => {
            let Some(scope) = goal(d, &upgrade.package) else {
                return Err("This activity does not admit configuration-upgrade decisions".into());
            };
            if scope.incoming_archive != upgrade.incoming_archive {
                return Err("The proposed incoming archive differs from this activity".into());
            }
        }
        Action::Remove { name } | Action::Purge { name } if goal(d, name).is_some() => {
            if let Some(record) = s.runtime().records.get(name) {
                let archive = &s.catalog().archives[&record.archive];
                for (path, _) in archive.files.iter().filter(|(_, v)| v.conffile) {
                    for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
                        if s.runtime().files.contains_key(&format!("{path}{suffix}")) {
                            return Err("Removing or purging configuration backups is outside this activity; no changes applied".into());
                        }
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

pub(super) fn action_size(upgrade: &core::ConffileUpgrade) -> bool {
    #[derive(Serialize)]
    struct Wire<'a> {
        kind: &'static str,
        upgrade: &'a core::ConffileUpgrade,
    }
    within(
        &Wire {
            kind: "UpgradeConffiles",
            upgrade,
        },
        MAX_ACTION_BYTES,
    )
}
/// Prepare only a draft authorized by this exact authored activity.
pub fn prepare_configuration(
    d: &Definition,
    s: &State,
    draft: &mut Draft,
    package: &str,
) -> Result<(), String> {
    if !d.same_assignment(s) || !draft.valid_for(s) {
        return Err("This review uses different sample packages".into());
    }
    let Some(scope) = goal(d, package) else {
        return Err("This activity has no configuration upgrade for that package".into());
    };
    let upgrade = s.review_conffiles(package).map_err(|e| notice(e.message))?;
    if upgrade.incoming_archive != scope.incoming_archive {
        return Err(
            "Cached metadata does not offer the required upgrade; inspect and refresh it first"
                .into(),
        );
    }
    draft.fill(s, &Action::UpgradeConffiles { upgrade })
}
/// Change one real conflict's draft decision; never publish package state here.
pub fn choose_configuration(
    d: &Definition,
    s: &State,
    draft: &mut Draft,
    path: &str,
    decision: Option<core::ConffileDecision>,
) -> Result<(), String> {
    if !bounded_raw(path, 160) || !d.same_assignment(s) || !draft.valid_for(s) {
        return Err("This configuration review uses different sample data".into());
    }
    let u = draft
        .conffile_upgrade
        .as_ref()
        .ok_or("Prepare a configuration review first")?;
    if goal(d, &u.package).is_none_or(|g| g.incoming_archive != u.incoming_archive) {
        return Err("This activity does not admit that configuration upgrade".into());
    }
    s.conffile_review_current(u)
        .map_err(|e| notice(e.message))?;
    let before = &s.catalog().archives[&u.expected_archive];
    let incoming = &s.catalog().archives[&u.incoming_archive];
    let row = u
        .files
        .iter()
        .find(|r| r.path == path)
        .ok_or("Unknown configuration review row")?;
    if core::conffile_disposition(
        &before.files[path].contents,
        &row.expected_local,
        &incoming.files[path].contents,
    ) != core::ConffileDisposition::Conflict
    {
        return Err("This file has an automatic outcome and needs no choice".into());
    }
    let mut next = draft.clone();
    next.conffile_upgrade
        .as_mut()
        .unwrap()
        .files
        .iter_mut()
        .find(|r| r.path == path)
        .unwrap()
        .decision = decision;
    if !next.valid_for(s) {
        return Err("Configuration choice exceeds saved-draft limits".into());
    }
    *draft = next;
    Ok(())
}
