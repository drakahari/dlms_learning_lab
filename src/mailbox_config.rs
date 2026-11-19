//! Bounded Dovecot2.4 mail-access profile. Pure parsing; never opens a path.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_TEXT: usize = 8192;
pub const MAX_LINES: usize = 160;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Kind {
    Syntax,
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub kind: Kind,
    pub line: Option<usize>,
    pub message: String,
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
fn fail<T>(kind: Kind, line: usize, message: impl Into<String>) -> Result<T, Fault> {
    Err(Fault {
        kind,
        line: (line != 0).then_some(line),
        message: message.into(),
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Imap,
    Pop3,
}
impl Protocol {
    pub fn name(self) -> &'static str {
        match self {
            Self::Imap => "imap",
            Self::Pop3 => "pop3",
        }
    }
    fn parse(s: &str, line: usize) -> Result<Self, Fault> {
        match s {
            "imap" => Ok(Self::Imap),
            "pop3" => Ok(Self::Pop3),
            _ => fail(
                Kind::Unsupported,
                line,
                "Only the imap and pop3 protocol names are modeled in this profile.",
            ),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Ssl {
    No,
    Yes,
    Required,
}
impl Ssl {
    pub fn name(self) -> &'static str {
        match self {
            Self::No => "no",
            Self::Yes => "yes",
            Self::Required => "required",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Listener {
    pub protocol: Protocol,
    pub port: u16,
    pub ssl: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Config {
    pub protocols: BTreeSet<Protocol>,
    pub ssl: Ssl,
    pub auth_allow_cleartext: bool,
    pub pop3_no_flag_updates: bool,
    pub auth_mechanisms: BTreeSet<String>,
    pub certificate: String,
    pub key: String,
    pub mail_path: String,
    pub listeners: BTreeMap<String, Listener>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Block {
    Protocols,
    Service(Protocol),
    Listener(&'static str),
}
fn boolean(s: &str, line: usize) -> Result<bool, Fault> {
    match s.to_ascii_lowercase().as_str() {
        "yes" | "y" | "1" => Ok(true),
        "no" => Ok(false),
        _ => fail(
            Kind::Invalid,
            line,
            "Use yes (also y or1) or no. Dovecot does not treat the word false or the number0 as no.",
        ),
    }
}
/// One double-quoted or unquoted scalar, with quote-aware comments handled first.
fn scalar(s: &str, line: usize) -> Result<String, Fault> {
    if !s.starts_with('"') {
        if s.contains(['"', '\'', '\\']) {
            return fail(
                Kind::Unsupported,
                line,
                "Use an unquoted value or one double-quoted value; continuations and other quoting forms are outside this profile.",
            );
        }
        return Ok(s.trim().into());
    }
    let mut result = String::new();
    let mut chars = s[1..].char_indices();
    while let Some((at, ch)) = chars.next() {
        match ch {
            '"' => {
                if !s[at + 2..].trim().is_empty() {
                    return fail(
                        Kind::Syntax,
                        line,
                        "Unexpected text after the closing quote.",
                    );
                }
                return Ok(result);
            }
            '\\' => match chars.next() {
                Some((_, c @ ('"' | '\\'))) => result.push(c),
                _ => {
                    return fail(
                        Kind::Unsupported,
                        line,
                        "Only escaped double quotes and backslashes are supported in quoted values.",
                    );
                }
            },
            _ => result.push(ch),
        }
    }
    fail(Kind::Syntax, line, "Close the double-quoted value.")
}
fn uncomment(s: &str, line: usize) -> Result<&str, Fault> {
    let mut quote = false;
    let mut escape = false;
    for (at, ch) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && quote {
            escape = true;
            continue;
        }
        if ch == '"' {
            quote = !quote;
        } else if ch == '#' && !quote {
            return Ok(s[..at].trim());
        }
    }
    if quote {
        return fail(
            Kind::Syntax,
            line,
            "Close the double-quoted value before the end of its line.",
        );
    }
    Ok(s.trim())
}
fn protocol_list(raw: &str, line: usize) -> Result<BTreeSet<Protocol>, Fault> {
    let mut out = BTreeSet::new();
    let mut at = 0;
    while at < raw.len() {
        let rest = raw[at..].trim_start_matches(|c: char| c.is_ascii_whitespace() || c == ',');
        at = raw.len() - rest.len();
        if rest.is_empty() {
            break;
        }
        let end = if let Some(inner) = rest.strip_prefix('"') {
            let mut escaped = false;
            let mut end = None;
            for (i, c) in inner.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    end = Some(i + 2);
                    break;
                }
            }
            end.ok_or_else(|| Fault {
                kind: Kind::Syntax,
                line: Some(line),
                message: "Close the quoted protocol name.".into(),
            })?
        } else {
            rest.find(|c: char| c.is_ascii_whitespace() || c == ',')
                .unwrap_or(rest.len())
        };
        let name = scalar(&rest[..end], line)?;
        out.insert(Protocol::parse(&name, line)?);
        at += end;
        if at < raw.len() && !raw.as_bytes()[at].is_ascii_whitespace() && raw.as_bytes()[at] != b','
        {
            return fail(
                Kind::Syntax,
                line,
                "Separate protocol names with spaces or commas.",
            );
        }
    }
    Ok(out)
}
pub fn virtual_path(s: &str, mail: bool) -> bool {
    if s.is_empty() || s.len() > 256 {
        return false;
    }
    let source = if mail {
        s.strip_prefix("~/").unwrap_or(s)
    } else {
        s
    };
    if !(s.starts_with('/') || mail && s.starts_with("~/")) {
        return false;
    }
    let expanded = if mail {
        source.replace("%{user}", "sample-user")
    } else {
        source.into()
    };
    expanded
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
        && expanded
            .split('/')
            .filter(|p| !p.is_empty())
            .all(|p| p != "." && p != "..")
        && !s.contains("//")
        && !expanded.contains("//")
}
impl Config {
    pub fn parse(text: &str) -> Result<Self, Fault> {
        if text.len() > MAX_TEXT || text.lines().count() > MAX_LINES {
            return fail(
                Kind::Limit,
                0,
                "Keep the mail-access profile within8KiB and160lines.",
            );
        }
        if !text
            .bytes()
            .all(|b| b.is_ascii_graphic() || b" \t\r\n".contains(&b))
        {
            return fail(
                Kind::Unsupported,
                0,
                "This profile supports plain ASCII configuration text, without control characters.",
            );
        }
        let mut root = BTreeMap::new();
        let mut protocols = BTreeSet::new();
        let mut specified = false;
        let mut blocks = Vec::new();
        let mut listeners: BTreeMap<String, Listener> = [
            ("imap", Protocol::Imap, 143, false),
            ("imaps", Protocol::Imap, 993, true),
            ("pop3", Protocol::Pop3, 110, false),
            ("pop3s", Protocol::Pop3, 995, true),
        ]
        .into_iter()
        .map(|(name, protocol, port, ssl)| {
            (
                name.into(),
                Listener {
                    protocol,
                    port,
                    ssl,
                },
            )
        })
        .collect();
        for (n, raw) in text.lines().enumerate() {
            let line = n + 1;
            let raw = raw.strip_suffix('\r').unwrap_or(raw);
            if raw.contains('\r') {
                return fail(
                    Kind::Syntax,
                    line,
                    "Use ordinary line breaks, not a carriage return inside a setting.",
                );
            }
            let row = uncomment(raw, line)?;
            if row.is_empty() {
                continue;
            }
            if root.is_empty() && !row.starts_with("dovecot_config_version") {
                return fail(
                    Kind::Syntax,
                    line,
                    "The first setting must be dovecot_config_version =2.4.0 for this versioned profile.",
                );
            }
            if row.starts_with('!') {
                return fail(
                    Kind::Unsupported,
                    line,
                    "Includes and external files are outside this profile; no host file is read.",
                );
            }
            if row == "}" {
                if blocks.pop().is_none() {
                    return fail(Kind::Syntax, line, "This closing brace has no open block.");
                }
                continue;
            }
            if let Some(head) = row.strip_suffix('{') {
                let words: Vec<_> = head.split_ascii_whitespace().collect();
                let next = match (blocks.as_slice(), words.as_slice()) {
                    ([], ["protocols"]) => {
                        specified = true;
                        Block::Protocols
                    }
                    ([], ["service", "imap-login"]) => Block::Service(Protocol::Imap),
                    ([], ["service", "pop3-login"]) => Block::Service(Protocol::Pop3),
                    ([Block::Service(protocol)], ["inet_listener", name]) => {
                        let item = listeners.get(*name).ok_or_else(|| Fault {
                            kind: Kind::Unsupported,
                            line: Some(line),
                            message:
                                "Use the named imap, imaps, pop3 or pop3s listener in this profile."
                                    .into(),
                        })?;
                        if item.protocol != *protocol {
                            return fail(
                                Kind::Unsupported,
                                line,
                                "This profile reserves imap/imaps for imap-login and pop3/pop3s for pop3-login. Arbitrarily named listeners in broader Dovecot are not modeled.",
                            );
                        }
                        let name = match *name {
                            "imap" => "imap",
                            "imaps" => "imaps",
                            "pop3" => "pop3",
                            "pop3s" => "pop3s",
                            _ => unreachable!(),
                        };
                        Block::Listener(name)
                    }
                    _ => {
                        return fail(
                            Kind::Unsupported,
                            line,
                            "Only protocols and the supplied mail-login/inet_listener blocks are modeled. Other filters, namespaces and authentication backends are supplied separately.",
                        );
                    }
                };
                blocks.push(next);
                continue;
            }
            let Some((key, raw_value)) = row.split_once('=') else {
                return fail(
                    Kind::Syntax,
                    line,
                    "Use setting = value, with block braces on separate lines.",
                );
            };
            let key = key.trim();
            let raw_value = raw_value.trim();
            if key.is_empty() || key.contains([' ', '\t']) {
                return fail(
                    Kind::Syntax,
                    line,
                    "Use one setting name before the equals sign.",
                );
            }
            match blocks.as_slice() {
                [] => {
                    if key == "protocols" {
                        protocols = protocol_list(raw_value, line)?;
                        specified = true;
                        continue;
                    }
                    let value = scalar(raw_value, line)?;
                    match key {
                        "dovecot_config_version" | "dovecot_storage_version" => {
                            if value != "2.4.0" {
                                return fail(
                                    Kind::Unsupported,
                                    line,
                                    "This Dovecot2.4profile uses explicit config/storage version2.4.0, as in the2.4.1upstream example. Other version grammars are not evaluated.",
                                );
                            }
                        }
                        "ssl" => {
                            if !matches!(value.as_str(), "no" | "yes" | "required") {
                                return fail(
                                    Kind::Invalid,
                                    line,
                                    "ssl accepts no, yes or required in this profile.",
                                );
                            }
                        }
                        "auth_allow_cleartext" | "pop3_no_flag_updates" => {
                            boolean(&value, line)?;
                        }
                        "auth_mechanisms" => {
                            if value.is_empty() {
                                return fail(
                                    Kind::Invalid,
                                    line,
                                    "Supply at least one authentication mechanism.",
                                );
                            }
                            if value
                                .split_ascii_whitespace()
                                .any(|v| !matches!(v, "plain" | "login"))
                            {
                                return fail(
                                    Kind::Unsupported,
                                    line,
                                    "Only plain and login authentication mechanisms are modeled; no external SASL provider runs.",
                                );
                            }
                        }
                        "ssl_server_cert_file" | "ssl_server_key_file" => {
                            if !virtual_path(&value, false) {
                                return fail(
                                    Kind::Unsupported,
                                    line,
                                    "Use a literal absolute virtual fixture path without traversal, variables or inline key material; host paths are never opened.",
                                );
                            }
                        }
                        "mail_driver" => {
                            if value != "maildir" {
                                return fail(
                                    Kind::Unsupported,
                                    line,
                                    "Only the supplied virtual Maildir storage is modeled, not mbox or other drivers.",
                                );
                            }
                        }
                        "mail_path" => {
                            if !virtual_path(&value, true) {
                                return fail(
                                    Kind::Unsupported,
                                    line,
                                    "Use an absolute virtual mail path or ~/Maildir; only the %{user} placeholder is supported, without traversal or other expansions.",
                                );
                            }
                        }
                        "ssl_cert" | "ssl_key" | "mail_location" => {
                            return fail(
                                Kind::Unsupported,
                                line,
                                "Those are older Dovecot setting names. This2.4profile uses ssl_server_cert_file, ssl_server_key_file, mail_driver and mail_path.",
                            );
                        }
                        _ => {
                            return fail(
                                Kind::Unsupported,
                                line,
                                format!(
                                    "Setting {key} is outside this disclosed mail-access profile; it was not applied."
                                ),
                            );
                        }
                    }
                    root.insert(key.to_owned(), (value, line));
                }
                [Block::Protocols] => {
                    let p = Protocol::parse(key, line)?;
                    if boolean(&scalar(raw_value, line)?, line)? {
                        protocols.insert(p);
                    } else {
                        protocols.remove(&p);
                    }
                }
                [Block::Service(_), Block::Listener(name)] => {
                    let value = scalar(raw_value, line)?;
                    let listener = listeners.get_mut(*name).expect("Known listener");
                    match key {
                        "port" | "inet_listener_port" => {
                            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                                return fail(
                                    Kind::Invalid,
                                    line,
                                    "Use a decimal listener port0through65535;0disables this listener.",
                                );
                            }
                            listener.port = value.parse().map_err(|_| Fault {
                                kind: Kind::Invalid,
                                line: Some(line),
                                message: "Listener ports must fit0through65535.".into(),
                            })?;
                        }
                        "ssl" | "inet_listener_ssl" => listener.ssl = boolean(&value, line)?,
                        _ => {
                            return fail(
                                Kind::Unsupported,
                                line,
                                "Only port and ssl are supported inside these virtual listeners.",
                            );
                        }
                    }
                }
                _ => {
                    return fail(
                        Kind::Unsupported,
                        line,
                        "Put port/ssl inside an inet_listener block; broader service settings are not modeled.",
                    );
                }
            }
        }
        if root.is_empty() {
            return fail(
                Kind::Syntax,
                0,
                "The configuration is blank. Begin with dovecot_config_version =2.4.0, then the explicit profile settings.",
            );
        }
        if !blocks.is_empty() {
            return fail(
                Kind::Syntax,
                0,
                "Close every opened configuration block with its own closing-brace line.",
            );
        }
        if !specified {
            return fail(
                Kind::Unsupported,
                0,
                "Define protocols explicitly in this profile; wider daemon defaults are not guessed.",
            );
        }
        let get = |key: &str| {
            root.get(key).map(|v|v.0.clone()).ok_or_else(||Fault{kind:Kind::Unsupported,line:None,message:format!("Set {key} explicitly in this profile; full Dovecot defaults and included base files are outside this parser.")})
        };
        get("dovecot_config_version")?;
        get("dovecot_storage_version")?;
        get("mail_driver")?;
        let ssl = match get("ssl")?.as_str() {
            "no" => Ssl::No,
            "yes" => Ssl::Yes,
            "required" => Ssl::Required,
            _ => unreachable!(),
        };
        let pop3_no_flag_updates = root
            .get("pop3_no_flag_updates")
            .map(|(v, line)| boolean(v, *line))
            .transpose()?
            .unwrap_or(false);
        Ok(Self {
            protocols,
            ssl,
            pop3_no_flag_updates,
            auth_allow_cleartext: boolean(&get("auth_allow_cleartext")?, 0)?,
            auth_mechanisms: get("auth_mechanisms")?
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect(),
            certificate: get("ssl_server_cert_file")?,
            key: get("ssl_server_key_file")?,
            mail_path: get("mail_path")?,
            listeners,
        })
    }
    pub fn active_listeners(&self) -> impl Iterator<Item = (&str, &Listener)> {
        self.listeners
            .iter()
            .filter(|(_, l)| {
                l.port != 0
                    && self.protocols.contains(&l.protocol)
                    && (!l.ssl || self.ssl != Ssl::No)
            })
            .map(|(name, l)| (name.as_str(), l))
    }
    /// All modeled mechanisms carry a password. Local/forwarded secured exceptions
    /// are explicit inputs; the initial learner profile uses remote clients only.
    pub fn permits_password(&self, tls: bool, secured_exception: bool) -> bool {
        secured_exception
            || ((self.ssl != Ssl::Required || tls) && (self.auth_allow_cleartext || tls))
    }
    pub fn normalized(&self) -> String {
        let protocols = self
            .protocols
            .iter()
            .map(|p| p.name())
            .collect::<Vec<_>>()
            .join(" ");
        let mechanisms = self
            .auth_mechanisms
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ");
        let mut s = format!(
            "dovecot_config_version = 2.4.0\ndovecot_storage_version = 2.4.0\nprotocols = {protocols}\nssl = {}\nauth_allow_cleartext = {}\nauth_mechanisms = {mechanisms}\nssl_server_cert_file = {}\nssl_server_key_file = {}\nmail_driver = maildir\nmail_path = {}\n",
            self.ssl.name(),
            if self.auth_allow_cleartext {
                "yes"
            } else {
                "no"
            },
            self.certificate,
            self.key,
            self.mail_path
        );
        s.push_str(&format!(
            "pop3_no_flag_updates = {}\n",
            if self.pop3_no_flag_updates {
                "yes"
            } else {
                "no"
            }
        ));
        for p in [Protocol::Imap, Protocol::Pop3] {
            s.push_str(&format!("service {}-login {{\n", p.name()));
            for (name, l) in self.listeners.iter().filter(|(_, l)| l.protocol == p) {
                s.push_str(&format!(
                    "  inet_listener {name} {{\n    port = {}\n    ssl = {}\n  }}\n",
                    l.port,
                    if l.ssl { "yes" } else { "no" }
                ));
            }
            s.push_str("}\n");
        }
        s
    }
}
