//! Finite supplied kernel-feature model, not Kconfig or a compiler.
//! No host paths, commands, executable bytes, kernel loading or real boot.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CONFIG_CAP: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Feature {
    Disabled,
    BuiltIn,
    Module,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub modules: bool,
    pub initrd: bool,
    pub nvme: Feature,
    pub ext4: Feature,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub localversion: String,
}

fn suffix(s: &str) -> bool {
    s.len() <= 24
        && (s.is_empty() || s.starts_with('-'))
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

fn source_token(s: &str, cap: usize) -> bool {
    !s.is_empty()
        && s.len() <= cap
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !suffix(&self.localversion) {
            return Err("Use an empty release suffix or - followed by letters, digits, dots, underscores or hyphens (at most 24 characters).".into());
        }
        if !self.modules && [self.nvme, self.ext4].contains(&Feature::Module) {
            return Err("A requested loadable driver needs CONFIG_MODULES=y. This finite configuration fragment rejects the conflict; it does not run Kconfig dependency resolution.".into());
        }
        Ok(())
    }

    /// Parse only the disclosed fragment; fixed source dependencies are supplied.
    /// Unknown settings, malformed lines and duplicates never disappear silently.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > CONFIG_CAP
            || text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        {
            return Err(
                "The supplied configuration fragment is limited to 8192 text bytes.".into(),
            );
        }
        let mut fields = std::collections::BTreeMap::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() {
                continue;
            }
            let (name, value) = if let Some(body) = line.strip_prefix("# ") {
                if let Some(name) = body.strip_suffix(" is not set") {
                    (name, "n")
                } else {
                    continue;
                }
            } else if line.starts_with('#') {
                continue;
            } else {
                line.split_once('=').ok_or_else(|| {
                    format!(
                        "Line {}: use one supported CONFIG_NAME=value setting.",
                        i + 1
                    )
                })?
            };
            if !matches!(
                name,
                "CONFIG_MODULES"
                    | "CONFIG_BLK_DEV_INITRD"
                    | "CONFIG_BLK_DEV_NVME"
                    | "CONFIG_EXT4_FS"
                    | "CONFIG_LOCALVERSION"
            ) {
                return Err(format!(
                    "Line {}: {name} is outside the disclosed configuration fragment; no setting was applied.",
                    i + 1
                ));
            }
            if fields.insert(name, value).is_some() {
                return Err(format!(
                    "Line {} repeats {name}; keep one unambiguous setting.",
                    i + 1
                ));
            }
        }
        let required = |key| {
            fields.get(key).copied().ok_or_else(|| {
                format!("Include {key}; use its supplied value or explicitly disable it.")
            })
        };
        let boolean = |key| match required(key)? {
            "y" => Ok(true),
            "n" => Ok(false),
            _ => Err(format!("{key} accepts y or n in this fragment.")),
        };
        let feature = |key| match required(key)? {
            "y" => Ok(Feature::BuiltIn),
            "m" => Ok(Feature::Module),
            "n" => Ok(Feature::Disabled),
            _ => Err(format!(
                "{key} accepts y (built in), m (loadable module), or n (disabled)."
            )),
        };
        let local = required("CONFIG_LOCALVERSION")?;
        let localversion = local.strip_prefix('"').and_then(|v| v.strip_suffix('"'))
            .ok_or("CONFIG_LOCALVERSION needs a double-quoted literal suffix, for example \"-training\".")?.to_owned();
        let config = Self {
            modules: boolean("CONFIG_MODULES")?,
            initrd: boolean("CONFIG_BLK_DEV_INITRD")?,
            nvme: feature("CONFIG_BLK_DEV_NVME")?,
            ext4: feature("CONFIG_EXT4_FS")?,
            localversion,
        };
        config.validate()?;
        Ok(config)
    }

    /// Supplied dependency graph only: these are inventories, not object files.
    pub fn loadable_inventory(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        if self.nvme == Feature::Module {
            result.extend(["nvme_core".into(), "nvme".into()]);
        }
        if self.ext4 == Feature::Module {
            result.extend(["jbd2".into(), "mbcache".into(), "ext4".into()]);
        }
        result
    }

    pub fn builtin_inventory(&self) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        if self.nvme == Feature::BuiltIn {
            result.extend(["nvme_core".into(), "nvme".into()]);
        }
        if self.ext4 == Feature::BuiltIn {
            result.extend(["jbd2".into(), "mbcache".into(), "ext4".into()]);
        }
        result
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub release: String,
    pub config: Config,
    pub image: bool,
    pub modules: bool,
}

