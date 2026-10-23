//! Bounded virtual Linux process metadata. No OS/process/filesystem APIs.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const HELP: &str = "Process simulator subset: ps -eo pid,ppid,user,stat,ni,comm; ps -p PID[,PID...] -o COLUMNS; pgrep -x NAME [-u USER]; kill [-s SIGNAL|-SIGNAL] [--] PID...; renice [--priority] VALUE [-p] PID.... Signals: 0, INT, TERM, KILL, STOP, CONT (Linux x86_64 numbers). No host processes, regex, groups, jobs, spawning or real scheduler. show is a simulator-only snapshot helper.";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub name: String,
    pub uid: u32,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Activity {
    Running,
    Sleeping,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Phase {
    Active {
        activity: Activity,
    },
    Stopped {
        resume: Activity,
        pending_termination: bool,
    },
    Exited,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Process {
    pub parent: u32,
    pub owner: Identity,
    pub session: u32,
    pub comm: String,
    pub nice: i8,
    pub phase: Phase,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub user: Identity,
    pub session: u32,
    #[serde(deserialize_with = "process_map")]
    pub processes: BTreeMap<u32, Process>,
}
fn process_map<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<u32, Process>, D::Error> {
    struct Entries;
    impl<'de> serde::de::Visitor<'de> for Entries {
        type Value = BTreeMap<u32, Process>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most64 distinct positive process IDs")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut processes = BTreeMap::new();
            while let Some(key) = map.next_key::<String>()? {
                // Internally tagged State buffers JSON object keys as strings;
                // validate canonical IDs explicitly rather than relying on JSON's u32 key adapter.
                let id = key
                    .parse::<u32>()
                    .map_err(|_| serde::de::Error::custom("Invalid process ID key"))?;
                if key != id.to_string()
                    || !bounded_id(id)
                    || processes.len() >= 64
                    || processes.contains_key(&id)
                {
                    return Err(serde::de::Error::custom(
                        "Invalid, duplicate or too many process IDs",
                    ));
                }
                processes.insert(id, map.next_value::<Process>()?);
            }
            Ok(processes)
        }
    }
    deserializer.deserialize_map(Entries)
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
impl Output {
    fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            status: 0,
        }
    }
    fn error(&mut self, text: String) {
        self.stderr.push_str(&text);
        self.status = 1;
    }
}
fn identifier(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        && s.as_bytes()[0] != b'-'
}
fn bounded_id(value: u32) -> bool {
    value > 0 && value <= i32::MAX as u32
}
fn pid(s: &str) -> Result<u32, String> {
    if !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_digit())
        && let Ok(n) = s.parse::<u32>()
        && bounded_id(n)
    {
        Ok(n)
    } else {
        Err("Unsupported PID: use a positive process ID up to2147483647; process groups and PID0/-1 are excluded".into())
    }
}
#[derive(Clone, Copy)]
enum Signal {
    Probe,
    Interrupt,
    Kill,
    Terminate,
    Continue,
    Stop,
}
fn signal(s: &str) -> Result<Signal, String> {
    let upper = s.to_ascii_uppercase();
    let name = upper.strip_prefix("SIG").unwrap_or(&upper);
    if upper.starts_with("SIG") && !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit()) {
        return Err("Unsupported numeric signal with SIG prefix; use a number or a symbolic name such as SIGTERM".into());
    }
    Ok(match name {"0"=>Signal::Probe,"2"|"INT"=>Signal::Interrupt,"9"|"KILL"=>Signal::Kill,
        "15"|"TERM"=>Signal::Terminate,"18"|"CONT"=>Signal::Continue,"19"|"STOP"=>Signal::Stop,
        _=>return Err("Unsupported signal; modeled names are0, INT, KILL, TERM, CONT and STOP (Linux x86_64 numbers)".into())})
}
fn unsupported() -> String {
    format!("Unsupported process command or syntax. {HELP}")
}
impl Table {
    pub fn prompt(&self) -> &'static str {
        if self.user.uid == 0 { "lab #" } else { "lab $" }
    }
    pub fn valid(&self) -> bool {
        if self.processes.is_empty()
            || self.processes.len() > 64
            || !bounded_id(self.session)
            || !identifier(&self.user.name, 32)
        {
            return false;
        }
        let mut names = BTreeMap::new();
        let mut ids = BTreeMap::new();
        for owner in std::iter::once(&self.user).chain(self.processes.values().map(|p| &p.owner)) {
            if owner.uid > i32::MAX as u32
                || !identifier(&owner.name, 32)
                || (owner.name == "root") != (owner.uid == 0)
                || names
                    .insert(&owner.name, owner.uid)
                    .is_some_and(|old| old != owner.uid)
                || ids
                    .insert(owner.uid, &owner.name)
                    .is_some_and(|old| old != &owner.name)
            {
                return false;
            }
        }
        let Some(init) = self.processes.get(&1) else {
            return false;
        };
        if init.parent != 0
            || init.owner.uid != 0
            || init.comm != "init"
            || init.nice != 0
            || init.session != 1
            || init.phase
                != (Phase::Active {
                    activity: Activity::Sleeping,
                })
        {
            return false;
        }
        for (&id, p) in &self.processes {
            if !bounded_id(id)
                || !bounded_id(p.session)
                || !identifier(&p.comm, 15)
                || !(-20..=19).contains(&p.nice)
                || (id != 1 && (!self.processes.contains_key(&p.parent) || p.parent == id))
            {
                return false;
            }
            let mut seen = BTreeSet::new();
            let mut at = id;
            while at != 1 {
                if !seen.insert(at) {
                    return false;
                }
                let Some(parent) = self.processes.get(&at).map(|p| p.parent) else {
                    return false;
                };
                at = parent;
            }
        }
        true
    }
    /// Immutable identities/shape stay pinned when saving or restoring an attempt.
    pub fn same_problem(&self, initial: &Self) -> bool {
        self.valid()
            && initial.valid()
            && self.user == initial.user
            && self.session == initial.session
            && self.processes.len() == initial.processes.len()
            && self.processes.iter().all(|(id, p)| {
                initial.processes.get(id).is_some_and(|old| {
                    p.parent == old.parent
                        && p.owner == old.owner
                        && p.session == old.session
                        && p.comm == old.comm
                })
            })
    }
    fn live(&self, id: u32) -> bool {
        self.processes
            .get(&id)
            .is_some_and(|p| p.phase != Phase::Exited)
    }
    /// Current observed metadata, including reparenting; origin fields stay pinned.
    pub fn observed_process(&self, pid: u32) -> Option<Process> {
        self.processes.get(&pid).map(|p| Process {
            parent: self.reported_parent(p),
            ..p.clone()
        })
    }
    fn reported_parent(&self, p: &Process) -> u32 {
        if p.parent == 0 || self.live(p.parent) {
            p.parent
        } else {
            1
        }
    }
    fn ps(&self, selected: Option<&BTreeSet<u32>>, columns: &str) -> Result<Output, String> {
        let columns: Vec<_> = columns.split(',').collect();
        if columns.is_empty()
            || columns.len() > 6
            || columns
                .iter()
                .any(|c| !matches!(*c, "pid" | "ppid" | "user" | "stat" | "ni" | "comm"))
        {
            return Err(unsupported());
        }
        let mut rows = vec![
            columns
                .iter()
                .map(|c| {
                    match *c {
                        "pid" => "PID",
                        "ppid" => "PPID",
                        "user" => "USER",
                        "stat" => "STAT",
                        "ni" => "NI",
                        _ => "COMMAND",
                    }
                    .to_string()
                })
                .collect::<Vec<_>>(),
        ];
        for (&id, p) in &self.processes {
            if !self.live(id) || selected.is_some_and(|ids| !ids.contains(&id)) {
                continue;
            }
            rows.push(
                columns
                    .iter()
                    .map(|column| match *column {
                        "pid" => id.to_string(),
                        "ppid" => self.reported_parent(p).to_string(),
                        "user" => p.owner.name.clone(),
                        "stat" => {
                            let mut state = match p.phase {
                                Phase::Active {
                                    activity: Activity::Running,
                                } => "R",
                                Phase::Active {
                                    activity: Activity::Sleeping,
                                } => "S",
                                Phase::Stopped { .. } => "T",
                                Phase::Exited => unreachable!(),
                            }
                            .to_string();
                            if p.nice < 0 {
                                state.push('<');
                            } else if p.nice > 0 {
                                state.push('N');
                            }
                            if p.session == id {
                                state.push('s');
                            }
                            state
                        }
                        "ni" => p.nice.to_string(),
                        _ => p.comm.clone(),
                    })
                    .collect(),
            );
        }
        let widths: Vec<_> = (0..columns.len())
            .map(|i| rows.iter().map(|r| r[i].len()).max().unwrap())
            .collect();
        let mut out = String::new();
        for row in &rows {
            for (i, value) in row.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                let width = widths[i];
                if matches!(columns[i], "pid" | "ppid" | "ni") {
                    out.push_str(&format!("{value:>width$}"));
                } else if i + 1 == columns.len() {
                    out.push_str(value);
                } else {
                    out.push_str(&format!("{value:<width$}"));
                }
            }
            out.push('\n');
        }
        Ok(Output {
            stdout: out,
            stderr: String::new(),
            status: i32::from(rows.len() == 1),
        })
    }
    fn pgrep(&self, args: &[&str]) -> Result<Output, String> {
        let mut exact = false;
        let mut owner = None;
        let mut executable = None;
        let mut i = 0;
        while i < args.len() {
            match args[i] {
                "-x" if !exact => exact = true,
                "-u" if owner.is_none() => {
                    i += 1;
                    owner = Some(*args.get(i).ok_or_else(unsupported)?);
                }
                s if !s.starts_with('-') && executable.is_none() => executable = Some(s),
                _ => return Err(unsupported()),
            }
            i += 1;
        }
        if !exact {
            return Err("Unsupported pgrep pattern mode: use -x NAME for an exact executable name; regex is not implemented".into());
        }
        let executable = executable.ok_or_else(unsupported)?;
        if !identifier(executable, 15) || executable.contains('.') {
            return Err("Unsupported pgrep regex: use -x with a name containing only letters, digits, underscores or hyphens; inspect other names with ps".into());
        }
        let uid = if let Some(name) = owner {
            Some(
                std::iter::once(&self.user)
                    .chain(self.processes.values().map(|p| &p.owner))
                    .find(|o| o.name == name)
                    .ok_or_else(|| format!("Unsupported unknown simulated user {name}"))?
                    .uid,
            )
        } else {
            None
        };
        let out = self
            .processes
            .iter()
            .filter(|(id, p)| {
                self.live(**id) && p.comm == executable && uid.is_none_or(|uid| p.owner.uid == uid)
            })
            .map(|(id, _)| format!("{id}\n"))
            .collect::<String>();
        Ok(Output {
            status: i32::from(out.is_empty()),
            stdout: out,
            stderr: String::new(),
        })
    }
    fn kill(&mut self, args: &[&str]) -> Result<Output, String> {
        let mut i = 0;
        let mut selected = Signal::Terminate;
        if args.first() == Some(&"-s") {
            i = 2;
            selected = signal(args.get(1).ok_or_else(unsupported)?)?;
        } else if let Some(flag) = args.first().filter(|s| s.starts_with('-') && **s != "--") {
            i = 1;
            selected = signal(&flag[1..])?;
        }
        if args.get(i) == Some(&"--") {
            i += 1;
        }
        let targets = args[i..]
            .iter()
            .map(|s| pid(s))
            .collect::<Result<Vec<_>, _>>()?;
        if targets.is_empty() {
            return Err(unsupported());
        }
        let mut out = Output::success(String::new());
        for id in targets {
            if !self.live(id) {
                out.error(format!("kill: ({id}): No such process\n"));
                continue;
            }
            let p = self.processes.get_mut(&id).unwrap();
            let permitted = self.user.uid == 0
                || self.user.uid == p.owner.uid
                || (matches!(selected, Signal::Continue) && self.session == p.session);
            if (id == 1 && !matches!(selected, Signal::Probe)) || !permitted {
                out.error(format!("kill: ({id}): Operation not permitted\n"));
                continue;
            }
            p.phase = match (selected, &p.phase) {
                (Signal::Probe, _) => p.phase.clone(),
                (Signal::Kill, _) => Phase::Exited,
                (
                    Signal::Continue,
                    Phase::Stopped {
                        pending_termination: true,
                        ..
                    },
                ) => Phase::Exited,
                (Signal::Continue, Phase::Stopped { resume, .. }) => {
                    Phase::Active { activity: *resume }
                }
                (Signal::Continue, _) | (Signal::Stop, Phase::Stopped { .. }) => p.phase.clone(),
                (Signal::Stop, Phase::Active { activity }) => Phase::Stopped {
                    resume: *activity,
                    pending_termination: false,
                },
                (Signal::Interrupt | Signal::Terminate, Phase::Stopped { resume, .. }) => {
                    Phase::Stopped {
                        resume: *resume,
                        pending_termination: true,
                    }
                }
                (Signal::Interrupt | Signal::Terminate, _) => Phase::Exited,
                (_, Phase::Exited) => unreachable!(),
            };
        }
        Ok(out)
    }
    fn renice(&mut self, args: &[&str]) -> Result<Output, String> {
        let i = usize::from(args.first() == Some(&"--priority"));
        let value=args.get(i).ok_or_else(unsupported)?.parse::<i8>().ok().filter(|n|(-20..=19).contains(n)).ok_or("Unsupported nice value: use an absolute integer from -20 through19; -n and relative mode are excluded")?;
        let mut i = i + 1;
        if args.get(i) == Some(&"-p") {
            i += 1;
        }
        let targets = args[i..]
            .iter()
            .map(|s| pid(s))
            .collect::<Result<Vec<_>, _>>()?;
        if targets.is_empty() {
            return Err(unsupported());
        }
        let mut out = Output::success(String::new());
        for id in targets {
            if !self.live(id) {
                out.error(format!(
                    "renice: failed to get priority for {id} (process ID): No such process\n"
                ));
                continue;
            }
            let p = self.processes.get_mut(&id).unwrap();
            if id == 1 || (self.user.uid != 0 && (self.user.uid != p.owner.uid || value < p.nice)) {
                out.error(format!(
                    "renice: failed to set priority for {id} (process ID): Permission denied\n"
                ));
                continue;
            }
            let old = p.nice;
            p.nice = value;
            out.stdout.push_str(&format!(
                "{id} (process ID) old priority {old}, new priority {value}\n"
            ));
        }
        Ok(out)
    }
    pub fn execute(&mut self, input: &str) -> Result<Output, String> {
        if !self.valid() {
            return Err("Invalid simulated process table".into());
        }
        if input.len() > 4096
            || input
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\t' | '\n'))
        {
            return Err(
                "Process command is too long or contains unsupported control characters".into(),
            );
        }
        // This command dispatcher is deliberately not a shell parser. No expansion,
        // quoting, redirection, pipelines, substitution or host lookup occurs.
        if input.contains([
            '\n', '\r', '\0', '\'', '"', '`', '$', '|', '&', ';', '<', '>', '\\', '(', ')', '{',
            '}', '[', ']', '/',
        ]) {
            return Err(unsupported());
        }
        let words: Vec<_> = input.split_whitespace().collect();
        if words.len() > 128 {
            return Err("Process command exceeds128 words".into());
        }
        match words.as_slice() {
            ["help"] => Ok(Output::success(HELP.into())),
            ["show"] => self.ps(None, "pid,ppid,user,stat,ni,comm"),
            ["ps", "-eo", columns] | ["ps", "-e", "-o", columns] => self.ps(None, columns),
            ["ps", "-p", ids, "-o", columns] => {
                let ids = ids
                    .split(',')
                    .map(pid)
                    .collect::<Result<BTreeSet<_>, _>>()?;
                self.ps(Some(&ids), columns)
            }
            ["pgrep", args @ ..] => self.pgrep(args),
            ["kill", args @ ..] => self.kill(args),
            ["renice", args @ ..] => self.renice(args),
            _ => Err(unsupported()),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExpectedPhase {
    Active,
    Stopped,
    Exited,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    NoPendingTermination { pid: u32 },
    Phase { pid: u32, expected: ExpectedPhase },
    Niceness { pid: u32, value: i8 },
    Unchanged { pid: u32, expected: Process },
}
impl Requirement {
    pub fn valid(&self, initial: &Table) -> bool {
        match self {
            Self::NoPendingTermination { pid } => initial.processes.contains_key(pid),
            Self::Phase { pid, .. } => initial.processes.contains_key(pid),
            Self::Niceness { pid, value } => {
                initial.processes.contains_key(pid) && (-20..=19).contains(value)
            }
            Self::Unchanged { pid, expected } => {
                initial.observed_process(*pid).as_ref() == Some(expected)
            }
        }
    }
    pub fn check(&self, table: &Table) -> (bool, String) {
        if !table.valid() {
            return (false, "The process table is invalid".into());
        }
        match self {
            Self::NoPendingTermination { pid } => {
                let pass = table.processes.get(pid).is_some_and(|p| {
                    matches!(
                        p.phase,
                        Phase::Active { .. }
                            | Phase::Stopped {
                                pending_termination: false,
                                ..
                            }
                    )
                });
                (
                    pass,
                    if pass {
                        format!("Process {pid} has no ending request waiting")
                    } else {
                        format!(
                            "Process {pid} has ended or has an ending request waiting, so continuing it cannot keep it available. Reset this attempt to practice pausing it safely."
                        )
                    },
                )
            }
            Self::Phase { pid, expected } => {
                let phase = table.processes.get(pid).map(|p| &p.phase);
                let pass = matches!(
                    (expected, phase),
                    (ExpectedPhase::Active, Some(Phase::Active { .. }))
                        | (ExpectedPhase::Stopped, Some(Phase::Stopped { .. }))
                        | (ExpectedPhase::Exited, Some(Phase::Exited))
                );
                let name = match expected {
                    ExpectedPhase::Active => "active",
                    ExpectedPhase::Stopped => "stopped",
                    ExpectedPhase::Exited => "terminated",
                };
                let feedback = if pass {
                    format!("Process {pid} is {name}")
                } else {
                    match (expected, phase) {
                        (ExpectedPhase::Stopped | ExpectedPhase::Active, Some(Phase::Exited)) => {
                            format!(
                                "Process {pid} has ended and cannot be resumed. Reset this attempt to restore it, then choose an action that keeps it present."
                            )
                        }
                        (ExpectedPhase::Exited, Some(Phase::Stopped { .. })) => format!(
                            "Process {pid} is still paused. A termination request can remain pending while paused; decide whether to resume it or end it immediately."
                        ),
                        (ExpectedPhase::Stopped, Some(Phase::Active { .. })) => format!(
                            "Process {pid} is still running or waiting, rather than paused. Check its STAT column after your action."
                        ),
                        _ => format!(
                            "Process {pid} needs to be {name}; inspect its current state with ps."
                        ),
                    }
                };
                (pass, feedback)
            }
            Self::Niceness { pid, value } => {
                let current = table
                    .processes
                    .get(pid)
                    .filter(|_| table.live(*pid))
                    .map(|p| p.nice);
                let pass = current == Some(*value);
                let message = match current {
                    Some(n) if pass => format!("Process {pid} has nice value {n}"),
                    Some(n) if n > *value && table.user.uid != 0 => format!(
                        "Process {pid} has nice value {n}; the task asks for {value}. Under this exercise’s ordinary-user limits you cannot lower it again; Reset this attempt before choosing the requested value."
                    ),
                    Some(n) => format!(
                        "Process {pid} has nice value {n}; the task asks for {value}. A larger nice value means lower urgency."
                    ),
                    None => format!(
                        "Process {pid} has ended. This task needs it present with nice value {value}; reset this attempt to try again."
                    ),
                };
                (pass, message)
            }
            Self::Unchanged { pid, expected } => (
                table.observed_process(*pid).as_ref() == Some(expected),
                format!("Preserve process {pid} and its settings"),
            ),
        }
    }
}
