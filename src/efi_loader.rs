//! Bounded EFI installation and firmware model over supplied GPT and FAT32 facts.
//! The model never opens a real device, file, EFI variable or boot executable.
use crate::{
    bios_loader::Directory,
    boot_entry, grub_menu,
    partition_fs::{self as disk, fat},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
mod commands;
mod removable;
pub use commands::HELP;
pub use removable::MediaTrace;

const MAX_ESPS: usize = 4;
const MAX_FILES: usize = 16;
const MAX_OPTIONS: usize = 8;
const MAX_IMAGE_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub writable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Esp {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub disk: String,
    pub number: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub partition_guid: String,
    pub mount: Option<Mount>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub removable: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub geometry: disk::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Esp,4>")]
    pub esps: Vec<Esp>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Directory,8>")]
    pub boot_directories: Vec<Directory>,
    pub grub_image_bytes: u64,
    pub firmware_writable: bool,
}

fn guid(text: &str) -> bool {
    text.len() == 36
        && text.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_digit() || matches!(b, b'a'..=b'f')
            }
        })
        && text.bytes().any(|b| b != b'0' && b != b'-')
}

fn description(text: &str) -> bool {
    !text.trim().is_empty() && text.len() <= 64 && text.bytes().all(|b| (b' '..=b'~').contains(&b))
}

impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        self.geometry.validate().map_err(|e| e.to_string())?;
        if self.esps.is_empty()
            || self.esps.len() > MAX_ESPS
            || self.boot_directories.is_empty()
            || self.boot_directories.len() > 8
            || !(1..=MAX_IMAGE_BYTES).contains(&self.grub_image_bytes)
        {
            return Err(
                "Supply bounded EFI partitions, boot directories and an image size.".into(),
            );
        }
        let mut identities = BTreeSet::new();
        let mut partitions = BTreeSet::new();
        let mut mounts = BTreeSet::new();
        for esp in &self.esps {
            if !guid(&esp.partition_guid)
                || !identities.insert(&esp.partition_guid)
                || !partitions.insert((&esp.disk, esp.number))
            {
                return Err("EFI partitions need unique supplied GPT partition identities.".into());
            }
            self.volume(esp)?;
            if let Some(mount) = &esp.mount
                && (!boot_entry::path(&mount.path) || !mounts.insert(&mount.path))
            {
                return Err("EFI mount paths must be normalized and unique.".into());
            }
        }
        for directory in &self.boot_directories {
            if !boot_entry::path(&directory.path) || !mounts.insert(&directory.path) {
                return Err(
                    "Boot directory paths must be normalized and distinct from EFI mounts.".into(),
                );
            }
            let partition = self.geometry.disks.get(&directory.disk).and_then(|disk| {
                disk.partitions()
                    .find(|part| part.number == directory.partition)
            });
            if !partition.is_some_and(|part| matches!(part.content, disk::Content::Ext4 { .. })) {
                return Err("A supplied GRUB boot directory needs its ext4 filesystem.".into());
            }
            if let Some(config) = &directory.configuration {
                config.inventory.validate().map_err(|e| e.to_string())?;
                grub_menu::Menu::parse(&config.menu).map_err(|e| e.to_string())?;
                for root in &config.inventory.roots {
                    if !self.geometry.disks.values().flat_map(|disk| disk.partitions()).any(|part| {
                        matches!(&part.content, disk::Content::Ext4 { uuid, .. } if uuid.eq_ignore_ascii_case(&root.uuid))
                    }) {
                        return Err("A supplied GRUB root must identify an ext4 filesystem in this machine.".into());
                    }
                }
            }
        }
        Ok(())
    }

    fn volume(&self, esp: &Esp) -> Result<&fat::Volume, String> {
        let partition = self
            .geometry
            .disks
            .get(&esp.disk)
            .and_then(|disk| disk.table.as_ref())
            .and_then(|table| {
                table
                    .partitions
                    .iter()
                    .find(|part| part.number == esp.number)
            })
            .ok_or("This EFI location must identify a supplied GPT partition.")?;
        if partition.type_hint != disk::TypeHint::EfiSystem {
            return Err(
                "The supplied partition does not have the EFI System Partition type.".into(),
            );
        }
        match &partition.content {
            disk::Content::Fat32 { volume } => Ok(volume),
            _ => Err(
                "The supplied EFI System Partition does not contain a modeled FAT32 volume.".into(),
            ),
        }
    }

    fn esp(&self, guid: &str) -> Result<&Esp, String> {
        self.esps
            .iter()
            .find(|esp| esp.partition_guid == guid)
            .ok_or_else(|| "This EFI partition identity is outside the supplied machine.".into())
    }

    fn directory(&self, path: &str) -> Result<&Directory, String> {
        self.boot_directories
            .iter()
            .find(|directory| directory.path == path)
            .ok_or_else(|| "Use a supplied GRUB boot directory.".into())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FileKind {
    Data,
    OtherApplication,
    Grub {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        boot_directory: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct File {
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub partition_guid: String,
    pub path: fat::Path,
    pub bytes: u64,
    pub kind: FileKind,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionEntry {
    pub number: u16,
    pub active: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub partition_guid: String,
    pub loader: fat::Path,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,File,16>")]
    pub files: Vec<File>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,OptionEntry,8>")]
    pub options: Vec<OptionEntry>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u16,8>")]
    pub boot_order: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootAttempt {
    pub number: u16,
    pub outcome: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trace {
    pub attempts: Vec<BootAttempt>,
    pub started: Option<u16>,
    pub handoff: Option<boot_entry::Report>,
    pub outcome: String,
}

impl World {
    pub fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || self.files.len() > MAX_FILES
            || self.options.len() > MAX_OPTIONS
            || self.boot_order.len() > MAX_OPTIONS
        {
            return Err("Saved EFI state exceeds the supported version or size.".into());
        }
        let mut paths = BTreeSet::new();
        for file in &self.files {
            if !guid(&file.partition_guid)
                || !paths.insert((&file.partition_guid, &file.path))
                || !(1..=MAX_IMAGE_BYTES).contains(&file.bytes)
            {
                return Err(
                    "EFI files need supplied partition identities, unique paths and bounded sizes."
                        .into(),
                );
            }
            if file.kind != FileKind::Data && !file.path.is_application() {
                return Err("A supplied EFI application needs an .efi path.".into());
            }
            if let FileKind::Grub { boot_directory } = &file.kind
                && !boot_entry::path(boot_directory)
            {
                return Err("A GRUB application needs a normalized boot directory.".into());
            }
        }
        let mut entries = BTreeSet::new();
        for entry in &self.options {
            if !guid(&entry.partition_guid)
                || !entries.insert(entry.number)
                || !description(&entry.label)
                || !entry.loader.is_application()
            {
                return Err("Firmware entries need unique numbers, readable labels and EFI application paths.".into());
            }
        }
        let order = self.boot_order.iter().copied().collect::<BTreeSet<_>>();
        if order.len() != self.boot_order.len() || !order.is_subset(&entries) {
            return Err("BootOrder must list known option numbers without duplicates.".into());
        }
        Ok(())
    }

    pub fn bound(&self, facts: &Facts) -> Result<(), String> {
        self.shape()?;
        facts.validate()?;
        let mut allocations = BTreeMap::<&str, u64>::new();
        for file in &self.files {
            let volume = facts.volume(facts.esp(&file.partition_guid)?)?;
            if let FileKind::Grub { boot_directory } = &file.kind {
                facts.directory(boot_directory)?;
                if file.bytes != facts.grub_image_bytes {
                    return Err("The GRUB application must match the supplied image size.".into());
                }
            }
            let used = allocations.entry(&file.partition_guid).or_default();
            *used += volume
                .allocation_bytes(file.bytes)
                .ok_or("Invalid cluster size.")?;
            // A fixed reservation for directory clusters is part of the supplied
            // profile. The model tracks payload allocations, not raw directories.
            let directory_reservation =
                volume.allocation_bytes(1).ok_or("Invalid cluster size.")? * 64;
            if *used + directory_reservation > volume.data_bytes() {
                return Err("The supplied FAT32 partition lacks room for the loader files.".into());
            }
        }
        for entry in &self.options {
            facts.esp(&entry.partition_guid)?;
        }
        Ok(())
    }

    /// Models a fixed x86_64-efi installation with --no-nvram.
    pub fn install(
        &mut self,
        facts: &Facts,
        esp_mount: &str,
        boot_directory: &str,
        loader_id: &str,
    ) -> Result<fat::Path, String> {
        if loader_id.is_empty()
            || loader_id.len() > 32
            || !loader_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return Err("Use a loader ID of 1–32 letters, digits, underscores or hyphens.".into());
        }
        let path = fat::Path::parse(&format!("/EFI/{loader_id}/grubx64.efi"))?;
        self.write_loader(facts, esp_mount, boot_directory, path, false)
    }

    fn write_loader(
        &mut self,
        facts: &Facts,
        esp_mount: &str,
        boot_directory: &str,
        path: fat::Path,
        removable: bool,
    ) -> Result<fat::Path, String> {
        self.bound(facts)?;
        let esp = facts
            .esps
            .iter()
            .find(|esp| {
                esp.mount
                    .as_ref()
                    .is_some_and(|mount| mount.path == esp_mount)
            })
            .ok_or("Choose a supplied mounted FAT32 EFI System Partition.")?;
        if removable && !esp.removable {
            return Err(
                "The selected EFI mount is on a fixed disk. Choose the supplied removable device."
                    .into(),
            );
        }
        let directory = facts.directory(boot_directory)?;
        if !esp.mount.as_ref().is_some_and(|mount| mount.writable)
            || !facts.geometry.disks[&esp.disk].writable
            || !directory.writable
            || !facts.geometry.disks[&directory.disk].writable
        {
            return Err(
                "Installation needs a writable EFI mount and a writable supplied boot directory."
                    .into(),
            );
        }
        let file = File {
            partition_guid: esp.partition_guid.clone(),
            path: path.clone(),
            bytes: facts.grub_image_bytes,
            kind: FileKind::Grub {
                boot_directory: boot_directory.into(),
            },
        };
        let mut next = self.clone();
        if let Some(old) = next
            .files
            .iter_mut()
            .find(|old| old.partition_guid == file.partition_guid && old.path == path)
        {
            if !matches!(old.kind, FileKind::Grub { .. }) {
                return Err("That EFI path contains a different supplied file. Choose a distinct loader ID.".into());
            }
            *old = file;
        } else {
            next.files.push(file);
        }
        next.bound(facts)?;
        *self = next;
        Ok(path)
    }

    /// A dangling path is representable. The later boot test checks the file.
    pub fn register(
        &mut self,
        facts: &Facts,
        disk: &str,
        partition: u8,
        label: &str,
        loader: fat::Path,
    ) -> Result<u16, String> {
        self.bound(facts)?;
        if !facts.firmware_writable {
            return Err("Firmware entries are read-only in this supplied machine.".into());
        }
        if self.options.len() >= MAX_OPTIONS {
            return Err("The supplied firmware entry limit is reached.".into());
        }
        let esp = facts
            .esps
            .iter()
            .find(|esp| esp.disk == disk && esp.number == partition)
            .ok_or("Choose the supplied disk and partition that contain the EFI loader.")?;
        let number = (0..=MAX_OPTIONS as u16)
            .find(|n| self.options.iter().all(|entry| entry.number != *n))
            .expect("at least one bounded entry number is unused");
        let mut next = self.clone();
        next.options.push(OptionEntry {
            number,
            active: true,
            label: label.into(),
            partition_guid: esp.partition_guid.clone(),
            loader,
        });
        next.boot_order.insert(0, number);
        next.bound(facts)?;
        *self = next;
        Ok(number)
    }

    pub fn reorder(&mut self, facts: &Facts, order: &[u16]) -> Result<(), String> {
        self.bound(facts)?;
        if !facts.firmware_writable {
            return Err("Firmware entries are read-only in this supplied machine.".into());
        }
        if order.is_empty() || order.len() > MAX_OPTIONS {
            return Err("Use one to eight supplied entry numbers.".into());
        }
        let mut next = self.clone();
        next.boot_order = order.to_vec();
        next.bound(facts)?;
        *self = next;
        Ok(())
    }

    pub fn trace(&self, facts: &Facts) -> Result<Trace, String> {
        self.bound(facts)?;
        let mut trace = Trace {
            attempts: Vec::new(),
            started: None,
            handoff: None,
            outcome: String::new(),
        };
        for number in &self.boot_order {
            let entry = self
                .options
                .iter()
                .find(|entry| entry.number == *number)
                .expect("validated entry");
            let file = self.files.iter().find(|file| {
                file.partition_guid == entry.partition_guid && file.path == entry.loader
            });
            let outcome = if !entry.active {
                "Entry inactive"
            } else if file.is_none() {
                "EFI application missing on the specified partition"
            } else if file.is_some_and(|f| f.kind == FileKind::Data) {
                "Path does not contain a supplied EFI application"
            } else {
                "Firmware starts the supplied application"
            };
            trace.attempts.push(BootAttempt {
                number: *number,
                outcome: outcome.into(),
            });
            let Some(file) = file.filter(|file| entry.active && file.kind != FileKind::Data) else {
                continue;
            };
            trace.started = Some(*number);
            if let FileKind::Grub { boot_directory } = &file.kind {
                if let Some(config) = &facts.directory(boot_directory)?.configuration {
                    trace.handoff = Some(
                        grub_menu::test(&config.inventory, &config.menu)
                            .map_err(|e| e.to_string())?,
                    );
                    trace.outcome = "Started GRUB and followed its supplied menu.".into();
                } else {
                    trace.outcome =
                        "Started GRUB, but its boot directory has no supplied menu.".into();
                }
            } else {
                trace.outcome = "Firmware started the other supplied application.".into();
            }
            return Ok(trace);
        }
        trace.outcome = "No active BootOrder entry started a supplied EFI application.".into();
        Ok(trace)
    }
}
