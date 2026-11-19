//! Custom-schema consumer verification. All supplied entries, schema definitions
//! and filter evaluation remain bounded, owned in-process data.
use crate::{
    ldap_query::{Dn, Truth},
    ldap_schema as model,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_SCHEMA: usize = 8192;
pub const MAX_FILTER: usize = 4096;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str, max: usize) -> bool {
    s.len() <= max
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && text(s, max)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleProbe {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub attribute: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub left: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub right: String,
    pub equal: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,model::EntrySource,12>")]
    pub entries: Vec<model::EntrySource>,
    #[serde(deserialize_with = "dn_list")]
    pub expected: Vec<String>,
}
fn dn_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Text(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] String);
    crate::cron_workspace::sequence::<_, Text, 12>(d).map(|v| v.into_iter().map(|t| t.0).collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub schema_name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub oid_branch: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter_schema: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub starter_filter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub reference_schema: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub reference_filter: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,RuleProbe,8>")]
    pub rules: Vec<RuleProbe>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,4>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub dn: String,
    pub truth: Truth,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub ran: bool,
    pub selection_matches: bool,
    pub rule_checks: Vec<(bool, String)>,
    pub label: String,
    pub passed: bool,
    pub expected: Vec<String>,
    pub actual: Vec<String>,
    pub observations: Vec<Observation>,
    pub issues: Vec<String>,
}
fn same_names(a: &[String], b: &[String]) -> bool {
    a.len() == b.len()
        && a.iter().all(|s| {
            b.iter().any(|t| {
                Dn::parse(s)
                    .ok()
                    .zip(Dn::parse(t).ok())
                    .is_some_and(|(a, b)| a.equivalent(&b))
            })
        })
}
impl Task {
    pub fn shape(&self) -> Result<(), String> {
        if !key(&self.id)
            || !prose(&self.instructions, 2048)
            || !key(&self.schema_name)
            || self.schema_name.len() > 32
            || !model::numeric_oid(&self.oid_branch)
            || !(2..=4).contains(&self.cases.len())
            || self.rules.is_empty()
            || self.rules.len() > 8
            || ![&self.starter_schema, &self.reference_schema]
                .iter()
                .all(|s| text(s, MAX_SCHEMA))
            || ![&self.starter_filter, &self.reference_filter]
                .iter()
                .all(|s| text(s, MAX_FILTER))
        {
            return Err("Invalid schema-consumer identity, text, rule or sample bounds.".into());
        }
        let mut ids = BTreeSet::new();
        for c in &self.cases {
            if !key(&c.id)
                || !ids.insert(&c.id)
                || !prose(&c.label, 120)
                || c.entries.is_empty()
                || c.entries.len() > 12
                || c.expected.len() > 12
            {
                return Err(
                    "Use distinct, labeled sample sets with1–12 entries and bounded expected names."
                        .into(),
                );
            }
            let mut names = Vec::new();
            for entry in &c.entries {
                entry.validate_shape().map_err(|e| e.to_string())?;
                let dn = Dn::parse(&entry.dn).map_err(|e| e.to_string())?;
                if names.iter().any(|n: &Dn| n.equivalent(&dn)) {
                    return Err("Sample entry names must be distinct.".into());
                }
                names.push(dn);
            }
            let mut expected = Vec::new();
            for name in &c.expected {
                let dn = Dn::parse(name).map_err(|e| e.to_string())?;
                if !names.iter().any(|n| n.equivalent(&dn))
                    || expected.iter().any(|n: &Dn| n.equivalent(&dn))
                {
                    return Err("Expected names must be distinct supplied entries.".into());
                }
                expected.push(dn);
            }
        }
        let mut labels = BTreeSet::new();
        for r in &self.rules {
            if !prose(&r.label, 120)
                || !labels.insert(&r.label)
                || r.attribute.is_empty()
                || r.attribute.len() > 32
                || !r
                    .attribute
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                || !text(&r.left, 256)
                || !text(&r.right, 256)
            {
                return Err(
                    "Invalid comparison-rule test; use bounded labels, descriptors and values."
                        .into(),
                );
            }
        }
        Ok(())
    }
    fn prepare(&self, schema: &str) -> Result<model::Directory, String> {
        self.shape()?;
        if !text(schema, MAX_SCHEMA) {
            return Err("Keep the schema within 8192 bytes and ordinary editor characters.".into());
        }
        let mut d = model::Directory::new(model::Registry::teaching_base());
        let load = d.load(schema);
        if let Some(error) = load.error {
            return Err(format!(
                "{} schema Add(s) applied before failure: {error}",
                load.applied.len()
            ));
        }
        if d.modules().len() != 1 || !d.modules()[0].name.eq_ignore_ascii_case(&self.schema_name) {
            return Err(format!(
                "Add one schema entry named {}; keep the supplied base definitions.",
                self.schema_name
            ));
        }
        let branch = format!("{}.", self.oid_branch);
        for module in d.modules() {
            for raw in &module.attributes {
                if !model::attribute_type(raw)
                    .map_err(|e| e.to_string())?
                    .oid
                    .starts_with(&branch)
                {
                    return Err(
                        "New attribute OIDs must stay below the supplied documentation branch."
                            .into(),
                    );
                }
            }
            for raw in &module.classes {
                if !model::object_class(raw)
                    .map_err(|e| e.to_string())?
                    .oid
                    .starts_with(&branch)
                {
                    return Err(
                        "New class OIDs must stay below the supplied documentation branch.".into(),
                    );
                }
            }
        }
        Ok(d)
    }
    fn run_prepared(
        &self,
        prepared: &model::Directory,
        filter: &model::Filter,
        index: usize,
    ) -> Result<Report, String> {
        let c = self
            .cases
            .get(index)
            .ok_or("Choose an existing sample set.")?;
        let mut d = prepared.clone();
        for entry in &c.entries {
            d.add(entry)
                .map_err(|e| format!("Supplied entry {} cannot be loaded: {e}", entry.dn))?;
        }
        let mut issues = Vec::new();
        let mut rule_checks = Vec::new();
        for r in &self.rules {
            let ok = d
                .registry()
                .attribute(&r.attribute)
                .ok_or_else(|| format!("{} is not registered", r.attribute))
                .and_then(|a| a.equivalent(&r.left, &r.right).map_err(|e| e.to_string()));
            rule_checks.push((
                ok.as_ref().is_ok_and(|value| *value == r.equal),
                r.label.clone(),
            ));
            match ok {
                Ok(value) if value == r.equal => (),
                Ok(_) => issues.push(format!(
                    "{}: the comparison rule treats these values differently from the requirement.",
                    r.label
                )),
                Err(e) => issues.push(format!("{}: {e}", r.label)),
            }
        }
        let observations = d
            .evaluate_filter(filter)
            .into_iter()
            .map(|(e, truth)| Observation {
                dn: e.dn().text().into(),
                truth,
            })
            .collect::<Vec<_>>();
        let actual = observations
            .iter()
            .filter(|o| o.truth == Truth::True)
            .map(|o| o.dn.clone())
            .collect::<Vec<_>>();
        if !same_names(&actual, &c.expected) {
            let extra = actual
                .iter()
                .filter(|name| {
                    !c.expected
                        .iter()
                        .any(|n| Dn::parse(name).unwrap().equivalent(&Dn::parse(n).unwrap()))
                })
                .count();
            let missing = c
                .expected
                .len()
                .saturating_sub(actual.len().saturating_sub(extra));
            issues.push(format!("Filter selected {} unexpected and missed {} required entries. Check both criteria and the field comparison rules.",extra,missing));
        }
        let undefined = observations
            .iter()
            .filter(|o| o.truth == Truth::Undefined)
            .count();
        if undefined > 0 {
            issues.push(format!("{undefined} entry comparisons are Undefined, so those entries are not selected. Check field names, comparison rules and assertion formats; NOT does not repair Undefined."));
        }
        Ok(Report {
            ran: true,
            selection_matches: same_names(&actual, &c.expected),
            rule_checks,
            label: c.label.clone(),
            passed: issues.is_empty(),
            expected: c.expected.clone(),
            actual,
            observations,
            issues,
        })
    }
    pub fn run(&self, schema: &str, filter: &str, index: usize) -> Result<Report, String> {
        if !text(filter, MAX_FILTER) {
            return Err("Filter exceeds4096 bytes or contains unsupported editor controls.".into());
        }
        let d = self.prepare(schema)?;
        let filter = model::Filter::parse(filter).map_err(|e| e.to_string())?;
        self.run_prepared(&d, &filter, index)
    }
    fn identity_report(
        &self,
        prepared: &model::Directory,
        filter: &model::Filter,
    ) -> Result<Report, String> {
        let mut used = filter
            .equality_values()
            .into_iter()
            .map(|s| model::Matching::CaseIgnore.key(s))
            .collect::<BTreeSet<_>>();
        let mut cursor = 0usize;
        let mut next_name = || -> Result<String, String> {
            for _ in 0..512 {
                let value = format!("VerifierIdentity{cursor}");
                cursor += 1;
                if used.insert(model::Matching::CaseIgnore.key(&value)) {
                    return Ok(value);
                }
            }
            Err("Identity verification exceeded its bounded sample-name limit.".into())
        };
        let mut changed = self.clone();
        for c in &mut changed.cases {
            let mut original = prepared.clone();
            for e in &c.entries {
                original.add(e).map_err(|e| {
                    format!("Cannot verify changed identities until supplied entries load: {e}")
                })?;
            }
            let mut entries = original.source();
            let mut expected = Vec::new();
            for e in &mut entries {
                let old = Dn::parse(&e.dn).map_err(|e| e.to_string())?;
                let cn = next_name()?;
                let sn = next_name()?;
                let uid = next_name()?;
                let mut changed_cn = false;
                for field in &mut e.fields {
                    let oid = prepared
                        .registry()
                        .attribute(&field.name)
                        .ok_or("A supplied field is not registered.")?
                        .definition
                        .oid
                        .as_str();
                    match oid {
                        "2.5.4.3" => {
                            field.values = vec![cn.clone()];
                            changed_cn = true;
                        }
                        "2.5.4.4" => field.values = vec![sn.clone()],
                        "0.9.2342.19200300.100.1.1" => field.values = vec![uid.clone()],
                        _ => (),
                    }
                }
                if !changed_cn {
                    return Err("This consumer activity requires person entries with a cn field for identity-change verification.".into());
                }
                e.dn = format!("cn={cn},dc=verification,dc=test");
                if c.expected
                    .iter()
                    .any(|name| Dn::parse(name).is_ok_and(|d| d.equivalent(&old)))
                {
                    expected.push(e.dn.clone());
                }
            }
            c.entries = entries;
            c.expected = expected;
        }
        let mut combined = Report {
            ran: true,
            selection_matches: true,
            rule_checks: Vec::new(),
            label: "Changed names and IDs".into(),
            passed: true,
            expected: Vec::new(),
            actual: Vec::new(),
            observations: Vec::new(),
            issues: Vec::new(),
        };
        for n in 0..changed.cases.len() {
            let row = changed.run_prepared(prepared, filter, n)?;
            combined.ran &= row.ran;
            combined.selection_matches &= row.selection_matches;
            combined.passed &= row.passed;
            if combined.rule_checks.is_empty() {
                combined.rule_checks = row.rule_checks;
            }
            combined.expected.extend(row.expected);
            combined.actual.extend(row.actual);
            combined.observations.extend(row.observations);
            for issue in row.issues {
                combined.issues.push(format!("{}: {issue}", row.label));
            }
        }
        if !combined.selection_matches {
            combined.issues.insert(0,"The filter stops selecting the required people when names and user IDs change. Select by team and state instead of enumerating identities.".into());
        }
        Ok(combined)
    }
    pub fn total_checks(&self) -> usize {
        self.cases.len() + 1
    }
    pub fn reports(&self, schema: &str, filter: &str) -> Result<Vec<Report>, String> {
        if !text(filter, MAX_FILTER) {
            return Err("Filter exceeds4096 bytes or contains unsupported editor controls.".into());
        }
        let d = self.prepare(schema)?;
        let filter = model::Filter::parse(filter).map_err(|e| e.to_string())?;
        let mut rows = (0..self.cases.len())
            .map(|n| match self.run_prepared(&d, &filter, n) {
                Ok(report) => report,
                Err(error) => Report {
                    ran: false,
                    selection_matches: false,
                    rule_checks: Vec::new(),
                    label: self.cases[n].label.clone(),
                    passed: false,
                    expected: self.cases[n].expected.clone(),
                    actual: Vec::new(),
                    observations: Vec::new(),
                    issues: vec![error],
                },
            })
            .collect::<Vec<_>>();
        let identity = match self.identity_report(&d, &filter) {
            Ok(r) => r,
            Err(e) => Report {
                ran: false,
                selection_matches: false,
                rule_checks: Vec::new(),
                label: "Changed names and IDs".into(),
                passed: false,
                expected: Vec::new(),
                actual: Vec::new(),
                observations: Vec::new(),
                issues: vec![e],
            },
        };
        rows.push(identity);
        Ok(rows)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if !self
            .reports(&self.reference_schema, &self.reference_filter)?
            .iter()
            .all(|r| r.passed)
        {
            return Err("Reference schema/filter must satisfy every behavioral sample.".into());
        }
        if self
            .reports(&self.starter_schema, &self.starter_filter)
            .is_ok_and(|r| r.iter().all(|r| r.passed))
        {
            return Err("The assignment starter must retain meaningful work.".into());
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
            || self.initial.starter_schema == self.example.starter_schema
            || self.initial.reference_filter == self.example.reference_filter
        {
            return Err(
                "Use version 1 and separate assignment/workshop identities, schema and filters."
                    .into(),
            );
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.initial)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub schema: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub filter: String,
    pub selected_case: u8,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub checked_schema: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub checked_filter: String,
    #[serde(skip)]
    pub output: Option<Report>,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            schema: t.starter_schema.clone(),
            filter: t.starter_filter.clone(),
            selected_case: 0,
            checked: false,
            checked_schema: String::new(),
            checked_filter: String::new(),
            output: None,
        }
    }
    fn shape(&self) -> Result<(), String> {
        if !text(&self.schema, MAX_SCHEMA)
            || !text(&self.checked_schema, MAX_SCHEMA)
            || !text(&self.filter, MAX_FILTER)
            || !text(&self.checked_filter, MAX_FILTER)
            || self.selected_case >= 4
            || !self.checked && (!self.checked_schema.is_empty() || !self.checked_filter.is_empty())
        {
            return Err(
                "Saved schema/filter drafts, selection or check markers are invalid.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        self.shape()?;
        t.shape()?;
        if usize::from(self.selected_case) >= t.cases.len() {
            return Err("Choose an existing sample set.".into());
        }
        Ok(())
    }
    fn invalidate(&mut self) {
        self.checked = false;
        self.checked_schema.clear();
        self.checked_filter.clear();
        self.output = None;
    }
    pub fn edit_schema(&mut self, schema: String) -> Result<(), String> {
        self.invalidate();
        if !text(&schema, MAX_SCHEMA) {
            return Err("Schema exceeds 8192 bytes or has unsupported editor controls; previous draft retained.".into());
        }
        self.schema = schema;
        Ok(())
    }
    pub fn edit_filter(&mut self, filter: String) -> Result<(), String> {
        self.invalidate();
        if !text(&filter, MAX_FILTER) {
            return Err("Filter exceeds4096 bytes or has unsupported editor controls; previous draft retained.".into());
        }
        self.filter = filter;
        Ok(())
    }
    pub fn select(&mut self, t: &Task, index: usize) -> Result<(), String> {
        if index >= t.cases.len() {
            return Err("Choose an existing sample set.".into());
        }
        self.selected_case = index as u8;
        self.output = None;
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.output = None;
        self.bound(t)?;
        self.output = Some(t.run(&self.schema, &self.filter, usize::from(self.selected_case))?);
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        self.checked = true;
        self.checked_schema = self.schema.clone();
        self.checked_filter = self.filter.clone();
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if !self.checked || self.schema != self.checked_schema || self.filter != self.checked_filter
        {
            return t
                .cases
                .iter()
                .map(|c| (false, format!("{}: unchecked", c.label)))
                .chain(std::iter::once((
                    false,
                    "Changed names and IDs: unchecked".into(),
                )))
                .collect();
        }
        match t.reports(&self.schema, &self.filter) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| {
                    (
                        r.passed,
                        format!(
                            "{}: {}",
                            r.label,
                            if r.passed {
                                "required comparisons and selected entries match".into()
                            } else {
                                r.issues.join(" ")
                            }
                        ),
                    )
                })
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
    pub context: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            context: d.initial.id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.initial),
            practice: Scope::new(&d.example),
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !key(&self.context) {
            return Err("Invalid saved schema-consumer version or identity.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != d.initial.id {
            return Err(
                "Saved schema-consumer work belongs to another task; original retained.".into(),
            );
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Scope::new(&d.initial);
    }
}
pub const PREFIX: &str = "LDAP SCHEMA QUERY WORK\n";
pub const HELP: &str = "Edit the schema and a parenthesized filter, then Apply schema and run filter against all supplied entries in the selected sample. Check work tests all samples and required comparison behavior. This native filter activity has no LDAP base/scope, projection, access policy, server, host command or network operation. It is not a full ldapsearch command.";
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        let raw = raw.strip_prefix(PREFIX).ok_or(HELP)?;
        if raw.len() > 64 * 1024 {
            return Err("Schema/query work envelope exceeds 64 KiB.".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Work {
            #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
            schema: String,
            #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
            filter: String,
        }
        let w: Work =
            serde_json::from_str(raw).map_err(|e| format!("Invalid schema/filter draft: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit_schema(w.schema)?;
        next.edit_filter(w.filter)?;
        next.check(&d.initial)?;
        let checks = next.checks(&d.initial);
        self.assignment = next;
        Ok(format!(
            "Schema/filter checks: {}/{} match",
            checks.iter().filter(|c| c.0).count(),
            checks.len()
        ))
    }
}

fn admitted(s: &crate::pack::Scenario) -> Result<(&Definition, &Session), String> {
    let (
        crate::engine::Goal::LdapSchemaQuery { definition },
        crate::engine::State::LdapSchemaQuery { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("Schema-consumer source/state mismatch.".into());
    };
    let fresh = Session {
        version: 1,
        context: definition.initial.id.clone(),
        view: View::Introduction,
        assignment: Scope::new(&definition.initial),
        practice: Scope::new(&definition.example),
    };
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != fresh
    {
        return Err(
            "Schema-consumer source identity, objective and fresh session must match.".into(),
        );
    }
    Ok((definition, simulation))
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (d, _) = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let (d, _) = admitted(s)?;
    let crate::engine::State::LdapSchemaQuery { simulation } = &a.state else {
        return Err("Schema-consumer work belongs to a different workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible schema-consumer work metadata; original retained.".into());
    }
    simulation.bound(d)
}
