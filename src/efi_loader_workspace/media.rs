//! Assessment for independent removable-media recovery.
use super::{Draft, Task};
use crate::{efi_loader::World, partition_fs::fat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MediaPolicy {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub unavailable_disk: String,
}

impl Task {
    pub(super) fn media_shape(&self, policy: &MediaPolicy) -> Result<(), String> {
        let media = self
            .facts
            .esps
            .iter()
            .find(|esp| esp.partition_guid == self.required_esp && esp.removable)
            .ok_or("Supply the intended removable EFI partition.")?;
        if media.disk == policy.unavailable_disk
            || !self
                .facts
                .geometry
                .disks
                .contains_key(&policy.unavailable_disk)
        {
            return Err(
                "The unavailable internal disk must be distinct from the removable device.".into(),
            );
        }
        let protected = self.archive()?;
        if !self.facts.esps.iter().any(|esp| {
            !esp.removable
                && esp.disk == policy.unavailable_disk
                && esp.partition_guid == protected.partition_guid
        }) {
            return Err(
                "The protected normal startup entry must belong to the supplied internal disk."
                    .into(),
            );
        }
        let initial = self.initial.trace(&self.facts)?;
        if initial.started != Some(self.archive_entry)
            || !initial
                .handoff
                .as_ref()
                .is_some_and(|report| report.passed())
        {
            return Err(
                "The supplied normal startup path must work before the recovery repair.".into(),
            );
        }
        Ok(())
    }

    pub(super) fn media_checks(&self, draft: &Draft, policy: &MediaPolicy) -> Vec<(bool, String)> {
        let normal = match draft.current.trace(&self.facts) {
            Ok(report) => report,
            Err(error) => return vec![(false, error)],
        };
        let recovery = match draft.current.trace_removable(
            &self.facts,
            &self.required_esp,
            Some(&policy.unavailable_disk),
        ) {
            Ok(report) => report,
            Err(error) => return vec![(false, error)],
        };
        let mut checks = Vec::new();
        let recovery_passed = recovery.passed();
        checks.push((recovery_passed, recovery.outcome));
        if let Some(report) = &recovery.handoff {
            if let Some(stage) = report.stages.iter().find(|stage| !stage.passed) {
                checks.push((false, format!("{}: {}", stage.title, stage.detail)));
            }
            let mode = if report.read_only { "ro" } else { "rw" };
            let required_mode = if self.read_only { "ro" } else { "rw" };
            checks.extend([
                (
                    report
                        .requested
                        .root
                        .eq_ignore_ascii_case(&self.required_root),
                    format!(
                        "The recovery menu requests root {}. The task requires {}.",
                        report.requested.root, self.required_root
                    ),
                ),
                (
                    report.release.as_deref() == Some(self.required_release.as_str()),
                    format!(
                        "Verify that the recovery kernel release is {}.",
                        self.required_release
                    ),
                ),
                (
                    mode == required_mode,
                    format!("The recovery root mode is {mode}. The task requires {required_mode}."),
                ),
                (
                    report.requested.target == self.target,
                    format!(
                        "The recovery menu requests {}. The task requires {}.",
                        report.requested.target, self.target
                    ),
                ),
            ]);
        }
        let normal_passed = normal.started == Some(self.archive_entry)
            && normal
                .handoff
                .as_ref()
                .is_some_and(|report| report.passed());
        checks.push((
            normal_passed,
            if normal_passed {
                format!("Normal startup still reaches Linux through Boot{:04X}.", self.archive_entry)
            } else {
                format!("Normal startup must still reach Linux through Boot{:04X}. Restore its original entry, order and files.", self.archive_entry)
            },
        ));
        let same_entries = draft.current.options == self.initial.options
            && draft.current.boot_order == self.initial.boot_order;
        checks.push((
            same_entries,
            if same_entries {
                "The original firmware entries and BootOrder are intact.".into()
            } else {
                "Keep the original firmware entries and BootOrder. The selected removable device uses its default application path.".into()
            },
        ));
        let default = fat::Path::parse("/EFI/BOOT/BOOTX64.EFI").expect("fixed default path");
        let preserved = |world: &World| {
            world
                .files
                .iter()
                .filter(|file| file.partition_guid != self.required_esp || file.path != default)
                .map(|file| {
                    (
                        (file.partition_guid.clone(), file.path.clone()),
                        file.clone(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        let same_files = preserved(&draft.current) == preserved(&self.initial);
        checks.push((
            same_files,
            if same_files {
                "All files outside the required removable loader path are intact.".into()
            } else {
                "Preserve every file outside the removable default loader path. Reset this attempt if you changed another file.".into()
            },
        ));
        checks
    }
}
