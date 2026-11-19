//! Versioned in-memory SSH lesson sessions. Authoring events are declarative data,
//! never scripts. Only source-listed virtual documents/attributes can be changed.
use crate::{
    script_teaching::View,
    ssh_access::{Expected, Request, Scope, World},
    ssh_identity::{AuthorizedKeys, KnownHosts},
    ssh_policy::Policy,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Document {
    KnownHosts,
    ServerConfig,
    KeyFile { path: String },
    Attributes { path: String },
}
impl Document {
    pub fn label(&self) -> String {
        match self {
            Self::KnownHosts => "Client · known_hosts".into(),
            Self::ServerConfig => "Server · sshd_config".into(),
            Self::KeyFile { path } | Self::Attributes { path } => path.clone(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyRule {
    StrictModes,
    OnlyPublicKey,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Access {
        label: String,
        host: String,
        user: String,
        identity: String,
        expected: Expected,
    },
    Published {},
    Policy {
        rule: PolicyRule,
    },
    KeySet {
        path: String,
        required: Vec<String>,
        forbidden: Vec<String>,
    },
    PreserveTrust {
        host: String,
    },
}
impl Requirement {
    pub fn label(&self) -> String {
        match self {
            Self::Access { label, .. } => label.clone(),
            Self::Published {} => "Saved server policy is applied".into(),
            Self::Policy {
                rule: PolicyRule::StrictModes,
            } => "Keep StrictModes enabled".into(),
            Self::Policy {
                rule: PolicyRule::OnlyPublicKey,
            } => "Offer only public-key authentication".into(),
            Self::KeySet { path, .. } => format!("Required public identities in {path}"),
            Self::PreserveTrust { host } => format!("Preserve the trusted key for {host}"),
        }
    }
    pub fn check(&self, fixture: &Fixture, w: &World) -> (bool, String) {
        let result: Result<(bool, String), String> = (|| {
            Ok(match self {
   Self::Access{host,user,identity,expected,..}=>fixture.scope.check_expected(w,&Request{host:host.clone(),user:user.clone(),identity:identity.clone()},*expected),
   Self::Published{}=>(w.published(),if w.published(){"Saved and active policy have the same supported behavior."}else{"Save the intended configuration, then apply it. Invalid or unapplied edits cannot satisfy this requirement."}.into()),
   Self::Policy{rule}=>{
    let text=w.active_config.as_ref().ok_or("No server policy is active.")?;let p=Policy::parse(text).map_err(|e|e.to_string())?;
    let pass=match rule{PolicyRule::StrictModes=>p.strict_modes,PolicyRule::OnlyPublicKey=>p.pubkey && (p.key_only||!p.password&&!p.keyboard_interactive)};
    (pass,match rule{PolicyRule::StrictModes=>"Keep the key-path safety check enabled; repair file access instead of bypassing it.",PolicyRule::OnlyPublicKey=>"The active policy must offer public keys and exclude both password and keyboard-interactive methods."}.into())
   },
   Self::KeySet{path,required,forbidden}=>{
    let text=w.tree.files.get(path).ok_or("The required virtual key file is missing.")?;let keys=AuthorizedKeys::parse(text)?;
    if required.len()+forbidden.len()>16{return Err("Public-identity requirement exceeds the supported case limit.".into());}
    let mut pass=true;
    for id in required{let key=fixture.scope.identities.get(id).ok_or("Requirement references an unknown supplied identity.")?;pass &= keys.contains(*key);}
    for id in forbidden{let key=fixture.scope.identities.get(id).ok_or("Requirement references an unknown supplied identity.")?;pass &= !keys.contains(*key);}
    (pass,"Include the required supplied public identities and remove the forbidden ones; comments and unrelated keys do not substitute for their actual public data.".into())
   },
   Self::PreserveTrust{host}=>{
    let before=KnownHosts::parse(&fixture.world.known_hosts)?;let after=KnownHosts::parse(&w.known_hosts)?;
    let keys:Vec<_>=before.entries.iter().filter(|e|e.names.iter().any(|h|h==host)).map(|e|e.key).collect();
    let pass=!keys.is_empty()&&keys.iter().all(|key|after.check(host,*key).is_ok_and(|t|t==crate::ssh_identity::Trust::Trusted));
    (pass,format!("Keep the originally supplied trusted public keys for {host}; repairing one host must not remove another host's trust."))
   },
  })
        })();
        result.unwrap_or_else(|e| (false, format!("Not evaluated: {e}")))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub scope: Scope,
    pub world: World,
    pub documents: Vec<Document>,
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub objective: String,
    pub initial: Fixture,
    pub example: Fixture,
    pub example_solution: Vec<Event>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TextDraft {
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MetadataDraft {
    pub path: String,
    pub owner: String,
    pub group: String,
    pub mode: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Drafts {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub known_hosts: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub config: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,TextDraft,16>")]
    pub files: Vec<TextDraft>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,MetadataDraft,32>")]
    pub metadata: Vec<MetadataDraft>,
    pub selected: Document,
}
impl Drafts {
    pub fn new(f: &Fixture) -> Self {
        Self {
            known_hosts: f.world.known_hosts.clone(),
            config: f.world.saved_config.clone(),
            files: f
                .scope
                .editable_text
                .iter()
                .map(|p| TextDraft {
                    path: p.clone(),
                    text: f.world.tree.files[p].clone(),
                })
                .collect(),
            metadata: f
                .scope
                .editable_metadata
                .iter()
                .map(|p| {
                    let n = &f.world.tree.nodes[p];
                    MetadataDraft {
                        path: p.clone(),
                        owner: n.owner.clone(),
                        group: n.group.clone(),
                        mode: format!("{:03o}", n.mode),
                    }
                })
                .collect(),
            selected: f.documents[0].clone(),
        }
    }
    pub fn valid_for(&self, f: &Fixture) -> bool {
        let mut file_paths: Vec<_> = self.files.iter().map(|d| d.path.as_str()).collect();
        file_paths.sort();
        let mut supplied: Vec<_> = f.scope.editable_text.iter().map(String::as_str).collect();
        supplied.sort();
        let mut meta_paths: Vec<_> = self.metadata.iter().map(|d| d.path.as_str()).collect();
        meta_paths.sort();
        let mut supplied_meta: Vec<_> = f
            .scope
            .editable_metadata
            .iter()
            .map(String::as_str)
            .collect();
        supplied_meta.sort();
        f.documents.contains(&self.selected)
            && self.known_hosts.len() <= 16384
            && self.config.len() <= 16384
            && file_paths == supplied
            && meta_paths == supplied_meta
            && self.files.iter().all(|d| d.text.len() <= 16384)
            && self.metadata.iter().all(|d| {
                d.path.len() <= 160
                    && d.owner.len() <= 32
                    && d.group.len() <= 32
                    && d.mode.len() <= 16
            })
    }
    pub fn matches(&self, w: &World) -> bool {
        self.known_hosts == w.known_hosts
            && self.config == w.saved_config
            && self
                .files
                .iter()
                .all(|d| w.tree.files.get(&d.path) == Some(&d.text))
            && self.metadata.iter().all(|d| {
                w.tree.nodes.get(&d.path).is_some_and(|n| {
                    d.owner == n.owner && d.group == n.group && mode(&d.mode).ok() == Some(n.mode)
                })
            })
    }
}
fn mode(value: &str) -> Result<u16, String> {
    if value.is_empty() || value.len() > 4 || !value.bytes().all(|b| (b'0'..=b'7').contains(&b)) {
        return Err("Permissions need an octal value from 000 to 777 in this field.".into());
    }
    u16::from_str_radix(value, 8)
        .ok()
        .filter(|v| *v <= 0o777)
        .ok_or("Permissions exceed 777.".into())
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Select {
        document: Document,
    },
    Text {
        document: Document,
        text: String,
    },
    Metadata {
        path: String,
        owner: String,
        group: String,
        mode: String,
    },
    Save {
        document: Document,
    },
    ApplyPolicy {},
    Check {},
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub world: World,
    pub drafts: Drafts,
    pub checked: bool,
    #[serde(default)]
    pub notice: Option<String>,
}
impl Attempt {
    pub fn new(f: &Fixture) -> Self {
        Self {
            world: f.world.clone(),
            drafts: Drafts::new(f),
            checked: false,
            notice: None,
        }
    }
    pub fn valid_for(&self, f: &Fixture) -> bool {
        if !self.drafts.valid_for(f)
            || f.scope.validate_world(&self.world).is_err()
            || self.notice.as_ref().is_some_and(|s| s.len() > 4096)
        {
            return false;
        }
        if !f.documents.contains(&Document::KnownHosts)
            && (self.world.known_hosts != f.world.known_hosts
                || self.drafts.known_hosts != self.world.known_hosts)
        {
            return false;
        }
        if !f.documents.contains(&Document::ServerConfig)
            && (self.world.saved_config != f.world.saved_config
                || self.world.active_config != f.world.active_config
                || self.drafts.config != self.world.saved_config)
        {
            return false;
        }
        true
    }
    pub fn checks(&self, f: &Fixture) -> Vec<(bool, String)> {
        if !self.valid_for(f) {
            return vec![(false,"Saved simulation data does not match the supplied activity. Preserve/export it before resetting this attempt.".into())];
        }
        let mut rows=vec![(self.drafts.matches(&self.world),"Save or discard pending document/permission edits before checking the complete assignment.".into())];
        rows.extend(f.requirements.iter().map(|r| {
            let (pass, why) = r.check(f, &self.world);
            (pass, format!("{} · {why}", r.label()))
        }));
        rows
    }
    pub fn solved(&self, f: &Fixture) -> bool {
        self.checked && self.checks(f).iter().all(|r| r.0)
    }
    pub fn apply(&mut self, f: &Fixture, event: &Event) -> Result<(), String> {
        if !self.valid_for(f) {
            return Err("This saved attempt does not match the supplied activity.".into());
        }
        match event {
            Event::Select { document } => {
                if !f.documents.contains(document) {
                    return Err("Unsupported document for this activity.".into());
                }
                self.drafts.selected = document.clone();
                return Ok(());
            }
            Event::Text { document, text } => {
                if text.len() > 16384 || !f.documents.contains(document) {
                    return Err(
                        "Document is unavailable or exceeds the 16,384-byte text limit.".into(),
                    );
                }
                match document {
                    Document::KnownHosts => self.drafts.known_hosts = text.clone(),
                    Document::ServerConfig => self.drafts.config = text.clone(),
                    Document::KeyFile { path } => {
                        self.drafts
                            .files
                            .iter_mut()
                            .find(|d| &d.path == path)
                            .ok_or("Key file is not editable here.")?
                            .text = text.clone()
                    }
                    Document::Attributes { .. } => {
                        return Err("A directory has permissions, not key-file text.".into());
                    }
                }
            }
            Event::Metadata {
                path,
                owner,
                group,
                mode,
            } => {
                if owner.len() > 32 || group.len() > 32 || mode.len() > 16 {
                    return Err("Permission fields exceed this activity's limits.".into());
                }
                let d = self
                    .drafts
                    .metadata
                    .iter_mut()
                    .find(|d| &d.path == path)
                    .ok_or("This path's attributes are not editable.")?;
                d.owner = owner.clone();
                d.group = group.clone();
                d.mode = mode.clone();
            }
            Event::Save { document } => {
                if !f.documents.contains(document) {
                    return Err("Unsupported document for this activity.".into());
                }
                let mut candidate = self.world.clone();
                match document {
                    Document::KnownHosts => {
                        KnownHosts::parse(&self.drafts.known_hosts)?;
                        candidate.known_hosts = self.drafts.known_hosts.clone();
                    }
                    Document::ServerConfig => candidate.saved_config = self.drafts.config.clone(),
                    Document::KeyFile { path } => {
                        let d = self
                            .drafts
                            .files
                            .iter()
                            .find(|d| &d.path == path)
                            .ok_or("Missing key-file draft.")?;
                        AuthorizedKeys::parse(&d.text)?;
                        candidate.tree.files.insert(path.clone(), d.text.clone());
                    }
                    Document::Attributes { .. } => {}
                }
                if let Document::KeyFile { path } | Document::Attributes { path } = document
                    && let Some(d) = self.drafts.metadata.iter().find(|d| &d.path == path)
                {
                    if !f.scope.initial_tree.users.contains_key(&d.owner)
                        || !f
                            .scope
                            .initial_tree
                            .users
                            .values()
                            .any(|groups| groups.contains(&d.group))
                    {
                        return Err(
                            "Choose a supplied owner and group; no system accounts are changed."
                                .into(),
                        );
                    }
                    let permissions = mode(&d.mode)?;
                    let n = candidate
                        .tree
                        .nodes
                        .get_mut(path)
                        .ok_or("Missing virtual path.")?;
                    n.owner = d.owner.clone();
                    n.group = d.group.clone();
                    n.mode = permissions;
                }
                f.scope.validate_world(&candidate)?;
                self.world = candidate;
                self.notice = Some(
                    if *document == Document::ServerConfig {
                        "Configuration saved. Apply policy to change the active server behavior."
                    } else {
                        "Virtual document/attributes saved. Check access to inspect their effect."
                    }
                    .into(),
                );
            }
            Event::ApplyPolicy {} => {
                if !f.documents.contains(&Document::ServerConfig) {
                    return Err("This activity does not expose the server policy editor.".into());
                }
                self.world.reload()?;
                self.notice = Some("Saved policy applied in the simulation.".into());
            }
            Event::Check {} => {
                self.checked = true;
                self.notice = None;
                return Ok(());
            }
        }
        self.checked = false;
        if !matches!(event, Event::Save { .. } | Event::ApplyPolicy {}) {
            self.notice = None;
        }
        Ok(())
    }
}
impl Fixture {
    pub fn validate(&self) -> Result<(), String> {
        self.scope.validate_world(&self.world)?;
        if self.documents.is_empty()
            || self.documents.len() > 12
            || self.requirements.is_empty()
            || self.requirements.len() > 24
        {
            return Err(
                "An SSH activity needs a bounded document list and meaningful requirements.".into(),
            );
        }
        if !self.requirements.iter().any(|r| {
            matches!(
                r,
                Requirement::Access {
                    expected: Expected::Permitted,
                    ..
                }
            )
        }) || !self.requirements.iter().any(|r| {
            matches!(
                r,
                Requirement::Access {
                    expected: Expected::PolicyDenies | Expected::KeyDenies,
                    ..
                }
            )
        }) {
            return Err(
                "Include a permitted request and a meaningful policy/key refusal case.".into(),
            );
        }
        let mut labels = std::collections::BTreeSet::new();
        if self.requirements.iter().any(|r| !labels.insert(r.label())) {
            return Err("SSH requirements need distinct, understandable labels.".into());
        }
        for (i, d) in self.documents.iter().enumerate() {
            if self.documents[..i].contains(d) {
                return Err("Duplicate SSH document.".into());
            }
            match d {
                Document::KeyFile { path } if !self.scope.editable_text.contains(path) => {
                    return Err("Key editor is not source-authorized.".into());
                }
                Document::Attributes { path } if !self.scope.editable_metadata.contains(path) => {
                    return Err("Attribute editor is not source-authorized.".into());
                }
                _ => {}
            }
        }
        for path in &self.scope.editable_text {
            if !self
                .documents
                .contains(&Document::KeyFile { path: path.clone() })
            {
                return Err("Every mutable key file needs a visible editor.".into());
            }
        }
        for path in &self.scope.editable_metadata {
            if !self
                .documents
                .contains(&Document::Attributes { path: path.clone() })
                && !self
                    .documents
                    .contains(&Document::KeyFile { path: path.clone() })
            {
                return Err("Every mutable attribute needs a visible document.".into());
            }
        }
        for r in &self.requirements {
            match r {
                Requirement::Access {
                    label,
                    host,
                    user,
                    identity,
                    ..
                } => {
                    if label.trim().is_empty()
                        || label.len() > 160
                        || host != &self.scope.host
                        || !crate::ssh_policy::user(user)
                        || !self.scope.identities.contains_key(identity)
                    {
                        return Err("Invalid named SSH access case.".into());
                    }
                }
                Requirement::KeySet {
                    path,
                    required,
                    forbidden,
                } => {
                    if !self.world.tree.files.contains_key(path)
                        || required.is_empty() && forbidden.is_empty()
                        || required
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                            != required.len()
                        || forbidden
                            .iter()
                            .collect::<std::collections::BTreeSet<_>>()
                            .len()
                            != forbidden.len()
                        || required.len() + forbidden.len() > 16
                        || required
                            .iter()
                            .chain(forbidden)
                            .any(|id| !self.scope.identities.contains_key(id))
                        || required.iter().any(|id| forbidden.contains(id))
                    {
                        return Err("Invalid required/forbidden public-identity set.".into());
                    }
                }
                Requirement::PreserveTrust { host }
                    if host == &self.scope.host
                        || !KnownHosts::parse(&self.world.known_hosts)?
                            .entries
                            .iter()
                            .any(|e| e.names.contains(host)) =>
                {
                    return Err("Protected trust must name an existing different host.".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 80
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.objective.trim().is_empty()
            || self.objective.len() > 1600
        {
            return Err("Use a stable SSH lesson identifier and a clear bounded objective.".into());
        }
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.scope.host == self.example.scope.host
            || self.initial.scope.host_key == self.example.scope.host_key
            || self.initial.scope.initial_tree.home == self.example.scope.initial_tree.home
            || self.example_solution.is_empty()
            || self.example_solution.len() > 64
        {
            return Err(
                "Use different supplied host/key/account values and a bounded worked repair."
                    .into(),
            );
        }
        let mut a = Attempt::new(&self.example);
        for e in &self.example_solution {
            a.apply(&self.example, e)?;
        }
        if !a.solved(&self.example) {
            return Err("Worked SSH repair cannot satisfy its declared outcomes.".into());
        }
        if Attempt::new(&self.initial)
            .checks(&self.initial)
            .iter()
            .all(|r| r.0)
        {
            return Err("Independent SSH assignment is already solved.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub id: String,
    pub version: u32,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            id: d.id.clone(),
            version: 1,
            view: View::Introduction,
            assignment: Attempt::new(&d.initial),
            practice: Attempt::new(&d.example),
        }
    }
    pub fn valid_for(&self, d: &Definition) -> bool {
        self.version == 1
            && self.id == d.id
            && self.assignment.valid_for(&d.initial)
            && self.practice.valid_for(&d.example)
    }
    pub fn apply(&mut self, d: &Definition, e: &Event, practice: bool) -> Result<(), String> {
        if !self.valid_for(d) {
            return Err("Invalid saved SSH lesson data.".into());
        }
        let (attempt, fixture) = if practice {
            (&mut self.practice, &d.example)
        } else {
            (&mut self.assignment, &d.initial)
        };
        let result = attempt.apply(fixture, e);
        if let Err(message) = &result {
            attempt.notice = Some(message.clone());
        }
        result
    }
    pub fn fill_example(&mut self, d: &Definition) -> Result<(), String> {
        for e in &d.example_solution {
            if matches!(e, Event::Text { .. } | Event::Metadata { .. }) {
                self.practice.apply(&d.example, e)?;
            }
        }
        self.practice.checked = false;
        Ok(())
    }
    pub fn reset_example(&mut self, d: &Definition) {
        self.practice = Attempt::new(&d.example);
        self.view = View::Example;
    }
}

pub const HELP: &str = "This native SSH workspace changes only supplied virtual documents and file attributes. It compares supplied host public keys, active authentication policy and ordinary nonroot key-file access. Public Ed25519 encodings are data, not private credentials or signature verification. Only the documented literal-host/plain-key/configuration subset is evaluated; unsupported syntax is diagnosed. No actual SSH client, server, shell, connection, cryptographic handshake or host file is used.";
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        let shape = |a: &Attempt| {
            let w = &a.world;
            let d = &a.drafts;
            w.version == 1
                && w.tree.valid()
                && w.known_hosts.len() <= 16384
                && w.saved_config.len() <= 16384
                && w.active_config
                    .as_ref()
                    .is_none_or(|c| c.len() <= 16384 && Policy::parse(c).is_ok())
                && d.known_hosts.len() <= 16384
                && d.config.len() <= 16384
                && d.files.len() <= 16
                && d.metadata.len() <= 32
                && d.files
                    .iter()
                    .all(|f| f.path.len() <= 160 && f.text.len() <= 16384)
                && d.metadata.iter().all(|m| {
                    m.path.len() <= 160
                        && m.owner.len() <= 32
                        && m.group.len() <= 32
                        && m.mode.len() <= 16
                })
                && a.notice.as_ref().is_none_or(|n| n.len() <= 4096)
        };
        if self.version != 1
            || self.id.is_empty()
            || self.id.len() > 80
            || !shape(&self.assignment)
            || !shape(&self.practice)
        {
            return Err("Invalid bounded SSH lesson session.".into());
        }
        Ok(())
    }
    pub fn bound(&self, d: &Definition) -> Result<(), String> {
        self.validate()?;
        if !self.valid_for(d) {
            return Err("Saved SSH work belongs to another supplied task or contains changes outside its editable files.".into());
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        self.bound(d)?;
        if input.len() > 32768 {
            return Err("SSH authoring event exceeds the supported size.".into());
        }
        let text=input.strip_prefix("ssh-lab ").ok_or("Use the native SSH workspace. ssh-lab JSON is a simulator authoring format, not a Linux command.")?;
        let event: Event = serde_json::from_str(text)
            .map_err(|e| format!("Invalid declarative SSH event: {e}"))?;
        self.apply(d, &event, false)?;
        Ok("Virtual SSH action applied.".into())
    }
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.bound(self) {
            return vec![(false, e)];
        }
        let (f, a) = if practice {
            (&self.example, &s.practice)
        } else {
            (&self.initial, &s.assignment)
        };
        let mut rows = vec![(
            a.checked,
            "Check access to evaluate the saved configuration and pending edits.".into(),
        )];
        rows.extend(a.checks(f));
        rows
    }
}

fn admitted(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::SshAccess { simulation },
        crate::engine::Goal::SshAccess { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("SSH task and workspace do not match.".into());
    };
    if !matches!(s.track.as_str(), "LPIC-1" | "LPIC-2")
        || definition.id != s.id
        || definition.objective != s.objective
        || **simulation != Session::new(definition)
    {
        return Err("Match the supplied Linux SSH lesson, objective and initial session.".into());
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
    let crate::engine::State::SshAccess { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved SSH progress metadata is incompatible.".into());
    }
    simulation.bound(d)
}
