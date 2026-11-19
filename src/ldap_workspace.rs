//! Source-held LDAP client tasks, live query practice and isolated persisted drafts.
use crate::{ldap_client as client, ldap_query as query, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_DEFINITION: usize = 4 * 1024 * 1024;
// Two scopes hold64KiB raw text in total; JSON can double escaping.
pub const MAX_SESSION: usize = 144 * 1024;
pub const PREFIX: &str = "LDAP CLIENT WORK\n";
pub const HELP: &str = "Edit the virtual URI and BASE defaults, then Apply configuration. Run search evaluates one literal ldapsearch command against the selected supplied directory. Check work evaluates every supplied case, including default-setting checks. Supported: -x -LLL, optional -H/-b/-s base|one|sub, explicit equality/presence/AND/OR/NOT filter, named attributes, quoted '*' or 1.1. Unknown syntax fails clearly. No host files, processes, authentication or network connections. Practice is separate from the assignment.";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && s.chars().all(|c| !c.is_control() || c == '\n')
}
fn config_text(s: &str) -> bool {
    s.len() <= client::MAX_CONFIG
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn command_text(s: &str) -> bool {
    s.len() <= client::MAX_COMMAND && s.chars().all(|c| !c.is_control() || c == '\t')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectorySource {
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub uri: String,
    pub directory: query::Source,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Entries {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,query::EntrySource,128>")]
        entries: Vec<query::EntrySource>,
    },
    NoSuchObject,
}
impl Expected {
    fn matches(&self, actual: &client::Outcome) -> bool {
        match (self, actual) {
            (Self::NoSuchObject, client::Outcome::NoSuchObject) => true,
            (Self::Entries { entries }, client::Outcome::Entries(rows)) => {
                entries.len() == rows.len()
                    && entries.iter().all(|expected| {
                        rows.iter().any(|row| {
                            query::Dn::parse(&expected.dn)
                                .ok()
                                .zip(query::Dn::parse(&row.dn).ok())
                                .is_some_and(|(a, b)| a.equivalent(&b))
                                && expected.attributes.len() == row.attributes.len()
                                && expected.attributes.iter().all(|a| {
                                    row.attributes.iter().any(|b| {
                                        a.name.eq_ignore_ascii_case(&b.name)
                                            && a.values.len() == b.values.len()
                                            && a.values.iter().all(|v| b.values.contains(v))
                                    })
                                })
                        })
                    })
            }
            _ => false,
        }
    }
    pub fn output(&self) -> Result<String, String> {
        match self {
            Self::NoSuchObject => Ok("No such object (32)".into()),
            Self::Entries { entries } => client::Outcome::Entries(
                entries
                    .iter()
                    .map(|e| client::ResultEntry {
                        dn: e.dn.clone(),
                        attributes: e.attributes.clone(),
                    })
                    .collect(),
            )
            .output()
            .map_err(|e| e.to_string()),
        }
    }
    fn validate(&self) -> Result<(), String> {
        let Self::Entries { entries } = self else {
            return Ok(());
        };
        if entries.len() > 128 {
            return Err("Expected result exceeds128entries.".into());
        }
        let mut names = Vec::new();
        for e in entries {
            let dn = query::Dn::parse(&e.dn).map_err(|e| e.to_string())?;
            if names.iter().any(|n: &query::Dn| n.equivalent(&dn)) {
                return Err("Expected entries need distinct DNs.".into());
            }
            names.push(dn);
            if e.attributes.len() > 16 {
                return Err("Expected result has too many attributes.".into());
            }
            let mut attrs = BTreeSet::new();
            for a in &e.attributes {
                if a.name.is_empty()
                    || a.name.len() > 32
                    || !a
                        .name
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    || !attrs.insert(a.name.to_ascii_lowercase())
                    || a.values.is_empty()
                    || a.values.len() > 16
                    || a.values.iter().any(|v| {
                        v.is_empty() || v.len() > 256 || !v.bytes().all(|b| (32..=126).contains(&b))
                    })
                    || a.values.iter().collect::<BTreeSet<_>>().len() != a.values.len()
                {
                    return Err("Expected attributes/values must be unique and bounded.".into());
                }
            }
        }
        self.output()?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,DirectorySource,4>")]
    pub directories: Vec<DirectorySource>,
    /// Empty uses the learner's search. Nonempty is a disclosed default-setting probe.
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub probe: String,
    pub expected: Expected,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub reason: String,
}
impl Case {
    pub fn environment(&self) -> Result<client::Environment, String> {
        client::Environment::new(
            &self
                .directories
                .iter()
                .map(|d| (d.uri.clone(), d.directory.clone()))
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())
    }
    fn run(&self, config: &str, command: &str) -> Result<client::Outcome, String> {
        let settings = client::Settings::parse(config).map_err(|e| e.to_string())?;
        self.environment()?
            .run(
                &settings,
                if self.probe.is_empty() {
                    command
                } else {
                    &self.probe
                },
            )
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4000>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub starter_query: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub reference_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub reference_query: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub ran: bool,
    pub label: String,
    pub passed: bool,
    pub feedback: String,
    pub actual: String,
    pub expected: String,
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 4000)
            || !config_text(&self.starter_config)
            || !config_text(&self.reference_config)
            || !command_text(&self.starter_query)
            || !command_text(&self.reference_query)
            || !(3..=8).contains(&self.cases.len())
            || !crate::cron_workspace::within(self, MAX_DEFINITION)
        {
            return Err(
                "Directory task requires version1, bounded text and3–8supplied cases.".into(),
            );
        }
        let mut ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        let mut own = 0;
        let mut probes = 0;
        for c in &self.cases {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !prose(&c.reason, 800)
                || !command_text(&c.probe)
            {
                return Err("Directory cases need distinct identities/labels and useful bounded explanations.".into());
            }
            c.environment()?;
            c.expected.validate()?;
            if c.probe.is_empty() {
                own += 1
            } else {
                probes += 1
            }
        }
        if own < 2 || probes == 0 {
            return Err("Test the learner query against at least two directories plus an independent default-setting probe.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        client::Settings::parse(&self.reference_config).map_err(|e| e.to_string())?;
        if self
            .reports_validated(&self.reference_config, &self.reference_query)
            .iter()
            .any(|r| !r.passed)
        {
            return Err("Worked configuration/query cannot reach every declared outcome.".into());
        }
        if self
            .reports_validated(&self.starter_config, &self.starter_query)
            .iter()
            .all(|r| r.passed)
        {
            return Err("Independent starter is already solved.".into());
        }
        Ok(())
    }
    fn reports_validated(&self, config: &str, command: &str) -> Vec<Report> {
        self.cases.iter().map(|c|{
   let expected=c.expected.output().unwrap_or_else(|e|e);let result=c.run(config,command);
   let ran=result.is_ok();
   let (passed,actual,feedback)=match result{
    Err(e)=>(false,String::new(),e),
    Ok(actual)=>{let passed=c.expected.matches(&actual);let text=actual.output().unwrap_or_else(|e|e.to_string());let why=if passed{"The actual entries and selected attributes match this case.".to_string()}else if matches!(actual,client::Outcome::Unavailable{..}){"The URI selected no supplied directory. Repair the virtual URI or compare an explicit -H override.".into()}else if matches!(actual,client::Outcome::NoSuchObject){"The directory was selected, but the search base does not exist. Check every DN component; this is different from zero matching entries.".into()}else{format!("The search ran, but its returned entries or attributes differ. {}",c.reason)};(passed,text,why)}
   };Report{ran,label:c.label.clone(),passed,feedback,actual,expected}
  }).collect()
    }
    pub fn reports(&self, config: &str, command: &str) -> Result<Vec<Report>, String> {
        self.shape()?;
        if !config_text(config) || !command_text(command) {
            return Err("Configuration/query exceeds this task's text bounds.".into());
        }
        Ok(self.reports_validated(config, command))
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
        if self.version != 1 || !crate::cron_workspace::within(self, MAX_DEFINITION) {
            return Err("Unsupported or oversized directory lesson.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.reference_config == self.example.reference_config
            || self.initial.cases == self.example.cases
        {
            return Err(
                "Guided practice needs a separate task, configuration and worked query.".into(),
            );
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        s.checks(self)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub applied: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub checked_config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub checked_command: String,
    pub checked: bool,
    pub selected_case: u8,
    #[serde(skip)]
    pub output: Option<String>,
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            config: t.starter_config.clone(),
            applied: t.starter_config.clone(),
            command: t.starter_query.clone(),
            checked_config: String::new(),
            checked_command: String::new(),
            checked: false,
            selected_case: 0,
            output: None,
        })
    }
    fn valid_shape(&self) -> bool {
        config_text(&self.config)
            && config_text(&self.applied)
            && config_text(&self.checked_config)
            && command_text(&self.command)
            && command_text(&self.checked_command)
            && self.selected_case < 8
            && self
                .output
                .as_ref()
                .is_none_or(|s| s.len() <= client::MAX_OUTPUT + 256)
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if !self.valid_shape() || usize::from(self.selected_case) >= t.cases.len() {
            return Err("Saved directory work exceeds this lesson's bounds.".into());
        }
        if self.applied != t.starter_config {
            client::Settings::parse(&self.applied).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn invalidate(&mut self) {
        self.checked = false;
        self.output = None;
    }
    pub fn edit_config(&mut self, text: String) -> Result<(), String> {
        if self.config != text {
            self.invalidate();
            if !config_text(&text) {
                return Err("Configuration exceeds8KiB or contains unsupported control text; prior draft retained, current success cleared.".into());
            }
            self.config = text;
        }
        Ok(())
    }
    pub fn edit_command(&mut self, text: String) -> Result<(), String> {
        if self.command != text {
            self.invalidate();
            if !command_text(&text) {
                return Err("Search exceeds4KiB or contains a line break; prior draft retained, current success cleared.".into());
            }
            self.command = text;
        }
        Ok(())
    }
    pub fn apply(&mut self) -> Result<(), String> {
        self.invalidate();
        client::Settings::parse(&self.config).map_err(|e| e.to_string())?;
        self.applied = self.config.clone();
        Ok(())
    }
    pub fn select_case(&mut self, t: &Task, index: usize) -> Result<(), String> {
        self.bound(t)?;
        if index >= t.cases.len() {
            return Err("Select a supplied directory case.".into());
        }
        if index != usize::from(self.selected_case) {
            self.selected_case = index as u8;
            self.output = None;
        }
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.output = None;
        if self.config != self.applied {
            return Err("Apply the edited configuration before running the search.".into());
        }
        let c = &t.cases[usize::from(self.selected_case)];
        let s = client::Settings::parse(&self.applied).map_err(|e| e.to_string())?;
        let actual = c
            .environment()?
            .run(&s, &self.command)
            .map_err(|e| e.to_string())?;
        self.output = Some(
            if matches!(&actual,client::Outcome::Entries(v)if v.is_empty()) {
                "No matching entries (successful search).".into()
            } else {
                actual.output().map_err(|e| e.to_string())?
            },
        );
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Report>, String> {
        self.invalidate();
        self.bound(t)?;
        let reports = t.reports(&self.config, &self.command)?;
        self.checked_config = self.config.clone();
        self.checked_command = self.command.clone();
        self.checked = true;
        if client::Settings::parse(&self.config).is_ok() {
            self.applied = self.config.clone();
        }
        Ok(reports)
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if !self.checked
            || self.config != self.checked_config
            || self.command != self.checked_command
        {
            return vec![(
                false,
                "Check the current configuration and search against every supplied case.".into(),
            )];
        }
        match t.reports(&self.config, &self.command) {
            Ok(r) => r
                .iter()
                .map(|r| (r.passed, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
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
            assignment: Scope::new(&d.initial)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.origin)
            || !self.assignment.valid_shape()
            || !self.practice.valid_shape()
            || !crate::cron_workspace::within(self, MAX_SESSION)
        {
            return Err("Invalid bounded directory session.".into());
        }
        Ok(())
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        d.validate()?;
        self.validate()?;
        if self.origin != d.initial.id {
            return Err("Directory work belongs to another lesson.".into());
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved directory work exceeds144KiB.".into());
        }
        let s: Self = serde_json::from_str(raw)
            .map_err(|_| "Malformed or unsupported directory work.".to_string())?;
        s.bound(d)?;
        Ok(s)
    }
    pub fn checks(&self, d: &Definition) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(d) {
            return vec![(false, e)];
        }
        self.assignment.checks(&d.initial)
    }
    pub fn fill_example(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.practice
            .edit_config(d.example.reference_config.clone())?;
        self.practice
            .edit_command(d.example.reference_query.clone())
    }
    pub fn reset_assignment(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.assignment = Scope::new(&d.initial)?;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<(), String> {
        self.bound(d)?;
        self.practice = Scope::new(&d.example)?;
        Ok(())
    }
    /// Internal source/reference adapter, never a learner command or host engine.
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > 16 * 1024 {
            return Err("Directory reference input exceeds16KiB.".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Work {
            #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
            config: String,
            #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
            command: String,
        }
        let work: Work = serde_json::from_str(raw)
            .map_err(|_| "Unsupported internal directory work data.".to_string())?;
        let mut next = self.assignment.clone();
        next.edit_config(work.config)?;
        next.edit_command(work.command)?;
        let rows = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "Directory work: {}/{} cases match",
            rows.iter().filter(|r| r.passed).count(),
            rows.len()
        ))
    }
}

pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LdapClient { simulation },
        crate::engine::Goal::LdapClient { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Directory source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Directory task identity, objective and fresh session must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::LdapClient { simulation } = &a.state else {
        return Err("Directory work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible directory attempt metadata; original retained.".into());
    }
    simulation.bound(d)
}
