//! Source-bound library-search practice with independent drafts; no host APIs.
pub use crate::shared_libraries as core;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

pub const VERSION: u32 = 1;
pub const MAX_DEFINITION: usize = 256 * 1024;
pub const MAX_WORKSPACE: usize = 160 * 1024;
pub const MAX_SESSION: usize = 512 * 1024;
pub const MAX_COMMAND: usize = 2048;
pub const MAX_ROWS: usize = 12;
pub const HELP: &str = "Supported simulated commands: ldd PROGRAM; env LD_LIBRARY_PATH=DIRS ldd PROGRAM; ldconfig; ldconfig -p. These names inspect supplied metadata or rebuild the virtual cache only. No real utility, ELF code or host path runs. DIRS uses supplied absolute directories separated by colons; semicolons must be quoted. Expansion, shell composition, redirection, flags other than -p, includes and link repair are unsupported. Results are typed simulator observations, not claims about real utility exit statuses. A one-off env inspection does not save a launch recipe. Launch recipes are simulated per-program settings, not a Linux configuration file.";

fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<String, E> {
            if value.len() > N {
                Err(E::custom("Library workspace text limit"))
            } else {
                Ok(value.to_owned())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> serde::de::Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} items")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
            let mut out = vec![];
            while out.len() < N {
                match a.next_element()? {
                    Some(value) => out.push(value),
                    None => return Ok(out),
                }
            }
            struct Excess;
            impl<'a> Deserialize<'a> for Excess {
                fn deserialize<D: Deserializer<'a>>(_: D) -> Result<Self, D::Error> {
                    Err(serde::de::Error::custom("Library workspace item limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn optional_environment<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Value(#[serde(deserialize_with = "text::<_,1024>")] String);
    Option::<Value>::deserialize(d).map(|v| v.map(|v| v.0))
}
#[derive(Deserialize)]
struct Id(#[serde(deserialize_with = "text::<_,64>")] String);
fn identities<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<Vec<String>, D::Error> {
    list::<D, Id, N>(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
#[derive(Deserialize)]
struct Path(#[serde(deserialize_with = "text::<_,256>")] String);
fn paths<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<Vec<String>, D::Error> {
    list::<D, Path, N>(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
fn fits<T: Serialize>(value: &T, cap: usize) -> bool {
    struct Counter {
        bytes: usize,
        cap: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|v| *v <= self.cap)
                .ok_or_else(|| std::io::Error::other("Library workspace byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { bytes: 0, cap }, value).is_ok()
}
fn unique<T: Ord>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = BTreeSet::new();
    items.into_iter().all(|v| seen.insert(v))
}
fn valid_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.starts_with("lib")
        && s.contains(".so")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
}
fn model(state: &core::State) -> Result<(), String> {
    state.validate().map_err(|e| e.to_string())?;
    if state.library_path.is_some() {
        return Err("Persisted loader state must be clean; use explicit simulated per-program launch recipes instead".into());
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    #[default]
    Introduction,
    Example,
    Assignment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Assignment,
    Example,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    Clean,
    SavedRecipe,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionRule {
    #[serde(deserialize_with = "text::<_,128>")]
    pub needed: String,
    #[serde(deserialize_with = "identities::<_,4>")]
    pub allowed_identities: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchGoal {
    #[serde(deserialize_with = "text::<_,256>")]
    pub program: String,
    pub mode: LaunchMode,
    #[serde(deserialize_with = "list::<_,SelectionRule,24>")]
    pub selections: Vec<SelectionRule>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "list::<_,LaunchGoal,4>")]
    pub launches: Vec<LaunchGoal>,
    #[serde(deserialize_with = "paths::<_,4>")]
    pub protected_programs: Vec<String>,
    pub require_current_cache: bool,
    pub require_clean_recipes: bool,
    /// Temporary-search assignments must not be bypassed through global changes.
    pub preserve_configuration_and_cache: bool,
}
impl Plan {
    fn validate(&self, initial: &core::State, declarations: &[LaunchRecipe]) -> Result<(), String> {
        let recipes = complete_recipes(initial, declarations)?;
        if self.launches.is_empty()
            || self.launches.len() > 4
            || self.protected_programs.len() > 4
            || !unique(self.launches.iter().map(|g| &g.program))
            || !unique(self.protected_programs.iter())
        {
            return Err("Invalid bounded library assignment plan".into());
        }
        for goal in &self.launches {
            if !initial
                .catalog
                .programs
                .iter()
                .any(|p| p.path == goal.program)
                || goal.selections.is_empty()
                || goal.selections.len() > 24
                || !unique(goal.selections.iter().map(|r| &r.needed))
            {
                return Err("Each launch goal needs a supplied program and distinct dependency selection rules".into());
            }
            for rule in &goal.selections {
                if !valid_name(&rule.needed)
                    || rule.allowed_identities.is_empty()
                    || rule.allowed_identities.len() > 4
                    || !unique(rule.allowed_identities.iter())
                    || rule
                        .allowed_identities
                        .iter()
                        .any(|id| !initial.catalog.objects.iter().any(|o| &o.identity == id))
                {
                    return Err("Dependency rules must name bounded supplied identities, not expected answer strings".into());
                }
            }
        }
        for path in &self.protected_programs {
            if self.require_clean_recipes
                && recipes
                    .iter()
                    .find(|r| &r.program == path)
                    .is_some_and(|r| r.environment.is_some())
            {
                return Err("A clean-recipe handover cannot also preserve an initially nonempty protected recipe".into());
            }
            if self.launches.iter().any(|g| &g.program == path)
                || !initial
                    .resolve(path, core::Environment::Clean)
                    .map_err(|e| e.to_string())?
                    .complete()
                || !recipe_resolution(initial, &recipes, path)
                    .map_err(|e| e.to_string())?
                    .complete()
            {
                return Err("Protect a distinct supplied program whose initial clean and saved-recipe dependency closures are complete".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "text::<_,100>")]
    pub title: String,
    #[serde(deserialize_with = "text::<_,1400>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkedAction {
    Configuration {
        #[serde(deserialize_with = "text::<_,4096>")]
        text: String,
    },
    LaunchRecipe {
        #[serde(deserialize_with = "text::<_,256>")]
        program: String,
        #[serde(default, deserialize_with = "optional_environment")]
        environment: Option<String>,
    },
    RebuildCache,
    Inspect {
        #[serde(deserialize_with = "text::<_,2048>")]
        command: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    #[serde(deserialize_with = "text::<_,120>")]
    pub title: String,
    #[serde(deserialize_with = "text::<_,900>")]
    pub explanation: String,
    pub action: WorkedAction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "text::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "list::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    pub initial: core::State,
    #[serde(default, deserialize_with = "list::<_,LaunchRecipe,8>")]
    pub initial_recipes: Vec<LaunchRecipe>,
    pub plan: Plan,
    #[serde(deserialize_with = "list::<_,Step,12>")]
    pub worked: Vec<Step>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    #[serde(default, deserialize_with = "list::<_,LaunchRecipe,8>")]
    pub initial_recipes: Vec<LaunchRecipe>,
    pub plan: Plan,
    pub lesson: Lesson,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if !fits(self, MAX_DEFINITION) {
            return Err("Library task definition exceeds its byte limit".into());
        }
        model(&self.initial)?;
        model(&self.lesson.initial)?;
        self.plan.validate(&self.initial, &self.initial_recipes)?;
        self.lesson
            .plan
            .validate(&self.lesson.initial, &self.lesson.initial_recipes)?;
        if self.lesson.prerequisite_note.is_empty()
            || self.lesson.prerequisite_note.len() > 1600
            || self.lesson.description.is_empty()
            || self.lesson.description.len() > 1600
            || self.lesson.concepts.is_empty()
            || self.lesson.concepts.len() > 6
            || self.lesson.worked.is_empty()
            || self.lesson.worked.len() > 12
            || self.lesson.concepts.iter().any(|c| {
                c.title.is_empty()
                    || c.title.len() > 100
                    || c.text.is_empty()
                    || c.text.len() > 1400
            })
            || self.lesson.worked.iter().any(|s| {
                s.title.is_empty()
                    || s.title.len() > 120
                    || s.explanation.is_empty()
                    || s.explanation.len() > 900
            })
        {
            return Err(
                "Supply bounded prerequisite teaching, worked actions and explanations".into(),
            );
        }
        if self.initial.catalog == self.lesson.initial.catalog {
            return Err("Use different supplied data for the separate worked example".into());
        }
        for (initial, recipes, plan) in [
            (&self.initial, &self.initial_recipes, &self.plan),
            (
                &self.lesson.initial,
                &self.lesson.initial_recipes,
                &self.lesson.plan,
            ),
        ] {
            if report(&Workspace::new(initial, recipes)?, initial, recipes, plan).passed() {
                return Err("Keep meaningful unfinished work in both library tasks".into());
            }
        }
        for step in &self.lesson.worked {
            match &step.action {
                WorkedAction::Configuration { text } => {
                    if text.len() > core::MAX_CONFIG {
                        return Err("Worked configuration exceeds limit".into());
                    }
                }
                WorkedAction::LaunchRecipe {
                    program,
                    environment,
                } => {
                    if !self
                        .lesson
                        .initial
                        .catalog
                        .programs
                        .iter()
                        .any(|p| &p.path == program)
                    {
                        return Err("Worked launch recipe names an unknown example program".into());
                    }
                    let mut probe = self.lesson.initial.clone();
                    probe
                        .set_library_path(environment.as_deref())
                        .map_err(|e| e.to_string())?;
                }
                WorkedAction::Inspect { command } => {
                    parse(command).map_err(|e| e.message)?;
                }
                WorkedAction::RebuildCache => {}
            }
        }
        Ok(())
    }
    /// Explicit authoring gate only; never runs as a side effect of opening/restoring.
    /// Closed typed model operations, not shell or executable content.
    pub fn verify_worked(&self) -> Result<(), String> {
        self.validate()?;
        let mut session = Session::new(self)?;
        for step in &self.lesson.worked {
            match &step.action {
                WorkedAction::Configuration { text } => {
                    session.edit_configuration(self, Scope::Example, text.clone())?;
                    session.save_configuration(self, Scope::Example)?;
                }
                WorkedAction::LaunchRecipe {
                    program,
                    environment,
                } => {
                    session.edit_launch(
                        self,
                        Scope::Example,
                        program,
                        environment.clone().unwrap_or_default(),
                    )?;
                    session.save_launch(self, Scope::Example, program)?;
                }
                WorkedAction::RebuildCache => {
                    session.rebuild_cache(self, Scope::Example)?;
                }
                WorkedAction::Inspect { command } => {
                    let row = session.run(self, Scope::Example, command)?;
                    if row.outcome != Outcome::Supported {
                        return Err(format!(
                            "Worked inspection was not supported: {}",
                            row.output
                        ));
                    }
                }
            }
        }
        let report = session.check(self, Scope::Example)?;
        if !report.passed() {
            return Err("Worked example does not satisfy its independent example plan".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("Library definition byte limit".into());
        }
        let value: Self =
            serde_json::from_slice(raw).map_err(|e| format!("Invalid library task: {e}"))?;
        value.validate()?;
        Ok(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchRecipe {
    #[serde(deserialize_with = "text::<_,256>")]
    pub program: String,
    #[serde(default, deserialize_with = "optional_environment")]
    pub environment: Option<String>,
}
/// Fill the exact source program roster; omitted declarations are clean.
/// Source declarations are sparse, bounded and independently pinned by Session.
fn complete_recipes(
    initial: &core::State,
    declarations: &[LaunchRecipe],
) -> Result<Vec<LaunchRecipe>, String> {
    if declarations.len() > core::MAX_PROGRAMS || !unique(declarations.iter().map(|r| &r.program)) {
        return Err(
            "Initial simulated recipes must have distinct bounded supplied program keys".into(),
        );
    }
    for recipe in declarations {
        if !initial
            .catalog
            .programs
            .iter()
            .any(|p| p.path == recipe.program)
        {
            return Err("Initial simulated recipe names an unknown supplied program".into());
        }
        let mut probe = initial.clone();
        probe
            .set_library_path(recipe.environment.as_deref())
            .map_err(|e| e.to_string())?;
    }
    Ok(initial
        .catalog
        .programs
        .iter()
        .map(|program| {
            declarations
                .iter()
                .find(|r| r.program == program.path)
                .cloned()
                .unwrap_or_else(|| LaunchRecipe {
                    program: program.path.clone(),
                    environment: None,
                })
        })
        .collect())
}
fn recipe_resolution(
    initial: &core::State,
    recipes: &[LaunchRecipe],
    program: &str,
) -> core::Result<core::Resolution> {
    let recipe = recipes
        .iter()
        .find(|r| r.program == program)
        .ok_or_else(|| core::Error {
            kind: core::ErrorKind::NotFound,
            message: "Choose a supplied simulated launch recipe".into(),
        })?;
    let mut probe = initial.clone();
    probe.set_library_path(recipe.environment.as_deref())?;
    probe.resolve(program, core::Environment::Configured)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchDraft {
    #[serde(deserialize_with = "text::<_,256>")]
    pub program: String,
    /// Empty means clean environment; not a real Linux configuration file.
    #[serde(deserialize_with = "text::<_,1024>")]
    pub value: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Supported,
    Unsupported,
    Invalid,
    NotFound,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "text::<_,2048>")]
    pub input: String,
    #[serde(deserialize_with = "text::<_,32768>")]
    pub output: String,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workspace {
    pub state: core::State,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub configuration_draft: String,
    #[serde(deserialize_with = "list::<_,LaunchRecipe,8>")]
    pub recipes: Vec<LaunchRecipe>,
    #[serde(deserialize_with = "list::<_,LaunchDraft,8>")]
    pub launch_drafts: Vec<LaunchDraft>,
    #[serde(deserialize_with = "text::<_,2048>")]
    pub command: String,
    #[serde(deserialize_with = "list::<_,Row,12>")]
    pub transcript: Vec<Row>,
    /// Display gate only. Report/credit always re-evaluate current state and drafts.
    pub checked: bool,
}
impl Workspace {
    fn new(initial: &core::State, declarations: &[LaunchRecipe]) -> Result<Self, String> {
        let recipes = complete_recipes(initial, declarations)?;
        let launch_drafts = recipes
            .iter()
            .map(|r| LaunchDraft {
                program: r.program.clone(),
                value: r.environment.clone().unwrap_or_default(),
            })
            .collect();
        Ok(Self {
            state: initial.clone(),
            configuration_draft: initial.configuration.clone(),
            recipes,
            launch_drafts,
            command: String::new(),
            transcript: vec![],
            checked: false,
        })
    }
    fn validate(&self, initial: &core::State) -> Result<(), String> {
        if !fits(self, MAX_WORKSPACE)
            || self.configuration_draft.len() > core::MAX_CONFIG
            || self.command.len() > MAX_COMMAND
            || self.command.contains(['\0', '\n', '\r'])
            || self.transcript.len() > MAX_ROWS
            || self.transcript.iter().any(|r| {
                r.input.len() > MAX_COMMAND
                    || r.output.len() > core::MAX_OUTPUT
                    || r.input.contains(['\0', '\n', '\r'])
            })
        {
            return Err("Saved library workspace exceeds its input/output limits".into());
        }
        model(&self.state)?;
        if !self
            .state
            .environment_matches(initial)
            .map_err(|e| e.to_string())?
            || self.recipes.len() != initial.catalog.programs.len()
            || self.launch_drafts.len() != self.recipes.len()
        {
            return Err(
                "Saved work does not match this supplied library inventory and program roster"
                    .into(),
            );
        }
        for ((program, recipe), draft) in initial
            .catalog
            .programs
            .iter()
            .zip(&self.recipes)
            .zip(&self.launch_drafts)
        {
            if recipe.program != program.path
                || draft.program != program.path
                || draft.value.len() > core::MAX_PATH_LIST
            {
                return Err(
                    "Saved simulated launch recipes do not match the supplied program order".into(),
                );
            }
            let mut probe = self.state.clone();
            probe
                .set_library_path(recipe.environment.as_deref())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn pending(&self) -> bool {
        self.configuration_draft != self.state.configuration
            || self
                .launch_drafts
                .iter()
                .zip(&self.recipes)
                .any(|(d, r)| d.value != r.environment.as_deref().unwrap_or_default())
    }
    fn launch(&self, program: &str, mode: LaunchMode) -> core::Result<core::Resolution> {
        let mut probe = self.state.clone();
        if mode == LaunchMode::SavedRecipe {
            let recipe = self
                .recipes
                .iter()
                .find(|r| r.program == program)
                .ok_or_else(|| core::Error {
                    kind: core::ErrorKind::NotFound,
                    message: "Choose a supplied simulated launch recipe".into(),
                })?;
            probe.set_library_path(recipe.environment.as_deref())?;
        }
        probe.resolve(program, core::Environment::Configured)
    }
    fn record(&mut self, row: Row) -> Result<(), String> {
        if row.input.len() > MAX_COMMAND || row.output.len() > core::MAX_OUTPUT {
            return Err("Inspection output exceeds the bounded transcript limit".into());
        }
        while self.transcript.len() >= MAX_ROWS {
            self.transcript.remove(0);
        }
        self.transcript.push(row);
        while !fits(self, MAX_WORKSPACE) && self.transcript.len() > 1 {
            self.transcript.remove(0);
        }
        if !fits(self, MAX_WORKSPACE) {
            return Err("Workspace byte limit; previous state and transcript retained".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub initial: core::State,
    pub example_initial: core::State,
    #[serde(deserialize_with = "list::<_,LaunchRecipe,8>")]
    pub initial_recipes: Vec<LaunchRecipe>,
    #[serde(deserialize_with = "list::<_,LaunchRecipe,8>")]
    pub example_initial_recipes: Vec<LaunchRecipe>,
    pub assignment_plan: Plan,
    pub example_plan: Plan,
    pub assignment: Workspace,
    pub example: Workspace,
    pub view: View,
    pub selected_example: usize,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub feedback: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub checks: Vec<Check>,
}
impl Report {
    pub fn passed(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|c| c.passed)
    }
}
fn snapshot(r: &core::Resolution) -> Vec<(String, String, Option<String>)> {
    r.bindings
        .iter()
        .map(|b| {
            (
                b.requester.clone(),
                b.needed.clone(),
                b.selected.as_ref().map(|s| s.identity.clone()),
            )
        })
        .collect()
}
fn report(
    work: &Workspace,
    initial: &core::State,
    declarations: &[LaunchRecipe],
    plan: &Plan,
) -> Report {
    let mut checks = vec![];
    if work.pending() {
        checks.push(Check { passed: false, label: "Pending edits".into(), feedback: "Your editor or launch-setting draft has not been applied. Save it, or restore the saved value, before checking. A successful one-off env inspection does not save the assignment's simulated launch recipe or rebuild its cache.".into() });
    }
    for goal in &plan.launches {
        let result = work.launch(&goal.program, goal.mode);
        let (passed, feedback) = match result {
            Err(e) => (
                false,
                format!(
                    "{}: {}. No successful dependency result was inferred from this error or unsupported behavior.",
                    goal.program, e
                ),
            ),
            Ok(result) => {
                let mut problems = vec![];
                for binding in &result.bindings {
                    match &binding.selected {
                        None => problems.push(format!("{} needs {}, which was not found", binding.requester, binding.needed)),
                        Some(selected) => match goal.selections.iter().find(|r| r.needed == binding.needed) {
                            Some(rule) if rule.allowed_identities.contains(&selected.identity) => {},
                            _ => problems.push(format!("{} selected supplied build {} for {}; inspect its directory and dependency metadata", binding.requester, selected.identity, binding.needed)),
                        }
                    }
                }
                for rule in &goal.selections {
                    if !result.bindings.iter().any(|b| b.needed == rule.needed) {
                        problems.push(format!(
                            "The required {} dependency was not inspected",
                            rule.needed
                        ));
                    }
                }
                if problems.is_empty() && result.complete() {
                    (true, "All direct and transitive dependencies select the required supplied builds.".into())
                } else {
                    (
                        false,
                        problems.into_iter().take(4).collect::<Vec<_>>().join("; "),
                    )
                }
            }
        };
        let feedback = if !passed && goal.mode == LaunchMode::SavedRecipe {
            format!(
                "{feedback} A one-off env inspection is temporary: save this program's simulated launch setting explicitly, then check it. It does not create a Linux configuration file."
            )
        } else {
            feedback
        };
        checks.push(Check {
            passed,
            label: format!(
                "{} · {}",
                goal.program,
                if goal.mode == LaunchMode::Clean {
                    "clean launch"
                } else {
                    "saved simulated recipe"
                }
            ),
            feedback,
        });
    }
    if plan.preserve_configuration_and_cache {
        let configuration_same = work.state.configuration == initial.configuration;
        // Generation counts explicit rebuilds; it is not a library selection.
        // Preserve cache presence, search directories and every functional entry.
        let cache_same = match (&work.state.cache, &initial.cache) {
            (None, None) => true,
            (Some(actual), Some(expected)) => {
                // Core validation recomputes both snapshots before this outcome check.
                // A past source comment cannot change validated cache selections.
                actual.directories == expected.directories && actual.entries == expected.entries
            }
            _ => false,
        };
        checks.push(Check { passed: configuration_same && cache_same, label: "Preserve system search settings".into(),
            feedback: format!("Expected: the supplied saved configuration and cache entries stay unchanged; use only the target's simulated launch recipe. Actual: configuration {}, cache {}.",
                if configuration_same { "unchanged" } else { "changed" }, if cache_same { "unchanged" } else { "changed" }) });
    }
    if plan.require_current_cache {
        let okay = work.state.cache_matches_configuration().unwrap_or(false);
        checks.push(Check { passed: okay, label: "Saved configuration and rebuilt cache".into(), feedback: if okay { "The virtual cache was explicitly rebuilt from the current saved directory list.".into() } else { "Save the required virtual directory list, then rebuild the cache. Editing configuration alone leaves the previous cache in use.".into() } });
    }
    if plan.require_clean_recipes {
        checks.push(Check { passed: work.recipes.iter().all(|r| r.environment.is_none()), label: "Clean launch settings".into(), feedback: "Keep every simulated per-program environment override empty; a temporary workaround must not hide a clean-cache failure.".into() });
    }
    for program in &plan.protected_programs {
        let baseline = initial.resolve(program, core::Environment::Clean).ok();
        let same_clean = baseline
            .as_ref()
            .zip(work.launch(program, LaunchMode::Clean).ok().as_ref())
            .is_some_and(|(a, b)| a.complete() && b.complete() && snapshot(a) == snapshot(b));
        let expected_recipe = declarations
            .iter()
            .find(|r| &r.program == program)
            .and_then(|r| r.environment.as_ref());
        let unchanged_recipe = work
            .recipes
            .iter()
            .find(|r| &r.program == program)
            .is_some_and(|r| r.environment.as_ref() == expected_recipe);
        let source_recipes = complete_recipes(initial, declarations)
            .expect("Source declarations validated before reporting");
        let baseline_saved = recipe_resolution(initial, &source_recipes, program).ok();
        let same_recipe = baseline_saved
            .as_ref()
            .zip(work.launch(program, LaunchMode::SavedRecipe).ok().as_ref())
            .is_some_and(|(a, b)| a.complete() && b.complete() && snapshot(a) == snapshot(b));
        checks.push(Check { passed: same_clean && unchanged_recipe && same_recipe, label: format!("Preserve {program}"),
            feedback: format!("Expected: keep the unrelated program's original clean selections and unchanged supplied simulated recipe. Actual: clean selections {}; saved recipe {}.",
                if same_clean { "preserved" } else { "changed or incomplete" },
                if unchanged_recipe && same_recipe { "unchanged" } else { "changed or incomplete" }) });
    }
    Report { checks }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        let s = Self {
            version: VERSION,
            initial: d.initial.clone(),
            example_initial: d.lesson.initial.clone(),
            initial_recipes: d.initial_recipes.clone(),
            example_initial_recipes: d.lesson.initial_recipes.clone(),
            assignment_plan: d.plan.clone(),
            example_plan: d.lesson.plan.clone(),
            assignment: Workspace::new(&d.initial, &d.initial_recipes)?,
            example: Workspace::new(&d.lesson.initial, &d.lesson.initial_recipes)?,
            view: View::Introduction,
            selected_example: 0,
        };
        s.bound(d)?;
        Ok(s)
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        d.validate()?;
        if self.version != VERSION
            || !fits(self, MAX_SESSION)
            || self.initial != d.initial
            || self.example_initial != d.lesson.initial
            || self.assignment_plan != d.plan
            || self.example_plan != d.lesson.plan
            || self.initial_recipes != d.initial_recipes
            || self.example_initial_recipes != d.lesson.initial_recipes
            || self.selected_example >= d.lesson.worked.len()
        {
            return Err("Saved library work belongs to another task/version or exceeds its limits; original retained".into());
        }
        self.assignment.validate(&d.initial)?;
        self.example.validate(&d.lesson.initial)
    }
    fn work(&self, scope: Scope) -> &Workspace {
        match scope {
            Scope::Assignment => &self.assignment,
            Scope::Example => &self.example,
        }
    }
    fn work_mut(&mut self, scope: Scope) -> &mut Workspace {
        match scope {
            Scope::Assignment => &mut self.assignment,
            Scope::Example => &mut self.example,
        }
    }
    fn publish(&mut self, d: &Definition, next: Self) -> Result<(), String> {
        next.bound(d)?;
        *self = next;
        Ok(())
    }
    pub fn set_view(&mut self, d: &Definition, view: View) -> Result<(), String> {
        self.bound(d)?;
        let mut next = self.clone();
        next.view = view;
        self.publish(d, next)
    }
    pub fn select_example(&mut self, d: &Definition, selected: usize) -> Result<(), String> {
        self.bound(d)?;
        if selected >= d.lesson.worked.len() {
            return Err("Choose one supplied worked action".into());
        }
        let mut next = self.clone();
        next.selected_example = selected;
        self.publish(d, next)
    }
    pub fn edit_configuration(
        &mut self,
        d: &Definition,
        scope: Scope,
        text: String,
    ) -> Result<(), String> {
        self.bound(d)?;
        if text.len() > core::MAX_CONFIG {
            return Err("Configuration draft exceeds 4096 bytes; previous draft retained".into());
        }
        let mut next = self.clone();
        let work = next.work_mut(scope);
        work.configuration_draft = text;
        work.checked = false;
        self.publish(d, next)
    }
    pub fn save_configuration(&mut self, d: &Definition, scope: Scope) -> Result<(), String> {
        self.bound(d)?;
        let mut next = self.clone();
        let work = next.work_mut(scope);
        work.state
            .save_configuration(&work.configuration_draft)
            .map_err(|e| e.to_string())?;
        work.checked = false;
        self.publish(d, next)
    }
    pub fn edit_launch(
        &mut self,
        d: &Definition,
        scope: Scope,
        program: &str,
        value: String,
    ) -> Result<(), String> {
        self.bound(d)?;
        if value.len() > core::MAX_PATH_LIST {
            return Err("Launch draft exceeds 1024 bytes; previous draft retained".into());
        }
        let mut next = self.clone();
        let work = next.work_mut(scope);
        let draft = work
            .launch_drafts
            .iter_mut()
            .find(|r| r.program == program)
            .ok_or("Choose a supplied simulated launch setting")?;
        draft.value = value;
        work.checked = false;
        self.publish(d, next)
    }
    pub fn save_launch(
        &mut self,
        d: &Definition,
        scope: Scope,
        program: &str,
    ) -> Result<(), String> {
        self.bound(d)?;
        let mut next = self.clone();
        let work = next.work_mut(scope);
        let value = work
            .launch_drafts
            .iter()
            .find(|r| r.program == program)
            .ok_or("Choose a supplied simulated launch setting")?
            .value
            .clone();
        let environment = if value.is_empty() { None } else { Some(value) };
        let mut probe = work.state.clone();
        probe
            .set_library_path(environment.as_deref())
            .map_err(|e| e.to_string())?;
        work.recipes
            .iter_mut()
            .find(|r| r.program == program)
            .ok_or("Unknown supplied launch recipe")?
            .environment = environment;
        work.checked = false;
        self.publish(d, next)
    }
    pub fn set_command(
        &mut self,
        d: &Definition,
        scope: Scope,
        command: String,
    ) -> Result<(), String> {
        self.bound(d)?;
        if command.len() > MAX_COMMAND || command.contains(['\0', '\n', '\r']) {
            return Err("Use one bounded simulated command; previous input retained".into());
        }
        let mut next = self.clone();
        next.work_mut(scope).command = command;
        self.publish(d, next)
    }
    pub fn rebuild_cache(&mut self, d: &Definition, scope: Scope) -> Result<(), String> {
        self.bound(d)?;
        let mut next = self.clone();
        let work = next.work_mut(scope);
        work.state.rebuild_cache().map_err(|e| e.to_string())?;
        work.checked = false;
        self.publish(d, next)
    }
    pub fn inspect_recipe(
        &self,
        d: &Definition,
        scope: Scope,
        program: &str,
    ) -> Result<core::Resolution, String> {
        self.bound(d)?;
        self.work(scope)
            .launch(program, LaunchMode::SavedRecipe)
            .map_err(|e| e.to_string())
    }
    pub fn run(&mut self, d: &Definition, scope: Scope, command: &str) -> Result<Row, String> {
        self.bound(d)?;
        if command.len() > MAX_COMMAND || command.contains(['\0', '\n', '\r']) {
            return Err("Use one bounded simulated command; previous work retained".into());
        }
        let mut next = self.clone();
        let work = next.work_mut(scope);
        let result = parse(command).and_then(|command| execute(work, command));
        let row = match result {
            Ok(output) => Row {
                input: command.into(),
                output,
                outcome: Outcome::Supported,
            },
            Err(failure) => Row {
                input: command.into(),
                output: failure.message,
                outcome: failure.outcome,
            },
        };
        work.command = command.into();
        work.record(row.clone())?;
        self.publish(d, next)?;
        Ok(row)
    }
    pub fn current_report(&self, d: &Definition, scope: Scope) -> Result<Report, String> {
        self.bound(d)?;
        let (initial, declarations, plan) = match scope {
            Scope::Assignment => (&d.initial, &d.initial_recipes, &d.plan),
            Scope::Example => (&d.lesson.initial, &d.lesson.initial_recipes, &d.lesson.plan),
        };
        Ok(report(self.work(scope), initial, declarations, plan))
    }
    pub fn check(&mut self, d: &Definition, scope: Scope) -> Result<Report, String> {
        self.bound(d)?;
        let mut next = self.clone();
        next.work_mut(scope).checked = true;
        self.publish(d, next)?;
        self.current_report(d, scope)
    }
    pub fn current_success(&self, d: &Definition) -> bool {
        self.assignment.checked
            && self
                .current_report(d, Scope::Assignment)
                .is_ok_and(|r| r.passed())
    }
    pub fn reset(&mut self, d: &Definition, scope: Scope) -> Result<(), String> {
        self.bound(d)?;
        let mut next = self.clone();
        let (initial, declarations) = match scope {
            Scope::Assignment => (&d.initial, &d.initial_recipes),
            Scope::Example => (&d.lesson.initial, &d.lesson.initial_recipes),
        };
        *next.work_mut(scope) = Workspace::new(initial, declarations)?;
        self.publish(d, next)
    }
    pub fn from_json(raw: &[u8], d: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved library session exceeds 512 KiB; original retained".into());
        }
        let session: Self = serde_json::from_slice(raw)
            .map_err(|e| format!("Invalid saved library session: {e}"))?;
        session.bound(d)?;
        Ok(session)
    }
    pub fn to_json(&self, d: &Definition) -> Result<Vec<u8>, String> {
        self.bound(d)?;
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Inspect {
        program: String,
        environment: Option<String>,
    },
    Rebuild,
    ShowCache,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandError {
    pub outcome: Outcome,
    pub message: String,
}
fn unsupported(message: &str) -> CommandError {
    CommandError {
        outcome: Outcome::Unsupported,
        message: format!("{message}\n{HELP}"),
    }
}
fn core_error(error: core::Error) -> CommandError {
    CommandError {
        outcome: match error.kind {
            core::ErrorKind::Invalid => Outcome::Invalid,
            core::ErrorKind::Unsupported => Outcome::Unsupported,
            core::ErrorKind::NotFound => Outcome::NotFound,
            core::ErrorKind::Limit => Outcome::Limit,
        },
        message: error.to_string(),
    }
}
/// Quote-aware tokenization only, never shell expansion or execution. Unsupported
/// unquoted composition/expansion is refused before interpreting any operation.
fn tokens(line: &str) -> Result<Vec<String>, CommandError> {
    if line.len() > MAX_COMMAND || line.chars().any(|c| c.is_control() && c != '\t') {
        return Err(unsupported(
            "Use one bounded command line; only spaces and tabs separate arguments.",
        ));
    }
    let mut out = vec![];
    let mut token = String::new();
    let mut quote = None;
    let mut started = false;
    for c in line.chars() {
        match quote {
            Some('\'') => {
                if c == '\'' {
                    quote = None;
                } else {
                    token.push(c);
                }
            }
            Some('"') => {
                if c == '"' {
                    quote = None;
                } else if matches!(c, '$' | '`' | '\\') {
                    return Err(unsupported(
                        "Expansion and escapes inside double quotes are unsupported.",
                    ));
                } else {
                    token.push(c);
                }
            }
            Some(_) => unreachable!(),
            None => {
                if matches!(c, '\'' | '"') {
                    quote = Some(c);
                    started = true;
                } else if c.is_ascii_whitespace() {
                    if started {
                        out.push(std::mem::take(&mut token));
                        started = false;
                    }
                } else if "|&;<>()$`\\*?[]{}~#".contains(c) {
                    return Err(unsupported(
                        "Shell composition, expansion and comments are outside this command subset. Quote literal semicolon path separators.",
                    ));
                } else {
                    token.push(c);
                    started = true;
                }
            }
        }
        if out.len() > 4 {
            return Err(unsupported("Use one of the four documented command forms."));
        }
    }
    if quote.is_some() {
        return Err(CommandError {
            outcome: Outcome::Invalid,
            message: "An opening quote needs a matching closing quote; no operation was applied."
                .into(),
        });
    }
    if started {
        out.push(token);
    }
    if out.len() > 4 {
        return Err(unsupported("Use one documented command form."));
    }
    Ok(out)
}
pub fn parse(line: &str) -> Result<Command, CommandError> {
    let words = tokens(line)?;
    match words.as_slice() {
        [name, program] if name == "ldd" && !program.starts_with('-') => Ok(Command::Inspect {
            program: program.clone(),
            environment: None,
        }),
        [env, value, ldd, program]
            if env == "env"
                && ldd == "ldd"
                && value.starts_with("LD_LIBRARY_PATH=")
                && !program.starts_with('-') =>
        {
            Ok(Command::Inspect {
                program: program.clone(),
                environment: Some(value[16..].to_owned()),
            })
        }
        [name] if name == "ldconfig" => Ok(Command::Rebuild),
        [name, flag] if name == "ldconfig" && flag == "-p" => Ok(Command::ShowCache),
        _ => Err(unsupported(
            "That command or option is not modeled; nothing was applied.",
        )),
    }
}
fn execute(work: &mut Workspace, command: Command) -> Result<String, CommandError> {
    match command {
        Command::Inspect {
            program,
            environment,
        } => {
            let mut probe = work.state.clone();
            probe
                .set_library_path(environment.as_deref())
                .map_err(core_error)?;
            let resolution = probe
                .resolve(&program, core::Environment::Configured)
                .map_err(core_error)?;
            let mut output = resolution.report().map_err(core_error)?;
            if environment.is_some() {
                output.push_str("One-off environment inspection only; no simulated launch recipe or cache was changed.\n");
            }
            if output.len() > core::MAX_OUTPUT {
                return Err(CommandError {
                    outcome: Outcome::Limit,
                    message: "Inspection output limit; no partial result published.".into(),
                });
            }
            Ok(output)
        }
        Command::Rebuild => {
            work.state.rebuild_cache().map_err(core_error)?;
            work.checked = false;
            Ok("Rebuilt the virtual cache from the saved configuration. Unsaved editor drafts were not used; no host cache or links changed.\n".into())
        }
        Command::ShowCache => {
            let mut output = String::from("Simulated cache entries; supplied metadata only.\n");
            if let Some(cache) = &work.state.cache {
                for e in &cache.entries {
                    let line = format!("{} => {}\n", e.name, e.path);
                    if output.len() + line.len() > core::MAX_OUTPUT {
                        return Err(CommandError {
                            outcome: Outcome::Limit,
                            message: "Cache display limit; no partial result published.".into(),
                        });
                    }
                    output.push_str(&line);
                }
            } else {
                output.push_str("No virtual cache has been built.\n");
            }
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests;

mod binding;
pub use binding::{Action, MAX_CONTEXT, PREFIX, context, metadata, source};
