//! Explicit-clock, in-process scheduling. No host clock, files or subprocesses.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const MAX_USERS: usize = 8;
pub const MAX_ENTRIES: usize = 16;
pub const MAX_TABLE_BYTES: usize = 8192;
pub const MAX_FILES: usize = 32;
pub const MAX_FILE_BYTES: usize = 96 * 1024;
pub const MAX_JSON_BYTES: usize = 1024 * 1024;
pub const MAX_WINDOW_MINUTES: u32 = 7 * 24 * 60;
pub const MAX_RUNS: usize = 64;
pub const MAX_RUN_OUTPUT: usize = 64 * 1024;
pub const MAX_TICK: usize = 64;

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
    pub line: Option<usize>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn err(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        line: None,
        message: message.into(),
    }
}
fn invalid(message: &str) -> Error {
    err(ErrorKind::Invalid, message)
}
fn syntax(message: &str) -> Error {
    err(ErrorKind::Syntax, message)
}
fn unsupported(message: &str) -> Error {
    err(ErrorKind::Unsupported, message)
}
fn limited(message: &str) -> Error {
    err(ErrorKind::Limit, message)
}
fn username(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_-".contains(&c))
}
fn variable(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && (s.as_bytes()[0].is_ascii_alphabetic() || s.starts_with('_'))
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moment {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}
fn leap(year: u16) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
fn month_days(year: u16, month: u8) -> u8 {
    match month {
        2 => 28 + u8::from(leap(year)),
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}
impl Moment {
    pub fn validate(&self) -> Result<()> {
        if !(2000..=2099).contains(&self.year)
            || !(1..=12).contains(&self.month)
            || self.day == 0
            || self.day > month_days(self.year, self.month)
            || self.hour > 23
            || self.minute > 59
        {
            return Err(invalid(
                "Use a valid virtual UTC date/time from 2000 through 2099",
            ));
        }
        Ok(())
    }
    pub fn parse(s: &str) -> Result<Self> {
        if s.len() != 16
            || !s.is_ascii()
            || &s[4..5] != "-"
            || &s[7..8] != "-"
            || &s[10..11] != " "
            || &s[13..14] != ":"
        {
            return Err(syntax("Use YYYY-MM-DD HH:MM in virtual UTC"));
        }
        let number = |s: &str| {
            if !s.bytes().all(|c| c.is_ascii_digit()) {
                return Err(syntax("Date and time fields must be decimal numbers"));
            }
            s.parse::<u16>()
                .map_err(|_| syntax("Date and time fields must be decimal numbers"))
        };
        let m = Self {
            year: number(&s[..4])?,
            month: number(&s[5..7])? as u8,
            day: number(&s[8..10])? as u8,
            hour: number(&s[11..13])? as u8,
            minute: number(&s[14..])? as u8,
        };
        m.validate()?;
        Ok(m)
    }
    pub fn text(&self) -> String {
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }
    fn ordinal(&self) -> u32 {
        let days: u32 = (2000..self.year)
            .map(|y| 365 + u32::from(leap(y)))
            .sum::<u32>()
            + (1..self.month)
                .map(|m| u32::from(month_days(self.year, m)))
                .sum::<u32>()
            + u32::from(self.day)
            - 1;
        days * 1440 + u32::from(self.hour) * 60 + u32::from(self.minute)
    }
    pub fn weekday(&self) -> Result<u8> {
        self.validate()?;
        Ok(((self.ordinal() / 1440 + 6) % 7) as u8)
    }
    pub fn next(&self) -> Result<Self> {
        self.validate()?;
        let mut next = *self;
        next.minute += 1;
        if next.minute == 60 {
            next.minute = 0;
            next.hour += 1;
        }
        if next.hour == 24 {
            next.hour = 0;
            next.day += 1;
        }
        if next.day > month_days(next.year, next.month) {
            next.day = 1;
            next.month += 1;
        }
        if next.month == 13 {
            next.month = 1;
            next.year += 1;
        }
        next.validate()?;
        Ok(next)
    }
    pub fn after(&self, minutes: u32) -> Result<Self> {
        self.validate()?;
        if minutes > MAX_WINDOW_MINUTES {
            return Err(limited(
                "A simulated scheduling window is limited to seven days",
            ));
        }
        let mut m = *self;
        for _ in 0..minutes {
            m = m.next()?;
        }
        Ok(m)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Field {
    values: BTreeSet<u8>,
    wildcard: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schedule {
    fields: [Field; 5],
}
fn field_value(s: &str, field: usize) -> Result<u8> {
    const MONTHS: [&str; 12] = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];
    const DAYS: [&str; 7] = ["sun", "mon", "tue", "wed", "thu", "fri", "sat"];
    let named = match field {
        3 => MONTHS
            .iter()
            .position(|n| s.eq_ignore_ascii_case(n))
            .map(|n| n + 1),
        4 => DAYS.iter().position(|n| s.eq_ignore_ascii_case(n)),
        _ => None,
    };
    if let Some(n) = named {
        return Ok(n as u8);
    }
    if s.is_empty() || !s.bytes().all(|c| c.is_ascii_digit()) {
        return Err(syntax(
            "Use numbers or three-letter month/weekday names in calendar fields",
        ));
    }
    s.parse()
        .map_err(|_| syntax("Calendar field number is too large"))
}
impl Schedule {
    pub fn parse(fields: [&str; 5]) -> Result<Self> {
        let limits = [(0, 59), (0, 23), (1, 31), (1, 12), (0, 7)];
        let mut parsed = Vec::with_capacity(5);
        for (i, text) in fields.iter().enumerate() {
            if text.is_empty() || text.len() > 128 {
                return Err(limited("Each calendar field is limited to 128 bytes"));
            }
            let wildcard = text.starts_with('*');
            let mut values = BTreeSet::new();
            let groups: Vec<_> = text.split(',').collect();
            if groups.len() > 32 {
                return Err(limited("Calendar lists are limited to 32 items"));
            }
            if groups.len() > 1 && text.contains('*') {
                return Err(unsupported(
                    "Wildcard/list combinations are outside this calendar subset; use explicit ranges or one wildcard field",
                ));
            }
            for group in groups {
                let (range, step) = match group.split_once('/') {
                    Some((range, step)) => {
                        if step.is_empty() || !step.bytes().all(|c| c.is_ascii_digit()) {
                            return Err(syntax("A step needs a positive decimal number"));
                        }
                        let step = step
                            .parse::<u16>()
                            .map_err(|_| limited("Calendar step exceeds 65535"))?;
                        if step == 0 {
                            return Err(syntax("A calendar step cannot be zero"));
                        }
                        if range != "*" && !range.contains('-') {
                            return Err(unsupported(
                                "Use */step or start-end/step; a single start/step is not included",
                            ));
                        }
                        (range, step)
                    }
                    None => (group, 1),
                };
                let (start, end) = if range == "*" {
                    limits[i]
                } else if let Some((start, end)) = range.split_once('-') {
                    (field_value(start, i)?, field_value(end, i)?)
                } else {
                    let n = field_value(range, i)?;
                    (n, n)
                };
                if start < limits[i].0 || end > limits[i].1 || start > end {
                    return Err(syntax(
                        "Calendar value is outside its field or its range is reversed",
                    ));
                }
                for n in u16::from(start)..=u16::from(end) {
                    if (n - u16::from(start)) % step == 0 {
                        values.insert(if i == 4 && n == 7 { 0 } else { n as u8 });
                    }
                }
            }
            if values.is_empty() {
                return Err(syntax("A calendar field cannot be empty"));
            }
            parsed.push(Field { values, wildcard });
        }
        Ok(Self {
            fields: parsed
                .try_into()
                .map_err(|_| invalid("Invalid calendar field count"))?,
        })
    }
    pub fn matches(&self, m: Moment) -> Result<bool> {
        let weekday = m.weekday()?;
        let f = &self.fields;
        let dom = f[2].values.contains(&m.day);
        let dow = f[4].values.contains(&weekday);
        let day = if f[2].wildcard || f[4].wildcard {
            dom && dow
        } else {
            dom || dow
        };
        Ok(f[0].values.contains(&m.minute)
            && f[1].values.contains(&m.hour)
            && f[3].values.contains(&m.month)
            && day)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub line: usize,
    pub owner: String,
    pub schedule: Schedule,
    pub source: String,
    pub environment: BTreeMap<String, String>,
}
fn env_assignment(line: &str, env: &mut BTreeMap<String, String>) -> Result<bool> {
    let Some((name, value)) = line.split_once('=') else {
        return Ok(false);
    };
    let name = name.trim();
    if !variable(name) {
        return Ok(false);
    }
    crate::shell::cron::environment_name(name).map_err(|message| unsupported(&message))?;
    let value = value.trim();
    let value = if value.starts_with(['\'', '"']) {
        if value.len() < 2 || !value.ends_with(value.chars().next().unwrap()) {
            return Err(syntax("Close the environment value's matching quote"));
        }
        let body = &value[1..value.len() - 1];
        if body.contains(value.chars().next().unwrap()) {
            return Err(unsupported(
                "Embedded matching quotes in cron environment values are outside this lesson's literal-value subset",
            ));
        }
        body
    } else {
        value
    };
    if value.len() > 256 {
        return Err(limited("Environment values are limited to 256 bytes"));
    }
    if matches!(
        name,
        "LOGNAME"
            | "USER"
            | "PATH"
            | "MAILTO"
            | "MAILFROM"
            | "CRON_TZ"
            | "RANDOM_DELAY"
            | "MLS_LEVEL"
            | "CONTENT_TYPE"
            | "CONTENT_TRANSFER_ENCODING"
    ) || (name == "HOME" && value != "/work")
        || (name == "SHELL" && value != "/bin/sh")
    {
        return Err(unsupported(
            "This scheduling lesson supports ordinary text variables and fixed HOME=/work, SHELL=/bin/sh only; identity, PATH, mail and timezone controls are not editable",
        ));
    }
    if !env.contains_key(name) && env.len() >= 16 {
        return Err(limited("A table is limited to 16 environment settings"));
    }
    env.insert(name.into(), value.into());
    Ok(true)
}
/// owner=None means a system table with an explicit account column.
pub fn parse_table(text: &str, owner: Option<&str>) -> Result<Vec<Entry>> {
    if text.len() > MAX_TABLE_BYTES {
        return Err(limited("A crontab is limited to 8 KiB"));
    }
    if text.contains(['\0', '\r']) {
        return Err(syntax(
            "Use ordinary newline-separated text, without NUL or carriage-return characters",
        ));
    }
    if !text.is_empty() && !text.ends_with('\n') {
        return Err(syntax("Finish the last crontab line with a newline"));
    }
    if owner.is_some_and(|n| !username(n)) {
        return Err(invalid("Invalid supplied account name"));
    }
    let mut env = BTreeMap::new();
    let mut result = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_start_matches([' ', '\t']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let entry = (|| {
            if env_assignment(line, &mut env)? { return Ok(None); }
            if line.starts_with('@') { return Err(unsupported("Cron nicknames/reboot are not included; use five calendar fields")); }
            let mut rest = line;
            let mut next = || -> Result<&str> {
                rest = rest.trim_start_matches([' ', '\t']);
                if rest.is_empty() { return Err(syntax("A cron entry needs five time fields followed by a command; a system table also needs its account")); }
                let end = rest.find([' ', '\t']).unwrap_or(rest.len()); let value = &rest[..end]; rest = &rest[end..]; Ok(value)
            };
            let fields = [next()?, next()?, next()?, next()?, next()?];
            let schedule = Schedule::parse(fields)?;
            let account = if let Some(owner) = owner { owner } else { next()? };
            if !username(account) { return Err(syntax("The system account column needs a supplied username")); }
            let command = rest.trim_start_matches([' ', '\t']);
            crate::shell::cron::prepare(command).map_err(|message| err(if message.contains("Unsupported") { ErrorKind::Unsupported } else { ErrorKind::Syntax }, &message))?;
            Ok(Some(Entry { line: index + 1, owner: account.into(), schedule, source: command.into(), environment: env.clone() }))
        })().map_err(|mut e: Error| { e.line = Some(index + 1); e })?;
        if let Some(entry) = entry {
            if result.len() >= MAX_ENTRIES {
                return Err(limited("At most 16 scheduled entries are supported"));
            }
            result.push(entry);
        }
    }
    Ok(result)
}

// A count sentinel rejects the N+1 item/key without decoding its contents.
// It intentionally does not deserialize IgnoredAny (which would still traverse
// a potentially large excess value).
struct Excess<const MAP: bool>;
impl<'de, const MAP: bool> Deserialize<'de> for Excess<MAP> {
    fn deserialize<D: Deserializer<'de>>(_: D) -> std::result::Result<Self, D::Error> {
        Err(D::Error::custom(if MAP {
            "Scheduling map count limit"
        } else {
            "Scheduling account count limit"
        }))
    }
}
struct TextSeed<const N: usize>;
impl<'de, const N: usize> serde::de::DeserializeSeed<'de> for TextSeed<N> {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<String, D::Error> {
        text::<D, N>(d)
    }
}
pub(crate) fn text<'de, D: Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> std::result::Result<String, E> {
            if value.len() > N {
                return Err(E::custom("Scheduling UTF-8 text byte limit"));
            }
            Ok(value.into())
        }
    }
    d.deserialize_str(Text::<N>)
}
fn names<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeSet<String>, D::Error> {
    struct Names;
    impl<'de> Visitor<'de> for Names {
        type Value = BTreeSet<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most eight distinct supplied account names")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut values = BTreeSet::new();
            while values.len() < MAX_USERS {
                let Some(v) = a.next_element_seed(TextSeed::<32>)? else {
                    return Ok(values);
                };
                if !username(&v) || !values.insert(v) {
                    return Err(A::Error::custom(
                        "Invalid, duplicate or excessive account names",
                    ));
                }
            }
            let _ = a.next_element::<Excess<false>>()?;
            Ok(values)
        }
    }
    d.deserialize_seq(Names)
}
#[derive(Deserialize)]
#[serde(transparent)]
struct NameSet(#[serde(deserialize_with = "names")] BTreeSet<String>);
fn optional_names<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<BTreeSet<String>>, D::Error> {
    Ok(Option::<NameSet>::deserialize(d)?.map(|s| s.0))
}
pub(crate) fn map<'de, D: Deserializer<'de>, const N: usize, const B: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    struct Maps<const N: usize, const B: usize>;
    impl<'de, const N: usize, const B: usize> Visitor<'de> for Maps<N, B> {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} distinct bounded text entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while values.len() < N {
                let Some(k) = a.next_key_seed(TextSeed::<256>)? else {
                    return Ok(values);
                };
                if values.contains_key(&k) {
                    return Err(A::Error::custom("Duplicate scheduling map key"));
                }
                let v = a.next_value_seed(TextSeed::<B>)?;
                values.insert(k, v);
            }
            let _ = a.next_key::<Excess<true>>()?;
            Ok(values)
        }
    }
    d.deserialize_map(Maps::<N, B>)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub clock: Moment,
    #[serde(deserialize_with = "names")]
    pub users: BTreeSet<String>,
    #[serde(deserialize_with = "map::<_,8,8192>")]
    pub tables: BTreeMap<String, String>,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub system_table: String,
    #[serde(deserialize_with = "optional_names")]
    pub allow: Option<BTreeSet<String>>,
    #[serde(deserialize_with = "optional_names")]
    pub deny: Option<BTreeSet<String>>,
    #[serde(deserialize_with = "map::<_,32,32768>")]
    pub files: BTreeMap<String, String>,
}
fn valid_files(files: &BTreeMap<String, String>) -> bool {
    files.len() <= MAX_FILES
        && files.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() <= MAX_FILE_BYTES
        && files.iter().all(|(k, v)| {
            k.strip_prefix("/work/").is_some_and(|n| {
                !n.is_empty()
                    && !matches!(n, "." | "..")
                    && !n.contains('/')
                    && !n.chars().any(char::is_control)
                    && n.len() <= 250
            }) && v.len() <= 32768
                && !v.contains('\0')
        })
}
impl State {
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_JSON_BYTES {
            return Err(limited("Scheduling JSON exceeds 1 MiB"));
        }
        let state: Self = serde_json::from_str(text).map_err(|_| {
            invalid("Invalid scheduling JSON: unknown, duplicate, excessive or malformed fields")
        })?;
        state.validate()?;
        Ok(state)
    }
    pub fn validate(&self) -> Result<()> {
        self.clock.validate()?;
        if !valid_files(&self.files) {
            return Err(invalid(
                "Use bounded ordinary /work filenames without control characters, and bounded NUL-free file text",
            ));
        }
        if self.users.is_empty()
            || self.users.len() > MAX_USERS
            || !self.users.contains("root")
            || self.users.iter().any(|s| !username(s))
            || self.tables.len() > MAX_USERS
        {
            return Err(invalid("Invalid supplied accounts or bounded /work files"));
        }
        for set in [&self.allow, &self.deny].into_iter().flatten() {
            if set.len() > MAX_USERS || !set.is_subset(&self.users) {
                return Err(invalid("Access files must name supplied accounts only"));
            }
        }
        self.entries()?;
        Ok(())
    }
    pub fn entries(&self) -> Result<Vec<Entry>> {
        let mut entries = Vec::new();
        for (owner, text) in &self.tables {
            if !self.users.contains(owner) {
                return Err(invalid("Crontab owner is not a supplied account"));
            }
            entries.extend(parse_table(text, Some(owner))?);
            if entries.len() > MAX_ENTRIES {
                return Err(limited("All tables together are limited to 16 entries"));
            }
        }
        entries.extend(parse_table(&self.system_table, None)?);
        if entries.len() > MAX_ENTRIES {
            return Err(limited("All tables together are limited to 16 entries"));
        }
        if entries.iter().any(|e| !self.users.contains(&e.owner)) {
            return Err(invalid("A system job names an unknown account"));
        }
        Ok(entries)
    }
    pub fn can_manage(&self, actor: &str, owner: &str) -> bool {
        self.users.contains(actor)
            && self.users.contains(owner)
            && (actor == "root"
                || (actor == owner
                    && match &self.allow {
                        Some(allow) => allow.contains(actor),
                        None => self.deny.as_ref().is_some_and(|deny| !deny.contains(actor)),
                    }))
    }
    pub fn install_user(&mut self, actor: &str, owner: &str, text: &str) -> Result<()> {
        self.validate()?;
        if !self.can_manage(actor, owner) {
            return Err(err(
                ErrorKind::Permission,
                "This account cannot manage that user crontab; installed jobs are unchanged",
            ));
        }
        let added = parse_table(text, Some(owner))?;
        // Count tables by source, not command/owner equality: a system entry may
        // legitimately equal a user entry and must still consume its own slot.
        let mut total = added.len() + parse_table(&self.system_table, None)?.len();
        for (name, table) in &self.tables {
            if name != owner {
                total += parse_table(table, Some(name))?.len();
            }
        }
        if total > MAX_ENTRIES {
            return Err(limited("All tables together are limited to 16 entries"));
        }
        self.tables.insert(owner.into(), text.into());
        Ok(())
    }
    pub fn remove_user(&mut self, actor: &str, owner: &str) -> Result<()> {
        self.validate()?;
        if !self.can_manage(actor, owner) {
            return Err(err(
                ErrorKind::Permission,
                "This account cannot remove that user crontab",
            ));
        }
        self.tables.remove(owner);
        Ok(())
    }
    pub fn install_system(&mut self, actor: &str, text: &str) -> Result<()> {
        self.validate()?;
        if actor != "root" {
            return Err(err(
                ErrorKind::Permission,
                "Only the supplied administrator may install the system table",
            ));
        }
        let entries = parse_table(text, None)?;
        if entries.iter().any(|e| !self.users.contains(&e.owner)) {
            return Err(invalid("A system entry names an unknown account"));
        }
        let total = entries.len()
            + self.tables.iter().try_fold(0, |n, (owner, text)| {
                parse_table(text, Some(owner)).map(|e| n + e.len())
            })?;
        if total > MAX_ENTRIES {
            return Err(limited("All tables together are limited to 16 entries"));
        }
        self.system_table = text.into();
        Ok(())
    }
    pub fn set_access(
        &mut self,
        actor: &str,
        allow: Option<&BTreeSet<String>>,
        deny: Option<&BTreeSet<String>>,
    ) -> Result<()> {
        self.validate()?;
        if actor != "root" {
            return Err(err(
                ErrorKind::Permission,
                "Only the supplied administrator may edit cron access files",
            ));
        }
        for set in [allow, deny].into_iter().flatten() {
            if set.len() > MAX_USERS || !set.is_subset(&self.users) {
                return Err(invalid("Access files must name supplied accounts only"));
            }
        }
        self.allow = allow.cloned();
        self.deny = deny.cloned();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub when: Moment,
    pub owner: String,
    pub line: usize,
    pub source: String,
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completed {
    pub state: State,
    pub records: Vec<Record>,
}
/// Tick-owned working state. Caller state is never mutated by this object.
pub struct Run {
    state: Option<State>,
    entries: Vec<Entry>,
    end: Moment,
    now: Moment,
    cursor: usize,
    active: Option<(Entry, crate::shell::Vm)>,
    records: Vec<Record>,
    output_bytes: usize,
    result: Option<Result<Completed>>,
}
impl Run {
    pub fn begin(state: &State, end: Moment) -> Result<Self> {
        state.validate()?;
        end.validate()?;
        let span = end
            .ordinal()
            .checked_sub(state.clock.ordinal())
            .ok_or_else(|| invalid("The scheduling window must move forward"))?;
        if span == 0 || span > MAX_WINDOW_MINUTES {
            return Err(limited(
                "Run a future window of one minute through seven days",
            ));
        }
        let entries = state.entries()?;
        Ok(Self {
            state: Some(state.clone()),
            entries,
            end,
            now: state.clock.next()?,
            cursor: 0,
            active: None,
            records: Vec::new(),
            output_bytes: 0,
            result: None,
        })
    }
    pub fn done(&self) -> bool {
        self.state.is_none()
    }
    pub fn cancel(&mut self) {
        if !self.done() {
            self.active = None;
            self.state = None;
            self.result = Some(Err(err(
                ErrorKind::Cancelled,
                "Stopped: unfinished simulated file changes were discarded",
            )));
        }
    }
    fn fail(&mut self, error: Error) {
        self.active = None;
        self.state = None;
        self.result = Some(Err(error));
    }
    pub fn take_result(&mut self) -> Option<Result<Completed>> {
        self.result.take()
    }
    pub fn tick(&mut self, budget: usize) {
        if self.done() || self.state.is_none() {
            return;
        }
        for _ in 0..budget.min(MAX_TICK) {
            if let Some((entry, vm)) = &mut self.active {
                vm.tick(1);
                if !vm.done() {
                    continue;
                }
                if let Some(message) = &vm.outcome.error {
                    let message = format!("Scheduled command could not finish: {message}");
                    self.fail(unsupported(&message));
                    return;
                }
                self.output_bytes += vm.outcome.stdout.len() + vm.outcome.stderr.len();
                if self.output_bytes > MAX_RUN_OUTPUT || !valid_files(&vm.outcome.files) {
                    self.fail(limited("The simulated window exceeded its output or virtual-file budget; nothing was committed"));
                    return;
                }
                self.records.push(Record {
                    when: self.now,
                    owner: entry.owner.clone(),
                    line: entry.line,
                    source: entry.source.clone(),
                    stdout: vm.outcome.stdout.clone(),
                    stderr: vm.outcome.stderr.clone(),
                    status: vm.outcome.status,
                });
                self.state.as_mut().unwrap().files = std::mem::take(&mut vm.outcome.files);
                self.active = None;
                continue;
            }
            if self.cursor == self.entries.len() {
                if self.now == self.end {
                    let mut state = self.state.take().unwrap();
                    state.clock = self.end;
                    self.result = Some(Ok(Completed {
                        state,
                        records: std::mem::take(&mut self.records),
                    }));
                    return;
                }
                match self.now.next() {
                    Ok(next) => self.now = next,
                    Err(e) => {
                        self.fail(e);
                        return;
                    }
                }
                self.cursor = 0;
                continue;
            }
            let entry = &self.entries[self.cursor];
            self.cursor += 1;
            if !entry.schedule.matches(self.now).unwrap_or(false) {
                continue;
            }
            if self.records.len() >= MAX_RUNS {
                self.fail(limited("The simulated window exceeds 64 job starts; shorten it or reduce an overly frequent schedule"));
                return;
            }
            let mut env = entry.environment.clone();
            for (key, value) in [
                ("HOME", "/work"),
                ("SHELL", "/bin/sh"),
                ("LOGNAME", &entry.owner),
                ("USER", &entry.owner),
            ] {
                env.insert(key.into(), value.into());
            }
            let case = crate::shell::Case {
                name: "scheduled command".into(),
                args: Vec::new(),
                stdin: String::new(),
                env,
                files: self.state.as_ref().unwrap().files.clone(),
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
                after: BTreeMap::new(),
            };
            match crate::shell::cron::start(&entry.source, &case) {
                Ok(vm) => self.active = Some((entry.clone(), vm)),
                Err(message) => {
                    self.fail(unsupported(&message));
                    return;
                }
            }
        }
    }
}
