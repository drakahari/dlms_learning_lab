//! Bounded versioned virtual placements, composed with the existing boot model.
//! Paths are logical keys only. No host files, installers, boot or subprocesses.
use crate::{boot_entry as boot, kernel_build as core, source_install};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageFile {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub build: core::Build,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModulesTree {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub build: core::Build,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitramfsFile {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    pub image: core::Initramfs,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryFile {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub source: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ImageFile,8>")]
    pub images: Vec<ImageFile>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ModulesTree,8>")]
    pub modules: Vec<ModulesTree>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,InitramfsFile,8>")]
    pub initramfs: Vec<InitramfsFile>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,EntryFile,8>")]
    pub entries: Vec<EntryFile>,
}

impl Store {
    pub fn validate(&self) -> Result<()> {
        if [
            self.images.len(),
            self.modules.len(),
            self.initramfs.len(),
            self.entries.len(),
        ]
        .iter()
        .any(|n| *n > 8)
        {
            return Err(
                "The supplied staging area supports at most eight items of each kind.".into(),
            );
        }
        let all: Vec<&str> = self
            .images
            .iter()
            .map(|f| f.path.as_str())
            .chain(self.modules.iter().map(|f| f.path.as_str()))
            .chain(self.initramfs.iter().map(|f| f.path.as_str()))
            .chain(self.entries.iter().map(|f| f.path.as_str()))
            .collect();
        let mut unique = BTreeSet::new();
        for path in &all {
            if !boot::path(path) || !unique.insert(*path) {
                return Err("Staged items need distinct canonical virtual paths without .. or empty components.".into());
            }
        }
        let files: Vec<&str> = self
            .images
            .iter()
            .map(|f| f.path.as_str())
            .chain(self.initramfs.iter().map(|f| f.path.as_str()))
            .chain(self.entries.iter().map(|f| f.path.as_str()))
            .collect();
        for file in files {
            if all.iter().any(|p| p.starts_with(&format!("{file}/"))) {
                return Err(
                    "A staged regular file cannot also contain other files or module directories."
                        .into(),
                );
            }
        }
        for file in &self.images {
            file.build.validate_shape()?;
            if !file.build.image {
                return Err("A staged image needs its actual image-build record.".into());
            }
        }
        for tree in &self.modules {
            tree.build.validate_shape()?;
            if !tree.build.modules
                || !tree
                    .path
                    .ends_with(&format!("/lib/modules/{}", tree.build.release))
            {
                return Err("Install built modules under the matching virtual lib/modules/release directory.".into());
            }
        }
        for file in &self.initramfs {
            file.image.validate_shape()?;
        }
        for file in &self.entries {
            if file.source.len() > 4096
                || file
                    .source
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            {
                return Err("A virtual boot entry is limited to 4096 plain text bytes.".into());
            }
        }
        Ok(())
    }

    fn transaction<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.validate()?;
        let mut next = self.clone();
        let result = f(&mut next)?;
        next.validate()?;
        *self = next;
        Ok(result)
    }

    /// Versioned placement helper, not a claim of general make-install/cp behavior.
    pub fn image(&mut self, prefix: &str, build: &core::Build) -> Result<String> {
        build.validate_shape()?;
        if !build.image {
            return Err("Build the image before staging it.".into());
        }
        let path = source_install::join(prefix, &format!("boot/vmlinuz-{}", build.release))?;
        self.transaction(|s| {
            let value = ImageFile {
                path: path.clone(),
                build: build.clone(),
            };
            if let Some(old) = s.images.iter_mut().find(|f| f.path == path) {
                *old = value;
            } else {
                s.images.push(value);
            }
            Ok(path)
        })
    }

    pub fn modules(&mut self, prefix: &str, build: &core::Build) -> Result<String> {
        build.validate_shape()?;
        if !build.modules {
            return Err("Build the matching modules before installing their virtual tree.".into());
        }
        let path = source_install::join(prefix, &format!("lib/modules/{}", build.release))?;
        self.transaction(|s| {
            let value = ModulesTree {
                path: path.clone(),
                build: build.clone(),
            };
            if let Some(old) = s.modules.iter_mut().find(|f| f.path == path) {
                *old = value;
            } else {
                s.modules.push(value);
            }
            Ok(path)
        })
    }

    pub fn assemble_initramfs(&mut self, prefix: &str, image_path: &str) -> Result<String> {
        self.validate()?;
        let root = source_install::join(prefix, "boot")?;
        if !image_path.starts_with(&format!("{root}/")) {
            return Err("Choose an image inside this supplied virtual boot directory.".into());
        }
        let kernel = self
            .images
            .iter()
            .find(|f| f.path == image_path)
            .ok_or("Stage the requested image before creating its modeled initramfs.")?;
        let module_path =
            source_install::join(prefix, &format!("lib/modules/{}", kernel.build.release))?;
        let modules = self
            .modules
            .iter()
            .find(|f| f.path == module_path)
            .map(|f| &f.build);
        if modules.is_none() && !kernel.build.config.loadable_inventory().is_empty() {
            return Err(format!(
                "Missing installed module tree {module_path}. Install the matching built modules into this virtual root before assembling its initramfs."
            ));
        }
        let image = core::Initramfs::from_builds(&kernel.build, modules)?;
        let path = source_install::join(
            prefix,
            &format!("boot/initramfs-{}.img", kernel.build.release),
        )?;
        self.transaction(|s| {
            let value = InitramfsFile {
                path: path.clone(),
                image,
            };
            if let Some(old) = s.initramfs.iter_mut().find(|f| f.path == path) {
                *old = value;
            } else {
                s.initramfs.push(value);
            }
            Ok(path)
        })
    }

    pub fn entry(&mut self, prefix: &str, filename: &str, source: &str) -> Result<String> {
        if filename.len() > 48
            || !filename.ends_with(".conf")
            || filename.contains('/')
            || !boot::path(&format!("/{filename}"))
        {
            return Err(
                "Use one virtual entry filename ending in .conf, without directories.".into(),
            );
        }
        if source.len() > 4096
            || source
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err("Keep the virtual boot-entry text within 4096 bytes without hidden control characters.".into());
        }
        let path = source_install::join(prefix, &format!("boot/loader/entries/{filename}"))?;
        self.transaction(|s| {
            let value = EntryFile {
                path: path.clone(),
                source: source.into(),
            };
            if let Some(old) = s.entries.iter_mut().find(|f| f.path == path) {
                *old = value;
            } else {
                s.entries.push(value);
            }
            Ok(path)
        })
    }

    pub fn inventory(&self, prefix: &str, root: boot::Root) -> Result<boot::Inventory> {
        self.validate()?;
        let boot_root = format!("{}/", source_install::join(prefix, "boot")?);
        let kernels = self
            .images
            .iter()
            .filter_map(|f| {
                f.path.strip_prefix(&boot_root).map(|leaf| boot::Kernel {
                    path: format!("/{leaf}"),
                    release: f.build.release.clone(),
                    builtin_drivers: f.build.config.builtin_inventory().into_iter().collect(),
                })
            })
            .collect();
        let initramfs = self
            .initramfs
            .iter()
            .filter_map(|f| {
                f.path.strip_prefix(&boot_root).map(|leaf| boot::Initramfs {
                    path: format!("/{leaf}"),
                    release: f.image.image_build.release.clone(),
                    drivers: f.image.included.iter().cloned().collect(),
                    uuid_lookup: true,
                })
            })
            .collect();
        let inventory = boot::Inventory {
            kernels,
            initramfs,
            roots: vec![root],
        };
        inventory.validate().map_err(|e| e.to_string())?;
        Ok(inventory)
    }

    pub fn test_entry(
        &self,
        prefix: &str,
        entry_path: &str,
        root: boot::Root,
    ) -> Result<boot::Report> {
        self.validate()?;
        let directory = source_install::join(prefix, "boot")?;
        if !entry_path.starts_with(&format!("{directory}/loader/entries/")) {
            return Err("Select an entry from this virtual boot directory.".into());
        }
        let file = self
            .entries
            .iter()
            .find(|f| f.path == entry_path)
            .ok_or("Save the selected virtual boot entry first.")?;
        let entry = boot::Entry::parse(&file.source).map_err(|e| e.to_string())?;
        let kernel_path = format!("{directory}{}", entry.linux);
        if let Some(kernel) = self.images.iter().find(|f| f.path == kernel_path) {
            if !entry.initrd.is_empty() && !kernel.build.config.initrd {
                return Err("This supplied image has external initramfs loading disabled; its selected initrd entry cannot be used.".into());
            }
            for path in &entry.initrd {
                let actual = format!("{directory}{path}");
                if let Some(initrd) = self.initramfs.iter().find(|f| f.path == actual)
                    && !initrd.image.image_build.same_snapshot(&kernel.build)
                {
                    return Err("The selected initramfs belongs to a different supplied source/configuration build, even if release labels match.".into());
                }
            }
        }
        let inventory = self.inventory(prefix, root)?;
        boot::test(&inventory, &file.source).map_err(|e| e.to_string())
    }
}
