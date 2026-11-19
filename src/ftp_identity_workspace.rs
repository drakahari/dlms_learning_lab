//! Replay-validated FTP configuration/client work. Sessions are reconstructed,
//! never imported as authority. Both terminals operate only on virtual state.
use crate::{
    ftp_identity as core, ftp_identity_config as config,
    ftp_identity_task::{Check, Task},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, String>;
const OUTPUT: usize = 64 * 1024;
fn within<T: Serialize>(x: &T, n: usize) -> bool {
    serde_json::to_vec(x).is_ok_and(|b| b.len() <= n)
}
fn local_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 120
        && s != "."
        && s != ".."
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn log(s: &mut String, text: &str) {
    s.push_str(text);
    if s.len() > OUTPUT {
        let mut cut = s.len() - OUTPUT;
        while !s.is_char_boundary(cut) {
            cut += 1;
        }
        s.drain(..cut);
    }
}
pub fn clients(t: &Task) -> Vec<core::Credential> {
    let mut out = vec![];
    for c in &t.cases {
        if !out.contains(&c.credential) {
            out.push(c.credential.clone());
        }
    }
    out
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    SaveConfig {
        #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
        text: String,
    },
    SaveFile {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
        text: String,
    },
    Admin {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    Connect {
        client: usize,
    },
    Client {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
}
struct Replay {
    world: core::World,
    connection: Option<core::Session>,
    local: BTreeMap<String, String>,
    admin_output: String,
    client_output: String,
}
fn admin(w: &mut core::World, input: &str) -> Result<String> {
    if input.len() > 512 {
        return Err("Command exceeds512bytes.".into());
    }
    let p = input.split_whitespace().collect::<Vec<_>>();
    // Project the separate typed configuration into readonly file inspections.
    // No host file exists, and its fixed metadata is not an editable permission task.
    if p.first() == Some(&"ls") {
        let mut view = w.files.clone();
        view.nodes.insert(
            "/etc/vsftpd.conf".into(),
            crate::engine::Node {
                directory: false,
                mode: 0o600,
                owner: view.user.clone(),
                group: view.users[&view.user][0].clone(),
            },
        );
        view.files
            .insert("/etc/vsftpd.conf".into(), w.saved.clone());
        return view.execute(input);
    }
    match p.as_slice() {
        ["systemctl",action,"vsftpd"|"vsftpd.service"]=>match *action {
            "restart"=>w.restart().map(|()|"Simulated FTP service restarted; reconnect the client.\n".into()).map_err(|e|e.detail),
            "start"=>if w.published.is_some() {Ok("Service is already running; use restart to apply saved configuration.\n".into())}else{w.restart().map(|()|"Simulated FTP service started.\n".into()).map_err(|e|e.detail)},
            "stop"=>w.stop().map(|()|"Simulated FTP service stopped.\n".into()).map_err(|e|e.detail),
            "status"=>Ok(format!("● vsftpd.service - supplied simulated FTP service\n   Active: {}\n",if w.published.is_some(){"active (running)"}else{"inactive (dead)"})),
            _=>Err("Supported systemctl actions here: status, start, restart, stop for vsftpd only.".into()),
        },
        ["cat","/etc/vsftpd.conf"]=>Ok(w.saved.clone()),
        ["pwd"]|["id"]|["id",_]|["id","-Gn",_] |["id","-nG",_]=>w.files.execute(input),
        ["chmod"|"chgrp",_,path] if config::canonical(path)=>w.files.execute(input),
        ["ls"]|["ls",_]|["ls",_,_]|["cat",_]=>w.files.execute(input),
        _=>Err("Unsupported command. Use ls, id, pwd, cat, owner-only chmod/chgrp, or systemctl status/start/restart/stop vsftpd. No host command or general shell runs.".into()),
    }
}
pub(crate) fn example_admin_supported(w: &core::World, input: &str) -> bool {
    admin(&mut w.clone(), input).is_ok()
}
fn client(r: &mut Replay, input: &str) -> Result<String> {
    let Some(s) = r.connection.as_mut() else {
        return Err("Connect using a supplied client identity first.".into());
    };
    let words = input.split_whitespace().collect::<Vec<_>>();
    match words.as_slice() {
        ["pwd"]=>Ok(format!("257 \"{}\" is the virtual working directory.\n",s.pwd())),
        ["cd",path]=>r.world.cd(s,path).map(|()|"250 Directory changed.\n".into()).map_err(|e|e.detail),
        ["ls"]|["ls",_]=>r.world.list(s,words.get(1).copied().unwrap_or(".")).map(|names|format!("{}\n226 Directory listing complete.\n",names.join("\n"))).map_err(|e|e.detail),
        ["get",remote]|["get",remote,_]=> {
            let local=words.get(2).copied().unwrap_or_else(||remote.rsplit('/').next().unwrap_or(""));
            if !local_name(local) {return Err("Choose a simple simulated client file name, for example report.txt; no host paths.".into());}
            let text=r.world.get(s,remote).map_err(|e|e.detail)?;
            let old=r.local.get(local).map_or(0,String::len);
            if (!r.local.contains_key(local)&&r.local.len()>=16)||r.local.values().map(String::len).sum::<usize>()-old+text.len()>32*1024 {return Err("Simulated client files exceed16entries or32KiB.".into());}
            let n=text.len();r.local.insert(local.into(),text);Ok(format!("226 Downloaded {n}bytes into simulated client file {local}.\n"))
        },
        ["put",local]|["put",local,_]=> {
            if !local_name(local) {return Err("put reads a supplied simulated client filename, not a host path.".into());}
            let remote=words.get(2).copied().unwrap_or(local);
            let text=r.local.get(*local).ok_or("No such simulated client file. Open Client files to inspect the supplied inputs.")?;
            r.world.put(s,remote,text).map_err(|e|e.detail)?;Ok(format!("226 Uploaded{}bytes from simulated client file {local}.\n",text.len()))
        },
        ["quit"]|["bye"]=>{r.connection=None;Ok("221 Connection closed.\n".into())},
        _=>Err("Supported FTP client commands: pwd, cd PATH, ls [PATH], get REMOTE [LOCAL], put LOCAL [REMOTE], quit. No shell, network or host files.".into()),
    }
}
impl Replay {
    fn new(t: &Task, inputs: &BTreeMap<String, String>) -> Self {
        Self {
            world: t.initial.clone(),
            connection: None,
            local: inputs.clone(),
            admin_output: String::new(),
            client_output: String::new(),
        }
    }
    fn event(&mut self, t: &Task, e: &Event) -> Result<()> {
        match e {
            Event::SaveConfig { text } => {
                self.world.save(text.clone()).map_err(|e| e.detail)?;
                log(
                    &mut self.admin_output,
                    "Saved /etc/vsftpd.conf; restart to apply it.\n",
                );
            }
            Event::SaveFile { path, text } => {
                if !t.editable_files.contains(path)
                    || text.len() > config::MAX_CONFIG
                    || text.contains('\0')
                {
                    return Err("Edit only a supplied bounded configuration/list file.".into());
                }
                let mut next = self.world.clone();
                next.files.files.insert(path.clone(), text.clone());
                next.validate().map_err(|e| e.detail)?;
                self.world = next;
                log(
                    &mut self.admin_output,
                    &format!("Saved virtual file {path}.\n"),
                );
            }
            Event::Admin { input } => {
                let generation = self.world.generation;
                let result = admin(&mut self.world, input);
                if self.world.generation != generation {
                    self.connection = None;
                }
                log(
                    &mut self.admin_output,
                    &format!(
                        "$ {input}\n{}",
                        result.unwrap_or_else(|e| format!("[Not successful] {e}\n"))
                    ),
                );
            }
            Event::Connect { client } => {
                let choices = clients(t);
                let proof = choices
                    .get(*client)
                    .ok_or("Choose a supplied client identity.")?;
                self.connection = None;
                match self.world.login(proof) {
                    Ok(s) => {
                        log(
                            &mut self.client_output,
                            &format!(
                                "230 Logged in as {}. Virtual directory: {}.\n",
                                s.user(),
                                s.pwd()
                            ),
                        );
                        self.connection = Some(s);
                    }
                    Err(e) => log(
                        &mut self.client_output,
                        &format!("530 Login refused: {}\n", e.detail),
                    ),
                }
            }
            Event::Client { input } => {
                if input.len() > 512 {
                    return Err("Client command exceeds512bytes.".into());
                }
                let result = client(self, input);
                log(
                    &mut self.client_output,
                    &format!(
                        "ftp> {input}\n{}",
                        result.unwrap_or_else(|e| format!("[Not successful] {e}\n"))
                    ),
                );
            }
        }
        t.bound(&self.world)?;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,96>")]
    pub history: Vec<Event>,
    pub local_files: BTreeMap<String, String>,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub admin_output: String,
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub client_output: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub admin_input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub client_input: String,
    /// Empty selects the separate saved vsftpd.conf string.
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub editor_path: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub editor: String,
    pub selected_client: usize,
    pub selected_case: usize,
    pub checked: bool,
    pub last_trial: Option<Vec<Check>>,
}
impl Scope {
    pub fn new(t: &Task, inputs: &BTreeMap<String, String>) -> Self {
        Self {
            world: t.initial.clone(),
            history: vec![],
            local_files: inputs.clone(),
            admin_output: String::new(),
            client_output: String::new(),
            admin_input: String::new(),
            client_input: String::new(),
            editor_path: String::new(),
            editor: t.initial.saved.clone(),
            selected_client: 0,
            selected_case: 0,
            checked: false,
            last_trial: None,
        }
    }
    pub fn shape(&self) -> Result<()> {
        self.world.validate().map_err(|e| e.detail)?;
        if !within(self, 768 * 1024)
            || !within(&self.history, 256 * 1024)
            || self.history.len() > 96
            || self.admin_output.len() > OUTPUT
            || self.client_output.len() > OUTPUT
            || self.admin_input.len() > 512
            || self.client_input.len() > 512
            || self.editor.len() > config::MAX_CONFIG
            || self.editor.contains('\0')
            || self.local_files.len() > 16
            || self
                .local_files
                .iter()
                .any(|(n, t)| !local_name(n) || t.len() > 4096 || t.contains('\0'))
            || self.local_files.values().map(String::len).sum::<usize>() > 32 * 1024
        {
            return Err(
                "Saved FTP workspace exceeds its bounded source context; prior work retained."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task, inputs: &BTreeMap<String, String>) -> Result<()> {
        self.shape()?;
        if self.selected_case >= t.cases.len()
            || self.selected_client >= clients(t).len()
            || (!self.editor_path.is_empty() && !t.editable_files.contains(&self.editor_path))
        {
            return Err(
                "Choose an existing source-owned client, transfer case and configuration file."
                    .into(),
            );
        }
        let mut r = Replay::new(t, inputs);
        for e in &self.history {
            r.event(t, e)?;
        }
        if self.world != r.world
            || self.local_files != r.local
            || self.admin_output != r.admin_output
            || self.client_output != r.client_output
        {
            return Err(
                "Saved FTP state differs from its supported edits; prior work retained.".into(),
            );
        }
        if self
            .last_trial
            .as_ref()
            .is_some_and(|c| *c != t.trial(&self.world, self.selected_case))
        {
            return Err("Saved FTP feedback is stale; prior work retained.".into());
        }
        Ok(())
    }
    pub fn saved_editor(&self) -> &str {
        if self.editor_path.is_empty() {
            &self.world.saved
        } else {
            self.world
                .files
                .files
                .get(&self.editor_path)
                .map(String::as_str)
                .unwrap_or("")
        }
    }
    pub fn checks(&self, t: &Task, inputs: &BTreeMap<String, String>) -> Vec<Check> {
        if let Err(e) = self.bound(t, inputs) {
            return vec![Check {
                passed: false,
                label: "Saved workspace".into(),
                detail: e,
            }];
        }
        let mut checks = vec![Check {
            passed: self.checked && self.editor == self.saved_editor(),
            label: "Current saved work".into(),
            detail: "Save the current editor and check the assignment after making changes.".into(),
        }];
        checks.extend(t.checks(&self.world));
        checks
    }
    pub fn apply(&mut self, t: &Task, inputs: &BTreeMap<String, String>, a: &Action) -> Result<()> {
        self.bound(t, inputs)?;
        let mut next = self.clone();
        match a {
            Action::AdminInput { text } => next.admin_input = text.clone(),
            Action::ClientInput { text } => next.client_input = text.clone(),
            Action::Edit { text } => {
                next.editor = text.clone();
                next.checked = false;
                next.last_trial = None;
            }
            Action::ChooseFile { path } => {
                if next.editor != next.saved_editor() {
                    return Err("Save this draft or explicitly reload it before switching configuration files.".into());
                }
                if !path.is_empty() && !t.editable_files.contains(path) {
                    return Err("Choose a supplied virtual configuration file.".into());
                }
                next.editor_path = path.clone();
                next.editor = next.saved_editor().into();
            }
            Action::Reload => next.editor = next.saved_editor().into(),
            Action::ChooseClient { client } => next.selected_client = *client,
            Action::ChooseCase { case } => {
                next.selected_case = *case;
                next.last_trial = None;
            }
            Action::Trial => next.last_trial = Some(t.trial(&next.world, next.selected_case)),
            Action::Check => next.checked = true,
            Action::Event { event } => {
                if next.history.len() >= 96 {
                    return Err("The attempt reached96recorded operations. Reset only this attempt to start again.".into());
                }
                let mut r = Replay::new(t, inputs);
                for e in &next.history {
                    r.event(t, e)?;
                }
                r.event(t, event)?;
                let changed = next.world != r.world;
                next.world = r.world;
                next.local_files = r.local;
                next.admin_output = r.admin_output;
                next.client_output = r.client_output;
                next.history.push(event.clone());
                if changed {
                    next.checked = false;
                    next.last_trial = None;
                }
                match event {
                    Event::Admin { .. } => next.admin_input.clear(),
                    Event::Client { .. } => next.client_input.clear(),
                    _ => {}
                }
            }
            Action::Save => {
                let e = if next.editor_path.is_empty() {
                    Event::SaveConfig {
                        text: next.editor.clone(),
                    }
                } else {
                    Event::SaveFile {
                        path: next.editor_path.clone(),
                        text: next.editor.clone(),
                    }
                };
                next.apply(t, inputs, &Action::Event { event: e })?;
            }
        }
        next.bound(t, inputs)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    AdminInput {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    ClientInput {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
        text: String,
    },
    ChooseFile {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        path: String,
    },
    Reload,
    Save,
    ChooseClient {
        client: usize,
    },
    ChooseCase {
        case: usize,
    },
    Trial,
    Check,
    Event {
        event: Event,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    pub assignment_inputs: BTreeMap<String, String>,
    pub example_inputs: BTreeMap<String, String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,32>")]
    pub worked: Vec<Action>,
}
fn inputs_ok(x: &BTreeMap<String, String>) -> bool {
    x.len() <= 8
        && x.iter()
            .all(|(n, t)| local_name(n) && t.len() <= 4096 && !t.contains('\0'))
        && x.values().map(String::len).sum::<usize>() <= 16 * 1024
}
impl Definition {
    pub fn bound(&self, s: &Session) -> Result<()> {
        if s.source_id != self.source_id {
            return Err(
                "Saved FTP work belongs to a different lesson; prior work retained.".into(),
            );
        }
        s.shape()?;
        s.assignment
            .bound(&self.assignment, &self.assignment_inputs)?;
        s.example.bound(&self.example, &self.example_inputs)
    }
    pub fn validate(&self) -> Result<()> {
        self.assignment.validate()?;
        self.example.validate()?;
        if self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.assignment.id == self.example.id
            || self.assignment.initial.server == self.example.initial.server
            || !inputs_ok(&self.assignment_inputs)
            || !inputs_ok(&self.example_inputs)
            || !within(self, 768 * 1024)
            || self.worked.is_empty()
            || self.worked.len() > 32
        {
            return Err("Use separate FTP example/assignment identities and bounded client inputs/worked actions.".into());
        }
        for t in [&self.assignment, &self.example] {
            if t.checks(&t.initial).iter().all(|c| c.passed) {
                return Err("Both tasks need meaningful repairs before checking.".into());
            }
        }
        let mut s = Scope::new(&self.example, &self.example_inputs);
        for a in &self.worked {
            s.apply(&self.example, &self.example_inputs, a)?;
        }
        if !s
            .checks(&self.example, &self.example_inputs)
            .iter()
            .all(|c| c.passed)
        {
            return Err(
                "The worked example must save, apply and check every required behavior.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.source_id.is_empty() || self.source_id.len() > 64 || !within(self, 1536 * 1024) {
            return Err("Invalid saved FTP source identity or size.".into());
        }
        self.assignment.shape()?;
        self.example.shape()
    }
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment, &d.assignment_inputs),
            example: Scope::new(&d.example, &d.example_inputs),
        })
    }
    pub fn bound(&self, d: &Definition) -> Result<()> {
        d.bound(self)
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.example.apply(&d.example, &d.example_inputs, a)?;
        } else {
            next.assignment
                .apply(&d.assignment, &d.assignment_inputs, a)?;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.example = Scope::new(&d.example, &d.example_inputs);
        } else {
            self.assignment = Scope::new(&d.assignment, &d.assignment_inputs);
        }
        Ok(())
    }
    pub fn reference(&mut self, d: &Definition, input: &str) -> Result<String> {
        let raw = input
            .strip_prefix("ftp-identity-workspace ")
            .ok_or("Use the native source-bound FTP workspace; no host command is executed.")?;
        if raw.len() > 16384 {
            return Err("FTP reference action exceeds16KiB.".into());
        }
        let a: Action = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        self.action(d, &a, false)?;
        Ok("Applied a source-bound simulated FTP action.".into())
    }
}
