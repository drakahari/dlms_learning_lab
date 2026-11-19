//! Pure bounded services-file lookup, never host NSS, files, sockets or commands.
use crate::transport_core::Protocol;
use serde::Serialize;
use std::collections::BTreeSet;
pub const MAX_FILE: usize = 32768;
pub const MAX_LINE: usize = 512;
pub const MAX_RECORDS: usize = 64;
pub const MAX_ALIASES: usize = 8;
pub const MAX_NAME: usize = 64;
pub const MAX_QUERY: usize = 80;
pub const MAX_COMMAND: usize = 256;
pub const MAX_OUTPUT: usize = 65536;
pub const HELP: &str = "Simulator reference (not a host shell): cat /etc/services; getent services; getent services NAME[/tcp|/udp]; getent services PORT[/tcp|/udp]. One literal key, no options or shell operators. /etc/services is supplied virtual text. A lookup does not start a service or prove that it is running. help is this simulator's reference helper.";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Syntax,
    Unsupported,
    Limit,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub line: Option<usize>,
    pub message: String,
}
fn fail(kind: FaultKind, line: Option<usize>, message: impl Into<String>) -> Fault {
    Fault {
        kind,
        line,
        message: message.into(),
    }
}
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(line) = self.line {
            write!(f, "Line {line}: ")?;
        }
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
pub fn protocol_name(p: Protocol) -> &'static str {
    match p {
        Protocol::Tcp => "tcp",
        Protocol::Udp => "udp",
    }
}
fn protocol(s: &str) -> Result<Protocol> {
    match s {
        "tcp" => Ok(Protocol::Tcp),
        "udp" => Ok(Protocol::Udp),
        _ => Err(fail(
            FaultKind::Unsupported,
            None,
            "This exercise supports lowercase tcp and udp records only; other Linux protocol names are outside its subset.",
        )),
    }
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= MAX_NAME
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
fn port(s: &str) -> Result<u16> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(fail(
            FaultKind::Syntax,
            None,
            "Use a decimal port number followed by /tcp or /udp.",
        ));
    }
    s.parse().map_err(|_| {
        fail(
            FaultKind::Invalid,
            None,
            "A service-database port must be between 0 and 65535.",
        )
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Record {
    pub name: String,
    pub port: u16,
    pub protocol: Protocol,
    pub aliases: Vec<String>,
    pub line: usize,
}
impl Record {
    pub fn output(&self) -> String {
        let mut line = format!(
            "{:<21} {}/{}",
            self.name,
            self.port,
            protocol_name(self.protocol)
        );
        for alias in &self.aliases {
            line.push(' ');
            line.push_str(alias);
        }
        line.push('\n');
        line
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Database {
    records: Vec<Record>,
}
impl Database {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_FILE {
            return Err(fail(
                FaultKind::Limit,
                None,
                "Keep the virtual services file within 32 KiB.",
            ));
        }
        if !text.is_ascii()
            || text
                .bytes()
                .any(|b| b.is_ascii_control() && !matches!(b, b'\n' | b'\r' | b'\t'))
        {
            return Err(fail(
                FaultKind::Unsupported,
                None,
                "The services-file subset supports printable ASCII, tabs and ordinary line endings only.",
            ));
        }
        let normalized = text.replace("\r\n", "\n");
        if normalized.contains('\r') {
            return Err(fail(
                FaultKind::Unsupported,
                None,
                "Use LF or CRLF line endings; isolated carriage returns are not interpreted.",
            ));
        }
        let mut records = Vec::new();
        let mut names = BTreeSet::new();
        let mut ports = BTreeSet::new();
        for (index, line) in normalized.lines().enumerate() {
            let line_number = Some(index + 1);
            if line.len() > MAX_LINE {
                return Err(fail(
                    FaultKind::Limit,
                    line_number,
                    "Keep each services-file line within 512 bytes.",
                ));
            }
            let text = line.split_once('#').map_or(line, |(before, _)| before);
            if text.trim().is_empty() {
                continue;
            }
            if text.starts_with([' ', '\t']) {
                return Err(fail(
                    FaultKind::Unsupported,
                    line_number,
                    "Start each service name in column 1 in this exercise; leading indentation is outside the supported file subset.",
                ));
            }
            let fields = text.split_ascii_whitespace().collect::<Vec<_>>();
            if fields.len() < 2 {
                return Err(fail(
                    FaultKind::Syntax,
                    line_number,
                    "Each entry needs a service name and a decimal port/protocol pair.",
                ));
            }
            if fields.len() > 2 + MAX_ALIASES {
                return Err(fail(
                    FaultKind::Limit,
                    line_number,
                    "Use at most 8 aliases for one service entry.",
                ));
            }
            if !name(fields[0]) || fields[2..].iter().any(|s| !name(s)) {
                return Err(fail(
                    FaultKind::Unsupported,
                    line_number,
                    "Use simple case-sensitive names: a letter, then letters, digits or hyphens, at most 64 characters.",
                ));
            }
            let (first, second) = fields[1]
                .split_once('/')
                .filter(|(_, p)| !p.contains('/'))
                .ok_or_else(|| {
                    fail(
                        FaultKind::Syntax,
                        line_number,
                        "Write the pair as a decimal port followed by /tcp or /udp.",
                    )
                })?;
            let value = port(first).map_err(|mut e| {
                e.line = line_number;
                e
            })?;
            let proto = protocol(second).map_err(|mut e| {
                e.line = line_number;
                e
            })?;
            let key = protocol_name(proto);
            if !ports.insert((key, value)) {
                return Err(fail(
                    FaultKind::Unsupported,
                    line_number,
                    "Repeated port/protocol entries are outside this unambiguous lookup exercise. Use one canonical entry with aliases; real NSS databases can differ.",
                ));
            }
            for field in fields.iter().take(1).chain(fields[2..].iter()) {
                if !names.insert((key, field.to_string())) {
                    return Err(fail(
                        FaultKind::Unsupported,
                        line_number,
                        "A name or alias occurs more than once for this protocol. Ambiguous/provider-dependent lookup is outside this exercise.",
                    ));
                }
            }
            records.push(Record {
                name: fields[0].into(),
                port: value,
                protocol: proto,
                aliases: fields[2..].iter().map(|s| s.to_string()).collect(),
                line: index + 1,
            });
            if records.len() > MAX_RECORDS {
                return Err(fail(
                    FaultKind::Limit,
                    line_number,
                    "Use at most 64 service entries.",
                ));
            }
        }
        Ok(Self { records })
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub fn lookup(&self, query: &Query) -> Option<&Record> {
        self.records.iter().find(|r| {
            query.protocol.is_none_or(|p| p == r.protocol)
                && match &query.target {
                    Target::Port(p) => *p == r.port,
                    Target::Name(n) => *n == r.name || r.aliases.contains(n),
                }
        })
    }
    pub fn output(&self) -> Result<String> {
        let mut text = String::new();
        for r in &self.records {
            text.push_str(&r.output());
            if text.len() > MAX_OUTPUT {
                return Err(fail(
                    FaultKind::Limit,
                    None,
                    "The services listing exceeds the simulator output limit.",
                ));
            }
        }
        Ok(text)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Target {
    Port(u16),
    Name(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    target: Target,
    protocol: Option<Protocol>,
}
impl Query {
    pub fn parse(text: &str) -> Result<Self> {
        if text.is_empty() {
            return Err(fail(
                FaultKind::Syntax,
                None,
                "Enter a service name or decimal port before looking it up.",
            ));
        }
        if text.len() > MAX_QUERY {
            return Err(fail(
                FaultKind::Limit,
                None,
                "Use one literal lookup key within 80 bytes.",
            ));
        }
        let (key, proto) = match text.split_once('/') {
            Some((key, p)) => {
                if p.contains('/') || p.is_empty() {
                    return Err(fail(
                        FaultKind::Syntax,
                        None,
                        "Use at most one /tcp or /udp suffix after the name or number.",
                    ));
                }
                (key, Some(protocol(p)?))
            }
            None => (text, None),
        };
        let target = if !key.is_empty() && key.bytes().all(|b| b.is_ascii_digit()) {
            Target::Port(port(key)?)
        } else if name(key) {
            Target::Name(key.into())
        } else {
            return Err(fail(
                FaultKind::Unsupported,
                None,
                "Use a literal case-sensitive service name or decimal port, optionally followed by /tcp or /udp. Shell syntax is not evaluated.",
            ));
        };
        Ok(Self {
            target,
            protocol: proto,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Output {
    pub stdout: String,
    pub status: u8,
}
/// A closed literal-argument dispatcher. An interpreter limitation is not a
/// manufactured getent result. All lookup text comes from the supplied database.
pub fn execute(text: &str, command: &str) -> Result<Output> {
    if command.len() > MAX_COMMAND {
        return Err(fail(
            FaultKind::Limit,
            None,
            "Keep a lookup command within 256 bytes.",
        ));
    }
    if !command.is_ascii()
        || command
            .bytes()
            .any(|b| (b.is_ascii_control() && b != b'\t') || b"'\"\\;$&|<>(){}[]?*#".contains(&b))
    {
        return Err(fail(
            FaultKind::Unsupported,
            None,
            "This terminal accepts literal arguments only. Shell quoting, comments, expansions, operators and scripts are not executed.",
        ));
    }
    let words = command.split_ascii_whitespace().collect::<Vec<_>>();
    if words == ["help"] {
        return Ok(Output {
            stdout: format!("{HELP}\n"),
            status: 0,
        });
    }
    if words == ["cat", "/etc/services"] {
        if text.len() > MAX_FILE {
            return Err(fail(
                FaultKind::Limit,
                None,
                "The virtual services file exceeds 32 KiB.",
            ));
        }
        return Ok(Output {
            stdout: text.to_owned(),
            status: 0,
        });
    }
    if words.len() < 2 || words[0] != "getent" || words[1] != "services" || words.len() > 3 {
        return Err(fail(FaultKind::Unsupported, None, HELP));
    }
    let data = Database::parse(text)?;
    if words.len() == 2 {
        return Ok(Output {
            stdout: data.output()?,
            status: 0,
        });
    }
    let query = Query::parse(words[2])?;
    Ok(data.lookup(&query).map_or_else(
        || Output {
            stdout: String::new(),
            status: 2,
        },
        |r| Output {
            stdout: r.output(),
            status: 0,
        },
    ))
}

#[cfg(test)]
mod tests;
