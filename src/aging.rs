//! Non-secret password-aging metadata and a versioned date policy. No host I/O.
use crate::accounts::EpochDay;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_RECORDS: usize = 16;
pub const MAX_DAYS: u32 = 365_000;
pub const MAX_COMMAND: usize = 1024;
pub const MAX_JSON: usize = 65_536;
pub const POLICY: &str =
    "Supplied UTC dates · Linux-PAM v1.7.1 aging-policy model; not a login decision";
pub const HELP: &str = "Virtual aging records only. chage -l [-i] LOGIN lists ISO metadata. Change fields with -m/--mindays, -M/--maxdays, -W/--warndays, -I/--inactive, -E/--expiredate or -d/--lastday followed by a literal value; long=value is also supported. Use -1 to disable a field; lastday0 means password change required. Dates: YYYY-MM-DD or positive epoch day. No interactive mode, short-option clusters, shell syntax, host paths/accounts, passwords or authentication. All queries use ISO dates. Open the dated observation table to inspect the separately modeled policy.";
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct Days(u32);
impl TryFrom<u32> for Days {
    type Error = String;
    fn try_from(n: u32) -> Result<Self> {
        if n <= MAX_DAYS {
            Ok(Self(n))
        } else {
            Err("Age values are limited to 0–365000 days in this simulator.".into())
        }
    }
}
impl From<Days> for u32 {
    fn from(d: Days) -> u32 {
        d.0
    }
}
impl Days {
    pub fn number(self) -> u32 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "day",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum LastChange {
    Disabled,
    ForceChange,
    Day(EpochDay),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub last_change: LastChange,
    pub minimum: Option<Days>,
    pub maximum: Option<Days>,
    pub warning: Option<Days>,
    pub inactive: Option<Days>,
    pub account_expires: Option<EpochDay>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Usable,
    PasswordChangeRequired,
    PasswordGraceEnded,
    AccountExpired,
}
impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::Usable => "Aging policy permits use",
            Self::PasswordChangeRequired => "Password change required",
            Self::PasswordGraceEnded => "Password-change grace has ended",
            Self::AccountExpired => "Account date has expired",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub verdict: Verdict,
    pub warning_days: Option<u32>,
    pub change_too_soon: bool,
    pub future_last_change: bool,
}
impl Record {
    /// Reimplements the disclosed ordering of the pinned expiry helper over
    /// supplied metadata. It never authenticates, changes a password or reads time.
    pub fn observe(&self, today: EpochDay) -> Observation {
        let mut out = Observation {
            verdict: Verdict::Usable,
            warning_days: None,
            change_too_soon: false,
            future_last_change: false,
        };
        if self.account_expires.is_some_and(|d| today >= d) {
            out.verdict = Verdict::AccountExpired;
            return out;
        }
        let changed = match self.last_change {
            LastChange::ForceChange => {
                out.verdict = Verdict::PasswordChangeRequired;
                return out;
            }
            LastChange::Disabled => return out,
            LastChange::Day(d) if d > today => {
                out.future_last_change = true;
                return out;
            }
            LastChange::Day(d) => d,
        };
        let elapsed = today.number() - changed.number();
        if let Some(max) = self.maximum {
            // Typed field bounds make this sum far smaller than u32::MAX.
            if self.inactive.is_some_and(|n| elapsed >= max.0 + n.0) {
                out.verdict = Verdict::PasswordGraceEnded;
                return out;
            }
            if elapsed >= max.0 {
                out.verdict = Verdict::PasswordChangeRequired;
                return out;
            }
            if self
                .warning
                .is_some_and(|w| w.0 > 0 && elapsed >= max.0.saturating_sub(w.0))
            {
                out.warning_days = Some(max.0 - elapsed);
            }
        }
        // A minimum age restricts a change attempt, not ordinary use.
        out.change_too_soon = self.minimum.is_some_and(|min| elapsed < min.0);
        out
    }
    pub fn list(&self) -> String {
        fn age(n: Option<Days>) -> String {
            n.map_or_else(|| "-1".into(), |n| n.0.to_string())
        }
        fn calendar(n: u32) -> String {
            EpochDay::try_from(n).map_or_else(
                |_| format!("epoch day {n} (outside supported calendar)"),
                |n| n.iso(),
            )
        }
        let last = match self.last_change {
            LastChange::Disabled => "never".into(),
            LastChange::ForceChange => "password must be changed".into(),
            LastChange::Day(d) => d.iso(),
        };
        let expires = match (self.last_change, self.maximum) {
            (LastChange::ForceChange, _) => "password must be changed".into(),
            (LastChange::Day(d), Some(max)) => calendar(d.number() + max.0),
            _ => "never".into(),
        };
        let inactive = match (self.last_change, self.maximum, self.inactive) {
            (LastChange::ForceChange, _, Some(_)) => "password must be changed".into(),
            (LastChange::Day(d), Some(max), Some(n)) => calendar(d.number() + max.0 + n.0),
            _ => "never".into(),
        };
        format!(
            "Last password change\t\t\t: {last}\nPassword expires\t\t\t: {expires}\nPassword inactive\t\t\t: {inactive}\nAccount expires\t\t\t\t: {}\nMinimum number of days between password change\t: {}\nMaximum number of days between password change\t: {}\nNumber of days of warning before password expires\t: {}\n",
            self.account_expires
                .map_or_else(|| "never".into(), |d| d.iso()),
            age(self.minimum),
            age(self.maximum),
            age(self.warning)
        )
    }
}
fn login(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b == b'_')
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub login: String,
    pub record: Record,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Entries", into = "Entries")]
