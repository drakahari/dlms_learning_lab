//! Pure, regular-file metadata repair over declared RPM records.
//! No payload replacement, archive parsing, accounts, filesystem or host utility.
use super::{Code, File, Identity, Result, State, fail};

/// Compare only the metadata represented by this finite regular-file model.
/// This deliberately does not claim content/digest/capability or full rpm -V checks.
pub fn same_recorded_metadata(actual: &File, expected: &File) -> bool {
    actual.mode == expected.mode
        && actual.owner == expected.owner
        && actual.group == expected.group
        && actual.mtime == expected.mtime
}

impl State {
    /// Return metadata repaired from the *installed* record, retaining all text.
    /// Missing payloads refuse before a candidate clone: real RPM's partial repair
    /// and file capabilities are explicitly outside this model, not approximated.
    pub fn restored_metadata(&self, package: &Identity) -> Result<Self> {
        let Some(record) = self.installed(package)? else {
            return fail(
                Code::NotInstalled,
                "No installed package record for metadata restoration",
            );
        };
        if record
            .files
            .keys()
            .any(|p| !self.runtime.files.contains_key(p))
        {
            return fail(
                Code::UnsupportedSubset,
                "Missing payload: this bounded metadata-only operation cannot recreate a file or model RPM's partial restore. Previous state retained; inspect the supplied exact archive for content repair.",
            );
        }
        let mut runtime = self.runtime.clone();
        for (path, payload) in &record.files {
            let live = runtime
                .files
                .get_mut(path)
                .expect("all payload paths preflighted");
            live.mode = payload.file.mode;
            live.owner.clone_from(&payload.file.owner);
            live.group.clone_from(&payload.file.group);
            live.mtime = payload.file.mtime;
        }
        Self::new(self.catalog.clone(), runtime)
    }
}
