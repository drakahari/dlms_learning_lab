//! Compose script-free package metadata and supplied module records. No host I/O,
//! ELF interpretation, kernel operation, package script, guest or hypervisor.
use crate::{kernel_modules as modules, rpm_packages as rpm};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const HELP: &str = "Simulated guest-driver console: uname -r; lspci -k; lsmod; modinfo NAME or supplied absolute object path; depmod -a; modprobe NAME [declared integer parameters]; modprobe -r NAME; insmod SUPPLIED_OBJECT_PATH [declared integer parameters]; rmmod NAME; dmesg; cat /etc/modprobe.d/lab.conf or /sys/module/NAME/parameters/PARAMETER. Use the native package inspector and supplied-package actions for image files. The supplied options are read-only in this lesson. No aliases, automatic device loading, shell expansion, pipelines, force, arbitrary paths, real package scripts, ELF execution, guest startup or host access.";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub module: String,
    /// An inert authored descriptor, explicitly not a real .ko/ELF payload.
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub modules: modules::Catalogue,
    pub packages: rpm::Catalog,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Object,16>")]
    pub objects: Vec<Object>,
}

impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        self.modules.validate()?;
        self.packages.validate().map_err(|e| e.message)?;
        if self.modules.modules.len() > 16
            || self.modules.devices.len() > 4
            || self.packages.archives.len() > 8
            || self.packages.allowed_paths.len() > 16
            || self.packages.snapshots.len() != 1
            || self.objects.len() != self.modules.modules.len()
        {
            return Err("Guest support uses at most sixteen supplied objects, four devices and eight local archives.".into());
        }
        let mut names = BTreeSet::new();
        for object in &self.objects {
            if !self.modules.modules.contains_key(&object.module)
                || !names.insert(&object.module)
                || object.contents.is_empty()
                || object.contents.len() > 160
                || !object.contents.is_ascii()
                || object.contents.chars().any(char::is_control)
            {
                return Err("Each known module needs one bounded inert object descriptor.".into());
            }
        }
        let paths: BTreeSet<_> = self.modules.modules.values().map(|m| &m.path).collect();
        if self.packages.allowed_paths.iter().collect::<BTreeSet<_>>() != paths {
            return Err(
                "The guest-image file universe must be exactly its supplied module-object paths."
                    .into(),
            );
        }
        let mut supplied = BTreeSet::new();
        for archive in self.packages.archives.values() {
            if archive.identity.arch != rpm::Architecture::X86_64 {
                return Err("These represented guest-driver archives declare x86_64, not noarch. No host architecture is inspected.".into());
            }
            for (path, payload) in &archive.files {
                let Some(object) = self
                    .objects
                    .iter()
                    .find(|o| self.modules.modules[&o.module].path == *path)
                else {
                    return Err(
                        "Guest support archives contain only declared virtual module-object paths."
                            .into(),
                    );
                };
                if payload.kind != rpm::FileKind::Ordinary
                    || payload.file.contents != object.contents
                {
                    return Err("Archive records must match the inert supplied module descriptors; executable payloads and configuration scripts are outside this model.".into());
                }
                supplied.insert(&object.module);
            }
        }
        if supplied != names {
            return Err("Every declared object must be supplied by a known local archive.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub packages: rpm::Runtime,
    pub modules: modules::Runtime,
}

impl Runtime {
    pub fn validate(&self, f: &Facts) -> Result<(), String> {
        f.validate()?;
        rpm::State::new(f.packages.clone(), self.packages.clone()).map_err(|e| e.message)?;
        self.modules.validate(&f.modules)?;
        // Missing objects are a modeled fault. Arbitrary changed module bytes are
        // not interpreted as ELF or silently assigned the source's properties.
        for (path, file) in &self.packages.files {
            if !f
                .packages
                .archives
                .values()
                .any(|a| a.files.get(path).is_some_and(|p| &p.file == file))
            {
                return Err("Changed module-object contents or attributes are outside this guest-image subset; original saved work must be retained rather than interpreted as executable code.".into());
            }
        }
        Ok(())
    }

    pub fn present(&self, f: &Facts) -> Result<BTreeSet<String>, String> {
        self.validate(f)?;
        Ok(self.present_inner(f))
    }

    fn present_inner(&self, f: &Facts) -> BTreeSet<String> {
        f.objects
            .iter()
            .filter(|o| {
                self.packages
                    .files
                    .contains_key(&f.modules.modules[&o.module].path)
            })
            .map(|o| o.module.clone())
            .collect()
    }

    /// A real logical package transaction from the existing bounded RPM model.
    /// It may add/remove virtual object files but does not load/unload drivers,
    /// run scripts or secretly rebuild the module index.
    pub fn package(&mut self, f: &Facts, action: &rpm::Action) -> Result<rpm::Plan, String> {
        self.validate(f)?;
        let mut state =
            rpm::State::new(f.packages.clone(), self.packages.clone()).map_err(|e| e.message)?;
        let plan = state.apply(action).map_err(|e| e.message)?;
        let mut next = self.clone();
        next.packages = state.runtime().clone();
        next.validate(f)?;
        *self = next;
        Ok(plan)
    }

    pub fn run(&mut self, f: &Facts, input: &str) -> Result<modules::Outcome, String> {
        self.validate(f)?;
        if input.len() > modules::MAX_COMMAND {
            return Err("Guest driver commands are limited to 4,096 bytes.".into());
        }
        let words = crate::shell::literal_command(input)?;
        if words.as_slice() == ["help"] {
            return Ok(modules::Outcome {
                status: 0,
                stdout: HELP.into(),
                stderr: String::new(),
            });
        }
        if words.first().is_some_and(|w| w == "rpm") {
            return Err("This terminal supports the listed driver commands. Use the package inspector to install, replace or reinstall a supplied package.".into());
        }
        let present = self.present_inner(f);
        let outcome = self
            .modules
            .execute_available(&f.modules, input, &present)?;
        self.validate(f)?;
        Ok(outcome)
    }

    /// Observe whether the stored files/index can load a driver from an empty
    /// supplied module state. This is not a boot, initramfs or connectivity test.
    pub fn fresh_request(&self, f: &Facts, driver: &str) -> Result<modules::Outcome, String> {
        self.validate(f)?;
        let Some(module) = f.modules.modules.get(driver) else {
            return Err("Choose a supplied physical driver name.".into());
        };
        if module.kernel_release != f.modules.kernel_release {
            return Err(
                "The requested driver must belong to the represented running kernel.".into(),
            );
        }
        let mut trial = self.clone();
        trial.modules.loaded.clear();
        trial.run(f, &format!("modprobe {driver}"))
    }

    /// Developer-only bounded fixture fault: remove one declared virtual object.
    /// No path from learner input ever reaches the host filesystem.
    pub fn remove_object(&mut self, f: &Facts, module: &str) -> Result<(), String> {
        self.validate(f)?;
        let object = f
            .modules
            .modules
            .get(module)
            .ok_or("Choose a supplied module object.")?;
        self.packages.files.remove(&object.path);
        self.validate(f)
    }
}
