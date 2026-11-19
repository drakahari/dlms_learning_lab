//! Source-bound configuration and protocol practice. Practice and assignment own
//! separate servers, stores, sessions, drafts and output; no host execution.
use crate::{
    mailbox_access::{self as access, Client, Facts, Flag, Issue, Runtime},
    mailbox_config::{Config, MAX_LINES, MAX_TEXT, Protocol},
    mailbox_protocol::{self as protocol, Code, Phase, Session as Connection},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_TASK: usize = 384 * 1024;
pub const MAX_SCOPE: usize = 512 * 1024;
pub const MAX_SESSION: usize = 1024 * 1024;
pub const MAX_ACTION: usize = 16 * 1024;
pub const MAX_CONSOLE: usize = 128 * 1024;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Requirement {
    Access {
        profile: String,
        user: String,
        upgrade: bool,
        allow: bool,
    },
    Message {
        user: String,
        mailbox: String,
        uid: u32,
        present: bool,
        seen: Option<bool>,
        deleted: Option<bool>,
    },
    Enabled,
    PopCommitted {
        user: String,
        uids: BTreeSet<u32>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub editable_config: bool,
    pub facts: Facts,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub saved: String,
    pub active: Option<String>,
    pub enabled: bool,
    pub clients: BTreeMap<String, Client>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Requirement,32>")]
    pub requirements: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !access::within(self, MAX_TASK)
            || self.clients.is_empty()
            || self.clients.len() > 8
            || self.requirements.is_empty()
            || self.requirements.len() > 32
        {
            return Err("Mail-access tasks need bounded fixtures,1to8client profiles and1to32explicit requirements.".into());
        }
        Runtime::initial(
            &self.facts,
            &self.saved,
            self.active.as_deref(),
            self.enabled,
        )
        .map_err(|e| e.message)?;
        for (name, client) in &self.clients {
            if !id(name) {
                return Err("Use stable client-profile identifiers.".into());
            }
            client.validate().map_err(|e| e.message)?;
        }
        let mut keys = BTreeSet::new();
        for req in &self.requirements {
            let key = serde_json::to_string(req).map_err(|e| e.to_string())?;
            if !keys.insert(key) {
                return Err("Mail-access requirements must not repeat the same case.".into());
            }
            match req {
                Requirement::Access {profile,user,upgrade,allow} => {
                    let c=self.clients.get(profile).ok_or("Access checks must name a supplied client profile.")?;
                    if !self.facts.accounts.get(user).is_some_and(|a| a.enabled) || *upgrade && c.implicit_tls || !allow && (c.implicit_tls || *upgrade) {
                        return Err("Access cases need enabled sample accounts; upgrades start plain and rejection cases test remote unprotected login.".into());
                    }
                }
                Requirement::Message {user,mailbox,uid,present,seen,deleted} => {
                    if !self.facts.mailboxes.get(user).and_then(|b| b.get(mailbox)).is_some_and(|b| b.messages.iter().any(|m| m.uid==*uid)) || !present && (seen.is_some() || deleted.is_some()) {
                        return Err("Message outcomes must name supplied UIDs; absent messages cannot have required flags.".into());
                    }
                }
                Requirement::Enabled => {}
                Requirement::PopCommitted {user,uids}=>{
                    if uids.is_empty()||uids.len()>16||uids.iter().any(|uid|!self.facts.mailboxes.get(user).is_some_and(|b|b["INBOX"].messages.iter().any(|m|m.uid==*uid))||!self.requirements.iter().any(|r|matches!(r,Requirement::Message {user:u,mailbox,uid:id,present:false,..}if u==user&&mailbox=="INBOX"&&id==uid))){return Err("POP commit requirements must name1to16supplied messages also required absent from that account's INBOX.".into());}
                }
            }
        }
        let mut targets = BTreeSet::new();
        for req in &self.requirements {
            if let Requirement::Message {
                user, mailbox, uid, ..
            } = req
                && !targets.insert((user, mailbox, uid))
            {
                return Err(
                    "Describe each message target once, including all its required flags.".into(),
                );
            }
        }
        Ok(())
    }
    fn access_check(
        &self,
        runtime: &Runtime,
        profile: &str,
        user: &str,
        upgrade: bool,
        allow: bool,
    ) -> Check {
        let client = &self.clients[profile];
        let observed = (|| {
            let mut server = runtime.clone();
            let (mut session, _) =
                Connection::open(&server, &self.facts, client).map_err(|e| (e.issue, e.message))?;
            if upgrade {
                let out = session.command(
                    &mut server,
                    &self.facts,
                    if client.protocol == Protocol::Imap {
                        "t1 STARTTLS"
                    } else {
                        "STLS"
                    },
                );
                if out.code != Code::Ok {
                    return Err((
                        out.issue.unwrap_or(Issue::Tls),
                        out.client_note.unwrap_or(out.text),
                    ));
                }
            }
            let password = &self.facts.accounts[user].sample_password;
            let login = if client.protocol == Protocol::Imap {
                format!("t2 LOGIN {user} {}", imap_quote(password))
            } else {
                let out = session.command(&mut server, &self.facts, &format!("USER {user}"));
                if out.code != Code::Ok {
                    return Err((out.issue.unwrap_or(Issue::Authentication), out.text));
                }
                format!("PASS {password}")
            };
            let out = session.command(&mut server, &self.facts, &login);
            if out.code != Code::Ok {
                return Err((out.issue.unwrap_or(Issue::Authentication), out.text));
            }
            if client.protocol == Protocol::Imap {
                let out = session.command(&mut server, &self.facts, "t3 EXAMINE INBOX");
                if out.code != Code::Ok {
                    return Err((out.issue.unwrap_or(Issue::Mailbox), out.text));
                }
                if !server
                    .mailbox(&self.facts, user, "INBOX")
                    .map_err(|e| (e.issue, e.message))?
                    .messages
                    .is_empty()
                {
                    let out = session.command(&mut server, &self.facts, "t4 FETCH 1 BODY.PEEK[]");
                    if out.code != Code::Ok {
                        return Err((out.issue.unwrap_or(Issue::Mailbox), out.text));
                    }
                }
            } else if !server
                .mailbox(&self.facts, user, "INBOX")
                .map_err(|e| (e.issue, e.message))?
                .messages
                .is_empty()
            {
                let out = session.command(&mut server, &self.facts, "RETR 1");
                if out.code != Code::Ok {
                    return Err((out.issue.unwrap_or(Issue::Mailbox), out.text));
                }
            }
            Ok(())
        })();
        let passed = if allow {
            observed.is_ok()
        } else {
            // A disabled port, wrong password or broken TLS is not evidence of
            // enforcing the required remote plaintext-authentication policy.
            matches!(&observed, Err((Issue::Authentication, _)))
                && runtime
                    .active_config()
                    .is_ok_and(|c| !c.permits_password(false, false))
        };
        let detail=match observed {
            Ok(()) if allow=>"The current server authenticated this account and opened/read its own supplied INBOX. Fresh checks leave stored mail unchanged.".into(),
            Ok(())=>"This unprotected remote connection accepted a password. Enforce the task's TLS policy instead of merely keeping the secure port available.".into(),
            Err((Issue::Authentication,_)) if passed=>"Remote login without TLS was rejected by the active password policy; the listener itself remained reachable.".into(),
            Err((kind,message))=>format!("Observed {kind:?}: {}",message.trim()),
        };
        Check {
            passed,
            label: format!(
                "{profile} · {user} · {}",
                if allow {
                    "open own mail"
                } else {
                    "reject unprotected login"
                }
            ),
            detail,
        }
    }
}
fn imap_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub runtime: Runtime,
    pub connection: Option<Connection>,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub input: String,
    pub client: String,
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub output: String,
    pub checked: bool,
}
impl Scope {
    pub fn new(task: &Task) -> Result<Self, String> {
        task.validate()?;
        Ok(Self {
            runtime: Runtime::initial(
                &task.facts,
                &task.saved,
                task.active.as_deref(),
                task.enabled,
            )
            .map_err(|e| e.message)?,
            connection: None,
            draft: task.saved.clone(),
            input: String::new(),
            client: task.clients.keys().next().unwrap().clone(),
            output: String::new(),
            checked: false,
        })
    }
    pub fn bound(&self, task: &Task) -> Result<(), String> {
        task.validate()?;
        if !access::within(self, MAX_SCOPE)
            || self.draft.len() > MAX_TEXT
            || self.draft.lines().count() > MAX_LINES
            || self.input.len() > protocol::MAX_COMMAND
            || self.output.len() > MAX_CONSOLE
            || !task.clients.contains_key(&self.client)
        {
            return Err(
                "Mail-access drafts, selection or output exceed the supplied workspace bounds."
                    .into(),
            );
        }
        self.runtime.validate(&task.facts).map_err(|e| e.message)?;
        if !task.editable_config
            && (self.draft != task.saved
                || self.runtime.saved != task.saved
                || self.runtime.active != task.active
                || self.runtime.enabled != task.enabled)
        {
            return Err("This protocol lesson preserves its supplied server configuration; only mail and connection state are editable.".into());
        }
        if let Some(session) = &self.connection {
            if !task.clients.values().any(|c| c == &session.link.client) {
                return Err(
                    "A restored connection must use a supplied virtual client profile.".into(),
                );
            }
            session
                .validate(&self.runtime, &task.facts)
                .map_err(|e| e.message)?;
        }
        Ok(())
    }
    pub fn checks(&self, task: &Task) -> Vec<Check> {
        if let Err(e) = self.bound(task) {
            return vec![Check {
                passed: false,
                label: "Read this saved work".into(),
                detail: e,
            }];
        }
        let saved = Config::parse(&self.runtime.saved);
        let active = self.runtime.active_config();
        let mut rows = vec![Check {
            passed: self.draft == self.runtime.saved,
            label: "Save the current editor".into(),
            detail: if self.draft == self.runtime.saved {
                "The editor matches the saved simulated configuration.".into()
            } else {
                "The editor has unsaved changes. Save them before checking; restart reads the saved file.".into()
            },
        }];
        rows.push(Check {passed:matches!((&saved,&active),(Ok(s),Ok(a))if s==a),label:"Run the current saved settings".into(),detail:match (&saved,&active) {(Err(e),_)=>e.to_string(),(_,Err(e))=>e.message.clone(),(Ok(s),Ok(a))if s!=a=>"Saved and running settings differ. Restart the simulated service to apply the saved configuration.".into(),_=>"Saved and running settings agree; equivalent supported spellings and setting order are accepted.".into()}});
        let mut may_remove = BTreeSet::new();
        for req in &task.requirements {
            match req {
                Requirement::Enabled=>rows.push(Check {passed:self.runtime.enabled,label:"Enable at boot".into(),detail:if self.runtime.enabled {"Enabled at boot in this simulation.".into()}else{"The service is disabled at boot; starting it and enabling it are separate actions.".into()}}),
                Requirement::PopCommitted {user,uids}=>{
                    let passed=self.runtime.pop_committed.get(user).is_some_and(|actual|uids.is_subset(actual));
                    rows.push(Check {passed,label:format!("{user} · committed POP cleanup"),detail:if passed{"The simulator recorded the required removals from an authenticated POP QUIT, and the messages remain absent.".into()}else{"This POP exercise needs an authenticated transaction that commits the required removals with QUIT. DELE alone, disconnecting, or deleting through IMAP does not demonstrate that outcome. Reset this attempt if you removed its target using another method.".into()}});
                }
                Requirement::Access {profile,user,upgrade,allow}=>rows.push(task.access_check(&self.runtime,profile,user,*upgrade,*allow)),
                Requirement::Message {user,mailbox,uid,present,seen,deleted}=>{
                    if !present {may_remove.insert((user,mailbox,*uid));}
                    let actual=self.runtime.store[user][mailbox].messages.iter().find(|m|m.uid==*uid);
                    let passed=match actual {Some(m)=>*present && seen.is_none_or(|s|m.flags.contains(&Flag::Seen)==s) && deleted.is_none_or(|d|m.flags.contains(&Flag::Deleted)==d),None=>!*present};
                    let detail=if passed {"This stored message outcome matches the requirement.".into()}else if !present && actual.is_some(){
                        let pending=matches!(&self.connection,Some(c)if matches!(&c.phase,Phase::Pop {user:Some(u),deleted,..}if u==user && deleted.contains(uid)));
                        if pending {"DELE only marked this POP message. The stored copy remains until QUIT commits the transaction; disconnect cancels the marks.".into()}else{"The stored message still exists. A deletion flag alone is not permanent removal; inspect the active protocol's commit behavior.".into()}
                    }else if let Some(message)=actual {
                        if *seen==Some(true)&&!message.flags.contains(&Flag::Seen){"The message has not been marked read. BODY.PEEK[] and read-only EXAMINE preserve the Seen flag; a read/write BODY[] fetch marks it.".into()}
                        else{format!("Stored flags: {}. Compare the requested Seen/Deleted state; reading with BODY[] can set Seen, while PEEK leaves it unchanged.",message.flag_text())}
                    }else{"This message is missing but must remain. Reset this attempt to recover the supplied sample mail, then target only the required message.".into()};
                    rows.push(Check {passed,label:format!("{user}/{mailbox} · UID {uid}"),detail});
                }
            }
        }
        let mut missing = Vec::new();
        for (user, boxes) in &task.facts.mailboxes {
            for (name, mailbox) in boxes {
                for message in &mailbox.messages {
                    if !may_remove.contains(&(user, name, message.uid))
                        && !self.runtime.store[user][name]
                            .messages
                            .iter()
                            .any(|m| m.uid == message.uid)
                    {
                        missing.push(format!("{user}/{name}: UID{}", message.uid));
                    }
                }
            }
        }
        rows.push(Check {passed:missing.is_empty(),label:"Preserve other supplied mail".into(),detail:if missing.is_empty(){"All messages outside the requested removals remain intact.".into()}else{format!("These required messages are missing: {}. Reset only this attempt to restore its sample mail.",missing.join(", "))}});
        rows
    }
    fn append(&mut self, text: &str) {
        if self.output.len() + text.len() > MAX_CONSOLE {
            self.output =
                "[Earlier console output cleared to retain this complete reply.]\n".into();
        }
        self.output.push_str(text);
    }
    fn apply_inner(&mut self, task: &Task, action: &Action) -> Result<(), String> {
        match action {
            Action::Edit { text } => {
                if !task.editable_config {
                    return Err(
                        "This lesson uses a supplied read-only server configuration.".into(),
                    );
                }
                if text.len() > MAX_TEXT || text.lines().count() > MAX_LINES {
                    return Err(
                        "Keep the configuration within8KiB/160lines; previous draft retained."
                            .into(),
                    );
                }
                self.draft = text.clone();
                self.checked = false;
            }
            Action::Input { text } => {
                if text.len() > protocol::MAX_COMMAND {
                    return Err("Protocol input exceeds2KiB.".into());
                }
                self.input = text.clone();
            }
            Action::Client { id } => {
                if !task.clients.contains_key(id) {
                    return Err("Choose a supplied virtual client profile.".into());
                }
                self.client = id.clone();
            }
            Action::Save => {
                self.runtime.save(&self.draft).map_err(|e| e.message)?;
                self.checked = false;
                self.append(
                    "[Simulator] Configuration saved; running settings have not changed.\n",
                );
            }
            Action::Restart => {
                self.connection = None;
                let result = self.runtime.restart(&task.facts);
                self.checked = false;
                self.append(&match result {
                    Ok(()) => {
                        "[Simulator] dovecot.service started with the saved settings.\n".into()
                    }
                    Err(e) => format!("[Simulator] Restart failed: {e}\n"),
                });
            }
            Action::Stop => {
                self.connection = None;
                self.runtime.stop();
                self.checked = false;
                self.append("[Simulator] Service stopped; stored mail retained.\n");
            }
            Action::Enable => {
                self.runtime.enabled = true;
                self.checked = false;
                self.append("[Simulator] Service enabled at boot.\n");
            }
            Action::Connect => {
                self.connection = None;
                self.checked = false;
                match Connection::open(&self.runtime, &task.facts, &task.clients[&self.client]) {
                    Ok((session, greeting)) => {
                        self.connection = Some(session);
                        self.append(&format!(
                            "[Simulator] Connected with {}\nS: {}",
                            self.client, greeting.text
                        ));
                    }
                    Err(e) => self.append(&format!("[Simulator] Connection failed: {e}\n")),
                }
            }
            Action::Command { line } => {
                let session = self
                    .connection
                    .as_mut()
                    .ok_or("Connect a virtual client before sending a protocol command.")?;
                let reply = session.command(&mut self.runtime, &task.facts, line);
                self.checked = false;
                self.input.clear();
                self.append(&format!(
                    "C: {line}\nS: {}{}",
                    reply.text,
                    reply
                        .client_note
                        .map_or(String::new(), |n| format!("[Virtual TLS client] {n}\n"))
                ));
            }
            Action::Admin { line } => {
                if line.len() > protocol::MAX_COMMAND {
                    return Err(
                        "Use one administration command within2KiB; previous input retained."
                            .into(),
                    );
                }
                self.checked = false;
                match crate::mailbox_admin::run(&mut self.runtime, &task.facts, line) {
                    Ok(out) => {
                        if out.changed {
                            self.connection = None;
                        }
                        self.append(&format!("$ {line}\n{}", out.text));
                        self.input.clear();
                    }
                    Err(e) => self.append(&format!("$ {line}\n[{:?}] {}\n", e.issue, e.message)),
                }
            }
            Action::Disconnect => {
                self.connection = None;
                self.checked = false;
                self.append(
                    "[Simulator] Disconnected; pending POP deletion marks were not committed.\n",
                );
            }
            Action::ClearOutput => self.output.clear(),
            Action::Check => self.checked = true,
            Action::Reset => *self = Self::new(task)?,
        }
        Ok(())
    }
    pub fn apply(&mut self, task: &Task, action: &Action) -> Result<Vec<Check>, String> {
        self.bound(task)?;
        if !access::within(action, MAX_ACTION) {
            return Err("Mail-access action exceeds16KiB.".into());
        }
        let mut next = self.clone();
        next.apply_inner(task, action)?;
        next.bound(task)?;
        *self = next;
        Ok(if matches!(action, Action::Check) {
            self.checks(task)
        } else {
            Vec::new()
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Edit { text: String },
    Input { text: String },
    Client { id: String },
    Save,
    Restart,
    Stop,
    Enable,
    Connect,
    Command { line: String },
    Admin { line: String },
    Disconnect,
    ClearOutput,
    Check,
    Reset,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema: u32,
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    pub worked_config: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema: u32,
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != 1 || !id(&self.source_id) || !access::within(self, MAX_SESSION) {
            return Err(
                "Use mail-access schema1, a stable source ID and bounded task data.".into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if let Some(text) = &self.worked_config {
            if !self.example.editable_config
                || text.len() > MAX_TEXT
                || text.lines().count() > MAX_LINES
            {
                return Err("Worked configuration needs an editable separate example and the normal file bounds.".into());
            }
            let mut server_task = self.example.clone();
            server_task.requirements.retain(|r| {
                !matches!(
                    r,
                    Requirement::Message { .. } | Requirement::PopCommitted { .. }
                )
            });
            server_task.validate()?;
            let mut worked = Scope::new(&server_task)?;
            for action in [
                Action::Edit { text: text.clone() },
                Action::Save,
                Action::Restart,
                Action::Enable,
            ] {
                worked.apply(&server_task, &action)?;
            }
            if worked.checks(&server_task).iter().any(|c| !c.passed) {
                return Err("The worked example configuration must satisfy every server/access requirement.".into());
            }
        }
        if self.assignment.facts == self.example.facts {
            return Err(
                "Worked practice must use separate supplied inputs from the assignment.".into(),
            );
        }
        let initial = Scope::new(&self.assignment)?;
        if initial.checks(&self.assignment).iter().all(|c| c.passed) {
            return Err("The independent assignment starter must require meaningful work.".into());
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        if s.schema != 1 || s.source_id != self.source_id || !access::within(s, MAX_SESSION) {
            return Err("Saved mail-access work does not match its authored source.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<Check> {
        if let Err(detail) = self.bound(s) {
            return vec![Check {
                passed: false,
                label: "Read this saved work".into(),
                detail,
            }];
        }
        if !s.assignment.checked {
            return vec![Check {passed:false,label:"Check the assignment".into(),detail:"Unchecked. Use Check work after changing the assignment; separate practice never earns assignment credit.".into()}];
        }
        s.assignment.checks(&self.assignment)
    }
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            schema: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn action(
        &mut self,
        d: &Definition,
        a: &Action,
        practice: bool,
    ) -> Result<Vec<Check>, String> {
        d.bound(self)?;
        if practice {
            self.practice.apply(&d.example, a)
        } else {
            self.assignment.apply(&d.assignment, a)
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        // Structural only: source-bound validation additionally checks fixtures.
        if self.schema != 1 || !id(&self.source_id) || !access::within(self, MAX_SESSION) {
            return Err("Invalid bounded mail-access session envelope.".into());
        }
        Ok(())
    }
}
