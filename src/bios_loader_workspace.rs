//! Source-bound BIOS installation console; no host execution or saved task authority.
use crate::{bios_loader as core, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const CHECK_ACTION: &str = "@bios-check";
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn strings<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
    crate::cron_workspace::sequence::<_, Text, N>(d).map(|v| v.into_iter().map(|v| v.0).collect())
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
    #[serde(deserialize_with = "strings::<_,8>")]
    pub reference: Vec<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub required_root: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub required_release: String,
    pub read_only: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub target: String,
    #[serde(deserialize_with = "strings::<_,4>")]
    pub preserve: Vec<String>,
}
impl Task {
    pub fn shape(&self) -> Result<(), String> {
        self.facts.validate().map_err(|e| e.to_string())?;
        self.initial.bound(&self.facts).map_err(|e| e.to_string())?;
        if !id(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 2048
            || self.initial.checked.is_some()
            || self.reference.is_empty()
            || self.reference.len() > 8
            || self
                .reference
                .iter()
                .any(|s| s.len() > 1024 || s.contains('\0'))
            || !matches!(self.target.as_str(), "multi-user.target" | "rescue.target")
            || self.preserve.len() > 4
            || self
                .preserve
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != self.preserve.len()
        {
            return Err(
                "Invalid supplied BIOS task identity, reference or preservation scope.".into(),
            );
        }
        if self
            .preserve
            .iter()
            .any(|name| !self.initial.records.iter().any(|r| &r.disk == name))
        {
            return Err(
                "Preservation requirements must name existing supplied boot records.".into(),
            );
        }
        if !self
            .facts
            .directories
            .iter()
            .filter_map(|d| d.configuration.as_ref())
            .any(|c| {
                c.inventory
                    .kernels
                    .iter()
                    .any(|k| k.release == self.required_release)
                    && c.inventory
                        .roots
                        .iter()
                        .any(|r| r.uuid.eq_ignore_ascii_case(&self.required_root))
            })
        {
            return Err(
                "Requested system/kernel must occur in the supplied boot-image facts.".into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, world: &core::World) -> Vec<(bool, String)> {
        if let Err(e) = world.bound(&self.facts) {
            return vec![(false, e.to_string())];
        }
        if !world.current_test() {
            return vec![(false,"Not tested for the current installation. Use Test boot chain after changing loader files.".into())];
        }
        match world.trace(&self.facts) {
            Err(e) => vec![(false, e.to_string())],
            Ok(trace) => {
                let mut checks = trace
                    .stages
                    .iter()
                    .map(|s| (s.passed, format!("{}: {}", s.title, s.detail)))
                    .collect::<Vec<_>>();
                if let Some(r) = &trace.handoff {
                    checks.extend([
                        (
                            r.requested.root.eq_ignore_ascii_case(&self.required_root),
                            "Configured root=UUID= must name the requested system filesystem."
                                .into(),
                        ),
                        (
                            r.release.as_deref() == Some(self.required_release.as_str()),
                            format!("Use the supplied kernel release {}.", self.required_release),
                        ),
                        (
                            r.read_only == self.read_only,
                            format!(
                                "The requested initial root mode is {}.",
                                if self.read_only { "ro" } else { "rw" }
                            ),
                        ),
                        (
                            r.requested.target == self.target,
                            format!("The configured startup request must be {}.", self.target),
                        ),
                    ]);
                }
                checks.push((
                    trace.passed(),
                    "Reach the complete modeled init handoff through the selected disk.".into(),
                ));
                for name in &self.preserve {
                    let expected = self.initial.records.iter().find(|r| &r.disk == name);
                    let actual = world.records.iter().find(|r| &r.disk == name);
                    let same = expected == actual;
                    checks.push((same,if same{format!("The supplied loader on {name} is preserved.")}else{format!("The supplied loader on {name} changed. This subset cannot restore other loaders; Reset this attempt to restore the supplied backup, then retry the intended disk.")}));
                }
                checks
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let mut start = self.initial.clone();
        start.test(&self.facts).map_err(|e| e.to_string())?;
        if self.checks(&start).iter().all(|c| c.0) {
            return Err(
                "A BIOS starter must need a real installation repair, not only a Test click."
                    .into(),
            );
        }
        let mut world = self.initial.clone();
        for command in &self.reference {
            if command == CHECK_ACTION {
                world.test(&self.facts).map_err(|e| e.to_string())?;
            } else {
                world
                    .execute(&self.facts, command)
                    .map_err(|e| e.to_string())?;
            }
        }
        if self.checks(&world).iter().any(|c| !c.0) {
            return Err("The supplied BIOS reference must reach every task outcome and preserve protected loaders.".into());
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
            || self
                .initial
                .required_root
                .eq_ignore_ascii_case(&self.example.required_root)
            || self.initial.reference == self.example.reference
        {
            return Err(
                "Use version1 and genuinely separate BIOS assignment/workshop facts.".into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        self.initial.checks(&s.assignment.current)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub current: core::World,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub transcript: String,
}
impl Draft {
    pub fn shape(&self) -> Result<(), String> {
        if self.command.len() > 1024
            || self.transcript.len() > 8192
            || self.command.contains('\0')
            || self.transcript.contains('\0')
        {
            return Err("Saved BIOS input/output exceeds its bounds; original retained.".into());
        }
        self.current.shape().map_err(|e| e.to_string())
    }
    pub fn new(t: &Task) -> Self {
        Self {
            current: t.initial.clone(),
            command: String::new(),
            transcript: String::new(),
        }
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        self.shape()?;
        if self.command.len() > 1024
            || self.transcript.len() > 8192
            || self.command.contains('\0')
            || self.transcript.contains('\0')
        {
            return Err("Saved BIOS command/output exceeds its bound; original retained.".into());
        }
        self.current.bound(&t.facts).map_err(|e| e.to_string())
    }
    fn append(&mut self, text: &str) {
        self.transcript.push_str(text);
        if self.transcript.len() > 8192 {
            let mut from = self.transcript.len() - 8100;
            while !self.transcript.is_char_boundary(from) {
                from += 1;
            }
            self.transcript.drain(..from);
            self.transcript
                .insert_str(0, "Earlier console output omitted.\n");
        }
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        let command = self.command.clone();
        if command.trim().is_empty() {
            return Err("Type one supported command, then choose Run command.".into());
        }
        self.append(&format!("# {command}\n"));
        match self.current.execute(&t.facts, &command) {
            Ok(out) => {
                self.append(&out);
                if !out.ends_with('\n') {
                    self.append("\n");
                }
                self.command.clear();
            }
            Err(e) => self.append(&format!("{:?}: {}\n", e.kind, e.message)),
        }
        Ok(())
    }
    pub fn test(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.current.test(&t.facts).map_err(|e| e.to_string())?;
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
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !id(&self.context) {
            return Err("Invalid saved BIOS workspace version/identity.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            context: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        })
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.version != 1 || self.context != d.initial.id {
            return Err(
                "Saved BIOS work belongs to another task/version; original retained.".into(),
            );
        }
        d.initial.shape()?;
        d.example.shape()?;
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    /// Reference/engine simulator action; the native terminal does not teach it as Linux.
    pub fn execute(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        if raw == CHECK_ACTION {
            self.assignment.test(&d.initial)?;
            let rows = d.checks(self);
            return Ok(format!(
                "Simulated BIOS task outcomes: {}/{} match",
                rows.iter().filter(|c| c.0).count(),
                rows.len()
            ));
        }
        let out = self
            .assignment
            .current
            .execute(&d.initial.facts, raw)
            .map_err(|e| e.to_string())?;
        self.assignment.append(&format!("# {raw}\n{out}"));
        Ok(out)
    }
}

fn admitted(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::Goal::BiosLoader { definition },
        crate::engine::State::BiosLoader { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("Boot-entry task/source mismatch.".into());
    };
    let fresh = Session {
        version: 1,
        context: definition.initial.id.clone(),
        view: View::Introduction,
        assignment: Draft::new(&definition.initial),
        practice: Draft::new(&definition.example),
    };
    if s.track != "LPIC-1"
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
    let crate::engine::State::BiosLoader { simulation } = &a.state else {
        return Err("Boot-entry work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible BIOS-loader metadata; original retained.".into());
    }
    simulation.bound(d)
}
