//! Versioned scheduling assignments, separate drafts and behavior-based checks.
//! No host I/O. A fresh probe uses the same tick-owned scheduler as native Run.
use crate::cron::{self as core, Moment, State};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, io,
};
pub mod budget;

pub const MAX_PROBES: usize = 16;
pub const MAX_REQUIREMENTS: usize = 16;
pub const MAX_CONTEXT: usize = 2 * 1024 * 1024;
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub const MAX_SESSION: usize = MAX_ATTEMPT;
pub const MAX_ACTION: usize = 65536;
pub const PREFIX: &str = "schedule ";
pub const HELP: &str = "Edit the supplied virtual user/system crontab or access document. Install stores a valid table; removing an installed table is separate from denying future crontab management. Run advances only the explicit virtual clock and uses the bounded in-process command subset. Stop discards unfinished simulated changes. No host cron, shell, clock, files, at, timers or locks are used.";

pub(crate) fn sequence<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    struct Excess;
    impl<'de> Deserialize<'de> for Excess {
        fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Err(D::Error::custom("Scheduling item limit"))
        }
    }
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut v = Vec::new();
            while v.len() < N {
                match a.next_element()? {
                    Some(x) => v.push(x),
                    None => return Ok(v),
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(v)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
fn texts<'de, D: Deserializer<'de>, const N: usize, const B: usize>(
    d: D,
) -> Result<Vec<String>, D::Error> {
    struct Seed<const B: usize>;
    impl<'de, const B: usize> serde::de::DeserializeSeed<'de> for Seed<B> {
        type Value = String;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
            core::text::<D, B>(d)
        }
    }
    struct Items<const N: usize, const B: usize>;
    impl<'de, const N: usize, const B: usize> Visitor<'de> for Items<N, B> {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} bounded text items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < N {
                let Some(v) = a.next_element_seed(Seed::<B>)? else {
                    return Ok(out);
                };
                out.push(v);
            }
            struct Excess;
            impl<'de> Deserialize<'de> for Excess {
                fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                    Err(D::Error::custom("Scheduling text item count limit"))
                }
            }
            let _ = a.next_element::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<N, B>)
}
struct Counter {
    bytes: usize,
    cap: usize,
}
impl io::Write for Counter {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| io::Error::other("Scheduling saved capacity limit"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn size<T: Serialize + ?Sized>(v: &T, cap: usize) -> Option<usize> {
    let mut w = Counter { bytes: 0, cap };
    serde_json::to_writer(&mut w, v).ok()?;
    Some(w.bytes)
}
pub fn within<T: Serialize + ?Sized>(v: &T, cap: usize) -> bool {
    size(v, cap).is_some()
}
pub fn string_bytes(text: &str) -> usize {
    2 + text
        .bytes()
        .map(|b| match b {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' | 8 | 12 => 2,
            0..=31 => 6,
            _ => 1,
        })
        .sum::<usize>()
}
fn ordinary(path: &str) -> bool {
    path.strip_prefix("/work/").is_some_and(|p| {
        !p.is_empty()
            && !matches!(p, "." | "..")
            && !p.contains('/')
            && !p.chars().any(char::is_control)
            && p.len() <= 250
    })
}
fn short(s: &str, n: usize) -> bool {
    !s.trim().is_empty() && s.len() <= n && !s.contains('\0')
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Document {
    User {
        #[serde(deserialize_with = "core::text::<_,32>")]
        owner: String,
    },
    System,
    Allow,
    Deny,
}
impl Document {
    pub fn key(&self) -> String {
        match self {
            Self::User { owner } => format!("user:{owner}"),
            Self::System => "system".into(),
            Self::Allow => "allow".into(),
            Self::Deny => "deny".into(),
        }
    }
    pub fn label(&self) -> String {
        match self {
            Self::User { owner } => format!("{owner}'s crontab"),
            Self::System => "System crontab".into(),
            Self::Allow => "cron.allow".into(),
            Self::Deny => "cron.deny".into(),
        }
    }
    pub fn installed(&self, s: &State) -> String {
        match self {
            Self::User { owner } => s.tables.get(owner).cloned().unwrap_or_default(),
            Self::System => s.system_table.clone(),
            Self::Allow => policy_text(s.allow.as_ref()),
            Self::Deny => policy_text(s.deny.as_ref()),
        }
    }
    pub fn exists(&self, s: &State) -> bool {
        match self {
            Self::User { owner } => s.tables.contains_key(owner),
            Self::System => !s.system_table.is_empty(),
            Self::Allow => s.allow.is_some(),
            Self::Deny => s.deny.is_some(),
        }
    }
    fn valid_for(&self, s: &State) -> bool {
        match self {
            Self::User { owner } => s.users.contains(owner),
            _ => true,
        }
    }
}
fn policy_text(s: Option<&BTreeSet<String>>) -> String {
    s.map(|s| s.iter().map(|n| format!("{n}\n")).collect())
        .unwrap_or_default()
}
fn policy(text: &str, initial: &State) -> Result<BTreeSet<String>, String> {
    if text.len() > 512
        || text.contains(['\0', '\r'])
        || (!text.is_empty() && !text.ends_with('\n'))
    {
        return Err(
            "Put one supplied username on each line and end the last line with a newline (maximum 512 bytes)"
                .into(),
        );
    }
    let mut names = BTreeSet::new();
    for line in text.lines() {
        if !initial.users.contains(line) || !names.insert(line.into()) {
            return Err("Use each supplied username at most once; no spaces or comments in this access-file subset".into());
        }
    }
    Ok(names)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    TableAbsent {
        #[serde(deserialize_with = "core::text::<_,32>")]
        owner: String,
    },
    CanManage {
        #[serde(deserialize_with = "core::text::<_,32>")]
        actor: String,
        #[serde(deserialize_with = "core::text::<_,32>")]
        owner: String,
        allowed: bool,
    },
    CurrentFile {
        #[serde(deserialize_with = "core::text::<_,256>")]
        path: String,
        #[serde(deserialize_with = "core::text::<_,32768>")]
        text: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedRun {
    #[serde(deserialize_with = "core::text::<_,32>")]
    pub owner: String,
    #[serde(deserialize_with = "core::text::<_,32768>")]
    pub stdout: String,
    #[serde(deserialize_with = "core::text::<_,32768>")]
    pub stderr: String,
    pub status: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "core::text::<_,120>")]
    pub name: String,
    pub start: Moment,
    #[serde(deserialize_with = "core::map::<_,32,32768>")]
    pub files: BTreeMap<String, String>,
    #[serde(deserialize_with = "sequence::<_,ExpectedRun,16>")]
    pub expected: Vec<ExpectedRun>,
    #[serde(deserialize_with = "core::map::<_,32,32768>")]
    pub after: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: State,
    #[serde(deserialize_with = "sequence::<_,Document,8>")]
    pub editable: Vec<Document>,
    #[serde(deserialize_with = "texts::<_,8,256>")]
    pub outputs: Vec<String>,
    #[serde(deserialize_with = "sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "sequence::<_,Probe,16>")]
    pub probes: Vec<Probe>,
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_CONTEXT {
            return Err("Scheduling context exceeds 2 MiB".into());
        }
        let d: Self = serde_json::from_slice(raw).map_err(|_| "Invalid scheduling context")?;
        d.validate()?;
        Ok(d)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.to_string())?;
        if !within(self, MAX_CONTEXT)
            || self.editable.is_empty()
            || self.editable.len() > 8
            || self.editable.iter().collect::<BTreeSet<_>>().len() != self.editable.len()
            || self.editable.iter().any(|t| !t.valid_for(&self.initial))
            || self.outputs.len() > 8
            || self.outputs.iter().collect::<BTreeSet<_>>().len() != self.outputs.len()
            || self.outputs.iter().any(|p| !ordinary(p))
            || self.requirements.is_empty()
            || self.requirements.len() > MAX_REQUIREMENTS
            || self.probes.is_empty()
            || self.probes.len() > MAX_PROBES
        {
            return Err("Invalid bounded scheduling assignment/context".into());
        }
        for r in &self.requirements {
            match r {
                Requirement::TableAbsent { owner } if self.initial.users.contains(owner) => {}
                Requirement::CanManage { actor, owner, .. }
                    if self.initial.users.contains(actor) && self.initial.users.contains(owner) => {
                }
                Requirement::CurrentFile { path, text }
                    if self.outputs.contains(path)
                        && text.len() <= 32768
                        && !text.contains('\0') => {}
                _ => {
                    return Err(
                        "Scheduling requirement references an unknown identity/output".into(),
                    );
                }
            }
        }
        for probe in &self.probes {
            probe.start.next().map_err(|e| e.to_string())?;
            if !short(&probe.name, 120)
                || probe.expected.len() > core::MAX_ENTRIES
                || probe.expected.iter().any(|r| {
                    !self.initial.users.contains(&r.owner)
                        || !(0..=255).contains(&r.status)
                        || r.stdout.len() + r.stderr.len() > 32768
                        || r.stdout.contains('\0')
                        || r.stderr.contains('\0')
                })
            {
                return Err("Invalid fresh scheduling outcome probe".into());
            }
            // Reuse the core's exact file admission; candidate commands cannot
            // access any path beyond this complete supplied flat virtual map.
            for files in [&probe.files, &probe.after] {
                let mut state = self.initial.clone();
                state.files = files.clone();
                state.validate().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
    /// Admission/binding, not a grade. A valid run may damage an unrelated
    /// virtual file; retain that wrong state for diagnosis, repair and resume.
    pub fn bound_state(&self, state: &State) -> bool {
        if state.validate().is_err() || state.users != self.initial.users {
            return false;
        }
        for owner in &self.initial.users {
            if !self.editable.contains(&Document::User {
                owner: owner.clone(),
            }) && state.tables.get(owner) != self.initial.tables.get(owner)
            {
                return false;
            }
        }
        if !self.editable.contains(&Document::System)
            && state.system_table != self.initial.system_table
        {
            return false;
        }
        if !self.editable.contains(&Document::Allow) && state.allow != self.initial.allow {
            return false;
        }
        if !self.editable.contains(&Document::Deny) && state.deny != self.initial.deny {
            return false;
        }
        true
    }
    pub fn compatible(&self, state: &State) -> bool {
        self.bound_state(state)
            && self
                .initial
                .files
                .iter()
                .all(|(p, v)| self.outputs.contains(p) || state.files.get(p) == Some(v))
            && state
                .files
                .keys()
                .all(|p| self.initial.files.contains_key(p) || self.outputs.contains(p))
    }
    fn current(&self, state: &State) -> Vec<(bool, String)> {
        let mut rows = vec![(
            self.compatible(state),
            "Supplied accounts, peer tables and unrelated files are preserved".into(),
        )];
        for r in &self.requirements {
            rows.push(match r {
            Requirement::TableAbsent{owner}=>(!state.tables.contains_key(owner),format!("Remove the installed crontab for {owner}; denying future edits does not stop it")),
            Requirement::CanManage{actor,owner,allowed}=>(state.can_manage(actor,owner)==*allowed,format!("{actor} {} manage {owner}'s crontab",if *allowed {"may"}else{"must not"})),
            Requirement::CurrentFile{path,text}=>(state.files.get(path)==Some(text),format!("Run the configured jobs so {path} contains the requested result")),
        });
        }
        rows
    }
    pub fn checks(&self, state: &State) -> Vec<(bool, String)> {
        if self.validate().is_err() {
            return vec![(
                false,
                "Scheduling context is invalid; previous work retained".into(),
            )];
        }
        let Ok(mut checks) = Check::new(self, state) else {
            return vec![(
                false,
                "Scheduling context is invalid; previous work retained".into(),
            )];
        };
        // One-minute probes and16 entries/cases bound this fallback. Native Check
        // owns the same continuation and yields between calendar/VM steps.
        while !checks.done() {
            checks.tick(core::MAX_TICK);
        }
        checks.rows
    }
    pub fn current_ready(&self, state: &State) -> bool {
        self.current(state).iter().all(|r| r.0)
    }
    pub fn file_paths(&self) -> impl Iterator<Item = &str> {
        self.initial
            .files
            .keys()
            .chain(self.outputs.iter())
            .map(String::as_str)
            .chain(
                self.probes
                    .iter()
                    .flat_map(|p| p.files.keys().chain(p.after.keys()).map(String::as_str)),
            )
    }
    pub fn row_count(&self) -> usize {
        1 + self.requirements.len() + self.probes.len()
    }
}
fn probe_label(probe: &Probe) -> String {
    let Ok(time) = probe.start.next() else {
        return probe.name.clone();
    };
    let days = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let day = time
        .weekday()
        .ok()
        .and_then(|n| days.get(n as usize))
        .copied()
        .unwrap_or("Date");
    format!("{} · {day} {} UTC", probe.name, time.text())
}
fn preview(text: Option<&String>) -> String {
    match text {
        None => "no file".into(),
        Some(text) if text.is_empty() => "an empty file".into(),
        Some(text) => {
            let mut shown = text.chars().take(96).collect::<String>();
            if shown.len() < text.len() {
                shown.push('…');
            }
            format!("{shown:?}")
        }
    }
}
pub struct Check {
    definition: Definition,
    state: State,
    index: usize,
    active: Option<core::Run>,
    pub rows: Vec<(bool, String)>,
    done: bool,
}
impl Check {
    pub fn new(d: &Definition, s: &State) -> Result<Self, String> {
        d.validate()?;
        if !d.bound_state(s) {
            return Err("Scheduling context differs; previous work retained".into());
        }
        let rows = d.current(s);
        let done = false;
        Ok(Self {
            definition: d.clone(),
            state: s.clone(),
            index: 0,
            active: None,
            rows,
            done,
        })
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn tick(&mut self, budget: usize) {
        if self.done {
            return;
        }
        for _ in 0..budget.min(core::MAX_TICK) {
            if self.index == self.definition.probes.len() {
                self.done = true;
                return;
            }
            let probe = &self.definition.probes[self.index];
            if self.active.is_none() {
                let mut state = self.state.clone();
                state.clock = probe.start;
                state.files = probe.files.clone();
                match probe
                    .start
                    .next()
                    .and_then(|end| core::Run::begin(&state, end))
                {
                    Ok(run) => self.active = Some(run),
                    Err(e) => {
                        self.rows
                            .push((false, format!("{}: {e}", probe_label(probe))));
                        self.done = true;
                        return;
                    }
                }
            }
            let run = self.active.as_mut().unwrap();
            run.tick(1);
            if !run.done() {
                continue;
            }
            let (pass, message) = match run.take_result().unwrap() {
                Err(error) => (false, format!("{}: {error}", probe_label(probe))),
                Ok(result) => {
                    let mut actual: Vec<_> = result
                        .records
                        .iter()
                        .map(|r| ExpectedRun {
                            owner: r.owner.clone(),
                            stdout: r.stdout.clone(),
                            stderr: r.stderr.clone(),
                            status: r.status,
                        })
                        .collect();
                    let mut expected = probe.expected.clone();
                    actual.sort();
                    expected.sort();
                    let pass = actual == expected && result.state.files == probe.after;
                    let reason = if actual.len() != expected.len() {
                        format!(
                            "{} job starts; expected {}. Check the calendar fields and unwanted extra entries.",
                            actual.len(),
                            expected.len()
                        )
                    } else if actual.iter().map(|r| &r.owner).collect::<Vec<_>>()
                        != expected.iter().map(|r| &r.owner).collect::<Vec<_>>()
                    {
                        format!(
                            "Run identity differs: expected {:?}; found {:?}. User tables use their owner; system tables need an explicit supplied username.",
                            expected
                                .iter()
                                .map(|r| r.owner.as_str())
                                .collect::<Vec<_>>(),
                            actual.iter().map(|r| r.owner.as_str()).collect::<Vec<_>>()
                        )
                    } else if actual != expected {
                        "Command output/status differs. Inspect variables, quoting, printf formatting and the command's error output.".into()
                    } else if result.state.files != probe.after {
                        let path = probe
                            .after
                            .keys()
                            .chain(result.state.files.keys())
                            .find(|path| probe.after.get(*path) != result.state.files.get(*path))
                            .unwrap();
                        format!(
                            "Simulated file {path}: expected {}; found {}. Check append versus replace, destination and use of the current sample input.",
                            preview(probe.after.get(path)),
                            preview(result.state.files.get(path))
                        )
                    } else {
                        "Scheduled behavior and complete simulated files match.".into()
                    };
                    (pass, format!("{}: {reason}", probe_label(probe)))
                }
            };
            self.rows.push((pass, message));
            self.active = None;
            self.index += 1;
        }
    }
    pub fn cancel(&mut self) {
        if !self.done {
            self.active = None;
            self.rows
                .push((false, "Check stopped; run it again when ready".into()));
            self.done = true;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunEvent {
    pub when: Moment,
    #[serde(deserialize_with = "core::text::<_,32>")]
    pub owner: String,
    #[serde(deserialize_with = "core::text::<_,65536>")]
    pub stdout: String,
    #[serde(deserialize_with = "core::text::<_,65536>")]
    pub stderr: String,
    pub status: i32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunLog {
    pub start: Moment,
    pub end: Moment,
    #[serde(deserialize_with = "sequence::<_,RunEvent,64>")]
    pub events: Vec<RunEvent>,
}
impl RunLog {
    fn valid(&self, users: &BTreeSet<String>) -> bool {
        self.start.validate().is_ok()
            && self.end.validate().is_ok()
            && self.start < self.end
            && self.events.len() <= core::MAX_RUNS
            && self.events.iter().all(|r| {
                r.when.validate().is_ok()
                    && r.when > self.start
                    && r.when <= self.end
                    && users.contains(&r.owner)
                    && (0..=255).contains(&r.status)
                    && !r.stdout.contains('\0')
                    && !r.stderr.contains('\0')
            })
            && self
                .events
                .iter()
                .try_fold(0usize, |n, r| {
                    n.checked_add(r.stdout.len())?.checked_add(r.stderr.len())
                })
                .is_some_and(|n| n <= core::MAX_RUN_OUTPUT)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub initial: State,
    pub selected: Document,
    #[serde(deserialize_with = "core::text::<_,32>")]
    pub actor: String,
    #[serde(deserialize_with = "core::map::<_,8,8192>")]
    pub drafts: BTreeMap<String, String>,
    #[serde(deserialize_with = "core::text::<_,32>")]
    pub window_end: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<RunLog>,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self, String> {
        Self::from_state(d, &d.initial)
    }
    pub fn from_state(d: &Definition, state: &State) -> Result<Self, String> {
        d.validate()?;
        if !d.bound_state(state) {
            return Err("Scheduling supplied state differs; existing work retained".into());
        }
        Ok(Self {
            initial: d.initial.clone(),
            selected: d.editable[0].clone(),
            actor: if d.initial.users.contains("alex") {
                "alex"
            } else {
                "root"
            }
            .into(),
            drafts: d
                .editable
                .iter()
                .map(|t| (t.key(), t.installed(state)))
                .collect(),
            window_end: state.clock.after(60).unwrap_or(state.clock).text(),
            last_run: None,
        })
    }
    pub fn valid(&self) -> bool {
        self.initial.validate().is_ok()
            && self.selected.valid_for(&self.initial)
            && self.initial.users.contains(&self.actor)
            && self.drafts.len() <= 8
            && self
                .drafts
                .values()
                .all(|v| v.len() <= core::MAX_TABLE_BYTES && !v.contains('\0'))
            && self.window_end.len() <= 32
            && !self.window_end.contains('\0')
            && self
                .last_run
                .as_ref()
                .is_none_or(|r| r.valid(&self.initial.users))
            && within(self, MAX_SESSION)
    }
    pub fn valid_for(&self, d: &Definition) -> bool {
        self.valid()
            && self.initial == d.initial
            && d.editable.contains(&self.selected)
            && self.drafts.keys().cloned().collect::<BTreeSet<_>>()
                == d.editable.iter().map(Document::key).collect()
    }
    pub fn applied(&self, d: &Definition, state: &State) -> bool {
        self.valid_for(d) && self.installed_matches(d, state)
    }
    /// Read-only comparison for an already admitted complete native context.
    /// This does not validate/admit a Session and is never completion authority.
    /// Keeping validation separate prevents JSON metering on idle UI frames.
    pub fn installed_matches(&self, d: &Definition, state: &State) -> bool {
        d.editable.iter().all(|t| {
            self.drafts.get(&t.key()).is_some_and(|draft| match t {
                Document::Allow | Document::Deny => {
                    let current = if *t == Document::Allow {
                        state.allow.as_ref()
                    } else {
                        state.deny.as_ref()
                    };
                    match current {
                        // An absent file has an initially empty editor. An
                        // explicit Install of that empty draft creates an
                        // empty policy file; the core keeps None/Some distinct.
                        None => draft.is_empty(),
                        Some(current) => {
                            policy(draft, &d.initial).is_ok_and(|parsed| parsed == *current)
                        }
                    }
                }
                Document::User { owner } => {
                    draft.as_str() == state.tables.get(owner).map(String::as_str).unwrap_or("")
                }
                Document::System => draft == &state.system_table,
            })
        })
    }
}
/// Explicit checking includes unsaved drafts. It never awards credit for a
/// successful installed table while a different broken edit is on screen.
pub fn checks_with_session(
    d: &Definition,
    state: &State,
    session: &Session,
) -> Vec<(bool, String)> {
    let mut rows = vec![(
        session.applied(d, state),
        "All edited documents are installed; pending drafts are not the running schedule".into(),
    )];
    rows.extend(d.checks(state));
    rows
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Edit {
        target: Document,
        #[serde(deserialize_with = "core::text::<_,8192>")]
        text: String,
    },
    Actor {
        #[serde(deserialize_with = "core::text::<_,32>")]
        actor: String,
    },
    Select {
        target: Document,
    },
    Install {
        target: Document,
    },
    Remove {
        target: Document,
    },
    Run {
        end: Moment,
    },
}
impl Action {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_ACTION {
            return Err("Scheduling action exceeds its raw bound".into());
        }
        serde_json::from_str(raw)
            .map_err(|_| "Unknown, duplicate, excessive or malformed scheduling action".into())
    }
}
pub fn apply(
    d: &Definition,
    state: &mut State,
    session: &mut Session,
    action: &Action,
) -> Result<(), String> {
    if !session.valid_for(d) || !d.bound_state(state) {
        return Err("Scheduling context differs; previous work retained".into());
    }
    let target = match action {
        Action::Edit { target, .. }
        | Action::Select { target }
        | Action::Install { target }
        | Action::Remove { target } => Some(target),
        _ => None,
    };
    if target.is_some_and(|t| !d.editable.contains(t)) {
        return Err("This document is not an editable part of the assignment".into());
    }
    match action {
        Action::Edit { target, text } => {
            if text.len() > core::MAX_TABLE_BYTES || text.contains('\0') {
                return Err(
                    "Keep this document within 8 KiB; the previous draft is retained".into(),
                );
            }
            let before = session.drafts.get(&target.key()).unwrap();
            let current = size(session, MAX_SESSION).ok_or("Saved scheduling capacity reached")?;
            if current - string_bytes(before) + string_bytes(text) > MAX_SESSION {
                return Err("Saved scheduling capacity reached; previous draft retained".into());
            }
            session.drafts.insert(target.key(), text.clone());
        }
        Action::Actor { actor } => {
            if !state.users.contains(actor) {
                return Err("Choose a supplied account".into());
            }
            session.actor = actor.clone();
        }
        Action::Select { target } => session.selected = target.clone(),
        Action::Install { target } => {
            let text = &session.drafts[&target.key()];
            match target {
                Document::User { owner } => state.install_user(&session.actor, owner, text),
                Document::System => state.install_system(&session.actor, text),
                Document::Allow | Document::Deny => {
                    let names = policy(text, &d.initial)?;
                    let (allow, deny) = if *target == Document::Allow {
                        (Some(&names), state.deny.as_ref())
                    } else {
                        (state.allow.as_ref(), Some(&names))
                    };
                    let allow = allow.cloned();
                    let deny = deny.cloned();
                    state.set_access(&session.actor, allow.as_ref(), deny.as_ref())
                }
            }
            .map_err(|e| e.to_string())?;
        }
        Action::Remove { target } => {
            match target {
                Document::User { owner } => state.remove_user(&session.actor, owner),
                Document::System => state.install_system(&session.actor, ""),
                Document::Allow | Document::Deny => {
                    let allow = if *target == Document::Allow {
                        None
                    } else {
                        state.allow.clone()
                    };
                    let deny = if *target == Document::Deny {
                        None
                    } else {
                        state.deny.clone()
                    };
                    state.set_access(&session.actor, allow.as_ref(), deny.as_ref())
                }
            }
            .map_err(|e| e.to_string())?;
            session.drafts.insert(target.key(), String::new());
        }
        Action::Run { end } => {
            if !session.applied(d, state) {
                return Err(
                    "Install or discard pending drafts before running the installed schedule"
                        .into(),
                );
            }
            let mut run = core::Run::begin(state, *end).map_err(|e| e.to_string())?;
            while !run.done() {
                run.tick(core::MAX_TICK);
            }
            *state = run.take_result().unwrap().map_err(|e| e.to_string())?.state;
        }
    }
    Ok(())
}

/// Self-contained authoring/test codec. Native editing uses Session and real
/// Install/Remove buttons; this is explicitly not a Linux shell command.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ReferenceAction {
    Install {
        #[serde(deserialize_with = "core::text::<_,32>")]
        actor: String,
        target: Document,
        #[serde(deserialize_with = "core::text::<_,8192>")]
        text: String,
    },
    Remove {
        #[serde(deserialize_with = "core::text::<_,32>")]
        actor: String,
        target: Document,
    },
    Run {
        end: Moment,
    },
}
impl ReferenceAction {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_ACTION {
            return Err("Scheduling action exceeds its raw bound".into());
        }
        serde_json::from_str(raw)
            .map_err(|_| "Unknown, duplicate, excessive or malformed scheduling action".into())
    }
}
/// Bounded synchronous fallback for declarative reachability and trusted tests.
/// The actual native Run/Stop workspace owns core::Run and yields every frame.
pub fn execute(d: &Definition, state: &mut State, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + MAX_ACTION {
        return Err("Scheduling input exceeds its raw bound before trimming".into());
    }
    d.validate()?;
    if !d.bound_state(state) {
        return Err("Scheduling context differs; previous work retained".into());
    }
    if input.trim() == "help" {
        return Ok(HELP.into());
    }
    let raw=input.trim().strip_prefix(PREFIX).ok_or("Scheduling uses native editors; schedule JSON is an internal simulator-only authoring codec")?;
    match ReferenceAction::from_json(raw)? {
        ReferenceAction::Install {
            actor,
            target,
            text,
        } => {
            let mut session = Session::new(d)?;
            apply(d, state, &mut session, &Action::Actor { actor })?;
            apply(
                d,
                state,
                &mut session,
                &Action::Edit {
                    target: target.clone(),
                    text,
                },
            )?;
            apply(d, state, &mut session, &Action::Install { target })?;
            Ok("Installed the supplied virtual scheduling document".into())
        }
        ReferenceAction::Remove { actor, target } => {
            let mut session = Session::new(d)?;
            apply(d, state, &mut session, &Action::Actor { actor })?;
            apply(d, state, &mut session, &Action::Remove { target })?;
            Ok("Removed the installed virtual scheduling document".into())
        }
        ReferenceAction::Run { end } => {
            let mut session = Session::new(d)?;
            // New authoring session starts from current installed text; pending
            // native drafts are deliberately absent from this codec.
            for target in &d.editable {
                session.drafts.insert(target.key(), target.installed(state));
            }
            apply(d, state, &mut session, &Action::Run { end })?;
            Ok(format!(
                "Advanced the simulated UTC clock to {}",
                state.clock.text()
            ))
        }
    }
}

/// Exact authored binding for writable, checking and restore paths. Wrong virtual
/// output files remain admitted; they are diagnosed by the outcome grader.
pub fn context<'a>(
    a: &crate::progress::Attempt,
    s: &'a crate::pack::Scenario,
) -> Result<&'a Definition, String> {
    if s.physical_link_teaching.is_some() {
        return Err("Physical-link teaching belongs to its own activity".into());
    }
    crate::workspace::reject_file_backup_workspace(a)?;
    crate::progress::validate_cron_metadata(a)?;
    if !source_shape(s) || s.version != a.scenario_version || !within(s, MAX_CONTEXT) {
        return Err("Scheduling source, version or complete Scenario bound differs".into());
    }
    let (
        crate::engine::State::Cron {
            simulation: initial,
        },
        crate::engine::Goal::Cron { definition },
        crate::engine::State::Cron { simulation },
    ) = (&s.initial, &s.goal, &a.state)
    else {
        return Err("Scheduling assignment kind differs".into());
    };
    definition.validate()?;
    if initial != &definition.initial || !definition.bound_state(simulation) {
        return Err("Scheduling supplied context differs; previous work retained".into());
    }
    if a.workspace.as_ref().is_some_and(|w| {
        w.cron_session.as_ref().is_some_and(|t| {
            !t.valid_for(definition)
                || t.last_run
                    .as_ref()
                    .is_some_and(|r| r.end > simulation.clock)
        }) || w.cron_learning.as_ref().is_some_and(|l| {
            s.cron_teaching
                .as_ref()
                .is_none_or(|lesson| !l.valid_for(lesson))
        })
    }) {
        return Err(
            "Scheduling draft or separate practice differs from its authored context".into(),
        );
    }
    if let Some(l) = &s.cron_teaching {
        l.validate_context()?;
        if !l.separate_from(definition) {
            return Err(
                "Scheduling example and assignment must use separate supplied users/files".into(),
            );
        }
    }
    Ok(definition)
}

pub fn source_shape(s: &crate::pack::Scenario) -> bool {
    if s.one_off_teaching.is_some() {
        return false;
    }
    let crate::pack::Scenario {
        one_off_teaching: _,
        cron_teaching: _,
        physical_link_teaching,
        file_backup_teaching,
        etherchannel_teaching,
        nfs_teaching,
        nat_teaching,
        rpm_packages_teaching,
        apache_static_teaching,
        spanning_tree_teaching,
        dhcpv4_teaching,
        id: _,
        version: _,
        track: _,
        topic: _,
        unit: _,
        title: _,
        summary: _,
        objective: _,
        retry_guidance: _,
        difficulty: _,
        minutes: _,
        steps: _,
        guide: _,
        explanation: _,
        hints: _,
        sources: _,
        simplifications: _,
        initial: _,
        goal: _,
        solution: _,
        prerequisites: _,
        teaching,
        terminal_teaching,
        process_teaching,
        accounts_teaching,
        unit_manager_teaching,
        ospf_teaching,
        discovery_teaching,
        storage_teaching,
        acl_teaching,
        ipv6_teaching,
        ipv4_routing_teaching,
        address_teaching,
        service_teaching,
        client_teaching,
        seed_dns_teaching,
        seed_mount_teaching,
        legacy_access_teaching,
        legacy_logs_teaching,
        dns_teaching,
        switching_teaching,
        mac_teaching,
        debian_packages_teaching,
    } = s;
    physical_link_teaching.is_none()
        && file_backup_teaching.is_none()
        && etherchannel_teaching.is_none()
        && nfs_teaching.is_none()
        && nat_teaching.is_none()
        && rpm_packages_teaching.is_none()
        && apache_static_teaching.is_none()
        && spanning_tree_teaching.is_none()
        && dhcpv4_teaching.is_none()
        && teaching.is_none()
        && terminal_teaching.is_none()
        && process_teaching.is_none()
        && accounts_teaching.is_none()
        && unit_manager_teaching.is_none()
        && ospf_teaching.is_none()
        && discovery_teaching.is_none()
        && storage_teaching.is_none()
        && acl_teaching.is_none()
        && ipv6_teaching.is_none()
        && ipv4_routing_teaching.is_none()
        && address_teaching.is_none()
        && service_teaching.is_none()
        && client_teaching.is_none()
        && seed_dns_teaching.is_none()
        && seed_mount_teaching.is_none()
        && legacy_access_teaching.is_none()
        && legacy_logs_teaching.is_none()
        && dns_teaching.is_none()
        && switching_teaching.is_none()
        && mac_teaching.is_none()
        && debian_packages_teaching.is_none()
        && matches!(
            (&s.initial, &s.goal),
            (
                crate::engine::State::Cron { .. },
                crate::engine::Goal::Cron { .. }
            )
        )
}

pub fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    if w.one_off_session.is_some() || w.one_off_learning.is_some() {
        return true;
    }
    let crate::workspace::WorkspaceData {
        one_off_session: _,
        one_off_learning: _,
        cron_session: _,
        physical_link_draft,
        physical_link_learning,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
        cron_learning: _,
        nfs_session,
        nfs_learning,
        nat_learning,
        rpm_session,
        rpm_learning,
        nat_draft,
        debian_packages_terminal,
        apache_static_draft,
        apache_static_learning,
        debian_packages_draft,
        debian_packages_learning,
        spanning_tree_draft,
        spanning_tree_learning,
        dhcpv4_draft,
        dhcpv4_learning,
        version: _,
        unit_manager_draft,
        unit_manager_learning,
        unit_manager_terminal,
        ospf_draft,
        ospf_learning,
        acl_draft,
        acl_learning,
        script_learning,
        script_history: _,
        script_history_more: _,
        terminal_learning,
        process_learning,
        accounts_learning,
        discovery_draft,
        discovery_learning,
        storage_draft,
        storage_learning,
        ipv6_draft,
        ipv6_learning,
        routing_v4_draft,
        routing_v4_learning,
        address_learning,
        service_learning,
        client_learning,
        seed_dns_learning,
        seed_mount_learning,
        legacy_access_learning,
        legacy_logs_learning,
        client_draft,
        dns_learning,
        dns_draft,
        switching_draft,
        switching_learning,
        mac_draft,
        mac_learning,
        command: _,
        service_port: _,
        dns_address: _,
        dns_serial: _,
        mount_device: _,
        checked: _,
        ipv4_check: _,
        practice_check: _,
        migrated_host: _,
        legacy_host_reviewed: _,
        legacy_practice_reviewed: _,
        migrated_practice_host: _,
    } = w;
    physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || file_backup_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || nfs_session.is_some()
        || nfs_learning.is_some()
        || nat_learning.is_some()
        || rpm_session.is_some()
        || rpm_learning.is_some()
        || nat_draft.is_some()
        || debian_packages_terminal.is_some()
        || apache_static_draft.is_some()
        || apache_static_learning.is_some()
        || debian_packages_draft.is_some()
        || debian_packages_learning.is_some()
        || spanning_tree_draft.is_some()
        || spanning_tree_learning.is_some()
        || dhcpv4_draft.is_some()
        || dhcpv4_learning.is_some()
        || unit_manager_draft.is_some()
        || unit_manager_learning.is_some()
        || unit_manager_terminal.is_some()
        || ospf_draft.is_some()
        || ospf_learning.is_some()
        || acl_draft.is_some()
        || acl_learning.is_some()
        || script_learning.is_some()
        || terminal_learning.is_some()
        || process_learning.is_some()
        || accounts_learning.is_some()
        || discovery_draft.is_some()
        || discovery_learning.is_some()
        || storage_draft.is_some()
        || storage_learning.is_some()
        || ipv6_draft.is_some()
        || ipv6_learning.is_some()
        || routing_v4_draft.is_some()
        || routing_v4_learning.is_some()
        || address_learning.is_some()
        || service_learning.is_some()
        || client_learning.is_some()
        || seed_dns_learning.is_some()
        || seed_mount_learning.is_some()
        || legacy_access_learning.is_some()
        || legacy_logs_learning.is_some()
        || client_draft.is_some()
        || dns_learning.is_some()
        || dns_draft.is_some()
        || switching_draft.is_some()
        || switching_learning.is_some()
        || mac_draft.is_some()
        || mac_learning.is_some()
        || w.script_history.is_some()
        || !w.script_history_more.is_empty()
}
