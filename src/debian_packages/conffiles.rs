//! Explicit, bounded per-file decisions. No host operations or force-option parser.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConffileDecision {
    KeepLocal,
    InstallNew,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConffileSelection {
    #[serde(deserialize_with = "scalar::<_,160>")]
    pub path: String,
    #[serde(deserialize_with = "scalar::<_,2048>")]
    pub expected_local: String,
    pub decision: Option<ConffileDecision>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConffileUpgrade {
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub package: String,
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub expected_archive: String,
    #[serde(deserialize_with = "scalar::<_,64>")]
    pub incoming_archive: String,
    #[serde(deserialize_with = "list::<_,ConffileSelection,4>")]
    pub files: Vec<ConffileSelection>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConffileDisposition {
    MatchesIncoming,
    UpdateUnmodified,
    KeepLocalOnly,
    Conflict,
}
pub fn conffile_disposition(old: &str, local: &str, incoming: &str) -> ConffileDisposition {
    if local == incoming {
        ConffileDisposition::MatchesIncoming
    } else if local == old {
        ConffileDisposition::UpdateUnmodified
    } else if incoming == old {
        ConffileDisposition::KeepLocalOnly
    } else {
        ConffileDisposition::Conflict
    }
}
impl ConffileUpgrade {
    pub fn valid_shape(&self) -> bool {
        name(&self.package)
            && id(&self.expected_archive)
            && id(&self.incoming_archive)
            && self.expected_archive != self.incoming_archive
            && !self.files.is_empty()
            && self.files.len() <= 4
            && self.files.iter().enumerate().all(|(n, r)| {
                path(&r.path)
                    && r.expected_local.len() <= 2048
                    && !r.expected_local.contains('\0')
                    && !self.files[..n].iter().any(|p| p.path == r.path)
            })
            && within(self, 16_384)
    }
}
impl State {
    /// Prepare an unchecked native draft. Choices stay empty; this never mutates.
    pub fn review_conffiles(&self, package: &str) -> Result<ConffileUpgrade> {
        self.validate()?;
        if !name(package) {
            return Err(error(ErrorCode::Invalid, "Invalid package name"));
        }
        let old = self
            .runtime
            .records
            .get(package)
            .filter(|r| r.status == Status::Installed)
            .ok_or_else(|| {
                error(
                    ErrorCode::Conffile,
                    "Choose an installed package for configuration review",
                )
            })?;
        let incoming = self.candidate(&Request {
            name: package.into(),
            version: None,
        })?;
        let before = &self.catalog.archives[&old.archive];
        let count = before.files.values().filter(|v| v.conffile).count();
        if !(1..=4).contains(&count) {
            return Err(error(
                ErrorCode::Limit,
                "Review needs1..4 managed configuration files",
            ));
        }
        let files=before.files.iter().filter(|(_,v)|v.conffile).map(|(path,_)|{
            let text=self.runtime.files.get(path).ok_or_else(||error(ErrorCode::Unsupported,"Deleted conffile decisions are outside this activity; previous state retained"))?;
            if text.len()>2048{return Err(error(ErrorCode::Limit,"Local configuration snapshot exceeds2048bytes"));}
            Ok(ConffileSelection{path:path.clone(),expected_local:text.clone(),decision:None})
        }).collect::<Result<Vec<_>>>()?;
        let draft = ConffileUpgrade {
            package: package.into(),
            expected_archive: old.archive.clone(),
            incoming_archive: incoming.into(),
            files,
        };
        self.validate_conffile_upgrade(&draft, false, &mut Budget(MAX_WORK))?;
        Ok(draft)
    }
    /// Freshness/admission check for an editable draft; empty choices are allowed.
    pub fn conffile_review_current(&self, u: &ConffileUpgrade) -> Result<()> {
        self.validate()?;
        self.validate_conffile_upgrade(u, false, &mut Budget(MAX_WORK))
    }
    pub(super) fn validate_conffile_upgrade(
        &self,
        u: &ConffileUpgrade,
        require_choices: bool,
        budget: &mut Budget,
    ) -> Result<()> {
        if !u.valid_shape() {
            return Err(error(
                ErrorCode::Limit,
                "Configuration review needs1..4 unique bounded file snapshots and two archive identities",
            ));
        }
        let record = self
            .runtime
            .records
            .get(&u.package)
            .filter(|r| r.status == Status::Installed && r.archive == u.expected_archive)
            .ok_or_else(|| {
                error(
                    ErrorCode::Conffile,
                    "Configuration review is stale: installed package changed; review again",
                )
            })?;
        let candidate = self.candidate(&Request {
            name: u.package.clone(),
            version: None,
        })?;
        if candidate != u.incoming_archive {
            return Err(error(
                ErrorCode::Conffile,
                "Configuration review is stale: incoming candidate changed; review again",
            ));
        }
        let old = &self.catalog.archives[&record.archive];
        let new = &self.catalog.archives[candidate];
        if old.name != u.package
            || new.name != u.package
            || !new.version.compare(&old.version).is_gt()
        {
            return Err(error(
                ErrorCode::Unsupported,
                "Review requires a newer cached package candidate; refresh the supplied metadata if needed",
            ));
        }
        let before: BTreeSet<_> = old
            .files
            .iter()
            .filter(|(_, v)| v.conffile)
            .map(|(p, _)| p.as_str())
            .collect();
        let after: BTreeSet<_> = new
            .files
            .iter()
            .filter(|(_, v)| v.conffile)
            .map(|(p, _)| p.as_str())
            .collect();
        let snapshots: BTreeSet<_> = u.files.iter().map(|r| r.path.as_str()).collect();
        if before != after || before != snapshots {
            return Err(error(
                ErrorCode::Unsupported,
                "Review must cover exactly the unchanged managed configuration paths",
            ));
        }
        // Bound traversal once. An admitted extra path is not ownership authority.
        let mut manifest = BTreeSet::new();
        for archive in self.catalog.archives.values() {
            for p in archive.files.keys() {
                budget.charge()?;
                manifest.insert(p.as_str());
            }
        }
        let mut proposed = BTreeSet::new();
        for row in &u.files {
            budget.charge()?;
            let live = self.runtime.files.get(&row.path).ok_or_else(|| {
                error(
                    ErrorCode::Unsupported,
                    "Deleted conffile decisions are outside this activity",
                )
            })?;
            if live != &row.expected_local {
                return Err(error(
                    ErrorCode::Conffile,
                    "Configuration review is stale: local text changed; review again",
                ));
            }
            let conflict = conffile_disposition(
                &old.files[&row.path].contents,
                live,
                &new.files[&row.path].contents,
            ) == ConffileDisposition::Conflict;
            if !conflict && row.decision.is_some() {
                return Err(error(
                    ErrorCode::Conffile,
                    "This file has an automatic outcome; remove its unnecessary choice",
                ));
            }
            if conflict && require_choices && row.decision.is_none() {
                return Err(error(
                    ErrorCode::Conffile,
                    format!(
                        "{}: choose whether to keep local settings or install the incoming default",
                        row.path
                    ),
                ));
            }
            for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
                budget.charge()?;
                if row.path.len() + suffix.len() > 160 {
                    return Err(error(
                        ErrorCode::Limit,
                        "Configuration backup path exceeds the160-byte path bound",
                    ));
                }
                let key = format!("{}{suffix}", row.path);
                if !path(&key)
                    || manifest.contains(key.as_str())
                    || !proposed.insert(key.clone())
                    || self.runtime.files.contains_key(&key)
                {
                    return Err(error(
                        ErrorCode::Collision,
                        "Existing or protected configuration backup neighbor is outside this clean-sidecar activity; no changes applied",
                    ));
                }
                if matches!(suffix, ".dpkg-old" | ".dpkg-dist")
                    && !self.catalog.extra_paths.contains(&key)
                {
                    return Err(error(
                        ErrorCode::Unsupported,
                        "Configuration backup path is not admitted by this scenario",
                    ));
                }
            }
        }
        Ok(())
    }
    fn admitted_conffile_sidecar(&self, file: &str, suffix: &str) -> Result<&str> {
        let key = format!("{file}{suffix}");
        self.catalog
            .extra_paths
            .get(key.as_str())
            .map(String::as_str)
            .ok_or_else(|| {
                error(
                    ErrorCode::Unsupported,
                    "Configuration backup path is not admitted by this scenario",
                )
            })
    }
    pub(super) fn install_reviewed_conffile<'a>(
        &'a self,
        p: &mut Prepared<'a>,
        path: &'a str,
        old: &'a Payload,
        incoming: &'a Payload,
        u: &ConffileUpgrade,
    ) -> Result<()> {
        let local = self.runtime.files.get(path).ok_or_else(|| {
            error(
                ErrorCode::Unsupported,
                "Deleted conffile decisions are outside this activity",
            )
        })?;
        match conffile_disposition(&old.contents, local, &incoming.contents) {
            ConffileDisposition::MatchesIncoming | ConffileDisposition::KeepLocalOnly => {}
            ConffileDisposition::UpdateUnmodified => {
                p.files.insert(path, &incoming.contents);
            }
            ConffileDisposition::Conflict => {
                let row = u
                    .files
                    .iter()
                    .find(|r| r.path == path)
                    .ok_or_else(|| error(ErrorCode::Conffile, "Review snapshot missing"))?;
                match row.decision.ok_or_else(|| {
                    error(
                        ErrorCode::Conffile,
                        "Explicit configuration choice required",
                    )
                })? {
                    ConffileDecision::KeepLocal => {
                        let side = self.admitted_conffile_sidecar(path, ".dpkg-dist")?;
                        p.files.insert(side, &incoming.contents);
                    }
                    ConffileDecision::InstallNew => {
                        let side = self.admitted_conffile_sidecar(path, ".dpkg-old")?;
                        p.files.insert(side, local);
                        p.files.insert(path, &incoming.contents);
                    }
                }
            }
        }
        Ok(())
    }
}