impl Build {
    pub fn from_config(
        source_id: &str,
        base_release: &str,
        config: Config,
    ) -> Result<Self, String> {
        config.validate()?;
        if !source_token(source_id, 64) || !source_token(base_release, 48) {
            return Err("Use the supplied source identity and kernel release.".into());
        }
        let release = format!("{base_release}{}", config.localversion);
        if release.len() > 64 {
            return Err("The kernel release, including its local suffix, must fit 64 characters. Shorten the suffix.".into());
        }
        Ok(Self {
            source_id: source_id.into(),
            release,
            config,
            image: false,
            modules: false,
        })
    }

    pub fn current_for(&self, source_id: &str, base_release: &str, config: &Config) -> bool {
        self.validate_shape().is_ok()
            && config.validate().is_ok()
            && source_token(source_id, 64)
            && source_token(base_release, 48)
            && self.source_id == source_id
            && self.release == format!("{base_release}{}", config.localversion)
            && self.config == *config
    }

    pub fn validate_shape(&self) -> Result<(), String> {
        self.config.validate()?;
        if !source_token(&self.source_id, 64) || !source_token(&self.release, 64) {
            return Err(
                "The build record needs its supplied source and a bounded release label.".into(),
            );
        }
        if self.modules && !self.config.modules {
            return Err("A modules-build record cannot claim module support was disabled.".into());
        }
        Ok(())
    }

    pub fn same_snapshot(&self, other: &Self) -> bool {
        self.validate_shape().is_ok()
            && other.validate_shape().is_ok()
            && self.source_id == other.source_id
            && self.release == other.release
            && self.config == other.config
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Image,
    Modules,
    All,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initramfs {
    pub image_build: Build,
    pub module_build: Option<Build>,
    #[serde(deserialize_with = "module_inventory")]
    pub included: BTreeSet<String>,
}

fn module_inventory<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeSet<String>, D::Error> {
    use serde::de::Error as _;
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,16>")] String);
    let names = crate::cron_workspace::sequence::<D, Name, 5>(d).map_err(|_| {
        D::Error::custom(
            "Kernel initramfs inventory needs at most five bounded supplied module names",
        )
    })?;
    let mut set = BTreeSet::new();
    for Name(name) in names {
        if !matches!(
            name.as_str(),
            "nvme" | "nvme_core" | "ext4" | "jbd2" | "mbcache"
        ) || !set.insert(name)
        {
            return Err(D::Error::custom(
                "Use each supplied kernel-module name at most once; no host paths or other modules",
            ));
        }
    }
    Ok(set)
}

impl Initramfs {
    pub fn validate_shape(&self) -> Result<(), String> {
        self.image_build.validate_shape()?;
        let needed = self.image_build.config.loadable_inventory();
        if !self.image_build.image || !self.included.is_subset(&needed) {
            return Err("The modeled initramfs needs its image/configuration identity and only the supplied module inventory.".into());
        }
        if let Some(modules) = &self.module_build {
            modules.validate_shape()?;
            if !modules.modules || !modules.same_snapshot(&self.image_build) {
                return Err("Initramfs module inputs must belong to its supplied source/configuration build.".into());
            }
        } else if !needed.is_empty() {
            return Err("A modular root configuration needs matching built module inputs in its initramfs record.".into());
        }
        Ok(())
    }
    pub fn from_builds(image: &Build, modules: Option<&Build>) -> Result<Self, String> {
        image.validate_shape()?;
        if !image.image {
            return Err(
                "Build the supplied kernel image before assembling its modeled initramfs.".into(),
            );
        }
        let needed = image.config.loadable_inventory();
        let module_build = if needed.is_empty() {
            None
        } else {
            let record = modules.ok_or(
                "This image needs loadable root drivers; build its matching modules first.",
            )?;
            record.validate_shape()?;
            if !record.modules || !record.same_snapshot(image) {
                return Err("The available modules belong to a different source/configuration build; rebuild them for this image.".into());
            }
            Some(record.clone())
        };
        // The supplied early-userspace fixture is distinct from loadable drivers.
        Ok(Self {
            image_build: image.clone(),
            module_build,
            included: needed,
        })
    }
}

/// Working outputs remain separate. A later partial build must neither claim the
/// other output was rebuilt nor erase an earlier usable initramfs/fallback.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    pub image: Option<Build>,
    pub modules: Option<Build>,
    pub initramfs: Option<Initramfs>,
}

impl Artifacts {
    pub fn validate_shape(&self) -> Result<(), String> {
        if let Some(image) = &self.image {
            image.validate_shape()?;
            if !image.image {
                return Err("An image output needs an actual image-build record.".into());
            }
        }
        if let Some(modules) = &self.modules {
            modules.validate_shape()?;
            if !modules.modules {
                return Err("A modules output needs an actual modules-build record.".into());
            }
        }
        if let Some(initrd) = &self.initramfs {
            initrd.validate_shape()?;
        }
        Ok(())
    }
    pub fn build(
        &mut self,
        source_id: &str,
        base_release: &str,
        config: &Config,
        target: Target,
    ) -> Result<(), String> {
        self.validate_shape()?;
        let record = Build::from_config(source_id, base_release, config.clone())?;
        if target == Target::Modules && !config.modules {
            return Err("The supplied configuration has modules disabled. Enable module support before requesting the modules target; built-in root support does not require it.".into());
        }
        if matches!(target, Target::Image | Target::All) {
            let mut image = record.clone();
            image.image = true;
            self.image = Some(image);
        }
        if config.modules && matches!(target, Target::Modules | Target::All) {
            let mut modules = record;
            modules.modules = true;
            self.modules = Some(modules);
        }
        Ok(())
    }

