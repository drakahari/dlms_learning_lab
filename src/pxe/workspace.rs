//! Saved policy and PXELINUX editor state over an authored inventory.
use super::{
    Architecture, Artifact, BootFile, Client, Diagnostic, Inventory, LinuxPair, Policy, Request,
    bootstrap, resolve_linux_pair,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

const MAX_EDITS: usize = 8;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Mode {
    Policy,
    Configuration {
        program: BootFile,
        #[serde(deserialize_with = "crate::cron::text::<_,127>")]
        starter_leaf: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Program {
        architecture: Architecture,
    },
    NoOffer,
    Release {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        release: String,
    },
    PreservePair,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub label: String,
    pub client: Client,
    pub requirement: Requirement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub server: String,
    #[serde(deserialize_with = "crate::cron::text::<_,127>")]
    pub filename: String,
}

impl Rule {
    fn from_offer(offer: Option<&BootFile>) -> Self {
        Self {
            enabled: offer.is_some(),
            server: offer.map_or_else(String::new, |f| f.server.to_string()),
            filename: offer.map_or_else(String::new, |f| f.name.clone()),
        }
    }

    fn valid_text(&self) -> bool {
        self.server.len() <= 15 && self.filename.len() <= 127
    }

    fn offer(&self, label: &str) -> Result<Option<BootFile>, String> {
        if !self.enabled {
            return Ok(None);
        }
        let server =
            self.server.trim().parse::<Ipv4Addr>().map_err(|_| {
                format!("{label}: enter a dotted IPv4 address for the boot server.")
            })?;
        let file = BootFile {
            server,
            name: self.filename.trim().into(),
        };
        if !file.valid() {
            return Err(format!(
                "{label}: enter a relative boot filename without spaces or parent directories."
            ));
        }
        Ok(Some(file))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyDraft {
    pub bios: Rule,
    pub x64_uefi: Rule,
    pub unconditional: Rule,
}

impl From<&Policy> for PolicyDraft {
    fn from(policy: &Policy) -> Self {
        Self {
            bios: Rule::from_offer(policy.bios.as_ref()),
            x64_uefi: Rule::from_offer(policy.x64_uefi.as_ref()),
            unconditional: Rule::from_offer(policy.unconditional.as_ref()),
        }
    }
}

impl PolicyDraft {
    fn valid_text(&self) -> bool {
        [&self.bios, &self.x64_uefi, &self.unconditional]
            .into_iter()
            .all(Rule::valid_text)
    }

    fn policy(&self) -> Result<Policy, String> {
        Ok(Policy {
            bios: self.bios.offer("PC BIOS rule")?,
            x64_uefi: self.x64_uefi.offer("x64 UEFI rule")?,
            unconditional: self.unconditional.offer("Unconditional rule")?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EntryDraft {
    #[serde(deserialize_with = "crate::cron::text::<_,127>")]
    pub leaf: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub source: String,
}

impl EntryDraft {
    fn valid_text(&self) -> bool {
        self.leaf.len() <= 127 && self.source.len() <= super::MAX_CONFIG
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "op", deny_unknown_fields)]
pub enum Action {
    EditPolicy { draft: PolicyDraft },
    SavePolicy,
    EditEntry { draft: EntryDraft },
    SaveEntry,
    Check,
    Reset,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub mode: Mode,
    pub starter_policy: Policy,
    pub inventory: Inventory,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,16>")]
    pub reference: Vec<Action>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub policy_draft: PolicyDraft,
    pub entry_draft: Option<EntryDraft>,
    pub policy: Policy,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,EntryDraft,8>")]
    pub saved_entries: Vec<EntryDraft>,
    pub checked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Check {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

const UNSAVED: &str =
    "The configuration has unsaved changes. Save the current edit, then check the client outcomes.";

impl Task {
    pub fn start(&self) -> Result<Work, String> {
        let entry_draft = match &self.mode {
            Mode::Policy => None,
            Mode::Configuration {
                program,
                starter_leaf,
            } => {
                let source = self
                    .entry_source(&self.inventory, program, starter_leaf)
                    .ok_or("The supplied starter configuration is missing.")?;
                Some(EntryDraft {
                    leaf: starter_leaf.clone(),
                    source: source.into(),
                })
            }
        };
        Ok(Work {
            policy_draft: PolicyDraft::from(&self.starter_policy),
            entry_draft,
            policy: self.starter_policy.clone(),
            saved_entries: Vec::new(),
            checked: false,
        })
    }

    fn entry_source<'a>(
        &self,
        inventory: &'a Inventory,
        program: &BootFile,
        leaf: &str,
    ) -> Option<&'a str> {
        if leaf.contains('/') {
            return None;
        }
        let path = program.from_program_directory(&format!("pxelinux.cfg/{leaf}"))?;
        match inventory.get(&path) {
            Some(Artifact::Configuration { source }) => Some(source),
            _ => None,
        }
    }

    pub fn effective_inventory(&self, work: &Work) -> Result<Inventory, String> {
        if !work.policy_draft.valid_text()
            || work
                .entry_draft
                .as_ref()
                .is_some_and(|entry| !entry.valid_text())
            || work.saved_entries.len() > MAX_EDITS
        {
            return Err("Keep the saved editor data within the exercise limits.".into());
        }
        self.inventory.validate()?;
        work.policy.validate()?;
        let mut inventory = self.inventory.clone();
        match &self.mode {
            Mode::Policy => {
                if work.entry_draft.is_some() || !work.saved_entries.is_empty() {
                    return Err("This activity changes only the boot-file policy.".into());
                }
            }
            Mode::Configuration { program, .. } => {
                if work.policy != self.starter_policy
                    || work.policy_draft != PolicyDraft::from(&self.starter_policy)
                    || work.entry_draft.is_none()
                {
                    return Err(
                        "Keep the supplied DHCP boot-file rules fixed in this activity.".into(),
                    );
                }
                let mut names = BTreeSet::new();
                for entry in &work.saved_entries {
                    if !entry.valid_text() || !names.insert(&entry.leaf) {
                        return Err(
                            "Saved configurations must have distinct supported names.".into()
                        );
                    }
                    inventory.save_configuration(program, &entry.leaf, entry.source.clone())?;
                }
            }
        }
        Ok(inventory)
    }

    pub fn has_unsaved_changes(&self, work: &Work, inventory: &Inventory) -> bool {
        match &self.mode {
            Mode::Policy => work.policy_draft.policy().as_ref() != Ok(&work.policy),
            Mode::Configuration { program, .. } => work.entry_draft.as_ref().is_some_and(|entry| {
                self.entry_source(inventory, program, &entry.leaf) != Some(entry.source.as_str())
            }),
        }
    }

    pub fn apply(&self, work: &mut Work, action: &Action) -> Result<(), String> {
        let mut inventory = self.effective_inventory(work)?;
        match (action, &self.mode) {
            (Action::EditPolicy { draft }, Mode::Policy) if draft.valid_text() => {
                if work.policy_draft != *draft {
                    work.policy_draft = draft.clone();
                    work.checked = false;
                }
            }
            (Action::SavePolicy, Mode::Policy) => {
                let policy = work.policy_draft.policy()?;
                if work.policy != policy {
                    work.policy = policy;
                    work.checked = false;
                }
            }
            (Action::EditEntry { draft }, Mode::Configuration { .. }) if draft.valid_text() => {
                if work.entry_draft.as_ref() != Some(draft) {
                    work.entry_draft = Some(draft.clone());
                    work.checked = false;
                }
            }
            (Action::SaveEntry, Mode::Configuration { program, .. }) => {
                let draft = work
                    .entry_draft
                    .as_ref()
                    .ok_or("Enter a virtual configuration first.")?;
                if self.entry_source(&inventory, program, &draft.leaf)
                    == Some(draft.source.as_str())
                {
                    return Ok(());
                }
                inventory.save_configuration(program, &draft.leaf, draft.source.clone())?;
                if let Some(previous) = work
                    .saved_entries
                    .iter_mut()
                    .find(|entry| entry.leaf == draft.leaf)
                {
                    *previous = draft.clone();
                } else if work.saved_entries.len() < MAX_EDITS {
                    work.saved_entries.push(draft.clone());
                } else {
                    return Err(
                        "Keep at most eight changed configuration files in this activity.".into(),
                    );
                }
                work.checked = false;
            }
            (Action::Check, _) => work.checked = true,
            (Action::Reset, _) => *work = self.start()?,
            _ => return Err(
                "Use the controls available in this activity and keep the draft within its limits."
                    .into(),
            ),
        }
        Ok(())
    }

    pub fn checks(&self, work: &Work) -> Vec<Check> {
        let result = |label: &str, passed, detail: String| Check {
            label: label.into(),
            passed,
            detail,
        };
        let inventory = match self.effective_inventory(work) {
            Ok(inventory) => inventory,
            Err(detail) => return vec![result("Saved workspace", false, detail)],
        };
        let unsaved = self.has_unsaved_changes(work, &inventory);
        let saved = if unsaved {
            UNSAVED
        } else {
            "The client traces use the current saved configuration."
        };
        let mut checks = vec![result("Saved changes", !unsaved, saved.into())];
        for case in &self.cases {
            let outcome = self.evaluate(&inventory, &work.policy, case);
            match outcome {
                Ok(detail) => checks.push(result(&case.label, true, detail)),
                Err(detail) => checks.push(result(&case.label, false, detail)),
            }
        }
        checks
    }

    fn evaluate(
        &self,
        inventory: &Inventory,
        policy: &Policy,
        case: &Case,
    ) -> Result<String, String> {
        match &case.requirement {
            Requirement::Program { architecture } => {
                let result = bootstrap(inventory, policy, &case.client).map_err(explain)?
                    .ok_or("The client received no boot file. Check its architecture rule and saved server and filename.")?;
                if result.architecture != *architecture {
                    return Err(
                        "The selected program does not satisfy this firmware requirement.".into(),
                    );
                }
                Ok(format!(
                    "{} supplies compatible program {}.",
                    result.file.server, result.file.name
                ))
            }
            Requirement::NoOffer => {
                match bootstrap(inventory, policy, &case.client).map_err(explain)? {
                    None => Ok(
                        "The ordinary client keeps its supplied lease and receives no boot file."
                            .into(),
                    ),
                    Some(_) => Err("This client should not receive a boot program.".into()),
                }
            }
            Requirement::Release { release } => {
                let current = pair(inventory, policy, &case.client)?;
                if &current.release != release {
                    return Err(format!(
                        "The selected files match release {}, but this client needs {release}.",
                        current.release
                    ));
                }
                Ok(format!(
                    "The selected kernel and initramfs both match {release}."
                ))
            }
            Requirement::PreservePair => {
                let previous = pair(&self.inventory, &self.starter_policy, &case.client)?;
                let current = pair(inventory, policy, &case.client)?;
                if previous != current {
                    return Err("The protected client selects a different boot pair. Keep its original kernel and initramfs.".into());
                }
                Ok("The protected client still selects its original kernel and initramfs.".into())
            }
        }
    }

    pub fn solved(&self, work: &Work) -> bool {
        work.checked
            && !self.cases.is_empty()
            && self.checks(work).iter().all(|result| result.passed)
    }
}

fn pair(inventory: &Inventory, policy: &Policy, client: &Client) -> Result<LinuxPair, String> {
    let selected = bootstrap(inventory, policy, client)
        .map_err(explain)?
        .ok_or("This client did not receive a boot file.")?;
    if selected.architecture != Architecture::PcBios {
        return Err("This activity traces the supplied BIOS PXELINUX program only.".into());
    }
    resolve_linux_pair(inventory, &selected.file, client).map_err(explain)
}

pub fn explain(diagnostic: Diagnostic) -> String {
    match diagnostic {
        Diagnostic::UnexpectedOffer(file) => format!(
            "The ordinary client received {} from an unconditional boot rule. Restrict boot offers to PXE clients.",
            file.name
        ),
        Diagnostic::MissingArchitecture => {
            "The supplied PXE request has no client architecture value.".into()
        }
        Diagnostic::UnsupportedArchitecture(value) => format!(
            "Client architecture {value} lies outside this exercise's BIOS and x64 UEFI profile."
        ),
        Diagnostic::InvalidPath => {
            "The current identity or file path does not fit the supplied relative boot-file form."
                .into()
        }
        Diagnostic::UnreachableServer(server) => {
            format!("This client cannot reach the supplied boot server {server}.")
        }
        Diagnostic::MissingFile(file) => format!(
            "{} does not supply {}. Inspect the server's file list.",
            file.server, file.name
        ),
        Diagnostic::WrongProgram { file, architecture } => format!(
            "{} exists, but its program type cannot run on {}. Renaming a file does not change its program type.",
            file.name,
            architecture.label()
        ),
        Diagnostic::MissingCore(file) => format!(
            "PXELINUX needs {} in its boot program directory.",
            file.name
        ),
        Diagnostic::MissingConfiguration => {
            "None of the generated client-specific or default configuration files are present."
                .into()
        }
        Diagnostic::WrongConfigurationType(file) => format!(
            "{} is the first present file but has the wrong type for a PXELINUX configuration.",
            file.name
        ),
        Diagnostic::InvalidConfiguration { file, detail } => format!("{}: {detail}", file.name),
        Diagnostic::WrongLinuxFileType(file) => format!(
            "{} is missing or has the wrong type. Paths are relative to the boot program's directory.",
            file.name
        ),
        Diagnostic::ReleaseMismatch { kernel, initramfs } => format!(
            "The selected kernel is {kernel}, but its initramfs belongs to {initramfs}. Point the selected configuration at a matching pair."
        ),
    }
}

impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.id.is_empty()
            || self.id.len() > 96
            || !self
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            || self.cases.len() > 8
            || !matches!(self.reference.last(), Some(Action::Check))
            || self.reference.len() > 16
        {
            return Err("A named network boot exercise needs bounded client cases and a checkable reference.".into());
        }
        self.inventory.validate()?;
        self.starter_policy.validate()?;
        let mut addresses = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut bios = false;
        let mut efi = false;
        let mut ordinary = false;
        let mut target = false;
        let mut fallback = false;
        for case in &self.cases {
            if case.label.trim().is_empty()
                || case.label.len() > 96
                || !labels.insert(&case.label)
                || !addresses.insert(case.client.address)
                || case.client.address.is_unspecified()
                || case.client.address.is_broadcast()
                || case.client.address.is_multicast()
                || case
                    .client
                    .uuid
                    .as_deref()
                    .is_some_and(|value| !super::canonical_uuid(value))
            {
                return Err(
                    "Use distinct labeled clients with valid supplied addresses and identifiers."
                        .into(),
                );
            }
            match (&self.mode, &case.requirement, case.client.request) {
                (
                    Mode::Policy,
                    Requirement::Program { architecture },
                    Request::Pxe {
                        option_93: Some(code),
                    },
                ) if Architecture::from_option_93(code) == Some(*architecture) => {
                    match architecture {
                        Architecture::PcBios => bios = true,
                        Architecture::X64Uefi => efi = true,
                    }
                }
                (Mode::Policy, Requirement::NoOffer, Request::OrdinaryDhcp) => ordinary = true,
                (
                    Mode::Configuration { .. },
                    Requirement::Release { release },
                    Request::Pxe { option_93: Some(0) },
                ) if super::release_name(release) => {
                    target = true;
                }
                (
                    Mode::Configuration { .. },
                    Requirement::PreservePair,
                    Request::Pxe { option_93: Some(0) },
                ) => {
                    pair(&self.inventory, &self.starter_policy, &case.client)?;
                    fallback = true;
                }
                _ => {
                    return Err(
                        "The required outcome must fit the supplied client and exercise type."
                            .into(),
                    );
                }
            }
        }
        match &self.mode {
            Mode::Policy if bios && efi && ordinary => {}
            Mode::Configuration { program, .. }
                if target
                    && fallback
                    && self.starter_policy.bios.as_ref() == Some(program)
                    && self.inventory.get(program) == Some(&Artifact::BiosPxelinux6) => {}
            _ => return Err("Include both firmware paths and ordinary DHCP, or an editable BIOS target and protected fallback.".into()),
        }
        let mut work = self.start()?;
        let checks = self.checks(&work);
        if checks.iter().all(|row| row.passed) {
            return Err("Supply a starter state that actually needs repair.".into());
        }
        for action in &self.reference {
            self.apply(&mut work, action)?;
        }
        if !self.solved(&work) {
            return Err(
                "The authored reference must satisfy every required client outcome.".into(),
            );
        }
        Ok(())
    }
}
