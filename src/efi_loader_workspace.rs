//! Source-bound EFI recovery exercises with independent practice and assignment drafts.
use crate::{efi_loader as core, script_teaching::View};
use serde::{Deserialize, Serialize};
mod media;
pub use media::MediaPolicy;

pub const CHECK_ACTION: &str = "@efi-check";
const MAX_COMMAND: usize = 1024;
const MAX_TRANSCRIPT: usize = 8192;

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn bounded_text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.contains('\0')
}
fn commands<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Command(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
    crate::cron_workspace::sequence::<_, Command, 10>(d)
        .map(|rows| rows.into_iter().map(|row| row.0).collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub instructions: String,
    pub facts: core::Facts,
    pub initial: core::World,
    #[serde(deserialize_with = "commands")]
    pub reference: Vec<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub required_esp: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub required_root: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_release: String,
    pub read_only: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub target: String,
    pub archive_entry: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaPolicy>,
}

impl Task {
    pub fn shape(&self) -> Result<(), String> {
        self.initial.bound(&self.facts)?;
        if !valid_id(&self.id)
            || self.instructions.trim().is_empty()
            || !bounded_text(&self.instructions, 2048)
            || !(2..=10).contains(&self.reference.len())
            || self.reference.iter().any(|c| !bounded_text(c, MAX_COMMAND))
            || self.reference.last().map(String::as_str) != Some(CHECK_ACTION)
            || !matches!(self.target.as_str(), "multi-user.target" | "rescue.target")
        {
            return Err("Invalid supplied EFI assignment, reference or startup target.".into());
        }
        if let Some(media) = &self.media {
            self.media_shape(media)?;
        } else {
            let archive = self.archive()?;
            if !archive.active
                || archive.partition_guid == self.required_esp
                || !self.initial.boot_order.contains(&self.archive_entry)
                || !self.initial.files.iter().any(|file| {
                    file.partition_guid == archive.partition_guid
                        && file.path == archive.loader
                        && file.kind == core::FileKind::OtherApplication
                })
            {
                return Err("The assignment needs an existing active archive application on a distinct EFI partition.".into());
            }
        }
        if !self
            .facts
            .esps
            .iter()
            .any(|esp| esp.partition_guid == self.required_esp)
            || !self
                .facts
                .boot_directories
                .iter()
                .filter_map(|directory| directory.configuration.as_ref())
                .any(|configuration| {
                    configuration
                        .inventory
                        .kernels
                        .iter()
                        .any(|k| k.release == self.required_release)
                        && configuration
                            .inventory
                            .roots
                            .iter()
                            .any(|root| root.uuid.eq_ignore_ascii_case(&self.required_root))
                })
        {
            return Err(
                "The intended EFI partition, system and kernel must be supplied facts.".into(),
            );
        }
        Ok(())
    }

    fn archive(&self) -> Result<&core::OptionEntry, String> {
        self.initial
            .options
            .iter()
            .find(|entry| entry.number == self.archive_entry)
            .ok_or_else(|| {
                "The protected archive entry is missing from the supplied machine.".into()
            })
    }

    pub fn checks(&self, draft: &Draft) -> Vec<(bool, String)> {
        if let Err(error) = draft.bound(self) {
            return vec![(false, error)];
        }
        if draft.checked.is_none() {
            return vec![(false, "Not tested for the current installation and boot order. Use Test boot chain after changing either.".into())];
        }
        if let Some(media) = &self.media {
            return self.media_checks(draft, media);
        }
        let trace = match draft.current.trace(&self.facts) {
            Ok(trace) => trace,
            Err(error) => return vec![(false, error)],
        };
        let started = trace.started.and_then(|number| {
            draft
                .current
                .options
                .iter()
                .find(|entry| entry.number == number)
        });
        let started_intended =
            started.is_some_and(|entry| entry.partition_guid == self.required_esp);
        let mut checks = Vec::new();
        if started_intended {
            let entry = started.expect("the intended entry started");
            checks.push((
                true,
                format!(
                    "Boot{:04X} ({}) starts from the intended EFI partition.",
                    entry.number, entry.label
                ),
            ));
            if let Some(report) = &trace.handoff {
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
                            "Compare the menu's root UUID {} with the required {}.",
                            report.requested.root, self.required_root
                        ),
                    ),
                    (
                        report.release.as_deref() == Some(self.required_release.as_str()),
                        format!(
                            "Confirm the loaded kernel release is {}.",
                            self.required_release
                        ),
                    ),
                    (
                        mode == required_mode,
                        format!(
                            "The initial root mode is {mode}. The task requires {required_mode}."
                        ),
                    ),
                    (
                        report.requested.target == self.target,
                        format!(
                            "The menu requests {}. The task requires {}.",
                            report.requested.target, self.target
                        ),
                    ),
                    (
                        report.passed(),
                        "The selected menu must reach the modeled Linux handoff.".into(),
                    ),
                ]);
            } else {
                checks.push((false, trace.outcome));
            }
        } else {
            let files = draft
                .current
                .files
                .iter()
                .filter(|file| {
                    file.partition_guid == self.required_esp
                        && matches!(file.kind, core::FileKind::Grub { .. })
                })
                .collect::<Vec<_>>();
            let entry_in_order = draft.current.options.iter().any(|entry| {
                entry.active
                    && entry.partition_guid == self.required_esp
                    && draft.current.boot_order.contains(&entry.number)
                    && files.iter().any(|file| file.path == entry.loader)
            });
            let message = if files.is_empty() {
                "The intended EFI partition has no installed GRUB application. Check the EFI mount used for installation.".into()
            } else if !entry_in_order {
                "The intended EFI partition has a GRUB file, but no active BootOrder entry points to it. Compare efibootmgr -v with the installed path.".into()
            } else if let Some(entry) = started {
                format!(
                    "Boot{:04X} ({}) starts from another EFI partition. Put the intended working entry first in BootOrder.",
                    entry.number, entry.label
                )
            } else {
                "No entry starts the intended application. Inspect the boot attempts for the blocked path.".into()
            };
            checks.push((false, message));
        }
        let archive = match self.archive() {
            Ok(entry) => entry,
            Err(error) => return vec![(false, error)],
        };
        let preserved_entry = draft
            .current
            .options
            .iter()
            .find(|entry| entry.number == self.archive_entry)
            == Some(archive);
        let files_on_archive = |world: &core::World| {
            world
                .files
                .iter()
                .filter(|file| file.partition_guid == archive.partition_guid)
                .map(|file| (file.path.clone(), file.clone()))
                .collect::<std::collections::BTreeMap<_, _>>()
        };
        let files_match = files_on_archive(&draft.current) == files_on_archive(&self.initial);
        let earlier_position = trace
            .started
            .and_then(|number| draft.current.boot_order.iter().position(|n| *n == number));
        let archive_position = draft
            .current
            .boot_order
            .iter()
            .position(|n| *n == self.archive_entry);
        let archive_intact = preserved_entry && files_match;
        if !archive_intact {
            checks.push((false, format!(
                "Preserve Boot{:04X} and every file on the archive EFI partition. Reset this attempt if the supplied archive was changed.",
                self.archive_entry
            )));
        } else {
            let kept_in_order = if started_intended {
                earlier_position
                    .zip(archive_position)
                    .is_some_and(|(first, archive)| first < archive)
            } else {
                archive_position.is_some()
            };
            checks.push((
                kept_in_order,
                if kept_in_order {
                    format!(
                        "Boot{:04X} and its files remain available as a fallback.",
                        self.archive_entry
                    )
                } else {
                    format!(
                        "Keep Boot{:04X} in BootOrder after the intended working entry.",
                        self.archive_entry
                    )
                },
            ));
        }
        checks
    }

    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let mut draft = Draft::new(self);
        draft.test(self)?;
        if self.checks(&draft).iter().all(|row| row.0) {
            return Err("The EFI starter must need an installation or firmware repair.".into());
        }
        for command in &self.reference {
            if command == CHECK_ACTION {
                draft.test(self)?;
            } else {
                draft.apply(self, command)?;
            }
        }
        if self.checks(&draft).iter().any(|row| !row.0) {
            return Err("The supplied EFI reference must boot the intended system and preserve its fallback.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self.initial.media.is_some() != self.example.media.is_some()
            || self
                .initial
                .required_root
                .eq_ignore_ascii_case(&self.example.required_root)
            || self.initial.required_esp == self.example.required_esp
            || self.initial.reference == self.example.reference
        {
            return Err(
                "Use version 1 and a separate workshop with different boot and root facts.".into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }

    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        if let Err(error) = session.bound(self) {
            return vec![(false, error)];
        }
        self.initial.checks(&session.assignment)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub current: core::World,
    pub checked: Option<core::World>,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub transcript: String,
}

impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            current: task.initial.clone(),
            checked: None,
            command: String::new(),
            transcript: String::new(),
        }
    }
    fn shape(&self) -> Result<(), String> {
        if !bounded_text(&self.command, MAX_COMMAND)
            || !bounded_text(&self.transcript, MAX_TRANSCRIPT)
            || self
                .checked
                .as_ref()
                .is_some_and(|checked| checked != &self.current)
        {
            return Err("Saved EFI input, output or tested state is incompatible.".into());
        }
        self.current.shape()
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.shape()?;
        self.current.bound(&task.facts)
    }
    fn append(&mut self, text: &str) {
        self.transcript.push_str(text);
        if self.transcript.len() > MAX_TRANSCRIPT {
            let mut offset = self.transcript.len() - (MAX_TRANSCRIPT - 64);
            while !self.transcript.is_char_boundary(offset) {
                offset += 1;
            }
            self.transcript.drain(..offset);
            self.transcript
                .insert_str(0, "Earlier console output omitted.\n");
        }
    }
    fn apply(&mut self, task: &Task, command: &str) -> Result<String, String> {
        self.bound(task)?;
        let before = self.current.clone();
        let output = self.current.execute(&task.facts, command)?;
        if self.current != before {
            self.checked = None;
        }
        Ok(output)
    }
    pub fn run(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        let command = self.command.clone();
        if command.trim().is_empty() {
            return Err("Type one supported command, then choose Run command.".into());
        }
        self.append(&format!("# {command}\n"));
        match self.apply(task, &command) {
            Ok(output) => {
                self.append(output.trim_end_matches('\n'));
                self.append("\n");
                self.command.clear();
            }
            Err(error) => self.append(&format!("Command refused: {error}\n")),
        }
        Ok(())
    }
    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        self.checked = Some(self.current.clone());
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub context: String,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}

