//! Source-bound standalone-share assignments. Grading runs fresh virtual trials.
use crate::{filesystem, samba_share as core, samba_share_config as config, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub label: String,
    pub credential: core::Credential,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Denial {
    Authentication,
    SharePolicy,
    ReadOnly,
    UnixAccess,
}
impl Denial {
    fn matches(self, k: core::FaultKind) -> bool {
        match self {
            Self::Authentication => matches!(
                k,
                core::FaultKind::Authentication
                    | core::FaultKind::AccountDisabled
                    | core::FaultKind::UnixIdentity
            ),
            Self::SharePolicy => k == core::FaultKind::SharePolicy,
            Self::ReadOnly => k == core::FaultKind::ReadOnly,
            Self::UnixAccess => matches!(
                k,
                core::FaultKind::DirectorySearch | core::FaultKind::FilePermission
            ),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Operation {
    Read {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    Replace {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub client: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub share: String,
    pub operation: Operation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Expected {
    Read {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Replaced {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Denied {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Denial,4>")]
        layers: Vec<Denial>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Input,8>")]
    pub inputs: Vec<Input>,
    pub request: Request,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountGoal {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub user: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum PolicyGoal {
    ReadOnly {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        share: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: core::World,
    pub reference: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,8>")]
    pub shares: Vec<Name>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,16>")]
    pub editable_accounts: Vec<Name>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,AccountGoal,16>")]
    pub accounts: Vec<AccountGoal>,
    #[serde(
        default,
        deserialize_with = "crate::cron_workspace::sequence::<_,PolicyGoal,8>"
    )]
    pub policy: Vec<PolicyGoal>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Client,8>")]
    pub clients: Vec<Client>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,filesystem::Requirement,32>")]
    pub filesystem: Vec<filesystem::Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,16>")]
    pub cases: Vec<Case>,
    pub trial: Request,
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
fn relative(p: &str) -> bool {
    !p.is_empty()
        && p.len() <= 512
        && !p.starts_with('/')
        && !p.contains('\\')
        && !p.chars().any(char::is_control)
        && p.split('/').all(|s| !s.is_empty() && s != "." && s != "..")
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    pub result: Result<Outcome, core::Fault>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Read(core::ReadFile),
    Replaced(core::Written),
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.detail)?;
        self.reference.validate().map_err(|e| e.detail)?;
        if !id(&self.id)
            || !(1..=8).contains(&self.shares.len())
            || !(2..=8).contains(&self.clients.len())
            || !(4..=16).contains(&self.cases.len())
            || self.filesystem.len() > 32
            || self.editable_accounts.len() > 16
            || self.accounts.len() > 16
        {
            return Err("Supply a bounded named share task with clients and substantive read/write/denial cases.".into());
        }
        let mut shares = BTreeSet::new();
        for n in &self.shares {
            if !id(&n.id) || n.id.len() > 32 || !shares.insert(&n.id) {
                return Err("Use distinct simple required share names.".into());
            }
        }
        let mut clients = BTreeSet::new();
        for c in &self.clients {
            if !id(&c.id)
                || !clients.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 96
                || c.label.chars().any(char::is_control)
            {
                return Err("Use distinct supplied client IDs and readable labels.".into());
            }
            if let core::Credential::Valid { user } | core::Credential::Invalid { user } =
                &c.credential
                && (user == "root" || !self.initial.files.users.contains_key(user))
            {
                return Err("Client proofs must name supplied non-root Unix identities.".into());
            }
        }
        let mut editable = BTreeSet::new();
        for u in &self.editable_accounts {
            if !self.initial.files.users.contains_key(&u.id)
                || u.id == "root"
                || !editable.insert(&u.id)
            {
                return Err("Editable accounts must name distinct supplied non-root users.".into());
            }
        }
        let mut accounts = BTreeSet::new();
        for u in &self.accounts {
            if !self.initial.files.users.contains_key(&u.user)
                || u.user == "root"
                || !accounts.insert(&u.user)
            {
                return Err("Account goals must name distinct supplied non-root users.".into());
            }
        }
        let mut policies = BTreeSet::new();
        if self.policy.len() > 8 {
            return Err("At most8share policy goals are supported.".into());
        }
        for PolicyGoal::ReadOnly { share } in &self.policy {
            if !self.shares.iter().any(|s| s.id == *share) || !policies.insert(share) {
                return Err("Policy goals must name distinct required shares.".into());
            }
        }
        if self
            .filesystem
            .iter()
            .any(|r| !r.valid_for(&self.initial.files))
        {
            return Err("Filesystem goals must refer to valid supplied resources.".into());
        }
        if self.filesystem.iter().any(|r| matches!(r, filesystem::Requirement::Contents {path,..} if path==&self.initial.configuration_path)) {
            return Err("Grade configuration policy and client outcomes, not an exact configuration-file answer.".into());
        }
        self.request_shape(&self.trial)?;
        let mut cases = BTreeSet::new();
        let (mut reads, mut writes, mut denied) = (0, 0, 0);
        for c in &self.cases {
            if !id(&c.id)
                || !cases.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
                || c.inputs.len() > 8
                || c.inputs.iter().map(|i| i.text.len()).sum::<usize>() > 8192
            {
                return Err(
                    "Use distinct bounded cases with readable labels and sample inputs.".into(),
                );
            }
            self.request_shape(&c.request)?;
            let mut paths = BTreeSet::new();
            for i in &c.inputs {
                if i.path == self.initial.configuration_path
                    || !paths.insert(&i.path)
                    || !self
                        .initial
                        .files
                        .nodes
                        .get(&i.path)
                        .is_some_and(|n| !n.directory)
                    || i.text.len() > 4096
                    || i.text.contains('\0')
                {
                    return Err("Sample inputs may replace distinct supplied regular data files, never configuration or invalid text.".into());
                }
            }
            match (&c.request.operation, &c.expected) {
                (Operation::Read { .. }, Expected::Read { path, text })
                    if self.initial.files.files.contains_key(path)
                        && path != &self.initial.configuration_path
                        && text.len() <= 4096
                        && !text.contains('\0') =>
                {
                    reads += 1
                }
                (Operation::Replace { .. }, Expected::Replaced { path, text })
                    if self.initial.files.files.contains_key(path)
                        && path != &self.initial.configuration_path
                        && text.len() <= 4096
                        && !text.contains('\0') =>
                {
                    writes += 1
                }
                (_, Expected::Denied { layers })
                    if !layers.is_empty()
                        && layers.len() <= 4
                        && layers.iter().collect::<BTreeSet<_>>().len() == layers.len() =>
                {
                    denied += 1
                }
                _ => return Err(
                    "Match each read/write/denial expectation to the actual supported operation."
                        .into(),
                ),
            }
        }
        if reads == 0 || writes == 0 || denied == 0 {
            return Err("Include actual read, successful write and denied-operation cases.".into());
        }
        self.owned(&self.reference)?;
        Ok(())
    }
    pub fn request_shape(&self, r: &Request) -> Result<(), String> {
        if !self.clients.iter().any(|c| c.id == r.client)
            || !self
                .shares
                .iter()
                .any(|s| s.id == r.share.to_ascii_lowercase())
        {
            return Err("Choose a supplied client and required share.".into());
        }
        let path = match &r.operation {
            Operation::Read { path } => path,
            Operation::Replace { path, text } => {
                if text.len() > 4096 || text.contains('\0') {
                    return Err(
                        "Trial replacement supports at most4096bytes of text without NUL.".into(),
                    );
                }
                path
            }
        };
        if !relative(path) {
            return Err("Enter a file name inside the selected share, such as draft.txt. Do not start with / or use ../.".into());
        }
        Ok(())
    }
    pub fn request_draft(&self, r: &Request) -> Result<(), String> {
        if !self.clients.iter().any(|c| c.id == r.client)
            || !self
                .shares
                .iter()
                .any(|s| s.id == r.share.to_ascii_lowercase())
        {
            return Err("Choose a supplied client and share.".into());
        }
        let path = match &r.operation {
            Operation::Read { path } => path,
            Operation::Replace { path, text } => {
                if text.len() > 4096 || text.contains('\0') {
                    return Err(
                        "Replacement text exceeds its bound or contains unsupported NUL.".into(),
                    );
                }
                path
            }
        };
        if path.len() > 512 || path.contains('\0') {
            return Err("Trial path draft exceeds its bound or contains unsupported NUL.".into());
        }
        Ok(())
    }
    pub fn owned(&self, w: &core::World) -> Result<(), String> {
        w.validate().map_err(|e| e.detail)?;
        let i = &self.initial;
        if w.server != i.server
            || w.guest != i.guest
            || w.configuration_path != i.configuration_path
            || w.files.users != i.files.users
            || w.files.user != i.files.user
            || w.files.home != i.files.home
            || w.files.nodes.keys().ne(i.files.nodes.keys())
            || w.files
                .nodes
                .iter()
                .any(|(p, n)| n.directory != i.files.nodes[p].directory)
        {
            return Err("Keep the supplied server, Unix identities and file/directory inventory. This unit edits existing files, modes and SMB accounts.".into());
        }
        for a in &i.accounts {
            if !self.editable_accounts.iter().any(|n| n.id == a.user)
                && !w.accounts.iter().any(|b| b == a)
            {
                return Err(format!(
                    "Keep the supplied {} SMB account unchanged.",
                    a.user
                ));
            }
        }
        if w.accounts.iter().any(|a| {
            !self.editable_accounts.iter().any(|n| n.id == a.user)
                && !i.accounts.iter().any(|b| b == a)
        }) {
            return Err("Configure only the named editable SMB accounts.".into());
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self.evaluate(&self.initial).iter().all(|c| c.passed) {
            return Err("The initial task already passes; provide meaningful repair work.".into());
        }
        if !self.evaluate(&self.reference).iter().all(|c| c.passed) {
            return Err(
                "The authored reference cannot meet every current-state and client-outcome goal."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn run(&self, w: &mut core::World, r: &Request) -> Result<Observed, String> {
        self.owned(w)?;
        self.request_shape(r)?;
        let client = self.clients.iter().find(|c| c.id == r.client).unwrap();
        let result = match w.connect(&r.share, &client.credential) {
            Err(e) => Err(e),
            Ok(connection) => match &r.operation {
                Operation::Read { path } => w.read_record(&connection, path).map(Outcome::Read),
                Operation::Replace { path, text } => {
                    w.replace(&connection, path, text).map(Outcome::Replaced)
                }
            },
        };
        self.owned(w)?;
        Ok(Observed { result })
    }
    fn evaluate(&self, w: &core::World) -> Vec<Check> {
        if let Err(e) = self.owned(w) {
            return vec![check(false, "Supplied workspace", e)];
        }
        let saved = w.check_saved();
        let applied = config::parse(&w.published);
        let mut checks = vec![
            check(
                saved.is_ok(),
                "Saved configuration",
                saved
                    .as_ref()
                    .map(|_| "Supported syntax; client trials still determine access.".into())
                    .unwrap_or_else(|e| e.detail.clone()),
            ),
            check(
                saved.as_ref().ok() == applied.as_ref().ok(),
                "Saved and applied settings",
                "The saved configuration must have the same supported policy as the applied settings; comments and equivalent aliases do not change policy.",
            ),
        ];
        let required = self
            .shares
            .iter()
            .map(|s| s.id.as_str())
            .collect::<BTreeSet<_>>();
        checks.push(check(
            applied.as_ref().is_ok_and(|c| {
                c.shares.keys().map(String::as_str).collect::<BTreeSet<_>>() == required
            }),
            "Published shares",
            "Publish exactly the required named shares; adding a different share is not a repair.",
        ));
        for a in &self.accounts {
            let actual = w.accounts.iter().find(|v| v.user == a.user);
            checks.push(check(
                actual.is_some_and(|v| v.enabled == a.enabled),
                format!("SMB account {}", a.user),
                format!(
                    "Required: {}. Actual: {}.",
                    if a.enabled { "Enabled" } else { "Disabled" },
                    actual.map_or("Not provisioned", |v| if v.enabled {
                        "Enabled"
                    } else {
                        "Disabled"
                    })
                ),
            ));
        }
        for PolicyGoal::ReadOnly { share } in &self.policy {
            let passed = applied.as_ref().is_ok_and(|c| {
                c.shares
                    .get(share)
                    .is_some_and(|s| s.read_only && s.write_list.is_empty())
            });
            checks.push(check(passed,format!("{share}: default read-only policy"),"Keep this share read-only by default with no write-list exceptions. Equivalent writable/writeable settings are supported."));
        }
        for r in &self.filesystem {
            let (passed, detail) = r.check(&w.files);
            checks.push(check(passed, "Unix files and permissions", detail));
        }
        for c in &self.cases {
            let mut trial = w.clone();
            for input in &c.inputs {
                trial
                    .files
                    .files
                    .insert(input.path.clone(), input.text.clone());
            }
            let observed = self.run(&mut trial, &c.request);
            let (passed, detail) = match observed {
                Err(e) => (false, e),
                Ok(o) => match (&c.expected, o.result) {
                    (Expected::Read { path, text }, Ok(Outcome::Read(actual))) => (
                        actual.path == *path && actual.text == *text,
                        format!(
                            "Expected file: {path}; output: {text:?}\nActual file: {}; output: {:?}",
                            actual.path, actual.text
                        ),
                    ),
                    (Expected::Replaced { path, text }, Ok(Outcome::Replaced(actual))) => (
                        actual.path == *path
                            && actual.bytes == text.len()
                            && trial.files.files.get(path) == Some(text),
                        format!(
                            "Actual write: {} ({} bytes).\nExpected file: {path}; contents: {text:?}. Actual contents: {:?}.",
                            actual.path,
                            actual.bytes,
                            trial.files.files.get(path)
                        ),
                    ),
                    (Expected::Denied { layers }, Err(e)) => (
                        layers.iter().any(|l| l.matches(e.kind)),
                        format!(
                            "Expected denial in {layers:?}. Actual {:?}: {}",
                            e.kind, e.detail
                        ),
                    ),
                    (Expected::Denied { .. }, Ok(actual)) => (
                        false,
                        format!("This operation must be denied, but it succeeded: {actual:?}"),
                    ),
                    (_, Err(e)) => (
                        false,
                        format!(
                            "This operation must succeed. Actual {:?}: {}",
                            e.kind, e.detail
                        ),
                    ),
                    (_, Ok(actual)) => (
                        false,
                        format!("A different operation outcome was returned: {actual:?}"),
                    ),
                },
            };
            checks.push(check(passed, &c.label, detail));
        }
        checks
    }
    pub fn checks(&self, w: &core::World) -> Vec<Check> {
        if let Err(e) = self.validate() {
            return vec![check(false, "Scenario data", e)];
        }
        self.evaluate(w)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
fn check(passed: bool, label: impl Into<String>, detail: impl Into<String>) -> Check {
    Check {
        passed,
        label: label.into(),
        detail: detail.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub world: core::World,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub editor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub terminal: String,
    pub request: Request,
    pub checked: bool,
    #[serde(skip)]
    pub last_trial: Option<Observed>,
    #[serde(skip)]
    pub output: Vec<String>,
}
impl Attempt {
    pub fn record(&mut self, mut text: String) {
        const NOTE: &str = "\n[History preview shortened; rerun the command for full output.]";
        if text.len() > 32768 {
            let mut end = 32768 - NOTE.len();
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            text.push_str(NOTE);
        }
        while !self.output.is_empty()
            && (self.output.len() >= 32
                || self.output.iter().map(String::len).sum::<usize>() + text.len() > 32768)
        {
            self.output.remove(0);
        }
        self.output.push(text);
    }
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            world: t.initial.clone(),
            editor: t.initial.saved_config().map_err(|e| e.detail)?.into(),
            terminal: String::new(),
            request: t.trial.clone(),
            checked: false,
            last_trial: None,
            output: vec![],
        })
    }
    pub fn validate(&self, t: &Task) -> Result<(), String> {
        t.owned(&self.world)?;
        t.request_draft(&self.request)?;
        if self.editor.len() > 16384
            || self.terminal.len() > 2048
            || self.output.len() > 32
            || self.output.iter().map(String::len).sum::<usize>() > 32768
        {
            return Err("Saved editor/input or transient output exceeds its bound.".into());
        }
        Ok(())
    }
    pub fn changed(&mut self) {
        self.checked = false;
        self.last_trial = None;
    }
    pub fn edit(&mut self, t: &Task, text: String) -> Result<(), String> {
        self.validate(t)?;
        if text.len() > 16384 {
            return Err("Configuration draft exceeds16KiB.".into());
        }
        if text != self.editor {
            self.editor = text;
            self.changed();
        }
        Ok(())
    }
    pub fn save(&mut self, t: &Task) -> Result<(), String> {
        self.validate(t)?;
        let before = self.world.clone();
        self.world.save(self.editor.clone()).map_err(|e| e.detail)?;
        if before != self.world {
            self.changed();
        }
        Ok(())
    }
    pub fn publish(&mut self, t: &Task) -> Result<(), String> {
        self.validate(t)?;
        let before = self.world.clone();
        self.world.publish().map_err(|e| e.detail)?;
        if before != self.world {
            self.changed();
        }
        Ok(())
    }
    pub fn account(&mut self, t: &Task, user: &str, enabled: bool) -> Result<(), String> {
        self.validate(t)?;
        if !t.editable_accounts.iter().any(|n| n.id == user) {
            return Err("Only the task's named account controls are editable.".into());
        }
        let before = self.world.clone();
        self.world
            .set_account(user, enabled)
            .map_err(|e| e.detail)?;
        if before != self.world {
            self.changed();
        }
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.last_trial = None;
        self.validate(t)?;
        let before = self.world.clone();
        let observed = t.run(&mut self.world, &self.request)?;
        if self.world != before {
            self.changed();
        }
        self.last_trial = Some(observed);
        Ok(())
    }
    pub fn command(&mut self, t: &Task, input: &str) -> Result<String, String> {
        self.validate(t)?;
        if input.len() > 2048
            || !matches!(
                input.split_whitespace().next(),
                Some("ls" | "id" | "cat" | "pwd" | "cd" | "chmod" | "chgrp")
            )
        {
            return Err("This simulated filesystem terminal supports ls, id, cat, pwd, cd, chmod and chgrp only. Use id -Gn USER to inspect groups. Use the configuration editor and supplied account/client controls for Samba; no host command is executed.".into());
        }
        let mut next = self.world.clone();
        let out = next.files.execute(input)?;
        t.owned(&next)?;
        if next != self.world {
            self.world = next;
            self.changed();
        }
        Ok(out)
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.validate(t)?;
        t.validate()?;
        self.checked = true;
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        let mut c = t.checks(&self.world);
        c.insert(
            0,
            check(
                self.world.saved_config().is_ok_and(|s| s == self.editor),
                "Editor draft saved",
                "Save the current editor draft before checking the full assignment.",
            ),
        );
        c
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checked && self.validate(t).is_ok() && self.checks(t).iter().all(|c| c.passed)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema_version: u32,
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub assignment: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema_version: u32,
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.version == 0
            || !id(&self.id)
            || self.assignment.id == self.example.id
            || self.assignment.initial.server == self.example.initial.server
        {
            return Err(
                "Use version1, a named lesson and distinct assignment/example server identities."
                    .into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()
    }
    pub fn new_session(&self) -> Result<Session, String> {
        self.validate()?;
        Ok(Session {
            schema_version: 1,
            version: self.version,
            id: self.id.clone(),
            view: View::Introduction,
            assignment: Attempt::new(&self.assignment)?,
            practice: Attempt::new(&self.example)?,
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        if s.schema_version != 1
            || s.version != self.version
            || s.id != self.id
            || serde_json::to_vec(s).map_err(|e| e.to_string())?.len() > 512 * 1024
        {
            return Err(
                "Saved share work belongs to a different lesson/version or exceeds its bound."
                    .into(),
            );
        }
        s.assignment.validate(&self.assignment)?;
        s.practice.validate(&self.example)
    }
}
