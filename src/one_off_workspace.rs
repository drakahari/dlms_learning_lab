//! Typed, bounded one-off assignments. No host execution; no event-ledger credit.
//! Completion recomputes current state and bounded replayed behavior.
use crate::{cron::Moment, cron_workspace as bounded, one_off as core};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub mod budget;
mod decode;
pub const MAX_CONTEXT: usize = 2 * 1024 * 1024;
pub const MAX_ATTEMPT: usize = 2 * 1024 * 1024;
pub const MAX_CASES: usize = 16;
pub const MAX_ACTION: usize = 64 * 1024;
pub const MAX_REQUIREMENTS: usize = 16;
fn path(p: &str) -> bool {
    p.strip_prefix("/work/").is_some_and(|n| {
        !n.is_empty()
            && n.len() <= 100
            && !matches!(n, "." | "..")
            && n.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    })
}
fn variable(k: &str) -> bool {
    !k.is_empty()
        && k.len() <= 32
        && (k.as_bytes()[0].is_ascii_alphabetic() || k.starts_with('_'))
        && k.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !k.starts_with("LC_")
        && !matches!(
            k,
            "HOME"
                | "SHELL"
                | "USER"
                | "LOGNAME"
                | "PWD"
                | "PATH"
                | "IFS"
                | "LANG"
                | "LANGUAGE"
                | "TZ"
        )
        && crate::shell::cron::environment_name(k).is_ok()
}
fn bounded_map<'de, D: serde::Deserializer<'de>, const N: usize, const K: usize, const V: usize>(
    d: D,
    check: fn(&str) -> bool,
) -> Result<BTreeMap<String, String>, D::Error> {
    use serde::de::{DeserializeSeed, Error as _, MapAccess, Visitor};
    struct Seed<const B: usize>;
    impl<'de, const B: usize> DeserializeSeed<'de> for Seed<B> {
        type Value = String;
        fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
            crate::cron::text::<D, B>(d)
        }
    }
    struct Excess;
    impl<'de> Deserialize<'de> for Excess {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Err(D::Error::custom(
                "One-off map count limit before excess key/value",
            ))
        }
    }
    struct Items<const N: usize, const K: usize, const V: usize> {
        check: fn(&str) -> bool,
    }
    impl<'de, const N: usize, const K: usize, const V: usize> Visitor<'de> for Items<N, K, V> {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most{N} distinct bounded one-off entries")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while out.len() < N {
                let Some(key) = a.next_key_seed(Seed::<K>)? else {
                    return Ok(out);
                };
                if !(self.check)(&key) || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate one-off key before value",
                    ));
                }
                out.insert(key, a.next_value_seed(Seed::<V>)?);
            }
            let _ = a.next_key::<Excess>()?;
            Ok(out)
        }
    }
    d.deserialize_map(Items::<N, K, V> { check })
}
fn environment_map<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    bounded_map::<D, 16, 32, 1024>(d, variable)
}
fn file_map<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    bounded_map::<D, N, 106, 98304>(d, path)
}

