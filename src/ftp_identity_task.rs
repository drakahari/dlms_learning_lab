//! Fresh, deterministic behavior trials over the FTP identity model.
//! A refused useful transfer never passes as a successful confinement check.
use crate::{ftp_identity as core, ftp_identity_config as config};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Denial {
    Authentication,
    Policy,
    Permission,
    Missing,
}
impl Denial {
    fn matches(self, k: core::Kind) -> bool {
        matches!(
            (self, k),
            (Self::Authentication, core::Kind::Authentication)
                | (Self::Policy, core::Kind::Policy)
                | (Self::Permission, core::Kind::Permission)
                | (Self::Missing, core::Kind::Missing)
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Pwd,
    Cd {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    List {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    Get {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    Put {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Directory {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    Names {
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,256>")]
        names: Vec<String>,
    },
    Download {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    Uploaded {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        server_path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
        /// When required by the assignment, grade resulting mode, not umask text.
        mode: Option<u16>,
    },
    Denied {
        layer: Denial,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    pub expected: Expected,
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
    pub credential: core::Credential,
    pub denied_login: Option<Denial>,
    pub confined: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Input,8>")]
    pub inputs: Vec<Input>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Step,8>")]
    pub steps: Vec<Step>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
fn check(ok: bool, label: impl Into<String>, detail: impl Into<String>) -> Check {
    Check {
        passed: ok,
        label: label.into(),
        detail: detail.into(),
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: core::World,
    /// Only these existing virtual text files are editable by the configuration pane.
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,8>")]
    pub editable_files: Vec<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,24>")]
    pub cases: Vec<Case>,
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.detail)?;
        if !id(&self.id)
            || !(2..=24).contains(&self.cases.len())
            || self.editable_files.len() > 8
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 256 * 1024
        {
            return Err(
                "Use a stable FTP task,2–24fresh behavior cases and bounded virtual inputs.".into(),
            );
        }
        let mut ids = BTreeSet::new();
        if self.editable_files.iter().collect::<BTreeSet<_>>().len() != self.editable_files.len() {
            return Err("Declare each editable virtual file once.".into());
        }
        if self
            .editable_files
            .iter()
            .any(|p| !config::canonical(p) || !self.initial.files.files.contains_key(p))
        {
            return Err(
                "Editable FTP files must be declared existing canonical virtual files.".into(),
            );
        }
        let mut useful = false;
        let mut denied = false;
        for c in &self.cases {
            if !id(&c.id)
                || !ids.insert(&c.id)
                || c.label.is_empty()
                || c.label.len() > 120
                || c.steps.len() > 8
                || c.inputs.len() > 8
                || (c.denied_login.is_some() && !c.steps.is_empty())
                || matches!(c.denied_login, Some(Denial::Missing))
                || (c.denied_login.is_none() && c.steps.is_empty())
            {
                return Err("Each case needs a distinct identifier and either an explicit login denial or1–8operations after login.".into());
            }
            denied |= c.denied_login.is_some();
            let mut paths = BTreeSet::new();
            let mut fixture = self.initial.clone();
            for input in &c.inputs {
                if !config::canonical(&input.path)
                    || !paths.insert(&input.path)
                    || !fixture.files.files.contains_key(&input.path)
                    || self.editable_files.contains(&input.path)
                    || !self
                        .initial
                        .accounts
                        .iter()
                        .any(|u| input.path.starts_with(&(u.home.clone() + "/")))
                    || input.text.len() > core::MAX_TRANSFER
                    || input.text.contains('\0')
                {
                    return Err("Sample inputs must replace distinct existing regular files with bounded text.".into());
                }
                fixture
                    .files
                    .files
                    .insert(input.path.clone(), input.text.clone());
            }
            fixture.validate().map_err(|e| e.detail)?;
            for s in &c.steps {
                let valid = match (&s.operation, &s.expected) {
                    (Operation::Pwd | Operation::Cd { .. }, Expected::Directory { path }) => {
                        config::canonical(path)
                    }
                    (Operation::List { .. }, Expected::Names { names }) => {
                        names.len() <= 256
                            && names
                                .iter()
                                .all(|n| !n.is_empty() && n.len() <= 512 && !n.contains('/'))
                    }
                    (Operation::Get { .. }, Expected::Download { text }) => {
                        useful = true;
                        text.len() <= core::MAX_TRANSFER && !text.contains('\0')
                    }
                    (
                        Operation::Put { text: input, .. },
                        Expected::Uploaded {
                            server_path,
                            text,
                            mode,
                        },
                    ) => {
                        useful = true;
                        config::canonical(server_path)
                            && text.len() <= core::MAX_TRANSFER
                            && !text.contains('\0')
                            && text == input
                            && mode.is_none_or(|m| m <= 0o777)
                    }
                    (Operation::Pwd, Expected::Denied { .. }) => false,
                    (_, Expected::Denied { .. }) => {
                        denied = true;
                        true
                    }
                    _ => false,
                };
                let path = match &s.operation {
                    Operation::Pwd => None,
                    Operation::Cd { path }
                    | Operation::List { path }
                    | Operation::Get { path }
                    | Operation::Put { path, .. } => Some(path),
                };
                if !valid
                    || path.is_some_and(|p| p.is_empty() || p.len() > 512)
                    || matches!(&s.operation,Operation::Put{text,..}if text.len()>core::MAX_TRANSFER||text.contains('\0'))
                {
                    return Err("Match each bounded operation to its expected result or a specific denial layer.".into());
                }
            }
        }
        if !useful || !denied {
            return Err("Require useful transfers and explicit denied behavior; an inaccessible server cannot pass.".into());
        }
        Ok(())
    }
    pub fn bound(&self, w: &core::World) -> Result<(), String> {
        w.validate().map_err(|e| e.detail)?;
        let a = &self.initial;
        if (w.version, &w.server, &w.accounts, &w.anonymous_home)
            != (a.version, &a.server, &a.accounts, &a.anonymous_home)
            || w.files.users != a.files.users
            || w.files.user != a.files.user
            || w.files.home != a.files.home
            || w.files.command_location != a.files.command_location
        {
            return Err("Supplied server, accounts, homes and Unix identities cannot change in this FTP task.".into());
        }
        for (p, n) in &a.files.nodes {
            if !w
                .files
                .nodes
                .get(p)
                .is_some_and(|x| x.directory == n.directory)
            {
                return Err("Preserve the supplied file/directory structure.".into());
            }
            let local_home = a
                .accounts
                .iter()
                .any(|u| p == &u.home || p.starts_with(&(u.home.clone() + "/")));
            if !local_home && w.files.nodes.get(p) != Some(n) {
                return Err(
                    "Preserve permissions and ownership outside the supplied local homes.".into(),
                );
            }
        }
        for (p, t) in &a.files.files {
            let transfer_file = a
                .accounts
                .iter()
                .any(|u| p.starts_with(&(u.home.clone() + "/")));
            if !self.editable_files.contains(p) && !transfer_file && w.files.files.get(p) != Some(t)
            {
                return Err(format!("Preserve the supplied outside file {p}; transfer tests provide their own inputs."));
            }
        }
        // Actual exploratory uploads may add regular files only below supplied homes.
        for (p, n) in &w.files.nodes {
            if !a.files.nodes.contains_key(p)
                && (n.directory
                    || !a
                        .accounts
                        .iter()
                        .any(|u| p.starts_with(&(u.home.clone() + "/"))))
            {
                return Err("New exploratory files must stay below supplied local homes.".into());
            }
        }
        Ok(())
    }
    pub fn trial(&self, w: &core::World, index: usize) -> Vec<Check> {
        let Some(c) = self.cases.get(index) else {
            return vec![check(
                false,
                "Traffic case",
                "Choose an existing FTP behavior case.",
            )];
        };
        if let Err(e) = self.bound(w) {
            return vec![check(false, &c.label, e)];
        }
        let mut fresh = w.clone();
        // Each behavior case gets its own source-owned transfer files, while
        // retaining the learner's configuration and permission changes.
        fresh
            .files
            .nodes
            .retain(|p, _| self.initial.files.nodes.contains_key(p));
        fresh
            .files
            .files
            .retain(|p, _| self.initial.files.files.contains_key(p));
        fresh
            .files
            .modified
            .retain(|p, _| self.initial.files.nodes.contains_key(p));
        for (p, text) in &self.initial.files.files {
            if !self.editable_files.contains(p) {
                fresh.files.files.insert(p.clone(), text.clone());
            }
        }
        for input in &c.inputs {
            fresh
                .files
                .files
                .insert(input.path.clone(), input.text.clone());
        }
        let mut s = match fresh.login(&c.credential) {
            Err(e) => {
                return vec![check(
                    c.denied_login.is_some_and(|d| d.matches(e.kind)),
                    &c.label,
                    format!("Login refused: {}", e.detail),
                )]
            }
            Ok(s) => {
                if c.denied_login.is_some() {
                    return vec![check(false,&c.label,"Login succeeded, but this identity must be refused. Check login policy; making every directory unreadable is not equivalent.")];
                }
                s
            }
        };
        let mut rows = vec![check(
            s.confined() == c.confined,
            format!("{} · login root", c.label),
            if s.confined() == c.confined {
                "Login uses the required root boundary."
            } else {
                "Login succeeded with the wrong root boundary. Check chroot_local_user and the enabled chroot list's inverted meaning."
            },
        )];
        for (i, step) in c.steps.iter().enumerate() {
            let before = fresh.clone();
            let actual = match &step.operation {
                Operation::Pwd => Ok(Expected::Directory {
                    path: s.pwd().into(),
                }),
                Operation::Cd { path } => fresh.cd(&mut s, path).map(|()| Expected::Directory {
                    path: s.pwd().into(),
                }),
                Operation::List { path } => {
                    fresh.list(&s, path).map(|names| Expected::Names { names })
                }
                Operation::Get { path } => {
                    fresh.get(&s, path).map(|text| Expected::Download { text })
                }
                Operation::Put { path, text } => fresh.put(&s, path, text).map(|()| {
                    let server_path = step_path(&before, &s, path);
                    let mode = fresh.files.nodes.get(&server_path).map(|n| n.mode);
                    Expected::Uploaded {
                        server_path,
                        text: text.clone(),
                        mode,
                    }
                }),
            };
            let (ok, detail) = match actual {
                Err(e) => (
                    matches!(step.expected,Expected::Denied{layer}if layer.matches(e.kind))
                        && fresh == before,
                    format!("Operation refused: {}", e.detail),
                ),
                Ok(result) => {
                    let equal = match (&result, &step.expected) {
                        (
                            Expected::Uploaded {
                                server_path: a,
                                text: b,
                                mode,
                            },
                            Expected::Uploaded {
                                server_path: c,
                                text: d,
                                mode: wanted,
                            },
                        ) => a == c && b == d && wanted.is_none_or(|m| *mode == Some(m)),
                        _ => result == step.expected,
                    };
                    let effect = if let Expected::Uploaded {
                        server_path, text, ..
                    } = &result
                    {
                        let mut expected = before.clone();
                        expected
                            .files
                            .files
                            .insert(server_path.clone(), text.clone());
                        if !before.files.nodes.contains_key(server_path) {
                            let mask = config::parse(before.published.as_deref().unwrap_or(""))
                                .map_or(0o777, |c| c.local_umask);
                            expected.files.nodes.insert(
                                server_path.clone(),
                                crate::engine::Node {
                                    directory: false,
                                    mode: 0o666 & !mask,
                                    owner: s.user().into(),
                                    group: before.files.users[s.user()][0].clone(),
                                },
                            );
                        }
                        fresh == expected
                    } else {
                        fresh == before
                    };
                    (
                        equal && effect,
                        if equal && effect {
                            "Actual result matches the required operation; unrelated files are unchanged.".into()
                        } else {
                            format!("Expected {}. Actual {}. Check root mapping, service policy and Unix file permissions.",describe(&step.expected),describe(&result))
                        },
                    )
                }
            };
            rows.push(check(ok, format!("{} · action{}", c.label, i + 1), detail));
            if !ok {
                break;
            }
        }
        rows
    }
    pub fn checks(&self, w: &core::World) -> Vec<Check> {
        if let Err(e) = self.validate().and_then(|()| self.bound(w)) {
            return vec![check(false, "Workspace", e)];
        }
        let published = w.published.as_deref().and_then(|s| config::parse(s).ok());
        let saved = config::parse(&w.saved).ok();
        let mut rows=vec![check(published.is_some()&&saved==published,"Saved and running configuration","Save valid configuration and restart the simulated service; equivalent formatting and comments are accepted.")];
        for i in 0..self.cases.len() {
            rows.extend(self.trial(w, i));
        }
        rows
    }
}
// Paths here were already component-walked by a successful core::put. This only
// reports the resulting canonical file key; it never authorizes an operation.
fn step_path(w: &core::World, s: &core::Session, p: &str) -> String {
    let mut parts: Vec<_> = if p.starts_with('/') {
        vec![]
    } else {
        s.pwd().split('/').filter(|x| !x.is_empty()).collect()
    };
    for x in p.split('/').filter(|x| !x.is_empty()) {
        match x {
            "." => {}
            ".." => {
                parts.pop();
            }
            _ => parts.push(x),
        }
    }
    let relative = format!("/{}", parts.join("/"));
    if !s.confined() {
        relative
    } else {
        let home = w
            .accounts
            .iter()
            .find(|a| a.user == s.user())
            .map_or(w.anonymous_home.as_str(), |a| a.home.as_str());
        if relative == "/" {
            home.into()
        } else {
            format!("{home}{relative}")
        }
    }
}
fn describe(v: &Expected) -> String {
    match v {
        Expected::Directory { path } => format!("directory {path}"),
        Expected::Names { names } => format!("names {}", names.join(", ")),
        Expected::Download { text } => format!("download {:?}", text),
        Expected::Uploaded {
            server_path,
            text,
            mode,
        } => {
            format!(
                "upload {} bytes to {server_path}{}",
                text.len(),
                mode.map_or(String::new(), |m| format!(" with mode {m:03o}"))
            )
        }
        Expected::Denied { layer } => format!(
            "a refusal at the {} layer",
            match layer {
                Denial::Authentication => "login",
                Denial::Policy => "service policy",
                Denial::Permission => "Unix permission",
                Denial::Missing => "virtual path",
            }
        ),
    }
}