impl Session {
    pub fn new(definition: &Definition) -> Result<Self, String> {
        definition.validate()?;
        Ok(Self::fresh(definition))
    }
    fn fresh(definition: &Definition) -> Self {
        Self {
            version: 1,
            context: definition.initial.id.clone(),
            view: View::Introduction,
            assignment: Draft::new(&definition.initial),
            practice: Draft::new(&definition.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !valid_id(&self.context) {
            return Err("Invalid saved EFI workspace version or identity.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != definition.initial.id {
            return Err("This saved EFI work belongs to another assignment.".into());
        }
        definition.initial.shape()?;
        definition.example.shape()?;
        self.assignment.bound(&definition.initial)?;
        self.practice.bound(&definition.example)
    }
    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        command: &str,
    ) -> Result<String, String> {
        self.bound(definition)?;
        if command == CHECK_ACTION {
            self.assignment.test(&definition.initial)?;
            let checks = definition.checks(self);
            return Ok(format!(
                "EFI requirements met: {}/{}",
                checks.iter().filter(|row| row.0).count(),
                checks.len()
            ));
        }
        let output = self.assignment.apply(&definition.initial, command)?;
        self.assignment.append(&format!("# {command}\n{output}"));
        Ok(output)
    }
}

fn admitted(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::Goal::EfiLoader { definition },
        crate::engine::State::EfiLoader { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("Boot-entry task/source mismatch.".into());
    };
    let fresh = Session::fresh(definition);
    let expected_track = if definition.initial.media.is_some() {
        "LPIC-2"
    } else {
        "LPIC-1"
    };
    if s.track != expected_track
        || definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != fresh
    {
        return Err("Boot-entry source, fresh session, track and objective must match.".into());
    }
    Ok(definition)
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let d = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = admitted(s)?;
    let crate::engine::State::EfiLoader { simulation } = &a.state else {
        return Err("Boot-entry work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible EFI-loader metadata; original retained.".into());
    }
    simulation.bound(d)
}