pub struct World {
    entries: Vec<Entry>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entries {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Entry,16>")]
    entries: Vec<Entry>,
}
impl TryFrom<Entries> for World {
    type Error = String;
    fn try_from(v: Entries) -> Result<Self> {
        Self::new(v.entries)
    }
}
impl From<World> for Entries {
    fn from(w: World) -> Self {
        Self { entries: w.entries }
    }
}
impl World {
    pub fn new(entries: Vec<Entry>) -> Result<Self> {
        let mut names = BTreeSet::new();
        if entries.is_empty()
            || entries.len() > MAX_RECORDS
            || entries
                .iter()
                .any(|e| !login(&e.login) || !names.insert(&e.login))
        {
            return Err("Use one to sixteen unique bounded virtual account names.".into());
        }
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    pub fn record(&self, name: &str) -> Option<&Record> {
        self.entries
            .iter()
            .find(|e| e.login == name)
            .map(|e| &e.record)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_JSON {
            return Err("Aging records exceed64KiB.".into());
        }
        serde_json::from_str(raw).map_err(|e| format!("Invalid virtual aging records: {e}"))
    }
    pub fn execute(&mut self, line: &str) -> Result<String> {
        match parse(line)? {
            Request::List(name) => self
                .record(&name)
                .map(Record::list)
                .ok_or_else(|| "That account is not supplied by this activity.".into()),
            Request::Edit { login, edits } => {
                let old = self
                    .record(&login)
                    .ok_or("That account is not supplied by this activity.")?;
                let mut candidate = old.clone();
                for edit in edits {
                    match edit {
                        Edit::Minimum(n) => candidate.minimum = n,
                        Edit::Maximum(n) => candidate.maximum = n,
                        Edit::Warning(n) => candidate.warning = n,
                        Edit::Inactive(n) => candidate.inactive = n,
                        Edit::Expiry(d) => candidate.account_expires = d,
                        Edit::Last(d) => candidate.last_change = d,
                    }
                }
                self.entries
                    .iter_mut()
                    .find(|e| e.login == login)
                    .unwrap()
                    .record = candidate;
                Ok(String::new())
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Edit {
    Minimum(Option<Days>),
    Maximum(Option<Days>),
    Warning(Option<Days>),
    Inactive(Option<Days>),
    Expiry(Option<EpochDay>),
    Last(LastChange),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    List(String),
    Edit { login: String, edits: Vec<Edit> },
}
fn age(s: &str) -> Result<Option<Days>> {
    if s == "-1" {
        return Ok(None);
    }
    if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err("Use a whole number of days, or -1 to disable this field.".into());
    }
    let n = s
        .parse::<u32>()
        .map_err(|_| "Day count exceeds the supported range.")?;
    Days::try_from(n).map(Some)
}
fn date(s: &str) -> Result<EpochDay> {
    EpochDay::parse(s).map_err(|_|"Use a valid YYYY-MM-DD date or positive epoch day (1970-01-02 through9999-12-31); ordinary day0 is outside this subset.".into())
}
pub fn parse(line: &str) -> Result<Request> {
    if line.len() > MAX_COMMAND {
        return Err("Command exceeds1024bytes; no changes made.".into());
    }
    if line.chars().any(|c| c.is_control() && c != '\t')
        || line.bytes().any(|b| b";|&<>$`\\\"'(){}[]*?!#".contains(&b))
    {
        return Err("Unsupported shell syntax. Enter one chage command with literal values; nothing is passed to a shell.".into());
    }
    let words: Vec<_> = line.split_whitespace().take(33).collect();
    if words.len() > 32 {
        return Err("Command exceeds32tokens; no changes made.".into());
    }
    if words.first() != Some(&"chage") {
        return Err(
            "Unsupported command. This workspace simulates the disclosed chage subset only.".into(),
        );
    }
    let mut at = 1;
    let mut target = None;
    let mut list = false;
    let mut iso = false;
    let mut fields = BTreeSet::new();
    let mut edits = Vec::new();
    while at < words.len() {
        let token = words[at];
        at += 1;
        if !token.starts_with('-') {
            if target.is_some() || !login(token) {
                return Err("Supply exactly one virtual account name, without a path.".into());
            }
            target = Some(token.to_string());
            continue;
        }
        if matches!(token, "-l" | "--list") {
            if list {
                return Err("List option was repeated; no changes made.".into());
            }
            list = true;
            continue;
        }
        if matches!(token, "-i" | "--iso8601") {
            if iso {
                return Err("ISO option was repeated; no changes made.".into());
            }
            iso = true;
            continue;
        }
        let (option, embedded) = if token.starts_with("--") {
            token
                .split_once('=')
                .map_or((token, None), |(k, v)| (k, Some(v)))
        } else {
            (token, None)
        };
        let field=match option {
            "-m"|"--mindays"=>0,"-M"|"--maxdays"=>1,"-W"|"--warndays"=>2,
            "-I"|"--inactive"=>3,"-E"|"--expiredate"=>4,"-d"|"--lastday"=>5,
            _=>return Err("Unsupported chage option. Use the separate literal options listed in Help; interactive, root/prefix and short clusters are not simulated.".into()),
        };
        if !fields.insert(field) {
            return Err("An aging field was specified twice; no changes made.".into());
        }
        let value = if let Some(v) = embedded {
            v
        } else {
            let v = *words
                .get(at)
                .ok_or("This option needs a value; no changes made.")?;
            at += 1;
            v
        };
        edits.push(match field {
            0 => Edit::Minimum(age(value)?),
            1 => Edit::Maximum(age(value)?),
            2 => Edit::Warning(age(value)?),
            3 => Edit::Inactive(age(value)?),
            4 => Edit::Expiry(if value == "-1" {
                None
            } else {
                Some(date(value)?)
            }),
            _ => Edit::Last(match value {
                "-1" => LastChange::Disabled,
                "0" => LastChange::ForceChange,
                _ => LastChange::Day(date(value)?),
            }),
        });
    }
    let target =
        target.ok_or("Supply a virtual account name; interactive mode is not supported.")?;
    if list && edits.is_empty() {
        Ok(Request::List(target))
    } else if !list && !iso && !edits.is_empty() {
        Ok(Request::Edit {
            login: target,
            edits,
        })
    } else {
        Err("Choose a list query or explicit field changes. Interactive mode and mixing list/ISO flags with changes are unsupported.".into())
    }
}
