//! Separate draft/editor/cache practice. No host files, APT command or transport.
use crate::{
    debian_packages::Version, debian_packages_workspace as wire, debian_source_cache as model,
    debian_sources as sources, script_teaching::View,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
pub const MAX_DEFINITION: usize = 524288;
pub const MAX_SESSION: usize = 98304;
pub const HELP: &str = "Edit the supplied deb822 selection excerpt, Save sources, Refresh simulated indexes, and Look up a package. These native actions operate only on supplied in-memory metadata. Saving never fetches; refresh never installs. Other APT fields, trust policy, transport and dependency solving are outside this activity.";
pub const PREFIX: &str = "APT SOURCES\n";
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn option_text<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<wire::Text<64>>::deserialize(d)?.map(|s| s.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Selected {
        #[serde(deserialize_with = "wire::list::<_,sources::Key,24>")]
        targets: Vec<sources::Key>,
    },
    Cached {
        target: sources::Key,
        #[serde(deserialize_with = "wire::string::<_,64>")]
        snapshot: String,
    },
    Available {
        target: sources::Key,
        #[serde(deserialize_with = "wire::string::<_,64>")]
        package: String,
        version: Version,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "wire::string::<_,4096>")]
    pub instructions: String,
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub id: String,
    pub model: model::Definition,
    pub starter: model::State,
    #[serde(deserialize_with = "wire::list::<_,Requirement,12>")]
    pub requirements: Vec<Requirement>,
}
fn location(key: &sources::Key) -> String {
    format!("{}/{} at {}", key.suite, key.component, key.uri)
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if self.instructions.trim().is_empty()
            || self.instructions.len() > 4096
            || self.instructions.contains('\0')
            || !identifier(&self.id)
            || self.requirements.is_empty()
            || self.requirements.len() > 12
        {
            return Err("Source task needs a bounded identifier and requirements".into());
        }
        self.starter
            .validate(&self.model)
            .map_err(|e| e.to_string())?;
        let selections = self
            .requirements
            .iter()
            .filter_map(|r| {
                if let Requirement::Selected { targets } = r {
                    Some(targets)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if selections.len() != 1
            || !self
                .requirements
                .iter()
                .any(|r| matches!(r, Requirement::Available { .. }))
        {
            return Err(
                "Source tasks need one selected-target goal and positive package availability"
                    .into(),
            );
        }
        let selected = selections[0];
        let keys = selected.iter().collect::<BTreeSet<_>>();
        if selected.is_empty()
            || selected.len() > sources::MAX_KEYS
            || keys.len() != selected.len()
            || selected
                .iter()
                .any(|k| !self.model.targets.iter().any(|t| &t.key == k))
        {
            return Err("Selected goals must name distinct supplied source targets".into());
        }
        let mut signatures = BTreeSet::new();
        for requirement in &self.requirements {
            if !signatures.insert(
                serde_json::to_string(requirement).map_err(|_| "Invalid source requirement")?,
            ) {
                return Err("Repeated source requirement".into());
            }
            match requirement {
                Requirement::Selected { .. } => {}
                Requirement::Cached { target, snapshot } => {
                    if !keys.contains(target)
                        || !self.model.targets.iter().any(|t| {
                            &t.key == target && t.snapshots.iter().any(|s| &s.id == snapshot)
                        })
                    {
                        return Err(
                            "Cached requirement must belong to a required active source".into()
                        );
                    }
                }
                Requirement::Available {
                    target,
                    package,
                    version,
                } => {
                    if !keys.contains(target)
                        || !self.model.targets.iter().any(|t| {
                            &t.key == target
                                && t.snapshots.iter().any(|s| {
                                    s.rows.iter().any(|r| {
                                        &r.name == package
                                            && &r.version == version
                                            && (r.architecture == self.model.native
                                                || r.architecture
                                                    == crate::debian_packages::Architecture::All)
                                    })
                                })
                        })
                    {
                        return Err("Availability requirement must have a supplied native/all package row in a required source".into());
                    }
                }
            }
        }
        if self.raw_checks(&self.starter).iter().all(|r| r.0) {
            return Err("Source starter must retain meaningful unfinished work".into());
        }
        Ok(())
    }
    fn raw_checks(&self, state: &model::State) -> Vec<(bool, String)> {
        if let Err(e) = state.validate(&self.model) {
            return vec![(false, e.to_string())];
        }
        let active = state.active(&self.model).expect("validated state");
        let selected = active.iter().collect::<BTreeSet<_>>();
        self.requirements.iter().map(|r|match r {
   Requirement::Selected{targets}=>{
    let expected=targets.iter().collect::<BTreeSet<_>>();let missing=expected.difference(&selected).map(|k|location(k)).collect::<Vec<_>>();let extra=selected.difference(&expected).map(|k|location(k)).collect::<Vec<_>>();
    (missing.is_empty()&&extra.is_empty(),if missing.is_empty()&&extra.is_empty(){"The required source targets are selected.".into()}else{format!("Saved sources: missing [{}]; extra [{}]. Saving selects targets; it does not refresh their metadata.",missing.join(", "),extra.join(", "))})
   },
   Requirement::Cached{target,snapshot}=>{
    let cached=state.cache.iter().find(|c|&c.key==target);let ok=selected.contains(target)&&cached.is_some_and(|c|&c.snapshot==snapshot);
    (ok,if ok{format!("{} has the required matching cached metadata.",location(target))}else{format!("{}: check the selected source and its cached snapshot. Refresh uses only the supplied publication; an unavailable source keeps its older cache.",location(target))})
   },
   Requirement::Available{target,package,version}=>{
    let rows=state.query(&self.model,package).unwrap_or_default();let same=rows.iter().filter(|r|&r.key==target).collect::<Vec<_>>();let ok=same.iter().any(|r|&r.row.version==version);
    (ok,if ok{format!("{package} {} is visible from {}.",version.as_str(),location(target))}else{format!("{package} from {}: expected {}; matching cached versions [{}]. Check suite/component selection, then refresh. A query alone changes no metadata.",location(target),version.as_str(),same.iter().map(|r|r.row.version.as_str()).collect::<Vec<_>>().join(", "))})
   }
  }).collect()
    }
    pub fn checks(&self, scope: &Scope) -> Vec<(bool, String)> {
        if let Err(e) = scope.bound(self) {
            return vec![(false, e)];
        }
        if scope.draft != scope.state.saved {
            return vec![(false,"Source edits are not saved. Use Save sources before checking the resulting configuration.".into())];
        }
        self.raw_checks(&scope.state)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worked {
    #[serde(deserialize_with = "wire::string::<_,8192>")]
    pub sources: String,
    pub refresh: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    pub example_worked: Worked,
    pub assignment_reference: Worked,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || !wire::within(self, MAX_DEFINITION) {
            return Err("Assignment and practice need separate bounded source tasks".into());
        }
        let a = self
            .initial
            .requirements
            .iter()
            .filter_map(|r| {
                if let Requirement::Available { package, .. } = r {
                    Some(package)
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();
        if self
            .example
            .requirements
            .iter()
            .any(|r| matches!(r,Requirement::Available{package,..}if a.contains(package)))
        {
            return Err("Worked package names must differ from assignment package names".into());
        }
        for (task, worked) in [
            (&self.example, &self.example_worked),
            (&self.initial, &self.assignment_reference),
        ] {
            let mut state = task.starter.clone();
            state
                .save(&task.model, &worked.sources)
                .map_err(|e| e.to_string())?;
            if worked.refresh {
                state.refresh(&task.model).map_err(|e| e.to_string())?;
            }
            if !task.raw_checks(&state).iter().all(|r| r.0) {
                return Err("Each source task needs a reference configuration that reaches its actual requirements".into());
            }
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("Source lesson exceeds512KiB".into());
        }
        let d: Self =
            serde_json::from_str(raw).map_err(|e| format!("Invalid bounded source lesson: {e}"))?;
        d.validate()?;
        Ok(d)
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.origin != self.initial.id {
            return Err("Source work belongs to another lesson; retained.".into());
        }
        s.assignment.bound(&self.initial)?;
        s.practice.bound(&self.example)?;
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked {
            return vec![(
                false,
                "Not checked · save and inspect the current sources, then Check work.".into(),
            )];
        }
        self.initial.checks(&s.assignment)
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.checks(s).iter().all(|r| r.0)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub origin: String,
    #[serde(deserialize_with = "wire::string::<_,8192>")]
    pub draft: String,
    pub state: model::State,
    pub checked: bool,
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub query: String,
    #[serde(default, deserialize_with = "option_text")]
    pub observed: Option<String>,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            origin: t.id.clone(),
            draft: t.starter.saved.clone(),
            state: t.starter.clone(),
            checked: false,
            query: String::new(),
            observed: None,
        }
    }
    fn bound(&self, t: &Task) -> Result<(), String> {
        if self.origin != t.id {
            return Err("The source workspace belongs to another task; retained.".into());
        }
        if self.draft.len() > sources::MAX_INPUT
            || self.draft.contains('\0')
            || self.query.len() > 64
            || self.query.contains('\0')
            || self.observed.as_ref().is_some_and(|s| s != &self.query)
        {
            return Err("Invalid source draft/query bounds or observation".into());
        }
        self.state.validate(&t.model).map_err(|e| e.to_string())?;
        if let Some(q) = &self.observed {
            self.state.query(&t.model, q).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn edit(&mut self, text: &str) -> Result<(), String> {
        if text.len() > sources::MAX_INPUT || text.contains('\0') {
            return Err("Use at most8KiB of source text without NUL characters".into());
        }
        if self.draft != text {
            self.draft = text.into();
            self.checked = false;
            self.observed = None;
        }
        Ok(())
    }
    pub fn save(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.checked = false;
        self.observed = None;
        self.state
            .save(&t.model, &self.draft)
            .map_err(|e| e.to_string())
    }
    pub fn refresh(&mut self, t: &Task) -> Result<Vec<model::Refresh>, String> {
        self.bound(t)?;
        if self.draft != self.state.saved {
            return Err("Save the edited sources before refreshing. The current saved selection has not changed.".into());
        }
        self.checked = false;
        self.observed = None;
        self.state.refresh(&t.model).map_err(|e| e.to_string())
    }
    pub fn set_query(&mut self, name: &str) -> Result<(), String> {
        if name.len() > 64 || name.contains('\0') {
            return Err("Use at most64bytes for a package query".into());
        }
        if self.query != name {
            self.query = name.into();
            self.observed = None;
        }
        Ok(())
    }
    pub fn lookup(&mut self, t: &Task) -> Result<Vec<model::Evidence>, String> {
        self.bound(t)?;
        let rows = self
            .state
            .query(&t.model, &self.query)
            .map_err(|e| e.to_string())?;
        self.observed = Some(self.query.clone());
        Ok(rows)
    }
    pub fn check(&mut self, t: &Task) -> Vec<(bool, String)> {
        self.checked = true;
        t.checks(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    #[serde(deserialize_with = "wire::string::<_,64>")]
    pub origin: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !identifier(&self.origin) || !wire::within(self, MAX_SESSION) {
            return Err("Invalid source-workspace version, identity or byte bound".into());
        }
        for scope in [&self.assignment, &self.practice] {
            if !identifier(&scope.origin)
                || scope.draft.len() > sources::MAX_INPUT
                || scope.draft.contains('\0')
                || scope.query.len() > 64
                || scope.query.contains('\0')
                || scope.state.saved.len() > sources::MAX_INPUT
                || scope.state.cache.len() > model::MAX_TARGETS
                || scope.observed.as_ref().is_some_and(|s| s != &scope.query)
            {
                return Err("Invalid source-workspace draft, cache or query bounds".into());
            }
            sources::select(&sources::parse(&scope.state.saved).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            for c in &scope.state.cache {
                c.key.validate().map_err(|e| e.to_string())?;
                if !model::id(&c.snapshot) {
                    return Err("Invalid cached snapshot identifier".into());
                }
            }
        }
        Ok(())
    }
    pub fn from_json(raw: &str, d: &Definition) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved source lesson exceeds96KiB".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid saved source workspace: {e}"))?;
        d.bound(&s)?;
        Ok(s)
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.bound(self)?;
        let raw = input
            .strip_prefix(PREFIX)
            .ok_or("Use the native source editor; this is internal data-only reference input")?;
        if raw.len() > 16384 {
            return Err("Source reference action exceeds16KiB".into());
        }
        #[derive(Deserialize)]
        #[serde(tag = "action", deny_unknown_fields)]
        enum Action {
            Save {
                #[serde(deserialize_with = "wire::string::<_,8192>")]
                sources: String,
            },
            Refresh {},
            Query {
                #[serde(deserialize_with = "wire::string::<_,64>")]
                name: String,
            },
            Check {},
        }
        let action: Action = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid source reference action: {e}"))?;
        match action {
            Action::Save { sources } => {
                self.assignment.edit(&sources)?;
                self.assignment.save(&d.initial)?;
                Ok("Source selection saved; metadata was not refreshed.".into())
            }
            Action::Refresh {} => {
                let rows = self.assignment.refresh(&d.initial)?;
                Ok(format!(
                    "{} refreshed; {} unavailable. Older matching caches retained.",
                    rows.iter()
                        .filter(|r| r.status == model::RefreshStatus::Updated)
                        .count(),
                    rows.iter()
                        .filter(|r| r.status == model::RefreshStatus::Unavailable)
                        .count()
                ))
            }
            Action::Query { name } => {
                self.assignment.set_query(&name)?;
                let rows = self.assignment.lookup(&d.initial)?;
                Ok(format!(
                    "{} matching cached metadata rows; no software installed.",
                    rows.len()
                ))
            }
            Action::Check {} => {
                let results = self.assignment.check(&d.initial);
                Ok(results
                    .into_iter()
                    .map(|(ok, t)| format!("{}: {t}", if ok { "PASS" } else { "Needs work" }))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
        }
    }
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::DebianSources { simulation },
        crate::engine::Goal::DebianSources { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Source-selection source/state mismatch".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Source-selection identity, task and initial work must match".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::DebianSources { simulation } = &a.state else {
        return Err("Source-selection work belongs to another workspace".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible source-selection metadata; previous work retained".into());
    }
    d.bound(simulation)
}
