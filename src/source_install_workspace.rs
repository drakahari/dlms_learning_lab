//! Source-bound drafts for the supplied-project activity. No serialized world or
//! cached outcome is trusted: the finite command history is replayed in process.
use crate::{script_teaching::View, source_install as core};
use core::{Artifact, Configuration, Facts, File, Output, World, command};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
pub const MAX_HISTORY: usize = 96;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Command(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Path(#[serde(deserialize_with = "crate::cron::text::<_,128>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Argument(#[serde(deserialize_with = "crate::cron::text::<_,128>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Argument,4>")]
    pub args: Vec<Argument>,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub stdout: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub stderr: String,
    pub status: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Destination {
    Build,
    Staged {
        #[serde(deserialize_with = "crate::cron::text::<_,128>")]
        directory: String,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Path,16>")]
    pub directories: Vec<Path>,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub initial_directory: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,16>")]
    pub starter: Vec<Command>,
    pub required: Configuration,
    pub destination: Destination,
    pub require_self_check: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,64>")]
    pub reference: Vec<Command>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
    pub expected: Option<Output>,
    pub actual: Option<Output>,
}
fn row(label: impl Into<String>, correct: bool, yes: &str, no: &str) -> Check {
    Check {
        label: label.into(),
        correct,
        feedback: if correct { yes } else { no }.into(),
        expected: None,
        actual: None,
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub world: World,
    pub cwd: String,
}
impl Task {
    pub fn check_label(&self, practice: bool) -> &'static str {
        match (&self.destination, practice) {
            (Destination::Build, false) => "Check build",
            (Destination::Build, true) => "Check example build",
            (Destination::Staged { .. }, false) => "Check installation",
            (Destination::Staged { .. }, true) => "Check example installation",
        }
    }
    pub fn initial(&self) -> Result<Frame> {
        let mut world = World::new(&self.facts)?;
        command::prepare_directories(
            &mut world,
            &self
                .directories
                .iter()
                .map(|p| p.0.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())?;
        if !world.directories.contains(&self.initial_directory) {
            return Err("Supply an existing initial virtual directory.".into());
        }
        Ok(Frame {
            world,
            cwd: self.initial_directory.clone(),
        })
    }
    pub fn replay(&self, history: &[Command]) -> Result<Frame> {
        if history.len() > MAX_HISTORY {
            return Err("Virtual command history exceeds96entries.".into());
        }
        let mut frame = self.initial()?;
        for command in history {
            core::command::run(&mut frame.world, &self.facts, &mut frame.cwd, &command.0)
                .map_err(|e| format!("Saved command cannot be replayed: {e}"))?;
        }
        Ok(frame)
    }
    pub fn program_path(&self, frame: &Frame) -> Result<String> {
        match &self.destination {
            Destination::Build => frame.world.binary_member(&self.facts),
            Destination::Staged { directory } => {
                World::installed_paths(&self.facts, &self.required, Some(directory)).map(|p| p.0)
            }
        }
    }
    pub fn grade(&self, frame: &Frame) -> Vec<Check> {
        let mut checks = vec![row(
            "Configured project",
            frame.world.configuration.as_ref() == Some(&self.required),
            "The selected prefix and project settings match the request.",
            "Inspect config.status. One or more configured project settings differ from the request; reconfigure before rebuilding.",
        )];
        if !checks[0].correct {
            checks[0].feedback=match &frame.world.configuration {
                None=>"No project configuration exists yet. Unpack the supplied archive, enter its source directory and configure the requested settings.".into(),
                Some(actual)=>{
                    let mut differences=Vec::new();
                    if actual.prefix!=self.required.prefix {differences.push(format!("prefix {:?} (requested {:?})",actual.prefix,self.required.prefix));}
                    if actual.greeting!=self.required.greeting {differences.push(format!("greeting {:?} (requested {:?})",actual.greeting,self.required.greeting));}
                    if actual.uppercase!=self.required.uppercase {differences.push(format!("uppercase {} (requested {})",if actual.uppercase{"enabled"}else{"disabled"},if self.required.uppercase{"enabled"}else{"disabled"}));}
                    format!("Configuration differs: {}. Reconfigure these settings before rebuilding.",differences.join("; "))
                }
            };
        }
        if matches!(self.destination, Destination::Build) {
            checks.push(row("Current build",frame.world.is_build_current(&self.facts),"The build matches the current supplied source/configuration.","The build is missing or belongs to an earlier configuration; build the supplied project again."));
        }
        if self.require_self_check {
            checks.push(row("Current package self-check",frame.world.is_check_current(&self.facts),"The package self-check applies to the current build.","Run the supplied check target after the current configuration. An old passing check does not verify a changed build."));
        }
        let path = self.program_path(frame);
        let artifact = path
            .as_ref()
            .ok()
            .and_then(|path| frame.world.files.get(path));
        let expected_artifact = Artifact {
            program: self.facts.program.clone(),
            configuration: self.required.clone(),
        };
        checks.push(row("Requested program and permissions",matches!(artifact,Some(File::Program{artifact,mode:0o755})if artifact==&expected_artifact),"The requested virtual program is present with executable permissions and the requested compiled settings.","Inspect the actual program path and run its --prefix/--version options. A build or staged copy can still have old compiled settings; configuring alone does not replace it."));
        if let Destination::Staged { directory } = &self.destination {
            let expected = World::installed_paths(&self.facts, &self.required, Some(directory));
            let valid = expected
                .as_ref()
                .ok()
                .and_then(|(_, manual)| frame.world.files.get(manual))
                .is_some_and(
                    |f| matches!(f,File::Text{text,mode:0o644}if text==&self.facts.manual),
                );
            checks.push(row("Staged documentation",valid,"The package documentation is staged at its requested destination with mode 0644.","The requested staged documentation is missing, changed or has the wrong mode. Install the complete supplied package with the requested prefix and staging root."));
        }
        for case in &self.cases {
            let expected = Output {
                stdout: case.stdout.clone(),
                stderr: case.stderr.clone(),
                status: case.status,
            };
            let actual = path.as_ref().map_err(Clone::clone).and_then(|path| {
                frame.world.run(
                    path,
                    &case.args.iter().map(|a| a.0.clone()).collect::<Vec<_>>(),
                )
            });
            let mut check = row(
                format!("Program case · {}", case.label),
                actual.as_ref().is_ok_and(|a| a == &expected),
                "The current program's output and exit status match this case.",
                "This case's output or exit status differs. Check the actual installed copy, its compiled settings and the expected versus actual results.",
            );
            if let Err(error) = &actual {
                check.feedback = format!("Cannot test this case: {error}");
            }
            check.expected = Some(expected);
            check.actual = actual.ok();
            checks.push(check);
        }
        for file in &self.facts.live_files {
            let expected = File::Text {
                text: file.text.clone(),
                mode: file.mode,
            };
            checks.push(row(format!("Preserve {}",file.path),frame.world.files.get(&file.path)==Some(&expected),"The supplied existing file and mode are preserved.","An existing virtual file changed. Omitting DESTDIR can install into the live prefix; reset this attempt if the original copy was overwritten, then stage the repair."));
        }
        checks
    }
    pub fn validate(&self) -> Result<()> {
        self.facts.validate()?;
        self.required.validate()?;
        if !identifier(&self.id)
            || self.directories.len() > 16
            || self.starter.len() > 16
            || self.reference.is_empty()
            || self.reference.len() > 64
            || self.starter.len() + self.reference.len() > MAX_HISTORY
            || !(3..=8).contains(&self.cases.len())
        {
            return Err("Supply a bounded identified project task, three to eight behavior cases and a reachable reference method.".into());
        }
        if let Destination::Staged { directory } = &self.destination {
            World::installed_paths(&self.facts, &self.required, Some(directory))?;
        }
        let approved = Artifact {
            program: self.facts.program.clone(),
            configuration: self.required.clone(),
        };
        let mut labels = std::collections::BTreeSet::new();
        let mut inputs = std::collections::BTreeSet::new();
        let mut names = std::collections::BTreeSet::new();
        for case in &self.cases {
            let args = case.args.iter().map(|a| a.0.clone()).collect::<Vec<_>>();
            if case.label.trim().is_empty()
                || case.label.len() > 64
                || case.label.chars().any(char::is_control)
                || !labels.insert(&case.label)
                || !inputs.insert(args.clone())
                || case.stdout.len() > 4096
                || case.stderr.len() > 4096
                || !(0..=125).contains(&case.status)
            {
                return Err(
                    "Give distinct labelled bounded program cases and valid exit statuses.".into(),
                );
            }
            if args.len() == 1 && !args[0].starts_with('-') {
                names.insert(args[0].clone());
            }
            let out = approved.run(&args)?;
            if (out.stdout, out.stderr, out.status)
                != (case.stdout.clone(), case.stderr.clone(), case.status)
            {
                return Err(format!(
                    "Authored expected output disagrees with supplied program semantics: {}",
                    case.label
                ));
            }
        }
        if names.len() < 2 {
            return Err(
                "Test at least two distinct name inputs, not just version/prefix constants.".into(),
            );
        }
        let starter = self.replay(&self.starter)?;
        if self.grade(&starter).iter().all(|r| r.correct) {
            return Err(
                "The assignment starter already passes; preserve meaningful independent work."
                    .into(),
            );
        }
        let mut method = self.starter.clone();
        method.extend(self.reference.clone());
        let solved = self.replay(&method)?;
        if !self.grade(&solved).iter().all(|r| r.correct) {
            return Err(
                "Reference commands do not reach every required state and behavior case.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.required == self.example.required
            || self.initial.facts == self.example.facts
        {
            return Err("Use separate source/task identities and different worked values.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub command: Command,
    pub before: usize,
    pub appended: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Command,96>")]
    pub history: Vec<Command>,
    pub last_run: Option<RunRecord>,
    pub checked: bool,
    pub attempted: bool,
}
impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            task: task.id.clone(),
            input: String::new(),
            history: task.starter.clone(),
            last_run: None,
            checked: false,
            attempted: false,
        }
    }
    fn shape(&self, task: &Task) -> Result<()> {
        if self.task != task.id
            || self.input.len() > 1024
            || self.history.len() > MAX_HISTORY
            || !self.history.starts_with(&task.starter)
        {
            return Err("Saved project draft belongs to another task, exceeds its limits or erased the inherited work; original retained.".into());
        }
        if self.history.iter().any(|c| c.0.len() > 1024) {
            return Err("Saved project command exceeds1024bytes.".into());
        }
        Ok(())
    }
    pub fn validate(&self, task: &Task) -> Result<()> {
        self.shape(task)?;
        let frame = task.replay(&self.history)?;
        if self.checked && (!self.attempted || !task.grade(&frame).iter().all(|r| r.correct)) {
            return Err(
                "Saved project success is inconsistent with replayed current state.".into(),
            );
        }
        if self.last_run.is_some() {
            self.last_output(task)?;
        }
        Ok(())
    }
    pub fn last_output(
        &self,
        task: &Task,
    ) -> Result<Option<std::result::Result<Output, crate::boot_entry::Error>>> {
        self.shape(task)?;
        let Some(record) = &self.last_run else {
            return Ok(None);
        };
        if record.command.0.len() > 1024
            || record.before < task.starter.len()
            || record.before > self.history.len()
        {
            return Err("Invalid saved project run position.".into());
        }
        if record.appended {
            if record.before + 1 != self.history.len()
                || self.history[record.before] != record.command
            {
                return Err("Saved project run claims a missing command.".into());
            }
        } else if record.before != self.history.len() {
            return Err("Failed project run must refer to the current history tip.".into());
        }
        if !record.appended && record.before == MAX_HISTORY {
            return Ok(Some(Err(crate::boot_entry::Error {
                kind: crate::boot_entry::ErrorKind::Limit,
                message:
                    "This attempt reached96commands; reset it to start over. No new command ran."
                        .into(),
            })));
        }
        let mut frame = task.replay(&self.history[..record.before])?;
        let output = command::run(
            &mut frame.world,
            &task.facts,
            &mut frame.cwd,
            &record.command.0,
        );
        if output.is_ok() != record.appended {
            return Err(
                "Saved project run's success flag disagrees with actual parsing/evaluation.".into(),
            );
        }
        Ok(Some(output))
    }
    pub fn run(
        &mut self,
        task: &Task,
    ) -> Result<std::result::Result<Output, crate::boot_entry::Error>> {
        self.validate(task)?;
        let mut frame = task.replay(&self.history)?;
        let before = self.history.len();
        let output = if before == MAX_HISTORY {
            Err(crate::boot_entry::Error {
                kind: crate::boot_entry::ErrorKind::Limit,
                message:
                    "This attempt reached96commands; reset it to start over. No new command ran."
                        .into(),
            })
        } else {
            command::run(&mut frame.world, &task.facts, &mut frame.cwd, &self.input)
        };
        let record = RunRecord {
            command: Command(self.input.clone()),
            before,
            appended: output.is_ok(),
        };
        if output.is_ok() {
            let prior = task.replay(&self.history)?;
            if frame.world != prior.world {
                self.checked = false;
                self.attempted = false;
            }
            self.history.push(record.command.clone());
        }
        self.last_run = Some(record);
        Ok(output)
    }
    pub fn check(&mut self, task: &Task) -> Result<Vec<Check>> {
        // Recompute rather than requiring a previously trusted success flag.
        self.shape(task)?;
        let frame = task.replay(&self.history)?;
        let checks = task.grade(&frame);
        self.attempted = true;
        self.checked = checks.iter().all(|r| r.correct);
        Ok(checks)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
    pub fn validate(&self, d: &Definition) -> Result<()> {
        self.shape()?;
        if self.version != 1 {
            return Err("Unknown supplied-project workspace version; original retained.".into());
        }
        self.assignment.validate(&d.initial)?;
        self.practice.validate(&d.example)
    }
    pub fn solved(&self, d: &Definition) -> bool {
        self.validate(d).is_ok()
            && self.assignment.checked
            && d.initial
                .replay(&self.assignment.history)
                .is_ok_and(|f| d.initial.grade(&f).iter().all(|r| r.correct))
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Draft::new(&d.initial);
    }
    pub fn reset_practice(&mut self, d: &Definition) {
        self.practice = Draft::new(&d.example);
    }
}

pub const HELP: &str = command::HELP;
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(error) = s.validate(self) {
            return vec![(false, error)];
        }
        let (draft, task) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !draft.attempted {
            return vec![(
                false,
                format!(
                    "Not checked. Run the supplied build/program operations, then choose {}.",
                    task.check_label(practice)
                ),
            )];
        }
        match task.replay(&draft.history) {
            Ok(frame) => task
                .grade(&frame)
                .into_iter()
                .map(|r| (r.correct, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(error) => vec![(false, error)],
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Reference {
    Check,
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unknown source-project workspace version; original retained.".into());
        }
        for draft in [&self.assignment, &self.practice] {
            if !identifier(&draft.task)
                || draft.input.len() > 1024
                || draft.history.len() > MAX_HISTORY
                || draft.history.iter().any(|c| c.0.len() > 1024)
                || draft
                    .last_run
                    .as_ref()
                    .is_some_and(|r| r.before > draft.history.len() || r.command.0.len() > 1024)
            {
                return Err(
                    "Saved source-project input, history or run observation exceeds its bounds."
                        .into(),
                );
            }
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, def: &Definition, raw: &str) -> Result<String> {
        self.validate(def)?;
        let mut next = self.clone();
        if let Some(raw) = raw.strip_prefix("source-action ") {
            if raw.len() > 128 {
                return Err("Simulator-only source operation exceeds128bytes.".into());
            }
            let action: Reference = serde_json::from_str(raw).map_err(
                |_| "Invalid simulator-only source operation; this helper is not a Linux command.",
            )?;
            match action {
                Reference::Check => {
                    next.assignment.check(&def.initial)?;
                }
            }
            next.validate(def)?;
            *self = next;
            Ok("Checked simulated source-installation requirements.".into())
        } else {
            next.assignment.input = raw.into();
            let output = next
                .assignment
                .run(&def.initial)?
                .map_err(|e| e.to_string())?;
            if output.status != 0 {
                return Err(format!(
                    "Program exited{}: {}",
                    output.status, output.stderr
                ));
            }
            next.validate(def)?;
            *self = next;
            Ok(output.stdout)
        }
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::SourceInstall { simulation },
        crate::engine::Goal::SourceInstall { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Source-install state and objective must use the same authored task.".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err(
            "Source-install work must start from separate declared assignment/practice drafts."
                .into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::SourceInstall { .. })
        || matches!(s.initial, crate::engine::State::SourceInstall { .. })
        || matches!(s.goal, crate::engine::Goal::SourceInstall { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let definition = source(s)?;
    let crate::engine::State::SourceInstall { simulation } = &a.state else {
        return Err("Saved source-install work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible source-install metadata; original progress retained.".into());
    }
    simulation.validate(definition)
}
