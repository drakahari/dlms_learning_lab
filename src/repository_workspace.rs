//! Source-bound repository evidence over the reviewed in-process metadata model.
//! This is not Zypper, libzypp, transport, trust verification or package execution.
use crate::{pack::Scenario, progress::Attempt, rpm_repositories as core, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const HELP: &str = "Repository metadata simulator: one or two supplied sources and fixed package snapshots. Native configuration changes enabled state and a URI label; Refresh metadata copies the supplied published snapshot only when that virtual source is enabled and available. Query exact package reads the cache without refreshing or installing. Wrong-source caches remain inspectable but are withheld from query candidates by this model policy. Older matching caches remain queryable when delivery is unavailable. No actual paths/URLs are opened, no signatures or trust checks occur, and no RPM/Zypper/Yum CLI, dependency solving, downloads or installation run here.";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Edit {
    Settings {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        alias: String,
        enabled: bool,
        #[serde(deserialize_with = "crate::cron::text::<_,160>")]
        uri: String,
    },
    Refresh {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        alias: String,
    },
}
fn explain(e: core::Error) -> String {
    match e.code{
 core::Code::Malformed=>"Use a complete supplied source address beginning file:/// or https://, without spaces or quotes. The address is an offline label; it is never opened.".into(),
 core::Code::UnknownAlias=>"Choose one of this lesson’s listed repository aliases.".into(),
 core::Code::UnknownSource=>"The configured address has no supplied virtual source. Compare the listed addresses; nothing was fetched.".into(),
 core::Code::Disabled=>"This repository is disabled. Enable it and Apply settings before Refresh metadata; its previous cache is unchanged.".into(),
 core::Code::Unavailable=>"The supplied source is unavailable. Refresh did not replace its existing cache; a matching older cache can still be queried.".into(),
 core::Code::Limit=>"The simulated repository resource limit was reached; previous work is unchanged.".into(),
 core::Code::Context=>"This repository work does not match its supplied sample inputs; previous work is retained.".into(),
}
}
fn apply(world: &mut core::State, source: &core::Source, edit: &Edit) -> Result<(), String> {
    let mut next = world.clone();
    match edit {
        Edit::Settings {
            alias,
            enabled,
            uri,
        } => {
            next.apply(
                source,
                &core::Action::SetUri {
                    alias: alias.clone(),
                    uri: uri.clone(),
                },
            )
            .map_err(explain)?;
            next.apply(
                source,
                &core::Action::SetEnabled {
                    alias: alias.clone(),
                    enabled: *enabled,
                },
            )
            .map_err(explain)?;
        }
        Edit::Refresh { alias } => next
            .apply(
                source,
                &core::Action::Refresh {
                    alias: alias.clone(),
                },
            )
            .map_err(explain)?,
    }
    *world = next;
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Edit {
        edit: Edit,
    },
    Query {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        package: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Current {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        alias: String,
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        endpoint: String,
    },
    Preserve {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        alias: String,
    },
    Query {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        package: String,
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Candidate,12>")]
        rows: Vec<Candidate>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub alias: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub archive: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial_source: core::Source,
    pub initial: core::State,
    pub example_source: core::Source,
    pub example: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,16>")]
    pub example_requirements: Vec<Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,24>")]
    pub example_actions: Vec<Event>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.initial_source == self.example_source
            || self.requirements.is_empty()
            || self.example_requirements.is_empty()
            || self.example_actions.is_empty()
            || self.example_actions.len() > 24
            || self.requirements.len() > 16
            || self.example_requirements.len() > 16
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 192 * 1024
        {
            return Err(
                "Repository task needs distinct supplied examples and bounded outcomes".into(),
            );
        }
        for (w, src, rs) in [
            (&self.initial, &self.initial_source, &self.requirements),
            (
                &self.example,
                &self.example_source,
                &self.example_requirements,
            ),
        ] {
            w.validate_context(src).map_err(|e| e.message.to_owned())?;
            for r in rs {
                match r {
                    Requirement::Current { alias, endpoint } => {
                        w.inspect(src, alias).map_err(|e| e.message.to_owned())?;
                        if !src.endpoints.iter().any(|e| e.id == *endpoint) {
                            return Err("Unknown required source".into());
                        }
                    }
                    Requirement::Preserve { alias } => {
                        w.inspect(src, alias).map_err(|e| e.message.to_owned())?;
                    }
                    Requirement::Query { package, rows } => {
                        w.query_exact(src, package)
                            .map_err(|e| e.message.to_owned())?;
                        if rows.len() > 12
                            || rows.iter().any(|r| {
                                !src.catalog.archives.contains_key(&r.archive)
                                    || w.inspect(src, &r.alias).is_err()
                            })
                        {
                            return Err("Invalid expected query evidence".into());
                        }
                    }
                }
            }
        }
        let mut s = Session::new(self);
        for e in &self.example_actions {
            s.run(self, e, true)?;
        }
        if !self.checks(&s, true).iter().all(|r| r.0) {
            return Err("Worked repository example does not reach its outcomes".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.example_action >= self.example_actions.len() {
            return Err("Unknown repository worked action".into());
        }
        for (scope, initial, src) in [
            (&s.assignment, &self.initial, &self.initial_source),
            (&s.practice, &self.example, &self.example_source),
        ] {
            let mut replay = initial.clone();
            for e in &scope.history {
                match e {
                    Event::Edit { edit } => apply(&mut replay, src, edit)?,
                    Event::Query { package } => {
                        replay
                            .query_exact(src, package)
                            .map_err(|e| e.message.to_owned())?;
                    }
                }
            }
            if replay != scope.world {
                return Err("Repository work differs from its supplied inputs and successful operations; prior work retained".into());
            }
            let loaded = scope
                .world
                .inspect(src, &scope.alias)
                .map_err(|e| e.message.to_owned())?;
            if !scope.dirty
                && (scope.uri != loaded.configuration.uri
                    || scope.enabled != loaded.configuration.enabled)
            {
                return Err("Repository editor differs from applied settings without an unapplied-draft marker; prior work retained".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (scope, initial, src, rs) = if practice {
            (
                &s.practice,
                &self.example,
                &self.example_source,
                &self.example_requirements,
            )
        } else {
            (
                &s.assignment,
                &self.initial,
                &self.initial_source,
                &self.requirements,
            )
        };
        let mut rows=vec![(!scope.dirty,if scope.dirty{"Apply or discard the edited repository settings; the query and grade inspect applied configuration."}else{"No unapplied repository settings."}.into())];
        for r in rs {
            match r {
                Requirement::Current { alias, endpoint } => {
                    let i = scope.world.inspect(src, alias).expect("bounded alias");
                    let state = match i.cache_status {
                        core::CacheStatus::Missing => "no cached metadata",
                        core::CacheStatus::WrongSource => "different cached source",
                        core::CacheStatus::Older => "older matching snapshot",
                        core::CacheStatus::Current => "matching current snapshot",
                    };
                    let enabled = if i.configuration.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    };
                    let configured = i
                        .endpoint
                        .map(|e| e.id.as_str())
                        .unwrap_or("unrecognized address");
                    rows.push((i.configuration.enabled&&i.endpoint.is_some_and(|e|e.id==*endpoint)&&i.cache_status==core::CacheStatus::Current,format!("{alias}: {enabled}; configured source {configured}; {state}. Use the supplied {endpoint} source with matching current metadata. Repair configuration first; refresh only when the cache needs updating.")));
                }
                Requirement::Preserve { alias } => {
                    let before = initial.inspect(src, alias).unwrap();
                    let now = scope.world.inspect(src, alias).unwrap();
                    rows.push((before.configuration==now.configuration&&before.cached==now.cached,format!("Retain {alias} configuration and cached source/snapshot exactly; do not replace useful historical evidence.")));
                }
                Requirement::Query {
                    package,
                    rows: expected,
                } => {
                    let last_edit = scope
                        .history
                        .iter()
                        .rposition(|e| matches!(e, Event::Edit { .. }));
                    let observed = scope.history.iter().enumerate().any(|(n, e)| {
                        matches!(e,Event::Query{package:p}if p==package)
                            && last_edit.is_none_or(|edit| n > edit)
                    });
                    let query = scope.world.query_exact(src, package).unwrap();
                    let mut actual: Vec<_> = query
                        .rows
                        .iter()
                        .map(|r| Candidate {
                            alias: r.alias.into(),
                            archive: r.archive_id.into(),
                        })
                        .collect();
                    actual.sort();
                    let mut expected = expected.clone();
                    expected.sort();
                    let feedback = if !observed {
                        format!(
                            "Run Query exact package for {package} after the last settings/refresh action. Earlier results describe the previous cache state."
                        )
                    } else if actual == expected {
                        format!(
                            "{package}: current inspection evidence matches the required source/version policy."
                        )
                    } else {
                        let unexpected: Vec<_> = query
                            .rows
                            .iter()
                            .filter(|r| {
                                !expected
                                    .iter()
                                    .any(|e| e.alias == r.alias && e.archive == r.archive_id)
                            })
                            .map(|r| format!("{} (version {})", r.alias, r.archive.evr.as_str()))
                            .collect();
                        let detail = if unexpected.is_empty() {
                            "Required candidate evidence is missing; read the source omissions and cache status.".to_string()
                        } else {
                            format!(
                                "Unexpected candidates: {}. Check which source/cache supplied these versions.",
                                unexpected.join(", ")
                            )
                        };
                        format!(
                            "{package}: {} actual candidate rows. {detail}",
                            actual.len()
                        )
                    };
                    rows.push((observed && actual == expected, feedback));
                }
            }
        }
        rows
    }
    pub fn observed(
        &self,
        s: &Session,
        practice: bool,
    ) -> Result<Option<(String, core::State, bool)>, String> {
        self.bound(s)?;
        let (scope, initial, src) = if practice {
            (&s.practice, &self.example, &self.example_source)
        } else {
            (&s.assignment, &self.initial, &self.initial_source)
        };
        let mut w = initial.clone();
        let mut last = None;
        for e in &scope.history {
            match e {
                Event::Edit { edit } => {
                    apply(&mut w, src, edit)?;
                    if let Some((_, _, fresh)) = &mut last {
                        *fresh = false;
                    }
                }
                Event::Query { package } => last = Some((package.clone(), w.clone(), true)),
            }
        }
        Ok(last)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,32>")]
    pub history: Vec<Event>,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub alias: String,
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub uri: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub package: String,
    pub dirty: bool,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub message: String,
}
impl Scope {
    fn new(world: &core::State, src: &core::Source) -> Self {
        let first = world.inspect_all(src).expect("validated initial source")[0].configuration;
        Self {
            world: world.clone(),
            history: vec![],
            alias: first.alias.clone(),
            enabled: first.enabled,
            uri: first.uri.clone(),
            package: String::new(),
            dirty: false,
            checked: false,
            message: String::new(),
        }
    }
    pub fn load(&mut self, src: &core::Source, alias: &str) -> Result<(), String> {
        let i = self
            .world
            .inspect(src, alias)
            .map_err(|e| e.message.to_owned())?;
        self.alias = alias.into();
        self.uri = i.configuration.uri.clone();
        self.enabled = i.configuration.enabled;
        self.dirty = false;
        self.checked = false;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub example_action: usize,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_action: 0,
            assignment: Scope::new(&d.initial, &d.initial_source),
            practice: Scope::new(&d.example, &d.example_source),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.example_action >= 24
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 256 * 1024
        {
            return Err("Repository workspace version or byte limit exceeded".into());
        }
        for s in [&self.assignment, &self.practice] {
            s.world.validate().map_err(|e| e.message.to_owned())?;
            if s.history.len() > 32
                || s.alias.len() > 32
                || s.uri.len() > 160
                || s.package.len() > 32
                || s.message.len() > 1024
            {
                return Err("Repository workspace text/history limit exceeded".into());
            }
        }
        Ok(())
    }
    pub fn run(&mut self, d: &Definition, event: &Event, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        let mut next = self.clone();
        let (scope, src) = if practice {
            (&mut next.practice, &d.example_source)
        } else {
            (&mut next.assignment, &d.initial_source)
        };
        if scope.history.len() == 32 {
            return Err("This attempt has reached32 retained operations. Reset this example or assignment to start another investigation.".into());
        }
        match event {
            Event::Edit { edit } => {
                apply(&mut scope.world, src, edit)?;
                if matches!(edit, Edit::Settings { .. }) {
                    let alias = scope.alias.clone();
                    scope.load(src, &alias)?;
                }
            }
            Event::Query { package } => {
                scope.world.query_exact(src,package).map_err(|_|"Enter one package name using letters, digits, dots, underscores, + or -. Patterns, spaces and shell commands are not supported by this query.".to_string())?;
            }
        }
        scope.history.push(event.clone());
        scope.checked = false;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String, String> {
        let raw=text.strip_prefix("repository-action ").ok_or("Use the native repository editor and query controls; no host package commands run here")?;
        if raw.len() > 4096 {
            return Err("Repository reference exceeds its bound".into());
        }
        let e: Event =
            serde_json::from_str(raw).map_err(|_| "Invalid simulator-only repository reference")?;
        self.run(d, &e, false)?;
        Ok("Applied simulated repository operation".into())
    }
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::Repositories { simulation },
        crate::engine::Goal::Repositories { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Repository state and authored task must agree".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Repository task must begin with its supplied separate worlds".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::Repositories { simulation } = &a.state else {
        return Err("Saved work belongs to another repository task".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible repository attempt metadata; prior work retained".into());
    }
    d.bound(simulation)
}
