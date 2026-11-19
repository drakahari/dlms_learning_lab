//! Authored LDIF editing tasks. All requests operate on fresh in-memory samples.
use crate::{
    ldap_changes as changes, ldap_client as client, ldap_ldif as ldif, ldap_query as q,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub mod utility;
pub const MAX_DEFINITION: usize = 4 * 1024 * 1024;
pub const MAX_DRAFT: usize = 16 * 1024;
pub const MAX_SESSION: usize = 144 * 1024;
pub const PREFIX: &str = "LDAP CHANGE WORK\n";
pub const HELP: &str = "Edit LDIF data, then Apply LDIF to inspect the selected sample. The whole document must parse before any request runs; a syntax error applies nothing. Check work tests every supplied sample. Each application starts from its own original directory; earlier successful records remain if a later record fails. Apply LDIF is a simulator action, not a Linux command. Version1, content or add/delete/modify records, printable ASCII, folded lines and bounded base64 are supported. Modify supports mail/description only; no URL loading, controls, rename, passwords, actual bind, network, host files or processes. The supplied identity and delegated rights are scenario facts, not real authentication or OpenLDAP ACL configuration.";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && s.chars().all(|c| !c.is_control() || c == '\n')
}
fn text(s: &str) -> bool {
    s.len() <= MAX_DRAFT
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grant {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub principal: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub subtree: String,
    pub add: bool,
    pub delete: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,SmallText,2>")]
    pub attributes: Vec<SmallText>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmallText(#[serde(deserialize_with = "crate::cron::text::<_,32>")] pub String);
impl Grant {
    fn model(&self) -> Result<changes::Grant, String> {
        if self.principal.len() > 1024 || self.subtree.len() > 1024 || self.attributes.len() > 2 {
            return Err("Delegated identity exceeds its bounds.".into());
        }
        let p = q::Dn::parse(&self.principal).map_err(|e| e.to_string())?;
        let b = q::Dn::parse(&self.subtree).map_err(|e| e.to_string())?;
        changes::Grant::new(
            p,
            b,
            self.add,
            self.delete,
            &self
                .attributes
                .iter()
                .map(|s| s.0.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub initial: q::Source,
    pub expected: q::Source,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub reason: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4000>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub reference: String,
    pub grant: Grant,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub utility: Option<utility::Utility>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub applied: bool,
    pub feedback: String,
    pub actual: String,
    pub directory: q::Source,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub passed: bool,
    pub feedback: String,
    pub actual: String,
}
pub fn output(source: &q::Source) -> Result<String, String> {
    q::Directory::new(source).map_err(|e| e.to_string())?;
    client::Outcome::Entries(
        source
            .entries
            .iter()
            .map(|e| client::ResultEntry {
                dn: e.dn.clone(),
                attributes: e.attributes.clone(),
            })
            .collect(),
    )
    .output()
    .map_err(|e| e.to_string())
}
fn prepared(s: &str) -> String {
    s.split(' ')
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}
// Compare semantic state, not LDIF record/attribute/value ordering or source strings.
fn difference(actual: &q::Source, expected: &q::Source, retirement: bool) -> Option<String> {
    for e in &expected.entries {
        let dn = match q::Dn::parse(&e.dn) {
            Ok(dn) => dn,
            Err(_) => return Some("The expected directory contains an invalid name.".into()),
        };
        let Some(a) = actual
            .entries
            .iter()
            .find(|a| q::Dn::parse(&a.dn).is_ok_and(|a| a.equivalent(&dn)))
        else {
            return Some(if retirement {
                format!("Missing entry: {}. This entry must remain. Delete only the requested full names; preserve the rest.", e.dn)
            } else {
                format!("Missing entry: {}. Check its DN and whether its Add record ran.", e.dn)
            });
        };
        for field in &e.attributes {
            let Some(got) = a
                .attributes
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(&field.name))
            else {
                return Some(format!(
                    "{} is missing {}. Check that the attribute was supplied and the request succeeded.",
                    e.dn, field.name
                ));
            };
            if field.values.len() != got.values.len()
                || field
                    .values
                    .iter()
                    .any(|v| !got.values.iter().any(|g| prepared(g) == prepared(v)))
            {
                return Some(format!(
                    "{} has a different {} value set. Add preserves old values; Replace sets the whole list. {}",
                    e.dn, field.name, "Compare the actual entry with the task requirements."
                ));
            }
        }
        if let Some(extra) = a.attributes.iter().find(|a| {
            !e.attributes
                .iter()
                .any(|e| a.name.eq_ignore_ascii_case(&e.name))
        }) {
            return Some(format!(
                "{} has an unwanted {} attribute. Remove only what the task asks you to remove.",
                e.dn, extra.name
            ));
        }
    }
    if let Some(extra) = actual.entries.iter().find(|a| {
        !expected.entries.iter().any(|e| {
            q::Dn::parse(&e.dn)
                .ok()
                .zip(q::Dn::parse(&a.dn).ok())
                .is_some_and(|(e, a)| e.equivalent(&a))
        })
    }) {
        return Some(format!(
            "Unexpected entry remains: {}. Check the retirement requirement; preserve every unrelated entry.",
            extra.dn
        ));
    }
    None
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 4000)
            || !text(&self.starter)
            || !text(&self.reference)
            || !(3..=8).contains(&self.cases.len())
            || !crate::cron_workspace::within(self, MAX_DEFINITION)
        {
            return Err("Directory-change task needs version1, bounded text and3–8samples.".into());
        }
        self.grant.model()?;
        let mut ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        for c in &self.cases {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || !prose(&c.label, 120)
                || !labels.insert(&c.label)
                || !prose(&c.reason, 800)
            {
                return Err(
                    "Directory samples need distinct identities/labels and clear reasons.".into(),
                );
            }
            for source in [&c.initial, &c.expected] {
                let d = changes::Directory::new(source).map_err(|e| e.to_string())?;
                output(d.source())?;
            }
            if !q::Dn::parse(&c.initial.suffix)
                .map_err(|e| e.to_string())?
                .equivalent(&q::Dn::parse(&c.expected.suffix).map_err(|e| e.to_string())?)
            {
                return Err("A sample cannot change its naming context.".into());
            }
        }
        if self
            .cases
            .iter()
            .skip(1)
            .all(|c| c.initial == self.cases[0].initial)
        {
            return Err("Use meaningfully different initial directory samples.".into());
        }
        if let Some(u) = &self.utility {
            u.validate(self)?;
        }
        Ok(())
    }
    fn run_case(&self, index: usize, draft: &str) -> Result<Run, String> {
        if !text(draft) {
            return Err("LDIF draft exceeds16KiB or contains unsupported control text.".into());
        }
        let c = self.cases.get(index).ok_or("Select an existing sample.")?;
        if let Some(u) = &self.utility {
            return u.run(self, index, draft);
        }
        let mut directory = changes::Directory::new(&c.initial).map_err(|e| e.to_string())?;
        let doc = ldif::parse(draft).map_err(|e| e.to_string())?;
        let batch = directory
            .apply_document(&doc, &self.grant.model()?)
            .map_err(|e| e.to_string())?;
        let mut feedback = String::new();
        for (i, result) in batch.results.iter().enumerate() {
            use std::fmt::Write;
            match result {
                Ok(r) => writeln!(feedback, "Record {} · {:?}: {}", i + 1, r.operation, r.dn),
                Err(e) => writeln!(feedback, "Record {} · Failed: {e}", i + 1),
            }
            .map_err(|_| "Could not format request results.")?;
        }
        if batch.unattempted > 0 {
            feedback.push_str(&format!("{} later record(s) not attempted. Earlier successful records remain in this sample.\n",batch.unattempted));
        }
        Ok(Run {
            applied: batch.succeeded(),
            feedback,
            actual: output(directory.source())?,
            directory: directory.source().clone(),
        })
    }
    pub fn run(&self, index: usize, draft: &str) -> Result<Run, String> {
        self.shape()?;
        self.run_case(index, draft)
    }
    fn reports_validated(&self, draft: &str) -> Vec<Report> {
        self.cases.iter().enumerate().map(|(index,c)|{
        let (passed,feedback,actual)=match self.run_case(index,draft){
            Err(e)=>(false,format!("Not applied: {e}"),String::new()),
            Ok(run)=>{
                let expected=changes::Directory::new(&c.expected).expect("validated expected sample");
                let diff=difference(&run.directory,expected.source(),self.utility.as_ref().is_some_and(utility::Utility::is_dn_list));
                let passed=run.applied&&diff.is_none();
                let feedback=if !run.applied{format!("{}\n{}",run.feedback,c.reason)}else if let Some(why)=diff{format!("{why} {}",c.reason)}else{"All requests succeeded and the resulting entries match; unrelated data is preserved.".into()};
                (passed,feedback,run.actual)
            }
        };Report{label:c.label.clone(),passed,feedback,actual}
    }).collect()
    }
    pub fn reports(&self, draft: &str) -> Result<Vec<Report>, String> {
        self.shape()?;
        if !text(draft) {
            return Err("LDIF draft exceeds its text bounds.".into());
        }
        Ok(self.reports_validated(draft))
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self
            .reports_validated(&self.reference)
            .iter()
            .any(|r| !r.passed)
        {
            return Err("Worked LDIF cannot reach every declared state.".into());
        }
        if self
            .reports_validated(&self.starter)
            .iter()
            .all(|r| r.passed)
        {
            return Err("Independent starter is already solved.".into());
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
        if self.version != 1 || !crate::cron_workspace::within(self, MAX_DEFINITION) {
            return Err("Unsupported or oversized directory-change definition.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.reference == self.example.reference
            || self.initial.cases == self.example.cases
        {
            return Err(
                "Practice needs separate sample inputs and a separate worked draft.".into(),
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
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub checked_draft: String,
    pub checked: bool,
    pub selected_case: u8,
    #[serde(skip)]
    pub output: Option<Run>,
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            draft: t.starter.clone(),
            checked_draft: String::new(),
            checked: false,
            selected_case: 0,
            output: None,
        })
    }
    fn shape(&self) -> bool {
        text(&self.draft)
            && text(&self.checked_draft)
            && self.selected_case < 8
            && self.output.as_ref().is_none_or(|r| {
                r.actual.len() <= client::MAX_OUTPUT
                    && r.feedback.len() <= client::MAX_OUTPUT
                    && crate::cron_workspace::within(&r.directory, q::MAX_JSON)
            })
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if !self.shape()
            || usize::from(self.selected_case) >= t.cases.len()
            || t.utility
                .as_ref()
                .is_some_and(|u| u.bound_draft(&self.draft).is_err())
        {
            Err("Saved directory-change work exceeds this task's bounds.".into())
        } else {
            Ok(())
        }
    }
    pub fn invalidate(&mut self) {
        self.checked = false;
        self.output = None;
    }
    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if text != self.draft {
            self.invalidate();
            if !crate::ldap_change_workspace::text(&text) {
                return Err("LDIF exceeds16KiB or contains unsupported control text; prior draft retained and current success cleared.".into());
            }
            self.draft = text;
        }
        Ok(())
    }
    pub fn select_case(&mut self, t: &Task, index: usize) -> Result<(), String> {
        self.bound(t)?;
        if index >= t.cases.len() {
            return Err("Select an existing sample.".into());
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
        self.output = Some(t.run(usize::from(self.selected_case), &self.draft)?);
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Report>, String> {
        self.invalidate();
        self.bound(t)?;
        let reports = t.reports(&self.draft)?;
        self.checked_draft = self.draft.clone();
        self.checked = true;
        Ok(reports)
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if !self.checked || self.draft != self.checked_draft {
            return vec![(
                false,
                "Check the current LDIF against every supplied sample.".into(),
            )];
        }
        match t.reports(&self.draft) {
            Ok(rows) => rows
                .into_iter()
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
            || !self.assignment.shape()
            || !self.practice.shape()
            || !crate::cron_workspace::within(self, MAX_SESSION)
        {
            Err("Invalid bounded directory-change session.".into())
        } else {
            Ok(())
        }
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        d.validate()?;
        self.validate()?;
        if self.origin != d.initial.id {
            return Err("Directory-change work belongs to another lesson.".into());
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn from_json(d: &Definition, raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_SESSION {
            return Err("Saved directory-change work exceeds144KiB.".into());
        }
        let s: Self = serde_json::from_str(raw).map_err(|_| "Malformed directory-change work.")?;
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
        self.practice.edit(d.example.reference.clone())
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
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        let draft = input.strip_prefix(PREFIX).ok_or(HELP)?;
        if !text(draft) {
            return Err(
                "Directory-change work exceeds16KiB or contains unsupported control text.".into(),
            );
        }
        let mut next = self.assignment.clone();
        next.edit(draft.into())?;
        let rows = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "Directory changes: {}/{} samples match",
            rows.iter().filter(|r| r.passed).count(),
            rows.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LdapChanges { simulation },
        crate::engine::Goal::LdapChanges { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Directory-change source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err(
            "Directory-change task identity, objective and fresh session must match.".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::LdapChanges { simulation } = &a.state else {
        return Err("Directory-change work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible directory-change metadata; original retained.".into());
    }
    simulation.bound(d)
}