    pub fn assemble_initramfs(&mut self) -> Result<(), String> {
        self.validate_shape()?;
        let record = self
            .image
            .as_ref()
            .ok_or("Build the supplied image before assembling its modeled initramfs.")?;
        let next = Initramfs::from_builds(record, self.modules.as_ref())?;
        self.initramfs = Some(next);
        Ok(())
    }

    pub fn readiness(&self) -> RootCheck {
        let fail = |reason: &str| RootCheck {
            passed: false,
            reason: reason.into(),
        };
        if self.validate_shape().is_err() {
            return fail(
                "The saved supplied-artifact record is malformed; no readiness claim is possible.",
            );
        }
        let Some(record) = &self.image else {
            return fail("Build a supplied image before checking root readiness.");
        };
        if record.validate_shape().is_err() {
            return fail(
                "The image build record is invalid; rebuild from the supplied source/configuration.",
            );
        }
        let mut image = record.clone();
        image.modules = self
            .modules
            .as_ref()
            .is_some_and(|r| r.modules && r.same_snapshot(&image));
        let empty = BTreeSet::new();
        let mut release = None;
        let mut included = &empty;
        if let Some(initrd) = &self.initramfs {
            let matching = initrd.image_build.same_snapshot(&image);
            if !matching {
                return fail(
                    "The initramfs belongs to a different source/configuration build. Matching release names alone do not establish matching supplied artifacts.",
                );
            }
            if !image.config.initrd {
                return fail(
                    "The image cannot load this selected external initramfs. Enable its initramfs-loading support and rebuild, or test a built-in-only image without an external initramfs.",
                );
            }
            // An assembled image retains its inputs; replacing working outputs
            // cannot retroactively alter its module or early-userspace records.
            image.modules |= initrd.module_build.is_some();
            release = Some(initrd.image_build.release.as_str());
            included = &initrd.included;
        }
        root_ready(&image, release, included)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootCheck {
    pub passed: bool,
    pub reason: String,
}

/// The supplied machine has an NVMe root disk with ext4. This is a finite
/// readiness check, not successful execution of a kernel or a universal boot test.
pub fn root_ready(
    build: &Build,
    initramfs_release: Option<&str>,
    initramfs_modules: &BTreeSet<String>,
) -> RootCheck {
    let fail = |reason: &str| RootCheck {
        passed: false,
        reason: reason.into(),
    };
    if build.validate_shape().is_err() {
        return fail(
            "The build record contains an invalid supplied configuration; no boot-readiness claim is possible.",
        );
    }
    if !build.image {
        return fail("Build the supplied image before testing its root-storage support.");
    }
    if build.config.nvme == Feature::Disabled {
        return fail("The image has no NVMe driver for the supplied root disk.");
    }
    if build.config.ext4 == Feature::Disabled {
        return fail("The image has no ext4 support for the supplied root filesystem.");
    }
    let needs_modules =
        build.config.nvme == Feature::Module || build.config.ext4 == Feature::Module;
    if needs_modules {
        if !build.config.initrd {
            return fail(
                "Root support is modular, but the image cannot load the supplied external initramfs. Build that support in, or enable initramfs loading and provide the matching modules.",
            );
        }
        if !build.modules {
            return fail(
                "The image expects loadable root drivers; build its matching modules too.",
            );
        }
        if initramfs_release != Some(build.release.as_str()) {
            return fail(
                "The initial RAM filesystem must belong to this image's exact kernel release.",
            );
        }
        if !build
            .config
            .loadable_inventory()
            .is_subset(initramfs_modules)
        {
            return fail(
                "The supplied initial RAM filesystem lacks a required driver or its supplied dependency.",
            );
        }
    }
    RootCheck { passed:true, reason:"The supplied image has the modeled support needed to reach this NVMe/ext4 root disk. This does not execute a real kernel or prove general hardware compatibility.".into() }
}
