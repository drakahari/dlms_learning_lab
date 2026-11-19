//! Declarative protected-password assignments over bounded in-process models.
//! Drafts contain commands, virtual configuration and fictional profile keys only.
use crate::{
    ldap_identity::{self as identity, protected},
    ldap_protection::{self as tls, client},
    ldap_query as q, ldap_utility as utility,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt};
pub const MAX_DEFINITION: usize = 4 * 1024 * 1024;
pub const MAX_SESSION: usize = 128 * 1024;
pub const PREFIX: &str = "LDAP PASSWORD WORK\n";
fn key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max && s.chars().all(|c| !c.is_control() || c == '\n')
}
fn text(s: &str, max: usize) -> bool {
    s.len() <= max
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Key(#[serde(deserialize_with = "crate::cron::text::<_,64>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(#[serde(deserialize_with = "crate::cron::text::<_,253>")] pub String);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Value {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub profile: String,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub sample_password: String,
}
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Value")
            .field("profile", &self.profile)
            .field("sample_password", &"[redacted]")
            .finish()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Person {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub dn: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub profile: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,4>")]
    pub names: Vec<Name>,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub issuer: String,
    pub not_before: u64,
    pub not_after: u64,
    pub confidential: bool,
}
impl Certificate {
    fn model(&self) -> Result<tls::Certificate, String> {
        if self.names.is_empty() || self.names.len() > tls::MAX_NAMES {
            return Err("Supply one to four bounded certificate-name facts.".into());
        }
        tls::Certificate::new(
            &self.names.iter().map(|n| n.0.as_str()).collect::<Vec<_>>(),
            &self.issuer,
            self.not_before,
            self.not_after,
            self.confidential,
        )
        .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RootInput {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Key,4>")]
    pub roots: Vec<Key>,
}
impl RootInput {
    fn model(&self) -> Result<tls::Trust, String> {
        if self.roots.len() > tls::MAX_ROOTS {
            return Err("A virtual CA input allows at most four symbolic roots.".into());
        }
        utility::Inputs::new("", &[(&self.path, "")]).map_err(|e| e.to_string())?;
        tls::Trust::new(&self.roots.iter().map(|k| k.0.as_str()).collect::<Vec<_>>())
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub supports_password_modify: bool,
    pub allow_self_change: bool,
    pub require_old_password: bool,
}
impl Policy {
    fn model(self) -> protected::Policy {
        protected::Policy {
            supports_password_modify: self.supports_password_modify,
            allow_self_change: self.allow_self_change,
            require_old_password: self.require_old_password,
        }
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
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Person,8>")]
    pub people: Vec<Person>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Value,4>")]
    pub values: Vec<Value>,
    pub certificate: Certificate,
    pub supports_start_tls: bool,
    pub sample_time: u64,
    pub policy: Policy,
}
impl Case {
    fn password(&self, key: &str) -> Result<&str, String> {
        self.values
            .iter()
            .find(|v| v.profile == key)
            .map(|v| v.sample_password.as_str())
            .ok_or_else(|| "The source sample is missing a required fictional profile.".into())
    }
    fn realm(&self, uri: &str) -> Result<protected::ProtectedRealm, String> {
        let mut identities = Vec::new();
        for person in &self.people {
            identities.push(
                identity::Identity::new(
                    q::Dn::parse(&person.dn).map_err(|e| e.to_string())?,
                    self.password(&person.profile)?,
                    q::Dn::parse(&self.initial.suffix).map_err(|e| e.to_string())?,
                    false,
                    false,
                    &[],
                )
                .map_err(|e| e.to_string())?,
            );
        }
        let endpoint = tls::Endpoint::new(
            uri,
            self.supports_start_tls,
            Some(self.certificate.model()?),
        )
        .map_err(|e| e.to_string())?;
        protected::ProtectedRealm::new(endpoint, &self.initial, &identities, self.policy.model())
            .map_err(|e| e.to_string())
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub config: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub bind_profile: String,
    pub old_profile: Option<Key>,
    pub replacement_profile: Option<Key>,
}
impl Draft {
    fn shape(&self) -> bool {
        text(&self.command, 4096)
            && !self.command.contains(['\n', '\r'])
            && text(&self.config, 8192)
            && key(&self.bind_profile)
            && self.old_profile.as_ref().is_none_or(|k| key(&k.0))
            && self.replacement_profile.as_ref().is_none_or(|k| key(&k.0))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2400>")]
    pub instructions: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub uri: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub actor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub desired_profile: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Profile,4>")]
    pub profiles: Vec<Profile>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,RootInput,4>")]
    pub ca_inputs: Vec<RootInput>,
    pub starter: Draft,
    pub reference: Draft,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probe {
    pub label: String,
    pub accepted: bool,
    pub expected: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Run {
    pub applied: bool,
    pub feedback: String,
    pub probes: Vec<Probe>,
    pub directory_preserved: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub label: String,
    pub passed: bool,
    pub feedback: String,
    pub probes: Vec<Probe>,
}
fn refused(message: String) -> Run {
    Run {
        applied: false,
        feedback: message,
        probes: vec![],
        directory_preserved: true,
    }
}
fn authenticated(
    realm: &protected::ProtectedRealm,
    c: &tls::Connection,
    dn: &q::Dn,
    password: &str,
) -> Result<bool, String> {
    match realm.authenticate(c, dn, password) {
        Ok(_) => Ok(true),
        Err(protected::Error::Identity(identity::Error::InvalidCredentials)) => Ok(false),
        Err(e) => Err(format!("Verification could not run: {e}")),
    }
}
impl Task {
    fn bound_draft(&self, d: &Draft) -> Result<(), String> {
        if !d.shape() {
            return Err("Keep one bounded command line, client settings and supplied profile keys; previous work is retained.".into());
        }
        for selected in [
            Some(d.bind_profile.as_str()),
            d.old_profile.as_ref().map(|k| k.0.as_str()),
            d.replacement_profile.as_ref().map(|k| k.0.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            if !self.profiles.iter().any(|p| p.id == selected) {
                return Err("Choose only fictional profiles supplied by this task.".into());
            }
        }
        Ok(())
    }
    fn shape(&self) -> Result<(), String> {
        if self.version != 1
            || !key(&self.id)
            || !prose(&self.instructions, 2400)
            || self.uri.len() > 256
            || self.actor.len() > 1024
            || !key(&self.desired_profile)
            || self.profiles.len() < 2
            || self.profiles.len() > 4
            || self.ca_inputs.is_empty()
            || self.ca_inputs.len() > 4
            || self.cases.len() < 2
            || self.cases.len() > 8
            || !crate::cron_workspace::within(self, MAX_DEFINITION)
        {
            return Err("Invalid or oversized protected-password task.".into());
        }
        let actor = q::Dn::parse(&self.actor).map_err(|e| e.to_string())?;
        let mut keys = BTreeSet::new();
        let mut labels = BTreeSet::new();
        for p in &self.profiles {
            if !key(&p.id)
                || !prose(&p.label, 120)
                || !keys.insert(p.id.as_str())
                || !labels.insert(p.label.as_str())
            {
                return Err("Use distinct bounded fictional profile keys and labels.".into());
            }
        }
        if !keys.contains(self.desired_profile.as_str()) {
            return Err("The desired replacement profile must be supplied.".into());
        }
        let mut paths = BTreeSet::new();
        for input in &self.ca_inputs {
            if !paths.insert(&input.path) {
                return Err("Each supplied virtual CA path must be distinct.".into());
            }
            input.model()?;
        }
        self.bound_draft(&self.starter)?;
        self.bound_draft(&self.reference)?;
        let (mut ids, mut case_labels) = (BTreeSet::new(), BTreeSet::new());
        for c in &self.cases {
            if !key(&c.id)
                || !prose(&c.label, 120)
                || !ids.insert(&c.id)
                || !case_labels.insert(&c.label)
                || c.people.is_empty()
                || c.people.len() > 8
                || c.values.len() != self.profiles.len()
            {
                return Err(
                    "Use distinct bounded samples with all supplied fictional profile values."
                        .into(),
                );
            }
            let mut values = BTreeSet::new();
            for v in &c.values {
                if !keys.contains(v.profile.as_str())
                    || !values.insert(v.profile.as_str())
                    || v.sample_password.is_empty()
                    || v.sample_password.len() > 128
                    || !v.sample_password.bytes().all(|b| (32..=126).contains(&b))
                {
                    return Err(
                        "Use one bounded printable fictional sample value per profile.".into(),
                    );
                }
            }
            let realm = c.realm(&self.uri)?;
            let person = c
                .people
                .iter()
                .find(|p| q::Dn::parse(&p.dn).is_ok_and(|n| n.equivalent(&actor)))
                .ok_or("The requested person must have an original identity in every sample.")?;
            if c.password(&person.profile)? == c.password(&self.desired_profile)? {
                return Err("Rotation must use a replacement distinct from the actor's original credential.".into());
            }
            if realm.uri() != crate::ldap_client::uri_key(&self.uri).map_err(|e| e.to_string())? {
                return Err("Protected endpoint/source mismatch.".into());
            }
        }
        if self.cases.iter().skip(1).all(|c| {
            c.initial == self.cases[0].initial
                && c.values == self.cases[0].values
                && c.people == self.cases[0].people
                && c.policy == self.cases[0].policy
                && c.certificate == self.cases[0].certificate
                && c.sample_time == self.cases[0].sample_time
                && c.supports_start_tls == self.cases[0].supports_start_tls
        }) {
            return Err(
                "Use meaningfully different sample data or policies, not just different labels."
                    .into(),
            );
        }
        Ok(())
    }
    fn run_case(&self, index: usize, d: &Draft) -> Result<Run, String> {
        self.bound_draft(d)?;
        let c = self.cases.get(index).ok_or("Choose an existing sample.")?;
        let config = match client::Settings::parse(&d.config) {
            Ok(c) => c,
            Err(e) => return Ok(refused(format!("Client settings: {e}"))),
        };
        let Some(input) = self.ca_inputs.iter().find(|i| i.path == config.ca_input()) else {
            return Ok(refused("That CA input is not supplied. Choose a provided virtual path; no host or system trust lookup occurs.".into()));
        };
        let trust = input.model()?;
        let mut realm = c.realm(&self.uri)?;
        // Directory construction canonicalizes source attribute spelling/order.
        let original_directory = realm.directory().source().clone();
        let prompts = utility::PasswordPrompts {
            bind: c.password(&d.bind_profile)?,
            old_target: d
                .old_profile
                .as_ref()
                .map(|k| c.password(&k.0))
                .transpose()?,
            replacement: d
                .replacement_profile
                .as_ref()
                .map(|k| c.password(&k.0))
                .transpose()?,
        };
        let report = utility::run_password(&d.command, &prompts, &trust, c.sample_time, &mut realm);
        let directory_preserved = realm.directory().source() == &original_directory;
        if let Some(failure) = &report.failure {
            let mut run = refused(format!("Exit status: {}\n{failure}", report.exit_status()));
            run.directory_preserved = directory_preserved;
            return Ok(run);
        }
        let connection = realm
            .open(
                &self.uri,
                tls::Request::RequireStartTls,
                &trust,
                c.sample_time,
            )
            .map_err(|e| format!("Verification connection: {e}"))?;
        let actor = q::Dn::parse(&self.actor).map_err(|e| e.to_string())?;
        let original = c
            .people
            .iter()
            .find(|p| q::Dn::parse(&p.dn).is_ok_and(|n| n.equivalent(&actor)))
            .ok_or("Missing original source identity.")?;
        let mut probes = vec![
            Probe {
                label: "Requested replacement credentials".into(),
                accepted: authenticated(
                    &realm,
                    &connection,
                    &actor,
                    c.password(&self.desired_profile)?,
                )?,
                expected: true,
            },
            Probe {
                label: "Previous credentials for the requested person".into(),
                accepted: authenticated(
                    &realm,
                    &connection,
                    &actor,
                    c.password(&original.profile)?,
                )?,
                expected: false,
            },
        ];
        for p in &c.people {
            let dn = q::Dn::parse(&p.dn).map_err(|e| e.to_string())?;
            if !dn.equivalent(&actor) {
                probes.push(Probe {
                    label: format!("Unrelated identity: {}", p.dn),
                    accepted: authenticated(&realm, &connection, &dn, c.password(&p.profile)?)?,
                    expected: true,
                });
            }
        }
        Ok(Run{applied:true,feedback:"Exit status: 0\nPassword modification succeeded. The checks below are actual fresh authentication attempts against its resulting credential state.".into(),probes,directory_preserved})
    }
    fn reports_validated(&self, d: &Draft) -> Vec<Report> {
        self.cases.iter().enumerate().map(|(index,c)|{
   match self.run_case(index,d){Err(e)=>Report{label:c.label.clone(),passed:false,feedback:e,probes:vec![]},Ok(run)=>{
    let wrong=run.probes.iter().find(|p|p.accepted!=p.expected);
    let passed=run.applied&&run.directory_preserved&&wrong.is_none();
    let feedback=if !run.applied{run.feedback}else if !run.directory_preserved{"Directory attributes changed; change only the requested credential.".into()}else if let Some(p)=wrong{format!("{}: actual {}; expected {}. Check the identity and the separate bind, old-target and replacement profile roles.",p.label,if p.accepted{"Accepted"}else{"Rejected"},if p.expected{"Accepted"}else{"Rejected"})}else{"Replacement credentials work, previous credentials are rejected, and unrelated identities and directory attributes are unchanged.".into()};
    Report{label:c.label.clone(),passed,feedback,probes:run.probes}
   }}
  }).collect()
    }
    pub fn run(&self, index: usize, d: &Draft) -> Result<Run, String> {
        self.shape()?;
        self.run_case(index, d)
    }
    pub fn reports(&self, d: &Draft) -> Result<Vec<Report>, String> {
        self.shape()?;
        self.bound_draft(d)?;
        Ok(self.reports_validated(d))
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self
            .reports_validated(&self.reference)
            .iter()
            .any(|r| !r.passed)
        {
            return Err("The worked password change cannot reach every required state.".into());
        }
        if self
            .reports_validated(&self.starter)
            .iter()
            .all(|r| r.passed)
        {
            return Err("The independent starter must retain meaningful work.".into());
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
            return Err("Unsupported or oversized protected-password definition.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.reference == self.example.reference
            || self.initial.cases == self.example.cases
        {
            return Err("Practice needs separate source inputs and worked values.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub draft: Draft,
    pub checked_draft: Option<Draft>,
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
            checked_draft: None,
            checked: false,
            selected_case: 0,
            output: None,
        })
    }
    fn shape(&self) -> bool {
        self.draft.shape()
            && self.checked_draft.as_ref().is_none_or(Draft::shape)
            && self.selected_case < 8
            && self.output.as_ref().is_none_or(|r| {
                r.feedback.len() <= crate::ldap_client::MAX_OUTPUT
                    && r.probes.len() <= 10
                    && r.probes.iter().all(|p| p.label.len() <= 1200)
            })
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        if !self.shape() || usize::from(self.selected_case) >= t.cases.len() {
            return Err("Saved password work exceeds this task's bounds.".into());
        }
        t.bound_draft(&self.draft)
    }
    pub fn invalidate(&mut self) {
        self.checked = false;
        self.checked_draft = None;
        self.output = None;
    }
    pub fn edit(&mut self, draft: Draft) -> Result<(), String> {
        if draft != self.draft {
            self.invalidate();
            if !draft.shape() {
                return Err("Keep bounded configuration, one command and supplied profile keys; prior draft retained and current success cleared.".into());
            }
            self.draft = draft;
        }
        Ok(())
    }
    pub fn select_case(&mut self, t: &Task, index: usize) -> Result<(), String> {
        self.bound(t)?;
        if index >= t.cases.len() {
            return Err("Choose an existing sample.".into());
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
        let rows = t.reports(&self.draft)?;
        self.checked = true;
        self.checked_draft = Some(self.draft.clone());
        Ok(rows)
    }
    pub fn checks(&self, t: &Task) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(t) {
            return vec![(false, e)];
        }
        if !self.checked || self.checked_draft.as_ref() != Some(&self.draft) {
            return vec![(
                false,
                "Check the current configuration against every sample.".into(),
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
            return Err("Invalid bounded password session.".into());
        }
        Ok(())
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.origin != d.initial.id {
            return Err("Password work belongs to another task.".into());
        }
        self.assignment.bound(&d.initial)?;
        self.practice.bound(&d.example)
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
}

pub const HELP: &str = "Edit only the supplied virtual client settings and literal ldappasswd command, using fictional prompt profiles. Run command starts from the selected original sample; Check work repeats every sample. Supported: -x -W -S -H -D, optional -A/-Z/-ZZ and one self DN; TLS_CACERT plus demand/hard or default TLS_REQCERT. No password arguments/files, personal password prompts, real TLS, network, host files/processes, administrative reset, generated passwords or actual server changes. Expected results are distinct from fresh authentication probes against actual simulated credential state.";
impl Definition {
    pub fn checks(&self, session: &Session) -> Vec<(bool, String)> {
        session.checks(self)
    }
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String, String> {
        self.bound(d)?;
        let text = raw.strip_prefix(PREFIX).ok_or(HELP)?;
        if text.len() > 16 * 1024 {
            return Err("Password work exceeds its bounded draft envelope.".into());
        }
        let draft: Draft =
            serde_json::from_str(text).map_err(|e| format!("Invalid password draft: {e}"))?;
        let mut next = self.assignment.clone();
        next.edit(draft)?;
        let rows = next.check(&d.initial)?;
        self.assignment = next;
        Ok(format!(
            "Password changes: {}/{} samples match",
            rows.iter().filter(|r| r.passed).count(),
            rows.len()
        ))
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LdapPassword { simulation },
        crate::engine::Goal::LdapPassword { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Password source/state mismatch.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id
        || definition.initial.instructions != s.objective
        || **simulation != Session::new(definition)?
    {
        return Err("Password task identity, objective and fresh session must match.".into());
    }
    Ok(definition)
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let definition = source(s)?;
    let crate::engine::State::LdapPassword { simulation } = &a.state else {
        return Err("Password work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible password work metadata; original retained.".into());
    }
    simulation.bound(definition)
}
