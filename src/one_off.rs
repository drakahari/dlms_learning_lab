//! Explicit-clock one-off queue. No host time, filesystem, shell or service I/O.
use crate::{cron::Moment, shell};
use serde::{
    Deserialize, Serialize,
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};

pub const MAX_JOBS: usize = 16;
pub const MAX_RECORDS: usize = 64;
pub const MAX_FILES: usize = 32;
pub const MAX_FILES_BYTES: usize = 96 * 1024;
pub const MAX_OUTPUT: usize = 64 * 1024;
pub const MAX_WIRE: usize = 1024 * 1024;
pub const MAX_TICK: usize = 32;
pub const MAX_ID: u32 = 1_000_000;
pub const HELP: &str = "Simulated one-off queue (UTC, virtual /work only): at -f /work/NAME -t YYYYMMDDhhmm; at -f /work/NAME now [+ N minutes|hours|days|weeks]; atq or at -l; atrm ID... or at -r ID...; at -c ID. Queue a only. Script source/environment are captured; virtual input files are read when the job runs. No host at, daemon, spool, shell, clock, mail or network runs.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Syntax,
    Unsupported,
    Permission,
    Limit,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn err(kind: ErrorKind, text: &str) -> Error {
    Error {
        kind,
        message: text.into(),
    }
}
fn valid_user(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c))
}
fn path(s: &str) -> bool {
    s.strip_prefix("/work/").is_some_and(|n| {
        !n.is_empty()
            && n.len() <= 100
            && n != "."
            && n != ".."
            && n.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    })
}
fn files_ok(files: &BTreeMap<String, String>) -> bool {
    files.len() <= MAX_FILES
        && files.iter().all(|(p, v)| path(p) && !v.contains('\0'))
        && files
            .iter()
            .map(|(p, v)| p.len().saturating_add(v.len()))
            .sum::<usize>()
            <= MAX_FILES_BYTES
}
fn environment_key(k: &str) -> bool {
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
        && shell::cron::environment_name(k).is_ok()
}
fn environment_ok(env: &BTreeMap<String, String>) -> bool {
    env.len() <= 16
        && env
            .iter()
            .all(|(k, v)| environment_key(k) && v.len() <= 1024 && !v.contains('\0'))
}
fn bytes<T: Serialize + ?Sized>(value: &T, cap: usize) -> Option<usize> {
    struct Count {
        n: usize,
        cap: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("One-off encoded byte limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Count { n: 0, cap };
    serde_json::to_writer(&mut c, value).ok()?;
    Some(c.n)
}
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    crate::cron::text::<D, N>(d)
}
struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        text::<D, N>(d).map(Self)
    }
}
struct Excess;
impl<'de> DeserializeSeed<'de> for Excess {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, _: D) -> std::result::Result<(), D::Error> {
        Err(D::Error::custom(
            "One-off collection bound before excess body",
        ))
    }
}
fn string_map<'de, D: serde::Deserializer<'de>, const N: usize, const K: usize, const V: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    struct Map<const N: usize, const K: usize, const V: usize>;
    impl<'de, const N: usize, const K: usize, const V: usize> Visitor<'de> for Map<N, K, V> {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} bounded one-off fields")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            loop {
                if out.len() == N {
                    if a.next_key_seed(Excess)?.is_some() {
                        unreachable!()
                    }
                    break;
                }
                let Some(Text(key)) = a.next_key::<Text<K>>()? else {
                    break;
                };
                if out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate one-off field before value"));
                }
                if (K == 106 && !path(&key)) || (K == 32 && !environment_key(&key)) {
                    return Err(A::Error::custom("Invalid one-off field before value"));
                }
                let Text(v) = a.next_value::<Text<V>>()?;
                out.insert(key, v);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Map::<N, K, V>)
}
// JSON object keys stay strings when an enclosing tagged enum buffers them.
// Decode the same bounded canonical decimal key in direct and buffered maps.
struct JobIdKey;
impl<'de> DeserializeSeed<'de> for JobIdKey {
    type Value = u32;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<u32, D::Error> {
        let key = text::<D, 10>(d)?;
        if key.is_empty() || key.starts_with('0') || !key.bytes().all(|b| b.is_ascii_digit()) {
            return Err(D::Error::custom(
                "Invalid/duplicate one-off job ID before value",
            ));
        }
        key.parse::<u32>()
            .ok()
            .filter(|id| *id <= MAX_ID)
            .ok_or_else(|| D::Error::custom("Invalid/duplicate one-off job ID before value"))
    }
}
fn jobs<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<u32, Job>, D::Error> {
    struct Jobs;
    impl<'de> Visitor<'de> for Jobs {
        type Value = BTreeMap<u32, Job>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most 16 one-off jobs")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            loop {
                if out.len() == MAX_JOBS {
                    if a.next_key_seed(Excess)?.is_some() {
                        unreachable!()
                    }
                    break;
                }
                let Some(id) = a.next_key_seed(JobIdKey)? else {
                    break;
                };
                if id == 0 || id > MAX_ID || out.contains_key(&id) {
                    return Err(A::Error::custom(
                        "Invalid/duplicate one-off job ID before value",
                    ));
                }
                out.insert(id, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Jobs)
}
fn list<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} one-off entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            loop {
                if out.len() == N {
                    if a.next_element_seed(Excess)?.is_some() {
                        unreachable!()
                    }
                    break;
                }
                let Some(v) = a.next_element()? else { break };
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(PhantomData))
}
fn users<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    let names = list::<D, Text<32>, 8>(d)?;
    let mut out = BTreeSet::new();
    for Text(n) in names {
        if !valid_user(&n) || !out.insert(n) {
            return Err(D::Error::custom("Invalid/duplicate one-off account"));
        }
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: u32,
    #[serde(deserialize_with = "text::<_,32>")]
    pub owner: String,
    pub submitted: Moment,
    pub due: Moment,
    #[serde(deserialize_with = "text::<_,106>")]
    pub script_path: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub source: String,
    #[serde(deserialize_with = "string_map::<_,16,32,1024>")]
    pub environment: BTreeMap<String, String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Completed,
    Failed,
    Removed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: u32,
    #[serde(deserialize_with = "text::<_,32>")]
    pub owner: String,
    pub when: Moment,
    pub status: Status,
    pub exit_status: Option<i32>,
    #[serde(deserialize_with = "text::<_,32768>")]
    pub stdout: String,
    #[serde(deserialize_with = "text::<_,32768>")]
    pub stderr: String,
    #[serde(default, deserialize_with = "diagnostic")]
    pub error: Option<String>,
}
fn diagnostic<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    struct Maybe;
    impl<'de> Visitor<'de> for Maybe {
        type Value = Option<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("optional bounded one-off diagnostic")
        }
        fn visit_none<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(
            self,
            d: D,
        ) -> std::result::Result<Self::Value, D::Error> {
            text::<D, 2048>(d).map(Some)
        }
    }
    d.deserialize_option(Maybe)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    pub clock: Moment,
    pub next_id: u32,
    #[serde(deserialize_with = "users")]
    pub users: BTreeSet<String>,
    #[serde(deserialize_with = "string_map::<_,32,106,98304>")]
    pub files: BTreeMap<String, String>,
    #[serde(deserialize_with = "jobs")]
    pub jobs: BTreeMap<u32, Job>,
    #[serde(deserialize_with = "list::<_,Record,64>")]
    pub records: Vec<Record>,
}
impl State {
    pub fn from_json(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_WIRE {
            return Err(err(
                ErrorKind::Limit,
                "One-off state exceeds 1 MiB before decoding",
            ));
        }
        let out: Self = serde_json::from_slice(raw)
            .map_err(|_| err(ErrorKind::Invalid, "Malformed or excessive one-off state"))?;
        out.validate()?;
        Ok(out)
    }
    pub fn validate(&self) -> Result<()> {
        self.clock
            .validate()
            .map_err(|_| err(ErrorKind::Invalid, "Invalid one-off virtual UTC clock"))?;
        if self.version != 1
            || self.users.is_empty()
            || self.users.len() > 8
            || !self.users.contains("root")
            || self.users.iter().any(|s| !valid_user(s))
            || !files_ok(&self.files)
            || self.jobs.len() > MAX_JOBS
            || self.records.len() > MAX_RECORDS
            || self.next_id == 0
            || self.next_id > MAX_ID + 1
            || bytes(self, MAX_WIRE).is_none()
        {
            return Err(err(
                ErrorKind::Limit,
                "One-off account/file/queue/history/encoded-state limit",
            ));
        }
        for (id, j) in &self.jobs {
            if *id != j.id
                || *id == 0
                || *id >= self.next_id
                || !self.users.contains(&j.owner)
                || j.submitted.validate().is_err()
                || j.submitted > self.clock
                || j.due.validate().is_err()
                || !path(&j.script_path)
                || j.source.is_empty()
                || j.source.len() > shell::one_off::MAX_SCRIPT
                || j.source.contains(['\0', '\r'])
                || !environment_ok(&j.environment)
            {
                return Err(err(ErrorKind::Invalid, "Invalid captured one-off job"));
            }
        }
        let mut ids = BTreeSet::new();
        for r in &self.records {
            if r.id == 0
                || r.id >= self.next_id
                || !ids.insert(r.id)
                || self.jobs.contains_key(&r.id)
                || !self.users.contains(&r.owner)
                || r.when.validate().is_err()
                || r.when > self.clock
                || r.stdout.len() > 32768
                || r.stderr.len() > 32768
                || r.stdout.contains('\0')
                || r.stderr.contains('\0')
                || r.error
                    .as_ref()
                    .is_some_and(|e| e.len() > 2048 || e.contains('\0'))
                || match r.status {
                    Status::Removed => {
                        r.exit_status.is_some()
                            || !r.stdout.is_empty()
                            || !r.stderr.is_empty()
                            || r.error.is_some()
                    }
                    Status::Completed => r.exit_status != Some(0) || r.error.is_some(),
                    Status::Failed => r.exit_status.is_none_or(|n| !(1..=255).contains(&n)),
                }
            {
                return Err(err(ErrorKind::Invalid, "Invalid one-off result record"));
            }
        }
        if self
            .records
            .iter()
            .map(|r| r.stdout.len() + r.stderr.len() + r.error.as_ref().map_or(0, String::len))
            .sum::<usize>()
            > MAX_OUTPUT
        {
            return Err(err(
                ErrorKind::Limit,
                "One-off retained output exceeds 64KiB",
            ));
        }
        Ok(())
    }
    fn actor(&self, actor: &str) -> Result<()> {
        if self.users.contains(actor) {
            Ok(())
        } else {
            Err(err(
                ErrorKind::Permission,
                "Choose a supplied simulated account",
            ))
        }
    }
    fn owned(&self, actor: &str, id: u32) -> Result<&Job> {
        self.actor(actor)?;
        let j = self
            .jobs
            .get(&id)
            .ok_or_else(|| err(ErrorKind::Invalid, "No such queued job"))?;
        if actor == "root" || j.owner == actor {
            Ok(j)
        } else {
            Err(err(
                ErrorKind::Permission,
                "This queued job belongs to another simulated account",
            ))
        }
    }
    pub fn edit_file(&mut self, p: &str, text: &str) -> Result<()> {
        self.validate()?;
        if !path(p) || text.len() > MAX_FILES_BYTES || text.contains('\0') {
            return Err(err(
                ErrorKind::Invalid,
                "Use an ordinary flat /work file with bounded NUL-free text",
            ));
        }
        let mut next = self.clone();
        next.files.insert(p.into(), text.into());
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn submit(
        &mut self,
        actor: &str,
        p: &str,
        due: Moment,
        env: &BTreeMap<String, String>,
    ) -> Result<u32> {
        self.validate()?;
        self.actor(actor)?;
        due.validate()
            .map_err(|_| err(ErrorKind::Invalid, "Invalid one-off due date"))?;
        if !environment_ok(env) {
            return Err(err(
                ErrorKind::Unsupported,
                "Use at most 16 ordinary captured variables; PATH/cwd/locale/special-variable customization is not modeled",
            ));
        }
        let source = self
            .files
            .get(p)
            .filter(|_| path(p))
            .ok_or_else(|| err(ErrorKind::Invalid, "No such virtual script file"))?;
        if source.is_empty()
            || source.len() > shell::one_off::MAX_SCRIPT
            || source.contains(['\0', '\r'])
        {
            return Err(err(
                ErrorKind::Limit,
                "One-off script must contain 1..8192 bytes without NUL or CR",
            ));
        }
        if self.jobs.len() == MAX_JOBS || self.next_id > MAX_ID {
            return Err(err(ErrorKind::Limit, "One-off queue/identifier limit"));
        }
        let id = self.next_id;
        let job = Job {
            id,
            owner: actor.into(),
            submitted: self.clock,
            due,
            script_path: p.into(),
            source: source.clone(),
            environment: env.clone(),
        };
        let mut next = self.clone();
        next.jobs.insert(id, job);
        next.next_id += 1;
        next.validate()?;
        *self = next;
        Ok(id)
    }
    pub fn queued(&self, actor: &str) -> Result<Vec<&Job>> {
        self.validate()?;
        self.actor(actor)?;
        let mut out = self
            .jobs
            .values()
            .filter(|j| actor == "root" || j.owner == actor)
            .collect::<Vec<_>>();
        out.sort_by_key(|j| (j.due, j.id));
        Ok(out)
    }
    fn record(&mut self, r: Record) {
        self.records.push(r);
        while self.records.len() > MAX_RECORDS
            || self
                .records
                .iter()
                .map(|r| r.stdout.len() + r.stderr.len() + r.error.as_ref().map_or(0, String::len))
                .sum::<usize>()
                > MAX_OUTPUT
        {
            self.records.remove(0);
        }
    }
    pub fn remove(&mut self, actor: &str, ids: &[u32]) -> Result<()> {
        self.validate()?;
        if ids.is_empty()
            || ids.len() > MAX_JOBS
            || ids.iter().collect::<BTreeSet<_>>().len() != ids.len()
        {
            return Err(err(ErrorKind::Syntax, "Name 1..16 distinct queued job IDs"));
        }
        for id in ids {
            self.owned(actor, *id)?;
        }
        let mut next = self.clone();
        for id in ids {
            let j = next.jobs.remove(id).unwrap();
            next.record(Record {
                id: *id,
                owner: j.owner,
                when: next.clock,
                status: Status::Removed,
                exit_status: None,
                stdout: String::new(),
                stderr: String::new(),
                error: None,
            });
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Scheduler-control input only; never parsed as a host/simulated shell line.
    pub fn command(
        &mut self,
        actor: &str,
        input: &str,
        env: &BTreeMap<String, String>,
    ) -> Result<String> {
        self.validate()?;
        self.actor(actor)?;
        if input.len() > 2048 || input.contains(['\0', '\n', '\r']) {
            return Err(err(
                ErrorKind::Limit,
                "Use one scheduler command up to 2 KiB",
            ));
        }
        let args = input.split_whitespace().collect::<Vec<_>>();
        match args.as_slice() {
            ["atq"] | ["at", "-l"] => Ok(self
                .queued(actor)?
                .iter()
                .map(|j| format!("{}\t{} UTC a {}\n", j.id, j.due.text(), j.owner))
                .collect()),
            ["at", "-c", id] => {
                let id = job_id(id)?;
                let j = self.owned(actor, id)?;
                // State validation bounds this captured map to 16 ordinary keys
                // and 1024 bytes per value. JSON quoting keeps embedded newlines,
                // quotes and backslashes visibly inside metadata, not fake shell
                // statements or another script/header. The saved source is exact.
                let environment = serde_json::to_string(&j.environment).map_err(|_| {
                    err(
                        ErrorKind::Invalid,
                        "Captured variables could not be displayed",
                    )
                })?;
                Ok(format!(
                    "# Simulator metadata: owner={}, cwd=/work, due={} UTC\n# Captured variables (JSON values, not shell commands): {}\n# Captured script follows unchanged\n{}",
                    j.owner,
                    j.due.text(),
                    environment,
                    j.source
                ))
            }
            ["atrm", tail @ ..] | ["at", "-r", tail @ ..] => {
                let ids = tail.iter().map(|s| job_id(s)).collect::<Result<Vec<_>>>()?;
                self.remove(actor, &ids)?;
                Ok(String::new())
            }
            ["at", tail @ ..] => {
                let (file, time) = submission(tail)?;
                let due = parse_time(&time, self.clock)?;
                let id = self.submit(actor, file, due, env)?;
                Ok(format!(
                    "job {id} at {} UTC (simulated queue a)\n",
                    due.text()
                ))
            }
            _ => Err(err(ErrorKind::Unsupported, HELP)),
        }
    }
}
fn job_id(s: &str) -> Result<u32> {
    if s.is_empty() || s.len() > 7 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(ErrorKind::Syntax, "Use a decimal queued job ID"));
    }
    s.parse::<u32>()
        .ok()
        .filter(|n| (1..=MAX_ID).contains(n))
        .ok_or_else(|| err(ErrorKind::Invalid, "Queued job ID is out of range"))
}
fn submission<'a>(args: &[&'a str]) -> Result<(&'a str, Vec<&'a str>)> {
    let mut file = None;
    let mut time = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-f" if file.is_none() && i + 1 < args.len() => {
                file = Some(args[i + 1]);
                i += 2;
            }
            "-t" if time.is_none() && i + 1 < args.len() => {
                time = Some(vec!["-t", args[i + 1]]);
                i += 2;
            }
            "now" if time.is_none() => {
                time = Some(args[i..].to_vec());
                break;
            }
            _ => {
                return Err(err(
                    ErrorKind::Unsupported,
                    "Use at -f /work/NAME -t YYYYMMDDhhmm, or at -f /work/NAME now [+ N minutes|hours|days|weeks]",
                ));
            }
        }
    }
    Ok((
        file.ok_or_else(|| {
            err(
                ErrorKind::Unsupported,
                "This subset reads jobs from -f /work/NAME; interactive input is not modeled",
            )
        })?,
        time.ok_or_else(|| err(ErrorKind::Syntax, "Give an explicit supported due time"))?,
    ))
}
pub fn parse_time(args: &[&str], now: Moment) -> Result<Moment> {
    now.validate()
        .map_err(|_| err(ErrorKind::Invalid, "Invalid virtual UTC clock"))?;
    match args {
        ["-t", s] if s.len() == 12 && s.bytes().all(|b| b.is_ascii_digit()) => {
            Moment::parse(&format!(
                "{}-{}-{} {}:{}",
                &s[..4],
                &s[4..6],
                &s[6..8],
                &s[8..10],
                &s[10..12]
            ))
            .map_err(|_| {
                err(
                    ErrorKind::Invalid,
                    "Invalid YYYYMMDDhhmm date in 2000..2099 virtual UTC",
                )
            })
        }
        ["now"] => Ok(now),
        ["now", "+", n, unit]
            if !n.is_empty() && n.len() <= 5 && n.bytes().all(|b| b.is_ascii_digit()) =>
        {
            let value = n
                .parse::<u32>()
                .map_err(|_| err(ErrorKind::Limit, "Relative one-off time is out of range"))?;
            let scale = match *unit {
                "minute" | "minutes" => 1,
                "hour" | "hours" => 60,
                "day" | "days" => 1440,
                "week" | "weeks" => 10080,
                _ => {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "Use minutes, hours, days or weeks",
                    ));
                }
            };
            let mins = value
                .checked_mul(scale)
                .filter(|n| *n <= crate::cron::MAX_WINDOW_MINUTES)
                .ok_or_else(|| {
                    err(
                        ErrorKind::Limit,
                        "Relative one-off time is limited to seven days",
                    )
                })?;
            now.after(mins).map_err(|_| {
                err(
                    ErrorKind::Invalid,
                    "Relative one-off time exceeds the supported calendar",
                )
            })
        }
        _ => Err(err(
            ErrorKind::Unsupported,
            "Unsupported one-off time: use -t YYYYMMDDhhmm or now [+ N minutes|hours|days|weeks], virtual UTC only",
        )),
    }
}
fn ordinal(m: Moment) -> u32 {
    let leap = |y: u16| y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400));
    let month = [
        31,
        28 + u32::from(leap(m.year)),
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    ((2000..m.year)
        .map(|y| 365 + u32::from(leap(y)))
        .sum::<u32>()
        + month[..usize::from(m.month) - 1].iter().sum::<u32>()
        + u32::from(m.day)
        - 1)
        * 1440
        + u32::from(m.hour) * 60
        + u32::from(m.minute)
}
struct Active {
    job: Job,
    vm: shell::Vm,
    when: Moment,
}
/// Transactional virtual preview. Only take_result publishes a finished candidate.
pub struct Run {
    state: Option<State>,
    end: Moment,
    active: Option<Active>,
    result: Option<Result<State>>,
    pub work_units: usize,
}
impl Run {
    pub fn begin(state: &State, end: Moment) -> Result<Self> {
        state.validate()?;
        end.validate()
            .map_err(|_| err(ErrorKind::Invalid, "Invalid end of virtual window"))?;
        ordinal(end)
            .checked_sub(ordinal(state.clock))
            .filter(|n| *n > 0 && *n <= crate::cron::MAX_WINDOW_MINUTES)
            .ok_or_else(|| {
                err(
                    ErrorKind::Limit,
                    "Advance one minute through seven days in virtual UTC",
                )
            })?;
        Ok(Self {
            state: Some(state.clone()),
            end,
            active: None,
            result: None,
            work_units: 0,
        })
    }
    pub fn done(&self) -> bool {
        self.state.is_none()
    }
    pub fn active_job(&self) -> Option<u32> {
        self.active.as_ref().map(|a| a.job.id)
    }
    pub fn cancel(&mut self) {
        if !self.done() {
            self.active = None;
            self.state = None;
            self.result = Some(Err(err(
                ErrorKind::Cancelled,
                "Stopped simulated preview: this window's staged queue/file/clock changes were discarded. Real at does not roll back completed jobs.",
            )));
        }
    }
    pub fn take_result(&mut self) -> Option<Result<State>> {
        self.result.take()
    }
    fn fail(&mut self, e: Error) {
        self.active = None;
        self.state = None;
        self.result = Some(Err(e));
    }
    pub fn tick(&mut self, budget: usize) {
        if self.done() {
            return;
        }
        for _ in 0..budget.min(MAX_TICK) {
            self.work_units = self.work_units.saturating_add(1);
            if let Some(a) = &mut self.active {
                a.vm.tick(1);
                if !a.vm.done() {
                    continue;
                }
                let outcome = &mut a.vm.outcome;
                if outcome.files.keys().any(|p| !path(p)) {
                    self.fail(err(ErrorKind::Unsupported,"One-off files require flat /work names using only ASCII letters, digits, dot, underscore or dash (up to100 name bytes). This job produced an unsupported name; the staged window was discarded."));
                    return;
                }
                if !files_ok(&outcome.files) {
                    self.fail(err(ErrorKind::Limit,"A job exceeded the one-off virtual-file count/text budget; the staged window was discarded"));
                    return;
                }
                let status = if outcome.error.is_none() && outcome.status == 0 {
                    Status::Completed
                } else {
                    Status::Failed
                };
                let exit_status = if status == Status::Completed {
                    0
                } else {
                    outcome.status.clamp(1, 255)
                };
                let record = Record {
                    id: a.job.id,
                    owner: a.job.owner.clone(),
                    when: a.when,
                    status,
                    exit_status: Some(exit_status),
                    stdout: std::mem::take(&mut outcome.stdout),
                    stderr: std::mem::take(&mut outcome.stderr),
                    error: outcome.error.take(),
                };
                let state = self.state.as_mut().unwrap();
                state.files = std::mem::take(&mut outcome.files);
                state.record(record);
                self.active = None;
                continue;
            }
            let state = self.state.as_mut().unwrap();
            let next = state
                .jobs
                .values()
                .filter(|j| j.due <= self.end)
                .min_by_key(|j| (j.due, j.id))
                .map(|j| j.id);
            let Some(id) = next else {
                state.clock = self.end;
                if let Err(e) = state.validate() {
                    self.fail(e);
                    return;
                }
                self.result = Some(Ok(self.state.take().unwrap()));
                return;
            };
            let job = state.jobs.remove(&id).unwrap();
            let when = job.due.max(state.clock);
            state.clock = when;
            let mut env = job.environment.clone();
            for (k, v) in [
                ("HOME", "/work"),
                ("SHELL", "/bin/sh"),
                ("USER", job.owner.as_str()),
                ("LOGNAME", job.owner.as_str()),
            ] {
                env.insert(k.into(), v.into());
            }
            let case = shell::Case {
                name: "one-off virtual job".into(),
                args: vec![],
                stdin: String::new(),
                env,
                files: state.files.clone(),
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                after: BTreeMap::new(),
            };
            match shell::one_off::start(&job.source, &case) {
                Ok(vm) => self.active = Some(Active { job, vm, when }),
                Err(message) => {
                    let mut diagnostic = message;
                    diagnostic.truncate(diagnostic.floor_char_boundary(1000));
                    state.record(Record {
                        id,
                        owner: job.owner,
                        when,
                        status: Status::Failed,
                        exit_status: Some(2),
                        stdout: String::new(),
                        stderr: String::new(),
                        error: Some(diagnostic),
                    });
                }
            }
        }
    }
}
