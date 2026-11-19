//! Finite IPv4 files/DNS name-source lookup. No OS resolver, host files or sockets.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, net::Ipv4Addr};
pub const MAX_CONFIG: usize = 16_384;
pub const MAX_ROWS: usize = 64;
pub const MAX_NAMES: usize = 128;
pub const MAX_NAME: usize = 253;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: Kind,
    pub file: &'static str,
    pub line: Option<usize>,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.line {
            write!(f, "{} line {n}: {}", self.file, self.message)
        } else {
            write!(f, "{}: {}", self.file, self.message)
        }
    }
}
impl std::error::Error for Error {}
fn err(kind: Kind, file: &'static str, line: Option<usize>, msg: impl Into<String>) -> Error {
    Error {
        kind,
        file,
        line,
        message: msg.into(),
    }
}
fn preview(text: &str) -> String {
    let mut value = text.chars().take(64).collect::<String>();
    if text.chars().nth(64).is_some() {
        value.push_str("...");
    }
    value
}
fn text(raw: &str, file: &'static str) -> Result<(), Error> {
    if raw.len() > MAX_CONFIG || raw.lines().count() > 128 {
        return Err(err(
            Kind::Limit,
            file,
            None,
            "Use at most16KiB and128 lines",
        ));
    }
    if raw
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(err(
            Kind::Invalid,
            file,
            None,
            "Unsupported control character",
        ));
    }
    Ok(())
}
pub fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_NAME
        && s.is_ascii()
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && p.as_bytes()[0].is_ascii_alphanumeric()
                && p.as_bytes()[p.len() - 1].is_ascii_alphanumeric()
                && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
        && s.parse::<Ipv4Addr>().is_err()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Success,
    NotFound,
    Unavailable,
    TryAgain,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Success => "SUCCESS",
            Self::NotFound => "NOTFOUND",
            Self::Unavailable => "UNAVAIL",
            Self::TryAgain => "TRYAGAIN",
        }
    }
    fn index(self) -> usize {
        match self {
            Self::Success => 0,
            Self::NotFound => 1,
            Self::Unavailable => 2,
            Self::TryAgain => 3,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Return,
    Continue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Service {
    Files,
    Dns,
}
impl Service {
    pub fn label(self) -> &'static str {
        match self {
            Self::Files => "files",
            Self::Dns => "dns",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub service: Service,
    actions: [Action; 4],
}
impl Source {
    pub fn action(&self, s: Status) -> Action {
        self.actions[s.index()]
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Switch {
    sources: Vec<Source>,
}
impl Switch {
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let file = "nsswitch.conf hosts excerpt";
        text(raw, file)?;
        let mut parsed = None;
        for (i, row) in raw.lines().enumerate() {
            let line = Some(i + 1);
            let row = row.split('#').next().unwrap_or("").trim();
            if row.is_empty() {
                continue;
            }
            if parsed.is_some() {
                return Err(err(
                    Kind::Unsupported,
                    file,
                    line,
                    "Use exactly one hosts database row in this excerpt",
                ));
            }
            let Some((database, rest)) = row.split_once(':') else {
                return Err(err(
                    Kind::Invalid,
                    file,
                    line,
                    "Start this database row with hosts:",
                ));
            };
            if database.trim() != "hosts" {
                return Err(err(
                    Kind::Unsupported,
                    file,
                    line,
                    "This activity edits only the hosts database, not other NSS databases",
                ));
            }
            let spaced = rest.replace('[', " [ ").replace(']', " ] ");
            let mut words = spaced.split_ascii_whitespace();
            let mut sources: Vec<Source> = vec![];
            while let Some(word) = words.next() {
                if word == "[" {
                    let Some(current) = sources.last_mut() else {
                        return Err(err(
                            Kind::Invalid,
                            file,
                            line,
                            "An action bracket follows the service it controls",
                        ));
                    };
                    let mut count = 0;
                    let mut closed = false;
                    for rule in words.by_ref() {
                        if rule == "]" {
                            closed = true;
                            break;
                        }
                        count += 1;
                        if count > 4 {
                            return Err(err(
                                Kind::Limit,
                                file,
                                line,
                                "Use at most four status rules in a bracket",
                            ));
                        }
                        let Some((status, action)) = rule.split_once('=') else {
                            return Err(err(
                                Kind::Invalid,
                                file,
                                line,
                                "Use a rule such as NOTFOUND=return",
                            ));
                        };
                        let status = match status.to_ascii_uppercase().as_str() {
                            "SUCCESS" => Status::Success,
                            "NOTFOUND" => Status::NotFound,
                            "UNAVAIL" => Status::Unavailable,
                            "TRYAGAIN" => Status::TryAgain,
                            _ => {
                                return Err(err(
                                    Kind::Unsupported,
                                    file,
                                    line,
                                    "Only SUCCESS, NOTFOUND, UNAVAIL and TRYAGAIN without negation are supported",
                                ));
                            }
                        };
                        let action = match action.to_ascii_lowercase().as_str() {
                            "return" => Action::Return,
                            "continue" => Action::Continue,
                            _ => {
                                return Err(err(
                                    Kind::Unsupported,
                                    file,
                                    line,
                                    "Use return or continue; merge is outside this hosts model",
                                ));
                            }
                        };
                        if status == Status::Success && action == Action::Continue {
                            return Err(err(
                                Kind::Unsupported,
                                file,
                                line,
                                "SUCCESS=continue is outside this bounded lookup model",
                            ));
                        }
                        current.actions[status.index()] = action;
                    }
                    if !closed || count == 0 {
                        return Err(err(
                            Kind::Invalid,
                            file,
                            line,
                            "Close a nonempty action bracket with ]",
                        ));
                    }
                } else {
                    let service = match word {
                        "files" => Service::Files,
                        "dns" => Service::Dns,
                        _ => {
                            return Err(err(
                                Kind::Unsupported,
                                file,
                                line,
                                format!(
                                    "Unsupported NSS service or token {:?}; only files and dns are modeled",
                                    preview(word)
                                ),
                            ));
                        }
                    };
                    if sources.iter().any(|s| s.service == service) {
                        return Err(err(
                            Kind::Unsupported,
                            file,
                            line,
                            "Each supported service may appear once",
                        ));
                    }
                    if sources.len() == 2 {
                        return Err(err(Kind::Limit, file, line, "At most two name sources"));
                    }
                    sources.push(Source {
                        service,
                        actions: [
                            Action::Return,
                            Action::Continue,
                            Action::Continue,
                            Action::Continue,
                        ],
                    });
                }
            }
            if sources.is_empty() {
                return Err(err(
                    Kind::Invalid,
                    file,
                    line,
                    "Supply at least one source after hosts:",
                ));
            }
            parsed = Some(Self { sources });
        }
        parsed.ok_or_else(|| {
            err(
                Kind::Invalid,
                file,
                None,
                "Supply a hosts: row; an omitted database default is not modeled",
            )
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hosts {
    names: BTreeMap<String, Ipv4Addr>,
    canonical: BTreeMap<String, String>,
}
impl Hosts {
    pub fn names(&self) -> &BTreeMap<String, Ipv4Addr> {
        &self.names
    }
    /// The original row's canonical name, including its authored case. Parsed
    /// provenance only: this does not change saved configuration or old lookups.
    pub fn canonical(&self, query: &str) -> Option<&str> {
        self.canonical.get(&query.to_ascii_lowercase()).map(String::as_str)
    }
    pub fn parse(raw: &str) -> Result<Self, Error> {
        let file = "hosts";
        text(raw, file)?;
        let mut names = BTreeMap::new();
        let mut canonical = BTreeMap::new();
        let mut rows = 0;
        for (i, row) in raw.lines().enumerate() {
            let line = Some(i + 1);
            let row = row.split('#').next().unwrap_or("").trim();
            if row.is_empty() {
                continue;
            }
            rows += 1;
            if rows > MAX_ROWS {
                return Err(err(Kind::Limit, file, line, "At most64 host rows"));
            }
            let fields = row.split_ascii_whitespace().take(10).collect::<Vec<_>>();
            if fields.len() < 2 {
                return Err(err(
                    Kind::Invalid,
                    file,
                    line,
                    "Use IPv4-address canonical-name followed by optional aliases",
                ));
            }
            if fields.len() > 9 {
                return Err(err(
                    Kind::Limit,
                    file,
                    line,
                    "At most eight names per host row",
                ));
            }
            let ip=fields[0].parse::<Ipv4Addr>().map_err(|_| {
                if fields[0].contains(':') || fields[0].split('.').count()!=4 {
                    err(Kind::Unsupported,file,line,"Use dotted-decimal IPv4 here; IPv6 and abbreviated numeric forms are outside this activity")
                } else {
                    err(Kind::Invalid,file,line,"Malformed IPv4 address: use four decimal octets from0 through255, without leading zeros")
                }
            })?;
            for alias in &fields[1..] {
                if !name(alias) {
                    return Err(err(
                        Kind::Invalid,
                        file,
                        line,
                        format!(
                            "Invalid supported host name {:?}; use ordinary labels without a final dot",
                            preview(alias)
                        ),
                    ));
                }
                let key = alias.to_ascii_lowercase();
                if names.contains_key(&key) {
                    return Err(err(
                        Kind::Unsupported,
                        file,
                        line,
                        "Duplicate host names are outside the single-answer subset; keep each canonical name/alias unique",
                    ));
                }
                if names.len() >= MAX_NAMES {
                    return Err(err(Kind::Limit, file, line, "At most128 host names"));
                }
                canonical.insert(key.clone(), fields[1].into());
                names.insert(key, ip);
            }
        }
        Ok(Self { names, canonical })
    }
}
fn records<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Ipv4Addr>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Records;
    impl<'de> Visitor<'de> for Records {
        type Value = BTreeMap<String, Ipv4Addr>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most64 unique bounded DNS A records")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            #[derive(Deserialize)]
            struct Key(#[serde(deserialize_with = "crate::cron::text::<_,253>")] String);
            let mut out = BTreeMap::new();
            while let Some(Key(key)) = a.next_key::<Key>()? {
                if out.len() >= 64 {
                    return Err(A::Error::custom("DNS record-count limit"));
                }
                if !name(&key) || key != key.to_ascii_lowercase() || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "DNS names must be unique, lowercase and supported",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Records)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dns {
    pub status: Status,
    // Domain-bound admission is rechecked before every lookup. Session loaders must
    // also bound the outer serialized payload before deserializing this context.
    #[serde(deserialize_with = "records")]
    pub records: BTreeMap<String, Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Context {
    pub files_available: bool,
    pub dns: Dns,
}
impl Context {
    pub fn validate(&self) -> Result<(), Error> {
        if self.dns.records.len() > 64
            || self
                .dns
                .records
                .keys()
                .any(|s| !name(s) || s != &s.to_ascii_lowercase())
        {
            return Err(err(
                Kind::Limit,
                "Supplied DNS sample",
                None,
                "Use at most64 unique lowercase supported names",
            ));
        }
        if self.dns.status != Status::Success && !self.dns.records.is_empty() {
            return Err(err(
                Kind::Invalid,
                "Supplied DNS sample",
                None,
                "An unavailable, retrying or not-found service cannot also supply successful answers",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub service: Service,
    pub status: Status,
    pub action: Action,
    pub answer: Option<Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lookup {
    pub name: String,
    pub address: Option<Ipv4Addr>,
    pub status: Status,
    pub steps: Vec<Step>,
}
pub fn lookup(
    switch: &Switch,
    hosts: &Hosts,
    context: &Context,
    query: &str,
) -> Result<Lookup, Error> {
    context.validate()?;
    if !name(query) {
        return Err(err(
            Kind::Invalid,
            "Lookup",
            None,
            "Use one supported host name; numeric addresses, final dots, shell syntax and paths are not lookup names here",
        ));
    }
    let query = query.to_ascii_lowercase();
    let mut out = Lookup {
        name: query.clone(),
        address: None,
        status: Status::NotFound,
        steps: vec![],
    };
    for source in &switch.sources {
        let (answer, status) = match source.service {
            Service::Files => {
                if context.files_available {
                    let a = hosts.names.get(&query).copied();
                    (
                        a,
                        if a.is_some() {
                            Status::Success
                        } else {
                            Status::NotFound
                        },
                    )
                } else {
                    (None, Status::Unavailable)
                }
            }
            Service::Dns => {
                if context.dns.status == Status::Success {
                    let a = context.dns.records.get(&query).copied();
                    (
                        a,
                        if a.is_some() {
                            Status::Success
                        } else {
                            Status::NotFound
                        },
                    )
                } else {
                    (None, context.dns.status)
                }
            }
        };
        let action = source.action(status);
        out.steps.push(Step {
            service: source.service,
            status,
            action,
            answer,
        });
        out.status = status;
        if action == Action::Return {
            out.address = answer;
            return Ok(out);
        }
    }
    Ok(out)
}