fn environment(e: &BTreeMap<String, String>) -> bool {
    e.len() <= 16
        && e.iter()
            .all(|(k, v)| variable(k) && v.len() <= 1024 && !v.contains('\0'))
}
fn files(f: &BTreeMap<String, String>) -> bool {
    f.len() <= core::MAX_FILES
        && f.iter().all(|(p, t)| path(p) && !t.contains('\0'))
        && f.iter()
            .try_fold(0usize, |n, (p, t)| {
                n.checked_add(p.len())?.checked_add(t.len())
            })
            .is_some_and(|n| n <= core::MAX_FILES_BYTES)
}
// Match the unchanged core's Gregorian UTC window bound without starting or
// cloning a Run during shape/admission validation.
fn ordinal(m: Moment) -> u32 {
    let leap = |y: u16| y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    let days = (2000..m.year)
        .map(|y| 365 + u32::from(leap(y)))
        .sum::<u32>()
        + (1..m.month)
            .map(|month| match month {
                2 => 28 + u32::from(leap(m.year)),
                4 | 6 | 9 | 11 => 30,
                _ => 31,
            })
            .sum::<u32>()
        + u32::from(m.day)
        - 1;
    days * 1440 + u32::from(m.hour) * 60 + u32::from(m.minute)
}
fn window(start: Moment, end: Moment) -> bool {
    start.validate().is_ok()
        && end.validate().is_ok()
        && ordinal(end)
            .checked_sub(ordinal(start))
            .is_some_and(|n| n > 0 && n <= crate::cron::MAX_WINDOW_MINUTES)
}
fn diagnostic<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Maybe;
    impl<'de> serde::de::Visitor<'de> for Maybe {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("optional bounded one-off diagnostic")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            crate::cron::text::<D, 2048>(d).map(Some)
        }
    }
    d.deserialize_option(Maybe)
}
fn paths<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "crate::cron::text::<_,106>")] String);
    bounded::sequence::<D, Name, 8>(d).map(|v| v.into_iter().map(|n| n.0).collect())
}
fn short(t: &str, max: usize) -> bool {
    !t.trim().is_empty() && t.len() <= max && !t.contains('\0')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunProof {
    pub before: core::State,
    pub end: Moment,
}
impl RunProof {
    fn shape(&self) -> bool {
        self.before.validate().is_ok() && window(self.before.clock, self.end)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub simulation: core::State,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<Box<RunProof>>,
}
impl Assignment {
    pub fn new(initial: &core::State) -> Result<Self, String> {
        initial.validate().map_err(|e| e.to_string())?;
        Ok(Self {
            simulation: initial.clone(),
            last_run: None,
        })
    }
    pub fn valid(&self) -> bool {
        self.simulation.validate().is_ok()
            && self
                .last_run
                .as_ref()
                .is_none_or(|p| p.shape() && p.end == self.simulation.clock)
            && bounded::within(self, MAX_ATTEMPT)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Timing {
    Absolute { submitted: Moment, due: Moment },
    AfterSubmission { minutes: u32 },
}
impl Timing {
    fn plan(&self, example: Moment) -> Option<(Moment, Moment)> {
        match self {
            Self::Absolute { submitted, due }
                if submitted.validate().is_ok() && due.validate().is_ok() =>
            {
                Some((*submitted, *due))
            }
            Self::AfterSubmission { minutes } if *minutes <= crate::cron::MAX_WINDOW_MINUTES => {
                Some((example, example.after(*minutes).ok()?))
            }
            _ => None,
        }
    }
    fn matches(&self, j: &core::Job) -> bool {
        match self {
            Self::Absolute { submitted, due } => j.submitted == *submitted && j.due == *due,
            Self::AfterSubmission { minutes } => {
                j.submitted.after(*minutes).is_ok_and(|due| due == j.due)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ProbeWindow {
    Absolute {
        start: Moment,
        end: Moment,
    },
    AfterSubmission {
        start_minutes: u32,
        end_minutes: u32,
    },
}
impl ProbeWindow {
    fn resolve(&self, submitted: Moment) -> Option<(Moment, Moment)> {
        let (start, end) = match self {
            Self::Absolute { start, end } => (*start, *end),
            Self::AfterSubmission {
                start_minutes,
                end_minutes,
            } => (
                submitted.after(*start_minutes).ok()?,
                submitted.after(*end_minutes).ok()?,
            ),
        };
        (start >= submitted && window(start, end)).then_some((start, end))
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ExpectedWhen {
    Absolute { at: Moment },
    Dispatch,
}
impl ExpectedWhen {
    fn resolve(&self, dispatch: Moment) -> Moment {
        match self {
            Self::Absolute { at } => *at,
            Self::Dispatch => dispatch,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub owner: String,
    #[serde(deserialize_with = "crate::cron::text::<_,106>")]
    pub script_path: String,
    pub timing: Timing,
}
impl Selector {
    fn matches(&self, j: &core::Job) -> bool {
        j.owner == self.owner && j.script_path == self.script_path && self.timing.matches(j)
    }
    fn valid(&self, initial: &core::State) -> bool {
        initial.users.contains(&self.owner)
            && path(&self.script_path)
            && self.timing.plan(initial.clock).is_some()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureSource {
    Queued,
    LastRun,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    QueueExactly {
        source: CaptureSource,
        #[serde(deserialize_with = "bounded::sequence::<_,Selector,16>")]
        jobs: Vec<Selector>,
    },
    CurrentFile {
        #[serde(deserialize_with = "crate::cron::text::<_,106>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,98304>")]
        text: String,
    },
    Clock {
        at: Moment,
    },
    CapturedEnvironment {
        source: CaptureSource,
        job: Selector,
        #[serde(deserialize_with = "environment_map")]
        values: BTreeMap<String, String>,
    },
    RanWindow {
        start: Moment,
        end: Moment,
    },
    Dispatched {
        job: Selector,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedEvent {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub owner: String,
    pub when: ExpectedWhen,
    pub status: core::Status,
    pub exit_status: Option<i32>,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub stdout: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub stderr: String,
    #[serde(default, deserialize_with = "diagnostic")]
    pub error: Option<String>,
}
impl ExpectedEvent {
    fn matches(&self, r: &core::Record, dispatch: Moment) -> bool {
        self.owner == r.owner
            && self.when.resolve(dispatch) == r.when
            && self.status == r.status
            && self.exit_status == r.exit_status
            && self.stdout == r.stdout
            && self.stderr == r.stderr
            && self.error == r.error
    }
    fn valid(&self, initial: &core::State) -> bool {
        initial.users.contains(&self.owner)
            && match self.when {
                ExpectedWhen::Absolute { at } => at.validate().is_ok(),
                ExpectedWhen::Dispatch => true,
            }
            && self.stdout.len() + self.stderr.len() <= 32768
            && !self.stdout.contains('\0')
            && !self.stderr.contains('\0')
            && self
                .error
                .as_ref()
                .is_none_or(|s| s.len() <= 2048 && !s.contains('\0'))
            && match self.status {
                core::Status::Completed => self.exit_status == Some(0) && self.error.is_none(),
                core::Status::Failed => self.exit_status.is_some_and(|n| (1..=255).contains(&n)),
                core::Status::Removed => false,
            }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub name: String,
    pub source: CaptureSource,
    pub job: Selector,
    pub window: ProbeWindow,
    #[serde(deserialize_with = "file_map::<_,32>")]
    pub files: BTreeMap<String, String>,
    pub expected: Option<ExpectedEvent>,
    #[serde(deserialize_with = "file_map::<_,32>")]
    pub after: BTreeMap<String, String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: core::State,
    #[serde(deserialize_with = "paths")]
    pub editable: Vec<String>,
    #[serde(deserialize_with = "paths")]
    pub outputs: Vec<String>,
    #[serde(deserialize_with = "bounded::sequence::<_,u32,16>")]
    pub preserve_jobs: Vec<u32>,
    #[serde(deserialize_with = "bounded::sequence::<_,Requirement,16>")]
    pub requirements: Vec<Requirement>,
    #[serde(deserialize_with = "bounded::sequence::<_,Probe,16>")]
    pub probes: Vec<Probe>,
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_CONTEXT {
            return Err("One-off context exceeds2MiB before decoding".into());
        }
        let out: Self =
            serde_json::from_slice(raw).map_err(|_| "Malformed/excessive one-off context")?;
        out.validate()?;
        Ok(out)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.to_string())?;
        if !bounded::within(self, MAX_CONTEXT)
            || self.editable.is_empty()
            || self.editable.len() > 8
            || self.editable.iter().collect::<BTreeSet<_>>().len() != self.editable.len()
            || self
                .editable
                .iter()
                .any(|p| !path(p) || !self.initial.files.contains_key(p))
            || self.outputs.len() > 8
            || self.outputs.iter().collect::<BTreeSet<_>>().len() != self.outputs.len()
            || self
                .outputs
                .iter()
                .any(|p| !path(p) || self.editable.contains(p))
            || self.preserve_jobs.len() > 16
            || self.preserve_jobs.iter().collect::<BTreeSet<_>>().len() != self.preserve_jobs.len()
            || self
                .preserve_jobs
                .iter()
                .any(|id| !self.initial.jobs.contains_key(id))
            || self.requirements.is_empty()
            || self.requirements.len() > MAX_REQUIREMENTS
            || !(2..=MAX_CASES).contains(&self.probes.len())
        {
            return Err("Invalid bounded one-off assignment".into());
        }
        for r in &self.requirements {
            let valid = match r {
                Requirement::QueueExactly { jobs, .. } => {
                    jobs.len() <= 16
                        && jobs.iter().all(|j| j.valid(&self.initial))
                        && jobs.iter().enumerate().all(|(n, j)| !jobs[..n].contains(j))
                }
                Requirement::CurrentFile { path, text } => {
                    self.outputs.contains(path)
                        && text.len() <= core::MAX_FILES_BYTES
                        && !text.contains('\0')
                }
                Requirement::Clock { at } => at.validate().is_ok(),
                Requirement::CapturedEnvironment { job, values, .. } => {
                    job.valid(&self.initial) && environment(values)
                }
                Requirement::RanWindow { start, end } => window(*start, *end),
                Requirement::Dispatched { job } => job.valid(&self.initial),
            };
            if !valid {
                return Err("Invalid one-off requirement binding".into());
            }
        }
        for (n, p) in self.probes.iter().enumerate() {
            let bound_queue=self.requirements.iter().any(|r|matches!(r,Requirement::QueueExactly{source,jobs} if *source==p.source && jobs.contains(&p.job)));
            let bound_window = p.source != CaptureSource::LastRun
                || self.requirements.iter().any(|r| {
                    matches!(r, Requirement::RanWindow { .. })
                        || matches!(r,Requirement::Dispatched{job} if *job==p.job)
                });
            let duplicate = self.probes[..n].iter().any(|q| {
                q.source == p.source
                    && q.job == p.job
                    && q.window == p.window
                    && q.files == p.files
                    && q.expected == p.expected
                    && q.after == p.after
            });
            if !bound_queue || !bound_window || duplicate {
                return Err("Fresh one-off cases need explicit complete queue/window requirements and distinct behavioral inputs".into());
            }
            let (submitted, due) = p
                .job
                .timing
                .plan(self.initial.clock)
                .ok_or("Invalid one-off timing")?;
            let (start, end) = p
                .window
                .resolve(submitted)
                .ok_or("Invalid one-off fresh time window")?;
            if !short(&p.name, 120)
                || self.probes[..n].iter().any(|q| q.name == p.name)
                || !p.job.valid(&self.initial)
                || !files(&p.files)
                || !files(&p.after)
                || p.expected.as_ref().is_some_and(|r| {
                    !r.valid(&self.initial)
                        || r.owner != p.job.owner
                        || r.when.resolve(due.max(start)) != due.max(start)
                })
                || (p.expected.is_some() != (due <= end))
            {
                return Err("Invalid fresh captured-job case".into());
            }
        }
        let tested = |source: CaptureSource, job: &Selector| {
            self.probes
                .iter()
                .filter(|p| p.source == source && p.job == *job && p.expected.is_some())
                .count()
                >= 2
        };
        let bound = |source: CaptureSource, job: &Selector| {
            self.requirements.iter().any(|r|matches!(r,Requirement::QueueExactly{source:expected,jobs} if *expected==source && jobs.contains(job)))
        };
        for requirement in &self.requirements {
            match requirement {
                Requirement::QueueExactly{source,jobs}=>for job in jobs {
                    let protected=self.preserve_jobs.iter().any(|id|job.matches(&self.initial.jobs[id]));
                    if !protected && !tested(*source,job){return Err("Each assessed captured job needs two distinct due-run cases; protected unchanged peer jobs are separate".into())}
                },
                Requirement::CapturedEnvironment{source,job,..} if !bound(*source,job) || !tested(*source,job)=>return Err("Captured variables need the same complete queue binding and behavioral cases".into()),
                Requirement::Dispatched{job} if !bound(CaptureSource::LastRun,job) || !tested(CaptureSource::LastRun,job)=>return Err("Completed jobs need a last-run queue binding and multiple fresh cases".into()),
                _=>{},
            }
        }
        Ok(())
    }
    pub fn bound(&self, a: &Assignment) -> bool {
        a.valid()
            && a.simulation.users == self.initial.users
            && a.last_run
                .as_ref()
                .is_none_or(|p| p.before.users == self.initial.users)
    }
    fn preserved(&self, s: &core::State) -> bool {
        s.users == self.initial.users
            && self.initial.files.iter().all(|(p, t)| {
                (self.editable.contains(p) && s.files.contains_key(p))
                    || self.outputs.contains(p)
                    || s.files.get(p) == Some(t)
            })
            && s.files
                .keys()
                .all(|p| self.initial.files.contains_key(p) || self.outputs.contains(p))
            && self
                .preserve_jobs
                .iter()
                .all(|id| s.jobs.get(id) == self.initial.jobs.get(id))
    }
    fn capture<'a>(&self, a: &'a Assignment, source: CaptureSource) -> Option<&'a core::State> {
        match source {
            CaptureSource::Queued => Some(&a.simulation),
            CaptureSource::LastRun => a.last_run.as_ref().map(|p| &p.before),
        }
    }
    fn selected<'a>(
        &self,
        a: &'a Assignment,
        source: CaptureSource,
        selector: &Selector,
    ) -> Option<&'a core::Job> {
        let state = self.capture(a, source)?;
        let mut found = state.jobs.values().filter(|j| selector.matches(j));
        let job = found.next()?;
        found.next().is_none().then_some(job)
    }
    fn exact_queue(&self, a: &Assignment, source: CaptureSource, jobs: &[Selector]) -> bool {
        let Some(state) = self.capture(a, source) else {
            return false;
        };
        if state.jobs.len() != jobs.len() {
            return false;
        }
        let mut selected = BTreeSet::new();
        jobs.iter().all(|selector| {
            self.selected(a, source, selector)
                .is_some_and(|job| selected.insert(job.id))
        })
    }
    fn current(&self, a: &Assignment) -> Vec<(bool, String)> {
        let s = &a.simulation;
        let mut rows = vec![(
            self.preserved(s),
            "Supplied accounts, protected queued work and unrelated files are preserved".into(),
        )];
        for r in &self.requirements {
            rows.push(match r {
            Requirement::QueueExactly{source,jobs}=>(self.exact_queue(a,*source,jobs),"The requested queue snapshot has exactly the required owners, submission times, due times and source filenames".into()),
            Requirement::CurrentFile{path,text}=>(s.files.get(path)==Some(text),format!("Current simulated file {path} contains the requested result")),
            Requirement::Clock{at}=>(s.clock==*at,format!("Virtual UTC clock is {}",at.text())),
            Requirement::CapturedEnvironment{source,job,values}=>(self.selected(a,*source,job).is_some_and(|j|j.environment==*values),format!("{} job captured the requested ordinary variable values",job.owner)),
            Requirement::RanWindow{start,end}=>(a.last_run.as_ref().is_some_and(|p|p.before.clock==*start && p.end==*end),"The requested virtual run window has replay-verifiable evidence".into()),
            Requirement::Dispatched{job}=>(self.selected(a,CaptureSource::LastRun,job).is_some_and(|captured|a.last_run.as_ref().is_some_and(|p|p.end>=captured.due) && !s.jobs.contains_key(&captured.id)),"Run the captured job through its due time; completion is verified by whole-state replay".into()),
        });
        }
        rows
    }
    pub fn row_count(&self) -> usize {
        2 + self.requirements.len() + self.probes.len()
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
    pub fn checks(&self, a: &Assignment) -> Vec<(bool, String)> {
        let Ok(mut check) = Check::new(self, a) else {
            return vec![(false, "Invalid one-off context; work retained".into())];
        };
        while !check.done() {
            check.tick(core::MAX_TICK);
        }
        check.rows
    }
}
fn preview(text: Option<&String>) -> String {
    text.map_or_else(
        || "no text/file".into(),
        |text| {
            let shown = text.chars().take(96).collect::<String>();
            format!(
                "{shown:?}{}",
                if shown.len() < text.len() { "…" } else { "" }
            )
        },
    )
}
/// Checker replay is in-process, serial, bounded and cancellable. It never
/// publishes virtual files and never trusts a serialized success flag/old log.
pub struct Check {
    definition: Definition,
    assignment: Assignment,
    phase: usize,
    active: Option<core::Run>,
    pub rows: Vec<(bool, String)>,
    done: bool,
}
impl Check {
    pub fn new(d: &Definition, a: &Assignment) -> Result<Self, String> {
        d.validate()?;
        if !d.bound(a) || !bounded::within(&(d, a), MAX_CONTEXT) {
            return Err("One-off complete context exceeds its bound".into());
        }
        Ok(Self {
            definition: d.clone(),
            assignment: a.clone(),
            phase: 0,
            active: None,
            rows: d.current(a),
            done: false,
        })
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn cancel(&mut self) {
        if self.done {
            return;
        }
        self.active = None;
        self.done = true;
        self.rows.push((
            false,
            "Check stopped; previous learner work retained".into(),
        ));
    }
    pub fn tick(&mut self, budget: usize) {
        if self.done {
            return;
        }
        for _ in 0..budget.min(core::MAX_TICK) {
            if self.phase == 0 {
                let Some(proof) = &self.assignment.last_run else {
                    self.rows
                        .push((true, "No completed-window evidence claimed".into()));
                    self.phase = 1;
                    continue;
                };
                if self.active.is_none() {
                    match core::Run::begin(&proof.before, proof.end) {
                        Ok(run) => self.active = Some(run),
                        Err(e) => {
                            self.rows
                                .push((false, format!("Cannot replay saved virtual window: {e}")));
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
                let pass = run
                    .take_result()
                    .is_some_and(|r| r.is_ok_and(|s| s == self.assignment.simulation));
                self.rows.push((pass,if pass {"Saved captured jobs and live files reproduce the entire current virtual state"}else{"The saved run does not reproduce current state; repair the work and run again"}.into()));
                self.active = None;
                if !pass {
                    self.done = true;
                    return;
                }
                self.phase = 1;
                continue;
            }
            let index = self.phase - 1;
            if index >= self.definition.probes.len() {
                self.done = true;
                return;
            }
            let p = &self.definition.probes[index];
            if self.active.is_none() {
                let Some(job) = self.definition.selected(&self.assignment, p.source, &p.job) else {
                    self.rows.push((
                        false,
                        format!(
                            "{}: no unique matching captured job; inspect owner and due time",
                            p.name
                        ),
                    ));
                    self.phase += 1;
                    continue;
                };
                let Some((start, end)) = p.window.resolve(job.submitted) else {
                    self.rows.push((
                        false,
                        format!(
                            "{}: this fresh time window is outside the supported UTC range",
                            p.name
                        ),
                    ));
                    self.phase += 1;
                    continue;
                };
                // One fresh captured job, no retained records. At most one32KiB
                // Outcome fits the64KiB ledger, so eviction cannot hide a run.
                let mut state = self
                    .definition
                    .capture(&self.assignment, p.source)
                    .unwrap()
                    .clone();
                state.jobs.retain(|id, _| *id == job.id);
                state.records.clear();
                state.files = p.files.clone();
                state.clock = start;
                match core::Run::begin(&state, end) {
                    Ok(run) => self.active = Some(run),
                    Err(e) => {
                        self.rows.push((false, format!("{}: {e}", p.name)));
                        self.phase += 1;
                        continue;
                    }
                }
            }
            let run = self.active.as_mut().unwrap();
            run.tick(1);
            if !run.done() {
                continue;
            }
            let (pass, reason) = match run.take_result().unwrap() {
                Err(e) => (false, format!("{}: {e}", p.name)),
                Ok(result) => {
                    let job = self
                        .definition
                        .selected(&self.assignment, p.source, &p.job)
                        .unwrap();
                    let (start, _) = p.window.resolve(job.submitted).unwrap();
                    let event = match &p.expected {
                        Some(expected) => {
                            result.records.len() == 1
                                && expected.matches(&result.records[0], job.due.max(start))
                                && result.jobs.is_empty()
                        }
                        None => result.records.is_empty() && result.jobs.len() == 1,
                    };
                    let pass = event && result.files == p.after;
                    let detail = if !event {
                        match (&p.expected,result.records.first()) {
                            (None,Some(_))=>"A job unexpectedly ran in this before-due window; inspect its due time".into(),
                            (Some(_),None)=>"No job result remained; inspect the due time and bounded-output diagnostics".into(),
                            (Some(expected),Some(actual)) if expected.owner!=actual.owner=>format!("Run identity differs: expected {}, found {}",expected.owner,actual.owner),
                            (Some(expected),Some(actual)) if expected.when.resolve(job.due.max(start))!=actual.when=>format!("Start time differs: expected {} UTC, found {} UTC",expected.when.resolve(job.due.max(start)).text(),actual.when.text()),
                            (Some(expected),Some(actual)) if expected.status!=actual.status || expected.exit_status!=actual.exit_status || expected.error!=actual.error=>format!("Script exit or diagnostic differs: expected status {:?}, found {:?}; {}",expected.exit_status,actual.exit_status,preview(actual.error.as_ref())),
                            (Some(expected),Some(actual)) if expected.stdout!=actual.stdout=>format!("Standard output differs: expected {}, found {}",preview(Some(&expected.stdout)),preview(Some(&actual.stdout))),
                            (Some(expected),Some(actual)) if expected.stderr!=actual.stderr=>format!("Error output differs: expected {}, found {}",preview(Some(&expected.stderr)),preview(Some(&actual.stderr))),
                            _=>"Queued/result count differs; this sample runs one captured job".into(),
                        }
                    } else if result.files != p.after {
                        let path = p
                            .after
                            .keys()
                            .chain(result.files.keys())
                            .find(|path| p.after.get(*path) != result.files.get(*path))
                            .unwrap();
                        format!(
                            "Simulated file {path} differs: expected {}, found {}. Inputs are read at dispatch",
                            preview(p.after.get(path)),
                            preview(result.files.get(path))
                        )
                    } else {
                        "Captured behavior and complete sample files match".into()
                    };
                    (pass, format!("{}: {detail}", p.name))
                }
            };
            self.rows.push((pass, reason));
            self.active = None;
            self.phase += 1;
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub initial: core::State,
    #[serde(deserialize_with = "crate::cron::text::<_,106>")]
    pub selected: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub actor: String,
    #[serde(deserialize_with = "file_map::<_,8>")]
    pub drafts: BTreeMap<String, String>,
    #[serde(deserialize_with = "environment_map")]
    pub environment: BTreeMap<String, String>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub window_end: String,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub output: String,
}
impl Session {
    pub fn from_state(d: &Definition, a: &Assignment) -> Result<Self, String> {
        d.validate()?;
        if !d.bound(a) || !bounded::within(&(d, a), MAX_CONTEXT) {
            return Err("Invalid one-off supplied context".into());
        }
        if budget::initial_session_size(d, &a.simulation).is_none() {
            return Err("One-off session exceeds its saved bound before copy".into());
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
                .map(|p| {
                    (
                        p.clone(),
                        a.simulation.files.get(p).cloned().unwrap_or_default(),
                    )
                })
                .collect(),
            environment: BTreeMap::new(),
            command: String::new(),
            window_end: a
                .simulation
                .clock
                .after(60)
                .unwrap_or(a.simulation.clock)
                .text(),
            output: String::new(),
        })
    }
    pub fn valid(&self) -> bool {
        self.initial.validate().is_ok()
            && self.initial.users.contains(&self.actor)
            && path(&self.selected)
            && self.drafts.contains_key(&self.selected)
            && self.drafts.len() <= 8
            && files(&self.drafts)
            && environment(&self.environment)
            && self.command.len() <= 2048
            && !self.command.contains(['\0', '\n', '\r'])
            && self.window_end.len() <= 32
            && !self.window_end.contains('\0')
            && self.output.len() <= 16384
            && !self.output.contains('\0')
            && bounded::within(self, MAX_ATTEMPT)
    }
    pub fn valid_for(&self, d: &Definition) -> bool {
        self.valid()
            && self.initial == d.initial
            && self.drafts.keys().collect::<BTreeSet<_>>() == d.editable.iter().collect()
            && d.editable.contains(&self.selected)
    }
    pub fn applied(&self, d: &Definition, a: &Assignment) -> bool {
        self.valid_for(d)
            && d.bound(a)
            && self
                .drafts
                .iter()
                .all(|(p, t)| a.simulation.files.get(p) == Some(t))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    CommandDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    WindowDraft {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        text: String,
    },
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,106>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,98304>")]
        text: String,
    },
    Save {
        #[serde(deserialize_with = "crate::cron::text::<_,106>")]
        path: String,
    },
    Select {
        #[serde(deserialize_with = "crate::cron::text::<_,106>")]
        path: String,
    },
    Actor {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        actor: String,
    },
    Environment {
        #[serde(deserialize_with = "environment_map")]
        values: BTreeMap<String, String>,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Run {
        end: Moment,
    },
}
impl Action {
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_ACTION {
            return Err("One-off action exceeds64KiB before decoding".into());
        }
        serde_json::from_str(raw).map_err(|_| "Malformed/excessive one-off action".into())
    }
}
/// Whole-Attempt guard supplied only by the typed Attempt adapter. Future UI
/// call sites must not invent a byte count. Conservative bounded-state reserve
/// can refuse a near-full profile even when a particular result would be small.
/// It never raises the2MiB persisted limit or copies before admission.
fn reserve(
    whole: usize,
    a: &Assignment,
    s: &Session,
    new_assignment: usize,
    new_session: usize,
) -> Result<(), String> {
    let old_a = bounded::size(a, MAX_ATTEMPT).ok_or("One-off state exceeds saved capacity")?;
    let old_s = bounded::size(s, MAX_ATTEMPT).ok_or("One-off drafts exceed saved capacity")?;
    let projected = whole
        .checked_sub(old_a)
        .and_then(|n| n.checked_sub(old_s))
        .and_then(|n| n.checked_add(new_assignment))
        .and_then(|n| n.checked_add(new_session))
        .and_then(|n| n.checked_add(1));
    if whole > MAX_ATTEMPT || projected.is_none_or(|n| n > MAX_ATTEMPT) {
        return Err("Saved one-off capacity reached; all previous work retained".into());
    }
    Ok(())
}
/// Immediate non-Run actions. Candidate allocation is preceded by conservative
/// complete-Attempt admission. A native Run uses PendingRun and yields per frame.
pub(crate) fn apply(
    d: &Definition,
    a: &mut Assignment,
    s: &mut Session,
    action: &Action,
    whole: usize,
) -> Result<String, String> {
    d.validate()?;
    if !d.bound(a) || !s.valid_for(d) {
        return Err("One-off context differs; prior work retained".into());
    }
    if matches!(action, Action::Run { .. }) {
        return Err("Start the bounded one-off Run continuation".into());
    }
    match action {
        Action::CommandDraft { text } if text.len() > 2048 || text.contains(['\0', '\n', '\r']) => {
            return Err("Use one bounded scheduler-command draft".into());
        }
        Action::WindowDraft { text } if text.len() > 32 || text.contains('\0') => {
            return Err("Keep a bounded UTC window draft".into());
        }
        Action::Edit { path, text }
            if !d.editable.contains(path)
                || text.len() > core::MAX_FILES_BYTES
                || text.contains('\0') =>
        {
            return Err("Edit only a supplied bounded virtual file".into());
        }
        Action::Save { path } | Action::Select { path } if !d.editable.contains(path) => {
            return Err("Select a supplied editable virtual file".into());
        }
        Action::Actor { actor } if !d.initial.users.contains(actor) => {
            return Err("Choose a supplied simulated account".into());
        }
        Action::Environment { values } if !environment(values) => {
            return Err("Unsupported captured variable settings".into());
        }
        Action::Command { text } if text.len() > 2048 || text.contains(['\0', '\n', '\r']) => {
            return Err("Use one bounded scheduler-control command".into());
        }
        _ => {}
    }
    let old_session = bounded::size(s, MAX_ATTEMPT).ok_or("One-off draft capacity")?;
    let changed_session = match action {
        Action::CommandDraft { text } => Some(
            old_session
                .checked_sub(bounded::string_bytes(&s.command))
                .and_then(|n| n.checked_add(bounded::string_bytes(text)))
                .ok_or("One-off command draft capacity")?,
        ),
        Action::WindowDraft { text } => Some(
            old_session
                .checked_sub(bounded::string_bytes(&s.window_end))
                .and_then(|n| n.checked_add(bounded::string_bytes(text)))
                .ok_or("One-off time draft capacity")?,
        ),
        Action::Edit { path, text } => {
            let old = &s.drafts[path];
            let total = s
                .drafts
                .iter()
                .map(|(p, t)| p.len() + t.len())
                .sum::<usize>();
            if total
                .checked_sub(old.len())
                .and_then(|n| n.checked_add(text.len()))
                .is_none_or(|n| n > core::MAX_FILES_BYTES)
            {
                return Err("Virtual file drafts exceed96KiB; previous text retained".into());
            }
            Some(
                old_session
                    .checked_sub(bounded::string_bytes(old))
                    .and_then(|n| n.checked_add(bounded::string_bytes(text)))
                    .ok_or("One-off text capacity")?,
            )
        }
        Action::Select { path } => Some(
            old_session
                .checked_sub(bounded::string_bytes(&s.selected))
                .and_then(|n| n.checked_add(bounded::string_bytes(path)))
                .ok_or("One-off selection capacity")?,
        ),
        Action::Actor { actor } => Some(
            old_session
                .checked_sub(bounded::string_bytes(&s.actor))
                .and_then(|n| n.checked_add(bounded::string_bytes(actor)))
                .ok_or("One-off identity capacity")?,
        ),
        Action::Environment { values } => Some(
            old_session
                .checked_sub(bounded::size(&s.environment, MAX_ATTEMPT).unwrap())
                .and_then(|n| n.checked_add(bounded::size(values, MAX_ATTEMPT)?))
                .ok_or("One-off variable capacity")?,
        ),
        _ => None,
    };
    if let Some(bytes) = changed_session {
        reserve(whole, a, s, bounded::size(a, MAX_ATTEMPT).unwrap(), bytes)?;
        // Only the admitted field is copied. Ordinary typing never clones the
        // current simulation, completed-window proof or other practice drafts.
        match action {
            Action::CommandDraft { text } => s.command = text.clone(),
            Action::WindowDraft { text } => s.window_end = text.clone(),
            Action::Edit { path, text } => {
                s.drafts.insert(path.clone(), text.clone());
            }
            Action::Select { path } => s.selected = path.clone(),
            Action::Actor { actor } => s.actor = actor.clone(),
            Action::Environment { values } => s.environment = values.clone(),
            _ => unreachable!(),
        }
        return Ok(String::new());
    }
    if let Action::Save { path } = action
        && a.simulation.files.get(path) == s.drafts.get(path)
    {
        reserve(
            whole,
            a,
            s,
            bounded::size(a, MAX_ATTEMPT).unwrap(),
            old_session,
        )?;
        return Ok("The simulated file already matches this saved draft".into());
    }
    let state_reserve = match action {
        Action::Save { .. } => core::MAX_WIRE + 64,
        Action::Command { .. } => {
            core::MAX_WIRE
                + bounded::size(&a.last_run, MAX_ATTEMPT).ok_or("Saved one-off proof capacity")?
                + 64
        }
        _ => bounded::size(a, MAX_ATTEMPT).unwrap(),
    };
    let session_reserve = bounded::size(s, MAX_ATTEMPT)
        .unwrap()
        .checked_add(128 * 1024)
        .ok_or("One-off draft capacity")?;
    reserve(whole, a, s, state_reserve, session_reserve)?;
    let mut next_a = a.clone();
    let mut next_s = s.clone();
    match action {
        Action::Edit { path, text } => {
            if !d.editable.contains(path)
                || text.len() > core::MAX_FILES_BYTES
                || text.contains('\0')
            {
                return Err("Edit only a supplied bounded virtual file".into());
            }
            next_s.drafts.insert(path.clone(), text.clone());
        }
        Action::Save { path } => {
            if !d.editable.contains(path) {
                return Err("This virtual file is not editable".into());
            }
            next_a
                .simulation
                .edit_file(path, &next_s.drafts[path])
                .map_err(|e| e.to_string())?;
            if next_a.simulation != a.simulation {
                next_a.last_run = None;
            }
        }
        Action::Select { path } => {
            if !d.editable.contains(path) {
                return Err("Select a supplied editable virtual file".into());
            }
            next_s.selected = path.clone();
        }
        Action::Actor { actor } => {
            if !d.initial.users.contains(actor) {
                return Err("Choose a supplied simulated account".into());
            }
            next_s.actor = actor.clone();
        }
        Action::Environment { values } => {
            if !environment(values) {
                return Err("Use bounded ordinary captured variables; special shell settings are outside this subset".into());
            }
            next_s.environment = values.clone();
        }
        Action::Command { text } => {
            let before = &a.simulation;
            let output = next_a
                .simulation
                .command(&next_s.actor, text, &next_s.environment)
                .map_err(|e| e.to_string())?;
            if output.len() > 16384 || output.contains('\0') {
                return Err("One-off response exceeds its display bound".into());
            }
            if next_a.simulation != *before {
                next_a.last_run = None
            }
            next_s.command = text.clone();
            next_s.output = output;
        }
        Action::Run { .. } | Action::CommandDraft { .. } | Action::WindowDraft { .. } => {
            unreachable!()
        }
    }
    if !d.bound(&next_a) || !next_s.valid_for(d) {
        return Err("One-off action exceeds its bound; prior work retained".into());
    }
    let old_a = bounded::size(a, MAX_ATTEMPT).unwrap();
    let old_s = bounded::size(s, MAX_ATTEMPT).unwrap();
    let actual = whole
        .checked_sub(old_a)
        .and_then(|n| n.checked_sub(old_s))
        .and_then(|n| n.checked_add(bounded::size(&next_a, MAX_ATTEMPT)?))
        .and_then(|n| n.checked_add(bounded::size(&next_s, MAX_ATTEMPT)?))
        .and_then(|n| n.checked_add(1));
    if actual.is_none_or(|n| n > MAX_ATTEMPT) {
        return Err("Saved one-off capacity reached; candidate discarded".into());
    }
    let restarted = a.last_run.is_some() && next_a.last_run.is_none();
    let message = if restarted {
        "Saved changes start a new verification window. Earlier outputs and history are retained; run the repaired job again to verify the current work".into()
    } else if matches!(action, Action::Save { .. }) {
        "Saved the simulated file".into()
    } else {
        next_s.output.clone()
    };
    *a = next_a;
    *s = next_s;
    Ok(message)
}
pub struct PendingRun {
    before: Option<core::State>,
    end: Moment,
    run: core::Run,
}
impl PendingRun {
    pub(crate) fn begin(
        d: &Definition,
        a: &Assignment,
        s: &Session,
        end: Moment,
        whole: usize,
    ) -> Result<Self, String> {
        d.validate()?;
        if !d.bound(a) || !s.valid_for(d) || !s.applied(d, a) || !window(a.simulation.clock, end) {
            return Err("Save pending file edits and choose a valid one-off virtual window".into());
        }
        let can_fold = a
            .last_run
            .as_ref()
            .is_some_and(|p| window(p.before.clock, end));
        let loses_required_capture = !can_fold
            && d.probes
                .iter()
                .filter(|p| p.source == CaptureSource::LastRun)
                .any(|p| {
                    d.selected(a, CaptureSource::LastRun, &p.job)
                        .is_some_and(|j| !a.simulation.jobs.contains_key(&j.id))
                });
        if loses_required_capture {
            return Err("This would exceed the activity's seven-day verification span and discard required completed-job evidence. Existing work is retained; use a shorter window or reset this activity to explore another timeline".into());
        }
        let evidence = a
            .last_run
            .as_ref()
            .filter(|p| window(p.before.clock, end))
            .map_or(&a.simulation, |p| &p.before);
        let before_bytes =
            bounded::size(evidence, core::MAX_WIRE).ok_or("One-off source capacity")?;
        reserve(
            whole,
            a,
            s,
            core::MAX_WIRE + before_bytes + 256,
            bounded::size(s, MAX_ATTEMPT).unwrap(),
        )?;
        Ok(Self {
            before: Some(evidence.clone()),
            end,
            run: core::Run::begin(&a.simulation, end).map_err(|e| e.to_string())?,
        })
    }
    pub fn tick(&mut self, budget: usize) {
        self.run.tick(budget.min(core::MAX_TICK));
    }
    pub fn done(&self) -> bool {
        self.run.done()
    }
    pub fn cancel(&mut self) {
        if !self.run.done() {
            self.run.cancel();
            self.before = None;
        }
    }
    pub(crate) fn take_result(&mut self) -> Option<Result<Assignment, String>> {
        let result = self.run.take_result()?;
        Some(result.map_err(|e| e.to_string()).and_then(|simulation| {
            let before = self
                .before
                .take()
                .ok_or("Stopped one-off window has no committed proof")?;
            let assignment = Assignment {
                simulation,
                last_run: Some(Box::new(RunProof {
                    before,
                    end: self.end,
                })),
            };
            if !assignment.valid() {
                return Err("Finished one-off candidate exceeds saved capacity".into());
            }
            Ok(assignment)
        }))
    }
}

pub fn checks_with_session(d: &Definition, a: &Assignment, s: &Session) -> Vec<(bool, String)> {
    let mut rows = vec![(
        s.applied(d, a),
        "All edited virtual files are saved; unsaved drafts are not the queued source".into(),
    )];
    rows.extend(d.checks(a));
    rows
}
/// Internal declarative authoring codec. It is never taught as a Linux command.
#[derive(Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum ReferenceAction {
    Write {
        #[serde(deserialize_with = "crate::cron::text::<_,106>")]
        path: String,
        #[serde(deserialize_with = "crate::cron::text::<_,98304>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        actor: String,
        #[serde(deserialize_with = "environment_map")]
        environment: BTreeMap<String, String>,
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Run {
        end: Moment,
    },
}
pub const PREFIX: &str = "one-off ";
pub const HELP: &str = "One-off scheduling uses native virtual-file editors, the simulated queue and explicit virtual UTC Run/Stop. All commands stay in process. Internal one-off JSON is an authoring codec, not a Linux command. No host at/daemon/shell/clock or access-policy/timer behavior is claimed.";
pub fn execute(d: &Definition, a: &mut Assignment, input: &str) -> Result<String, String> {
    if input.len() > PREFIX.len() + MAX_ACTION {
        return Err("One-off authoring input exceeds its raw bound".into());
    }
    if input.trim() == "help" {
        return Ok(HELP.into());
    }
    let raw = input.trim().strip_prefix(PREFIX).ok_or(HELP)?;
    let action: ReferenceAction =
        serde_json::from_str(raw).map_err(|_| "Malformed/excessive one-off authoring action")?;
    let mut session = Session::from_state(d, a)?;
    let total = bounded::size(a, MAX_ATTEMPT)
        .and_then(|n| n.checked_add(bounded::size(&session, MAX_ATTEMPT)?))
        .and_then(|n| n.checked_add(256))
        .ok_or("One-off authoring capacity")?;
    match action {
        ReferenceAction::Write { path, text } => {
            apply(
                d,
                a,
                &mut session,
                &Action::Edit {
                    path: path.clone(),
                    text,
                },
                total,
            )?;
            let total = bounded::size(a, MAX_ATTEMPT).unwrap()
                + bounded::size(&session, MAX_ATTEMPT).unwrap()
                + 256;
            apply(d, a, &mut session, &Action::Save { path }, total)
        }
        ReferenceAction::Command {
            actor,
            environment,
            text,
        } => {
            apply(d, a, &mut session, &Action::Actor { actor }, total)?;
            let total = bounded::size(a, MAX_ATTEMPT).unwrap()
                + bounded::size(&session, MAX_ATTEMPT).unwrap()
                + 256;
            apply(
                d,
                a,
                &mut session,
                &Action::Environment {
                    values: environment,
                },
                total,
            )?;
            let total = bounded::size(a, MAX_ATTEMPT).unwrap()
                + bounded::size(&session, MAX_ATTEMPT).unwrap()
                + 256;
            apply(d, a, &mut session, &Action::Command { text }, total)
        }
        ReferenceAction::Run { end } => {
            let mut run = PendingRun::begin(d, a, &session, end, total)?;
            while !run.done() {
                run.tick(core::MAX_TICK)
            }
            let result = run
                .take_result()
                .ok_or("One-off run has no final result")??;
            *a = result;
            Ok("Completed the bounded simulated one-off window".into())
        }
    }
}

// BEGIN GENERATED BINDINGS

pub fn source_shape(s: &crate::pack::Scenario) -> bool {
    let crate::pack::Scenario {
        physical_link_teaching,
        one_off_teaching: _,
        file_backup_teaching,
        etherchannel_teaching,
        cron_teaching,
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
        && cron_teaching.is_none()
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
}
pub fn reject_foreign_source(s: &crate::pack::Scenario) -> Result<(), String> {
    if s.one_off_teaching.is_some()
        || matches!(s.initial, crate::engine::State::OneOff { .. })
        || matches!(s.goal, crate::engine::Goal::OneOff { .. })
    {
        return Err("One-off source belongs to its one-off activity".into());
    }
    Ok(())
}
pub fn foreign(w: &crate::workspace::WorkspaceData) -> bool {
    let crate::workspace::WorkspaceData {
        physical_link_draft,
        physical_link_learning,
        one_off_session: _,
        one_off_learning: _,
        file_backup_learning,
        etherchannel_draft,
        etherchannel_learning,
        cron_session,
        cron_learning,
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
        script_history,
        script_history_more,
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
        command,
        service_port,
        dns_address,
        dns_serial,
        mount_device,
        checked: _,
        ipv4_check,
        practice_check,
        migrated_host,
        legacy_host_reviewed,
        legacy_practice_reviewed,
        migrated_practice_host,
    } = w;
    physical_link_draft.is_some()
        || physical_link_learning.is_some()
        || file_backup_learning.is_some()
        || etherchannel_draft.is_some()
        || etherchannel_learning.is_some()
        || cron_session.is_some()
        || cron_learning.is_some()
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
        || script_history.is_some()
        || !script_history_more.is_empty()
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
        || !command.is_empty()
        || service_port.is_some()
        || dns_address.is_some()
        || dns_serial.is_some()
        || mount_device.is_some()
        || ipv4_check.is_some()
        || practice_check.is_some()
        || *migrated_host
        || *legacy_host_reviewed
        || *legacy_practice_reviewed
        || *migrated_practice_host
}
pub fn metadata(a: &crate::progress::Attempt) -> Result<(), String> {
    let present = crate::workspace::has_one_off_metadata(a);
    let crate::engine::State::OneOff { simulation } = &a.state else {
        return if present {
            Err("One-off metadata on another activity".into())
        } else {
            Ok(())
        };
    };
    if !simulation.valid()
        || a.transcript.len() > 120
        || a.ipv4.is_some()
        || a.workspace
            .as_ref()
            .is_some_and(|w| foreign(w) || !w.valid())
        || !bounded::within(a, MAX_ATTEMPT)
    {
        return Err("One-off state/metadata/wholeAttempt differs; prior work retained".into());
    }
    Ok(())
}
pub fn context<'a>(
    a: &crate::progress::Attempt,
    s: &'a crate::pack::Scenario,
) -> Result<&'a Definition, String> {
    metadata(a)?;
    if !source_shape(s)
        || s.version != a.scenario_version
        || !bounded::within(s, MAX_CONTEXT)
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
    {
        return Err("One-off source/version/wholeScenario differs".into());
    }
    let (
        crate::engine::State::OneOff {
            simulation: initial,
        },
        crate::engine::Goal::OneOff { definition },
        crate::engine::State::OneOff { simulation },
    ) = (&s.initial, &s.goal, &a.state)
    else {
        return Err("One-off authored kinds differ".into());
    };
    definition.validate()?;
    if initial.simulation != definition.initial
        || initial.last_run.is_some()
        || !definition.bound(simulation)
    {
        return Err("One-off initial or current context differs".into());
    }
    if let Some(l) = &s.one_off_teaching {
        l.validate_context()?;
        if !l.separate_from(definition) {
            return Err("One-off practice is not separate".into());
        }
    }
    if a.workspace.as_ref().is_some_and(|w| {
        w.one_off_session
            .as_ref()
            .is_some_and(|t| !t.valid_for(definition))
            || w.one_off_learning.as_ref().is_some_and(|l| {
                s.one_off_teaching
                    .as_ref()
                    .is_none_or(|source| !l.valid_for(source))
            })
    }) {
        return Err("One-off drafts/practice differ from authored context".into());
    }
    Ok(definition)
}
