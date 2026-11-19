//! Explicit x64 removable-media selection over supplied EFI state.
use super::{Facts, FileKind, World, fat};
use crate::{boot_entry, grub_menu, partition_fs as disk};

const DEFAULT_APPLICATION: &str = "/EFI/BOOT/BOOTX64.EFI";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaTrace {
    pub application_started: bool,
    pub handoff: Option<boot_entry::Report>,
    pub outcome: String,
}
impl MediaTrace {
    fn blocked(application_started: bool, reason: impl Into<String>) -> Self {
        Self {
            application_started,
            handoff: None,
            outcome: reason.into(),
        }
    }
    pub fn passed(&self) -> bool {
        self.handoff
            .as_ref()
            .is_some_and(boot_entry::Report::passed)
    }
}

impl World {
    pub fn install_removable(
        &mut self,
        facts: &Facts,
        esp_mount: &str,
        boot_directory: &str,
    ) -> Result<fat::Path, String> {
        let path = fat::Path::parse(DEFAULT_APPLICATION).expect("fixed EFI application path");
        self.write_loader(facts, esp_mount, boot_directory, path, true)
    }

    /// Select the supplied removable device with the given test disk absent.
    /// This observation leaves normal BootOrder and the editable machine intact.
    pub fn trace_removable(
        &self,
        facts: &Facts,
        partition_guid: &str,
        unavailable_disk: Option<&str>,
    ) -> Result<MediaTrace, String> {
        self.bound(facts)?;
        let media = facts.esp(partition_guid)?;
        if !media.removable {
            return Err("Select a supplied removable EFI partition for this test.".into());
        }
        if unavailable_disk.is_some_and(|name| !facts.geometry.disks.contains_key(name)) {
            return Err("The absent disk must be part of the supplied machine.".into());
        }
        if unavailable_disk == Some(media.disk.as_str()) {
            return Ok(MediaTrace::blocked(
                false,
                "The selected removable device is unavailable.",
            ));
        }
        let path = fat::Path::parse(DEFAULT_APPLICATION).expect("fixed EFI application path");
        let Some(file) = self
            .files
            .iter()
            .find(|file| file.partition_guid == media.partition_guid && file.path == path)
        else {
            return Ok(MediaTrace::blocked(
                false,
                "This removable partition lacks the default x64 application at \\EFI\\BOOT\\BOOTX64.EFI.",
            ));
        };
        let FileKind::Grub { boot_directory } = &file.kind else {
            return Ok(MediaTrace::blocked(
                file.kind == FileKind::OtherApplication,
                "The default path does not contain the supplied GRUB loader.",
            ));
        };
        let directory = facts.directory(boot_directory)?;
        if unavailable_disk == Some(directory.disk.as_str()) {
            return Ok(MediaTrace::blocked(
                true,
                format!(
                    "GRUB starts from removable media, but {boot_directory} is on the unavailable disk {}.",
                    directory.disk
                ),
            ));
        }
        let Some(configuration) = &directory.configuration else {
            return Ok(MediaTrace::blocked(
                true,
                format!("GRUB starts, but {boot_directory} has no supplied menu."),
            ));
        };
        let menu =
            grub_menu::Menu::parse(&configuration.menu).map_err(|error| error.to_string())?;
        let entry = &menu.selected().map_err(|error| error.to_string())?.entry;
        let report = boot_entry::trace_entry_with_root_availability(
            &configuration.inventory,
            entry,
            |root| {
                facts.geometry.disks.iter().any(|(name, disk)| {
                    unavailable_disk != Some(name.as_str())
                        && disk.partitions().any(|part| {
                            matches!(&part.content, disk::Content::Ext4 { uuid, .. } if uuid.eq_ignore_ascii_case(&root.uuid))
                        })
                })
            },
        ).map_err(|error| error.to_string())?;
        let outcome = if report.passed() {
            "The selected removable application reaches the supplied Linux recovery system."
        } else {
            "GRUB starts from removable media, but the Linux recovery handoff stops."
        };
        Ok(MediaTrace {
            application_started: true,
            handoff: Some(report),
            outcome: outcome.into(),
        })
    }
}
