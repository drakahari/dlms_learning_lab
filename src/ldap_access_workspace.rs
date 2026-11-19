//! Source-held ACL editing tasks. Runs apply real bounded modeled Modify requests.
//! Given sessions are established before the editable policy; no ACL-aware Bind,
//! host file, server command, transport or external credential is executed here.
use crate::{
    ldap_access as acl, ldap_changes as changes, ldap_configuration as config,
    ldap_identity::{self as identity, protected},
    ldap_ldif::{Change, Edit, EditKind},
    ldap_protection as tls, ldap_query as q,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_DRAFT: usize = 8192;
pub const MAX_SESSION: usize = 256 * 1024;
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn text(s: &str, n: usize) -> bool {
    s.len() <= n
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && text(s, n)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,1024>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub attribute: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub value: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub actor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub target: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Field,2>")]
    pub fields: Vec<Field>,
    pub allow: bool,
}
impl Probe {
    fn request(&self) -> Result<Change, String> {
        Ok(Change::Modify {
            dn: q::Dn::parse(&self.target).map_err(|e| e.to_string())?,
            edits: self
                .fields
                .iter()
                .map(|f| Edit {
                    kind: EditKind::Replace,
                    attribute: q::Attribute {
                        name: f.attribute.clone(),
                        values: vec![f.value.clone()],
                    },
                })
                .collect(),
        })
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
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,8>")]
    pub identities: Vec<Name>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,16>")]
    pub probes: Vec<Probe>,
}
// Source-held fiction for given established sessions, never a learner password.
const SAMPLE: &str = "Fictional-established-context-only-91";
const URI: &str = "ldap://acl.example.test";
impl Case {
    fn realm(&self) -> Result<protected::ProtectedRealm, String> {
        let suffix = q::Dn::parse(&self.initial.suffix).map_err(|e| e.to_string())?;
        let people = self
            .identities
            .iter()
            .map(|n| {
                identity::Identity::new(
                    q::Dn::parse(&n.0).map_err(|e| e.to_string())?,
                    SAMPLE,
                    suffix.clone(),
                    false,
                    false,
                    &[],
                )
                .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let certificate =
            tls::Certificate::new(&["acl.example.test"], "scenario-root", 0, 100, true)
                .map_err(|e| e.to_string())?;
        let endpoint =
            tls::Endpoint::new(URI, true, Some(certificate)).map_err(|e| e.to_string())?;
        protected::ProtectedRealm::new(
            endpoint,
            &self.initial,
            &people,
            protected::Policy {
                supports_password_modify: false,
                allow_self_change: false,
                require_old_password: false,
            },
        )
        .map_err(|e| e.to_string())
    }
    fn run(&self, index: usize, prepared: &Prepared) -> Result<Run, String> {
        let p = self
            .probes
            .get(index)
            .ok_or("Choose an existing change request.")?;
        let mut realm = self.realm()?;
        let connection = realm
            .open(
                URI,
                tls::Request::RequireStartTls,
                &tls::Trust::new(&["scenario-root"]).map_err(|e| e.to_string())?,
                50,
            )
            .map_err(|e| e.to_string())?;
        let actor = q::Dn::parse(&p.actor).map_err(|e| e.to_string())?;
        let auth = realm
            .authenticate(&connection, &actor, SAMPLE)
            .map_err(|e| e.to_string())?;
        let request = p.request()?;
        let before = realm.directory().source().clone();
        // Expected state uses authored allow/deny requirements and the ordinary schema
        // engine, never the learner's policy or the reference policy's decision.
        let mut expected = changes::Directory::new(&self.initial).map_err(|e| e.to_string())?;
        if p.allow {
            let grant = changes::Grant::new(
                actor,
                expected.read().suffix().clone(),
                false,
                false,
                &["mail", "description"],
            )
            .map_err(|e| e.to_string())?;
            expected
                .apply(&request, &grant)
                .map_err(|e| e.to_string())?;
        }
        let (allowed, feedback, decisions) = match realm.modify_with_acl(
            &auth,
            &prepared.policy,
            prepared.root_dn.as_ref(),
            &request,
        ) {
            Ok(r) => (
                true,
                "Allowed: the requested contact fields changed.".into(),
                r.decisions,
            ),
            Err(protected::access::Error::Denied {
                attribute,
                decision,
            }) => (
                false,
                format!(
                    "Denied: no fields changed. This change needs write access to {attribute}. {}",
                    decision.explanation()
                ),
                vec![(attribute, decision)],
            ),
            Err(e) => return Err(e.to_string()),
        };
        let after = realm.directory().source().clone();
        let data_passed = allowed == p.allow && after == *expected.source();
        let configuration_preserved = prepared.configuration.as_ref().is_none_or(|c| c.preserved);
        let passed = data_passed && configuration_preserved;
        let explanation = if !configuration_preserved {
            "The configuration change also altered an unrelated database. Restore its access values and deploy the policy only to the required data backend.".into()
        } else if passed {
            if allowed{"This request is allowed by the task, and the complete resulting directory matches the requested change."}else{"This request must be denied by the task, and the complete directory is unchanged."}.into()
        } else if allowed && !p.allow {
            "This caller changed data the task requires you to protect. Inspect the first matching target rule and its first matching by clause; a later restriction cannot undo that grant.".into()
        } else if !allowed && p.allow {
            "This required change was blocked. Check the target scope, attribute names and caller order in the rule shown by the actual result.".into()
        } else {
            "The resulting directory differs from the full requested state; preserve every unrelated value.".into()
        };
        Ok(Run {
            label: p.label.clone(),
            expected_allow: p.allow,
            allowed,
            passed,
            feedback,
            explanation,
            before,
            after,
            decisions,
            configuration: prepared.configuration.clone(),
        })
    }
}
/// An opt-in configuration deployment. Existing static-policy tasks omit this
/// field entirely, retaining their serialized values and existing behavior.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deployment {
    pub initial: config::Source,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub target: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationRun {
    pub before: config::Source,
    pub after: config::Source,
    pub requests: Vec<config::Applied>,
    pub preserved: bool,
    pub feedback: String,
}
struct Prepared {
    policy: acl::Policy,
    root_dn: Option<q::Dn>,
    configuration: Option<std::sync::Arc<ConfigurationRun>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<Deployment>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2400>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub reference: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,4>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub label: String,
    pub expected_allow: bool,
    pub allowed: bool,
    pub passed: bool,
    pub feedback: String,
    pub explanation: String,
    pub before: q::Source,
    pub after: q::Source,
    pub decisions: Vec<(String, acl::Decision)>,
    pub configuration: Option<std::sync::Arc<ConfigurationRun>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub passed: bool,
    pub feedback: String,
    pub outcomes: Vec<Run>,
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 2400)
            || !text(&self.starter, MAX_DRAFT)
            || !text(&self.reference, MAX_DRAFT)
            || !(2..=4).contains(&self.cases.len())
        {
            return Err(
                "Use a versioned bounded ACL task with two to four sample directories.".into(),
            );
        }
        if let Some(deployment) = &self.configuration {
            let server =
                config::Configuration::new(&deployment.initial).map_err(|e| e.to_string())?;
            let target = config::Name::parse(&deployment.target).map_err(|e| e.to_string())?;
            let inventory = server.source();
            let database = inventory
                .databases
                .iter()
                .find(|d| config::Name::parse(&d.name).is_ok_and(|n| n == target))
                .ok_or("Choose an actual supplied data configuration entry.")?;
            let suffix = database
                .suffix
                .as_deref()
                .ok_or("The deployment target must be a data backend.")?;
            let suffix = q::Dn::parse(suffix).map_err(|e| e.to_string())?;
            for case in &self.cases {
                if !q::Dn::parse(&case.initial.suffix)
                    .map_err(|e| e.to_string())?
                    .equivalent(&suffix)
                {
                    return Err("Every sample directory must belong to the supplied deployment target's suffix.".into());
                }
            }
        }
        let mut ids = BTreeSet::new();
        let mut labels = BTreeSet::new();
        for c in &self.cases {
            if !key(&c.id)
                || !prose(&c.label, 120)
                || !ids.insert(&c.id)
                || !labels.insert(&c.label)
                || !(2..=8).contains(&c.identities.len())
                || !(2..=16).contains(&c.probes.len())
                || c.initial.entries.len() > 16
            {
                return Err("Use distinct sample IDs/labels, two to eight identities, two to sixteen requests, and at most sixteen entries.".into());
            }
            let realm = c.realm()?;
            let mut people = Vec::new();
            for n in &c.identities {
                let name = q::Dn::parse(&n.0).map_err(|e| e.to_string())?;
                if people.iter().any(|p: &q::Dn| p.equivalent(&name)) {
                    return Err("Each established identity must be distinct.".into());
                }
                people.push(name);
            }
            let mut requests = BTreeSet::new();
            for p in &c.probes {
                let actor = q::Dn::parse(&p.actor).map_err(|e| e.to_string())?;
                if !prose(&p.label, 120)
                    || !requests.insert(&p.label)
                    || !people.iter().any(|name| name.equivalent(&actor))
                    || p.fields.is_empty()
                    || p.fields.len() > 2
                {
                    return Err("Each request needs a distinct label, an established actor and one or two contact fields.".into());
                }
                let mut fields = BTreeSet::new();
                for f in &p.fields {
                    let name = f.attribute.to_ascii_lowercase();
                    if !matches!(name.as_str(), "mail" | "description")
                        || !fields.insert(name)
                        || f.value.is_empty()
                        || f.value.len() > 256
                        || !f.value.bytes().all(|b| (32..=126).contains(&b))
                    {
                        return Err("Requests name distinct mail/description fields and bounded printable values.".into());
                    }
                }
                // Even expected denials must be meaningful, schema-valid requests. Otherwise
                // malformed source requests could be mistaken for correct access denials.
                let mut staged = realm.directory().clone();
                let grant = changes::Grant::new(
                    actor,
                    staged.read().suffix().clone(),
                    false,
                    false,
                    &["mail", "description"],
                )
                .map_err(|e| e.to_string())?;
                staged
                    .apply(&p.request()?, &grant)
                    .map_err(|e| e.to_string())?;
                if staged.source() == realm.directory().source() {
                    return Err(
                        "Each request must actually change a contact value if authorized.".into(),
                    );
                }
            }
            if !c.probes.iter().any(|p| p.allow) || c.probes.iter().all(|p| p.allow) {
                return Err("Every sample needs required permissions and required denials.".into());
            }
        }
        if self.cases.iter().skip(1).all(|c| {
            c.initial == self.cases[0].initial
                && c.identities == self.cases[0].identities
                && c.probes == self.cases[0].probes
        }) {
            return Err(
                "Use meaningfully different directories or requests, not renamed copies.".into(),
            );
        }
        Ok(())
    }
    fn prepare(&self, draft: &str) -> Result<Prepared, String> {
        let Some(deployment) = &self.configuration else {
            return Ok(Prepared {
                policy: acl::Policy::parse(draft).map_err(|e| e.to_string())?,
                root_dn: None,
                configuration: None,
            });
        };
        let mut server =
            config::Configuration::new(&deployment.initial).map_err(|e| e.to_string())?;
        let before = server.source();
        let result = server.apply_document(draft);
        if let Some(e) = result.error {
            return Err(format!(
                "Configuration change stopped after {} successful request(s). Earlier changes remain in this trial. {e}",
                result.applied.len()
            ));
        }
        let after = server.source();
        let target = config::Name::parse(&deployment.target).map_err(|e| e.to_string())?;
        let untouched: Vec<_> = before
            .databases
            .iter()
            .filter(|d| config::Name::parse(&d.name).is_ok_and(|n| n != target))
            .collect();
        let changed: Vec<_> = untouched
            .iter()
            .filter(|d| !after.databases.iter().any(|new| new == **d))
            .map(|d| d.name.as_str())
            .collect();
        let preserved = changed.is_empty();
        let feedback = if preserved {
            format!(
                "{} configuration request(s) applied. Other databases and their settings are unchanged.",
                result.applied.len()
            )
        } else {
            format!(
                "Unrelated configuration changed: {}. Restore those access values; this task only changes {}.",
                changed.join(", "),
                deployment.target
            )
        };
        let suffix = q::Dn::parse(&self.cases[0].initial.suffix).map_err(|e| e.to_string())?;
        let (policy, root_dn) = server.policy_for(&suffix).map_err(|e| e.to_string())?;
        Ok(Prepared {
            policy,
            root_dn,
            configuration: Some(std::sync::Arc::new(ConfigurationRun {
                before,
                after,
                requests: result.applied,
                preserved,
                feedback,
            })),
        })
    }
    fn reports_with(&self, draft: &str) -> Vec<Report> {
        let prepared = match self.prepare(draft) {
            Ok(p) => p,
            Err(e) => {
                return self
                    .cases
                    .iter()
                    .map(|c| Report {
                        label: c.label.clone(),
                        passed: false,
                        feedback: e.to_string(),
                        outcomes: vec![],
                    })
                    .collect();
            }
        };
        self.cases.iter().map(|c|{
   let outcomes=(0..c.probes.len()).map(|index|c.run(index,&prepared)).collect::<Result<Vec<_>,_>>();
   match outcomes{Err(e)=>Report{label:c.label.clone(),passed:false,feedback:e,outcomes:vec![]},Ok(outcomes)=>{let passed=outcomes.iter().all(|r|r.passed);let feedback=outcomes.iter().find(|r|!r.passed).map(|r|format!("{}: {} {}",r.label,r.explanation,r.feedback)).unwrap_or_else(||"Every required change works, every protected change is denied, and complete resulting directories are correct.".into());Report{label:c.label.clone(),passed,feedback,outcomes}}}
  }).collect()
    }
    pub fn reports(&self, draft: &str) -> Result<Vec<Report>, String> {
        self.shape()?;
        if !text(draft, MAX_DRAFT) {
            return Err("Keep the ACL draft within 8192 bytes and ordinary text controls.".into());
        }
        Ok(self.reports_with(draft))
    }
    pub fn run(&self, draft: &str, case: usize, probe: usize) -> Result<Run, String> {
        self.shape()?;
        if !text(draft, MAX_DRAFT) {
            return Err("Keep the ACL draft within 8192 bytes.".into());
        }
        let prepared = self.prepare(draft)?;
        self.cases
            .get(case)
            .ok_or("Choose an existing sample directory.")?
            .run(probe, &prepared)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self.reports_with(&self.reference).iter().any(|r| !r.passed) {
            return Err(
                "The worked ACL cannot reach every authored permission/preservation requirement."
                    .into(),
            );
        }
        if self.reports_with(&self.starter).iter().all(|r| r.passed) {
            return Err("The independent ACL starter must leave meaningful work.".into());
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
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 1024 * 1024
        {
            return Err("Use a bounded version 1 ACL definition.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.cases == self.example.cases
            || self.initial.reference == self.example.reference
        {
            return Err("Practice needs a separate directory, context ID and worked draft.".into());
        }
        Ok(())
    }
}
fn checked_draft<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Bounded(#[serde(deserialize_with = "crate::cron::text::<_,8192>")] String);
    Ok(Option::<Bounded>::deserialize(d)?.map(|s| s.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    pub selected_case: u8,
    pub selected_probe: u8,
    pub checked: bool,
    #[serde(deserialize_with = "checked_draft")]
    pub checked_draft: Option<String>,
    #[serde(skip)]
    pub output: Option<Run>,
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            draft: t.starter.clone(),
            selected_case: 0,
            selected_probe: 0,
            checked: false,
            checked_draft: None,
            output: None,
        }
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if !text(&self.draft, MAX_DRAFT)
            || self
                .checked_draft
                .as_ref()
                .is_some_and(|s| !text(s, MAX_DRAFT))
            || usize::from(self.selected_case) >= t.cases.len()
            || usize::from(self.selected_probe)
                >= t.cases[usize::from(self.selected_case)].probes.len()
            || self.checked != self.checked_draft.is_some()
        {
            return Err("Saved ACL work has invalid bounds, selection or check markers.".into());
        }
        Ok(())
    }
    pub fn edit(&mut self, draft: String) -> Result<(), String> {
        self.checked = false;
        self.checked_draft = None;
        self.output = None;
        if !text(&draft, MAX_DRAFT) {
            return Err(
                "Keep the ACL draft within 8192 bytes; the previous draft is retained.".into(),
            );
        }
        self.draft = draft;
        Ok(())
    }
    pub fn select(&mut self, t: &Task, case: usize, probe: usize) -> Result<(), String> {
        if t.cases
            .get(case)
            .and_then(|c| c.probes.get(probe))
            .is_none()
        {
            return Err("Choose an existing sample and request.".into());
        }
        self.selected_case = case as u8;
        self.selected_probe = probe as u8;
        self.output = None;
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.output = None;
        let output = t.run(
            &self.draft,
            usize::from(self.selected_case),
            usize::from(self.selected_probe),
        )?;
        self.output = Some(output);
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.bound(t)?;
        let _ = t.reports(&self.draft)?;
        self.checked = true;
        self.checked_draft = Some(self.draft.clone());
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if !self.checked || self.checked_draft.as_deref() != Some(&self.draft) {
            return t
                .cases
                .iter()
                .map(|c| (false, format!("{}: unchecked", c.label)))
                .collect();
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
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        if self.version != 1
            || self.context != d.initial.id
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err("Saved ACL work belongs to a different or oversized context.".into());
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
    }
    pub fn reset_assignment(&mut self, d: &Definition) {
        self.assignment = Scope::new(&d.initial);
    }
}

pub const PREFIX: &str = "LDAP ACCESS WORK\n";
pub const HELP: &str = "Edit a static access to policy for given established identities. Run request tests one real simulated mail/description Modify request; Check work repeats all source-held requests. A denied request changes no fields. Supported: bounded DN scopes, attrs, self/users/anonymous/explicit caller selectors and cumulative levels with stop behavior. This is not a whole server configuration, ACL-aware fresh Bind/search, add/delete, schema or password operation. No host files, commands or network are used.";
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.context)
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > MAX_SESSION
        {
            return Err("Invalid ACL session version, identity or size.".into());
        }
        for s in [&self.assignment, &self.practice] {
            if !text(&s.draft, MAX_DRAFT)
                || s.checked_draft
                    .as_ref()
                    .is_some_and(|s| !text(s, MAX_DRAFT))
                || s.checked != s.checked_draft.is_some()
                || s.selected_case >= 4
                || s.selected_probe >= 16
            {
                return Err(
                    "Saved ACL draft, check markers or selection exceed this subset.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        let value = raw.strip_prefix(PREFIX).ok_or(HELP)?;
        if value.len() > 64 * 1024 {
            return Err("ACL reference envelope exceeds 64 KiB.".into());
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Work {
            #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
            policy: String,
        }
        let work: Work =
            serde_json::from_str(value).map_err(|e| format!("Invalid ACL draft: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit(work.policy)?;
        next.check(&d.initial)?;
        let rows = next.checks(&d.initial);
        self.assignment = next;
        Ok(format!(
            "ACL samples: {}/{} match",
            rows.iter().filter(|r| r.0).count(),
            rows.len()
        ))
    }
}
impl Definition {
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        s.assignment.checks(&self.initial)
    }
}
fn admitted(s: &crate::pack::Scenario) -> Result<(&Definition, &Session), String> {
    let (
        crate::engine::Goal::LdapAccess { definition },
        crate::engine::State::LdapAccess { simulation },
    ) = (&s.goal, &s.initial)
    else {
        return Err("ACL source/state mismatch.".into());
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
        return Err("ACL source identity, objective and fresh session must match.".into());
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
    let crate::engine::State::LdapAccess { simulation } = &a.state else {
        return Err("ACL work belongs to a different workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible ACL work metadata; original retained.".into());
    }
    simulation.validate()?;
    simulation.bound(d)
}
