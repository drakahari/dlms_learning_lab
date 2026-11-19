//! Virtual mailbox/server lifecycle and logical TLS facts. No sockets, host
//! credentials, file reads, processes, real clock or cryptographic operations.
use crate::mailbox_config::{Config, Fault, Kind, Protocol, Ssl};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_MESSAGES: usize = 32;
pub const MAX_WIRE: usize = 8192;
pub const MAX_FACTS: usize = 256 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Issue {
    Configuration,
    Syntax,
    Unsupported,
    Limit,
    Service,
    Connection,
    Tls,
    Authentication,
    Mailbox,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub issue: Issue,
    pub message: String,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Failure {}
impl From<Fault> for Failure {
    fn from(f: Fault) -> Self {
        Self {
            issue: match f.kind {
                Kind::Syntax => Issue::Syntax,
                Kind::Invalid => Issue::Configuration,
                Kind::Unsupported => Issue::Unsupported,
                Kind::Limit => Issue::Limit,
            },
            message: f.to_string(),
        }
    }
}
fn fail<T>(issue: Issue, message: impl Into<String>) -> Result<T, Failure> {
    Err(Failure {
        issue,
        message: message.into(),
    })
}
pub(crate) fn within<T: Serialize>(value: &T, limit: usize) -> bool {
    struct Budget {
        used: usize,
        limit: usize,
    }
    impl std::io::Write for Budget {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.used = self
                .used
                .checked_add(b.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("Mail-access serialization limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget { used: 0, limit }, value).is_ok()
}
fn simple(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
}
pub fn dns(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_err() && crate::smtp_config::domain(s).is_ok()
}
pub fn mailbox_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.trim() == s
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" ._-".contains(&b))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Flag {
    Seen,
    Deleted,
}
impl Flag {
    pub fn wire(self) -> &'static str {
        match self {
            Self::Seen => "\\Seen",
            Self::Deleted => "\\Deleted",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub uid: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,(String,String),16>")]
    pub headers: Vec<(String, String)>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub body: String,
    pub flags: BTreeSet<Flag>,
}
impl Message {
    pub fn validate(&self) -> Result<(), Failure> {
        if self.uid == 0
            || self.headers.len() > 16
            || self.body.len() > 2048
            || !self.body.is_ascii()
        {
            return fail(
                Issue::Limit,
                "Use a nonzero UID and bounded plain-message fixtures.",
            );
        }
        if self
            .body
            .bytes()
            .any(|b| b < 32 && !b"\t\r\n".contains(&b) || b == 127)
        {
            return fail(
                Issue::Unsupported,
                "Plain message bodies cannot contain control bytes other than tabs and line breaks.",
            );
        }
        crate::sieve_eval::Message::new(self.headers.clone(), &self.body).map_err(|e| Failure {
            issue: Issue::Unsupported,
            message: format!("Plain mail fixture: {e}"),
        })?;
        if self.body.replace("\r\n", "").contains('\r') {
            return fail(
                Issue::Unsupported,
                "Use LF or CRLF body lines; bare carriage returns are not modeled.",
            );
        }
        if self.raw().len() > MAX_WIRE {
            return fail(
                Issue::Limit,
                "A represented message exceeds 8KiB of canonical wire text.",
            );
        }
        Ok(())
    }
    pub fn raw(&self) -> String {
        let mut s = String::new();
        for (n, v) in &self.headers {
            s.push_str(n);
            s.push_str(": ");
            s.push_str(v);
            s.push_str("\r\n");
        }
        s.push_str("\r\n");
        s.push_str(&self.body.replace("\r\n", "\n").replace('\n', "\r\n"));
        if !s.ends_with("\r\n") {
            s.push_str("\r\n");
        }
        s
    }
    pub fn flag_text(&self) -> String {
        self.flags
            .iter()
            .map(|f| f.wire())
            .collect::<Vec<_>>()
            .join(" ")
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mailbox {
    pub uid_validity: u32,
    pub uid_next: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Message,16>")]
    pub messages: Vec<Message>,
}
impl Mailbox {
    pub fn validate(&self) -> Result<(), Failure> {
        if self.uid_validity == 0 || self.uid_next == 0 || self.messages.len() > 16 {
            return fail(
                Issue::Configuration,
                "A mailbox needs nonzero UIDVALIDITY/UIDNEXT and at most 16 messages.",
            );
        }
        let mut last = 0;
        for m in &self.messages {
            m.validate()?;
            if m.uid <= last || m.uid >= self.uid_next {
                return fail(
                    Issue::Configuration,
                    "Message UIDs must increase strictly and stay below the persistent UIDNEXT value.",
                );
            }
            last = m.uid;
        }
        Ok(())
    }
    pub fn expunge(&mut self) -> Vec<usize> {
        let mut removed = Vec::new();
        let mut i = 0;
        while i < self.messages.len() {
            if self.messages[i].flags.contains(&Flag::Deleted) {
                removed.push(i + 1);
                self.messages.remove(i);
            } else {
                i += 1;
            }
        }
        removed
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub sample_password: String,
    pub enabled: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub home: String,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub maildir_path: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Certificate {
    pub key_id: String,
    pub issuer_id: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,String,8>")]
    pub dns_names: Vec<String>,
    pub valid_from_day: u32,
    pub valid_through_day: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub key_id: String,
    pub owner: String,
    pub mode: u16,
}
pub type Store = BTreeMap<String, BTreeMap<String, Mailbox>>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub server_name: String,
    pub day: u32,
    pub accounts: BTreeMap<String, Account>,
    pub mailboxes: Store,
    pub certificates: BTreeMap<String, Certificate>,
    pub keys: BTreeMap<String, Key>,
}
impl Facts {
    pub fn validate(&self) -> Result<(), Failure> {
        if !dns(&self.server_name)
            || self.accounts.is_empty()
            || self.accounts.len() > 4
            || self.accounts.keys().ne(self.mailboxes.keys())
            || self.certificates.len() > 8
            || self.keys.len() > 8
        {
            return fail(
                Issue::Configuration,
                "Supply one virtual DNS endpoint,1to4accounts with matching stores, and bounded certificate/key fixtures.",
            );
        }
        if !within(self, MAX_FACTS) {
            return fail(Issue::Limit, "Mail access facts exceed256KiB.");
        }
        let mut count = 0;
        for (user, a) in &self.accounts {
            if !name(user)
                || a.sample_password.is_empty()
                || a.sample_password.len() > 64
                || !a.sample_password.bytes().all(|b| b.is_ascii_graphic())
                || !crate::mailbox_config::virtual_path(&a.home, false)
                || !crate::mailbox_config::virtual_path(&a.maildir_path, false)
            {
                return fail(
                    Issue::Configuration,
                    "Use short supplied usernames, synthetic one-word passwords and absolute virtual home/store paths.",
                );
            }
            let boxes = &self.mailboxes[user];
            if boxes.is_empty() || boxes.len() > 4 || !boxes.contains_key("INBOX") {
                return fail(
                    Issue::Configuration,
                    "Each account needs INBOX and at most4flat virtual mailboxes.",
                );
            }
            for (n, b) in boxes {
                if !mailbox_name(n) || n.eq_ignore_ascii_case("INBOX") && n != "INBOX" {
                    return fail(
                        Issue::Configuration,
                        "Use flat supplied mailbox names; store the special INBOX name in uppercase.",
                    );
                }
                b.validate()?;
                count += b.messages.len();
            }
        }
        if count > MAX_MESSAGES {
            return fail(
                Issue::Limit,
                "The virtual endpoint supports at most 32 sample messages in total.",
            );
        }
        for (p, c) in &self.certificates {
            if !crate::mailbox_config::virtual_path(p, false)
                || !simple(&c.key_id)
                || !simple(&c.issuer_id)
                || c.dns_names.is_empty()
                || c.dns_names.len() > 8
                || c.dns_names.iter().any(|n| !dns(n))
                || c.valid_from_day > c.valid_through_day
            {
                return fail(
                    Issue::Configuration,
                    "Certificates need virtual paths, bounded logical key/issuer IDs, exact DNS names and an ordered day interval.",
                );
            }
        }
        for (p, k) in &self.keys {
            if !crate::mailbox_config::virtual_path(p, false)
                || !simple(&k.key_id)
                || !name(&k.owner)
                || k.mode > 0o7777
            {
                return fail(
                    Issue::Configuration,
                    "Key fixtures need virtual paths, logical key IDs, known-style owners and Unix mode bits, never real key material.",
                );
            }
        }
        Ok(())
    }
    pub fn resolved_maildir(&self, c: &Config, user: &str) -> Result<String, Failure> {
        let a = self.accounts.get(user).ok_or_else(|| Failure {
            issue: Issue::Authentication,
            message: "Unknown sample account.".into(),
        })?;
        let path = c.mail_path.replace("%{user}", user);
        let result = if let Some(tail) = path.strip_prefix("~/") {
            format!("{}/{tail}", a.home.trim_end_matches('/'))
        } else {
            path
        };
        if result != a.maildir_path {
            return fail(
                Issue::Mailbox,
                format!(
                    "The resolved virtual Maildir is {result}, but this account's supplied mail is at {}. Compare mail_path with the home/store facts.",
                    a.maildir_path
                ),
            );
        }
        Ok(result)
    }
    pub fn startup(&self, c: &Config) -> Result<(), Failure> {
        let mut ports = BTreeSet::new();
        for (_, l) in c.active_listeners() {
            if !ports.insert(l.port) {
                return fail(
                    Issue::Service,
                    format!(
                        "Two enabled virtual listeners claim port {}. The service cannot bind both at this endpoint.",
                        l.port
                    ),
                );
            }
        }
        // Explicit fixture-profile boundary: declared certificate paths always name
        // supplied metadata. No behavior of real missing host files is invented.
        let cert=self.certificates.get(&c.certificate).ok_or_else(||Failure{issue:Issue::Unsupported,message:"The certificate path is not a supplied virtual fixture. This profile never reads a host file.".into()})?;
        let key=self.keys.get(&c.key).ok_or_else(||Failure{issue:Issue::Unsupported,message:"The key path is not a supplied virtual fixture. This profile never reads a host file.".into()})?;
        if c.ssl != Ssl::No && cert.key_id != key.key_id {
            return fail(
                Issue::Service,
                "The selected virtual certificate and private key have different key identities. Choose a matching pair.",
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub protocol: Protocol,
    pub port: u16,
    pub implicit_tls: bool,
    pub server_name: String,
    pub trusted_issuers: BTreeSet<String>,
}
impl Client {
    pub fn validate(&self) -> Result<(), Failure> {
        if self.port == 0
            || !dns(&self.server_name)
            || self.trusted_issuers.len() > 8
            || self.trusted_issuers.iter().any(|s| !simple(s))
        {
            return fail(
                Issue::Configuration,
                "Supply a nonzero client port, exact TLS DNS name and bounded virtual trust IDs.",
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Log {
    pub number: u32,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub saved: String,
    pub active: Option<String>,
    pub running: bool,
    pub enabled: bool,
    pub generation: u32,
    pub store: Store,
    pub logs: Vec<Log>,
    pub error: Option<Failure>,
    /// Simulator grading observations, not a real Dovecot setting or mailbox flag.
    /// Older under-development snapshots receive no invented POP commit credit.
    #[serde(default)]
    pub pop_committed: BTreeMap<String, BTreeSet<u32>>,
}
impl Runtime {
    pub fn initial(
        f: &Facts,
        saved: &str,
        active: Option<&str>,
        enabled: bool,
    ) -> Result<Self, Failure> {
        f.validate()?;
        if saved.len() > crate::mailbox_config::MAX_TEXT
            || saved.lines().count() > crate::mailbox_config::MAX_LINES
        {
            return fail(
                Issue::Limit,
                "Saved configuration exceeds its text/line limit.",
            );
        }
        if let Some(text) = active {
            f.startup(&Config::parse(text)?)?;
        }
        Ok(Self {
            saved: saved.into(),
            active: active.map(str::to_owned),
            running: active.is_some(),
            enabled,
            generation: 0,
            store: f.mailboxes.clone(),
            logs: vec![],
            error: None,
            pop_committed: BTreeMap::new(),
        })
    }
    pub fn save(&mut self, text: &str) -> Result<(), Failure> {
        if text.len() > crate::mailbox_config::MAX_TEXT
            || text.lines().count() > crate::mailbox_config::MAX_LINES
        {
            return fail(
                Issue::Limit,
                "Keep the saved mail-access file within 8KiB / 160 lines; the previous file is retained.",
            );
        }
        self.saved = text.into();
        Ok(())
    }
    fn log(&mut self, message: String) {
        let number = self.logs.last().map_or(1, |l| l.number.saturating_add(1));
        if self.logs.len() == 64 {
            self.logs.remove(0);
        }
        self.logs.push(Log { number, message });
    }
    pub fn restart(&mut self, f: &Facts) -> Result<(), Failure> {
        self.running = false;
        self.active = None;
        self.generation = self.generation.checked_add(1).ok_or_else(|| Failure {
            issue: Issue::Limit,
            message: "Virtual restart counter exhausted; reset this attempt.".into(),
        })?;
        let result = Config::parse(&self.saved)
            .map_err(Failure::from)
            .and_then(|c| f.startup(&c));
        if let Err(e) = result {
            self.error = Some(e.clone());
            self.log(format!("dovecot.service: Failed to start: {e}"));
            return Err(e);
        }
        self.active = Some(self.saved.clone());
        self.running = true;
        self.error = None;
        self.log("dovecot.service: Started virtual mail access.".into());
        Ok(())
    }
    pub fn stop(&mut self) {
        self.running = false;
        self.active = None;
        self.generation = self.generation.saturating_add(1);
        self.log("dovecot.service: Stopped; stored messages retained.".into());
    }
    pub fn active_config(&self) -> Result<Config, Failure> {
        if !self.running {
            return fail(
                Issue::Service,
                "The virtual dovecot.service is stopped. Inspect its saved configuration and last restart result.",
            );
        }
        Config::parse(self.active.as_deref().ok_or_else(|| Failure {
            issue: Issue::Service,
            message: "There is no active server configuration.".into(),
        })?)
        .map_err(Into::into)
    }
    pub fn connect(&self, f: &Facts, client: &Client) -> Result<Link, Failure> {
        client.validate()?;
        let cfg = self.active_config()?;
        let listener=cfg.active_listeners().find(|(_,l)|l.port==client.port).map(|(_,l)|l).ok_or_else(||Failure{issue:Issue::Connection,message:format!("Connection refused on virtual port {}. Compare enabled protocols and listener ports; 0 disables a listener.",client.port)})?;
        if listener.protocol != client.protocol {
            return fail(
                Issue::Connection,
                "This port serves a different mail protocol. Check the client's IMAP/POP choice and selected port.",
            );
        }
        if listener.ssl != client.implicit_tls {
            return fail(
                Issue::Tls,
                if listener.ssl {
                    "This listener requires TLS on connect, before any mail protocol command."
                } else {
                    "This listener expects a plain protocol greeting and optional STARTTLS/STLS, not TLS on connect."
                },
            );
        }
        let mut link = Link {
            protocol: client.protocol,
            tls: false,
            generation: self.generation,
            client: client.clone(),
            closed: false,
        };
        if client.implicit_tls {
            link.start_tls(self, f)?;
        }
        Ok(link)
    }
    pub fn authenticate(
        &self,
        f: &Facts,
        link: &Link,
        user: &str,
        password: &str,
    ) -> Result<(), Failure> {
        link.current(self)?;
        let cfg = self.active_config()?;
        if !cfg.permits_password(link.tls, false) {
            return fail(
                Issue::Authentication,
                "Plaintext authentication is disabled for this remote connection. Establish verified TLS before sending a sample password.",
            );
        }
        if !cfg.auth_mechanisms.contains("plain") {
            return fail(
                Issue::Unsupported,
                "This profile's USER/PASS and IMAP LOGIN paths use PLAIN authentication; other SASL exchanges are not modeled.",
            );
        }
        if f.accounts
            .get(user)
            .is_none_or(|a| !a.enabled || a.sample_password != password)
        {
            return fail(
                Issue::Authentication,
                "Authentication failed for the supplied sample account; verify the username and sample password.",
            );
        }
        Ok(())
    }
    pub fn mailbox<'a>(
        &'a self,
        f: &Facts,
        user: &str,
        name: &str,
    ) -> Result<&'a Mailbox, Failure> {
        let cfg = self.active_config()?;
        f.resolved_maildir(&cfg, user)?;
        let name = if name.eq_ignore_ascii_case("INBOX") {
            "INBOX"
        } else {
            name
        };
        self.store
            .get(user)
            .and_then(|m| m.get(name))
            .ok_or_else(|| Failure {
                issue: Issue::Mailbox,
                message: "No such supplied mailbox for this authenticated user.".into(),
            })
    }
    pub fn mailbox_mut<'a>(
        &'a mut self,
        f: &Facts,
        user: &str,
        name: &str,
    ) -> Result<&'a mut Mailbox, Failure> {
        let cfg = self.active_config()?;
        f.resolved_maildir(&cfg, user)?;
        let name = if name.eq_ignore_ascii_case("INBOX") {
            "INBOX"
        } else {
            name
        };
        self.store
            .get_mut(user)
            .and_then(|m| m.get_mut(name))
            .ok_or_else(|| Failure {
                issue: Issue::Mailbox,
                message: "No such supplied mailbox for this authenticated user.".into(),
            })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub protocol: Protocol,
    pub tls: bool,
    pub generation: u32,
    pub client: Client,
    pub closed: bool,
}
impl Link {
    pub fn current(&self, r: &Runtime) -> Result<(), Failure> {
        if self.closed || !r.running || self.generation != r.generation {
            return fail(
                Issue::Connection,
                "The simulated connection is closed or belongs to a previous server start. Connect again; pending POP deletions have not been committed.",
            );
        }
        Ok(())
    }
    pub fn start_tls(&mut self, r: &Runtime, f: &Facts) -> Result<(), Failure> {
        self.current(r)?;
        if self.tls {
            return fail(Issue::Tls, "TLS is already active on this connection.");
        }
        let cfg = r.active_config()?;
        if cfg.ssl == Ssl::No {
            return fail(
                Issue::Tls,
                "TLS is disabled in the active server configuration.",
            );
        }
        let cert = f
            .certificates
            .get(&cfg.certificate)
            .ok_or_else(|| Failure {
                issue: Issue::Tls,
                message: "No supplied virtual certificate matches the active path.".into(),
            })?;
        let result = if !self.client.trusted_issuers.contains(&cert.issuer_id) {
            fail(
                Issue::Tls,
                "TLS verification failed: the client does not trust this virtual issuing CA.",
            )
        } else if !cert.dns_names.iter().any(|n| {
            n.trim_end_matches('.')
                .eq_ignore_ascii_case(self.client.server_name.trim_end_matches('.'))
        }) {
            fail(
                Issue::Tls,
                "TLS verification failed: the requested server name is absent from this certificate's virtual DNS names.",
            )
        } else if f.day < cert.valid_from_day || f.day > cert.valid_through_day {
            fail(
                Issue::Tls,
                "TLS verification failed: the virtual certificate is not valid on this exercise's supplied day.",
            )
        } else {
            Ok(())
        };
        if result.is_err() {
            self.closed = true;
        } else {
            self.tls = true;
        }
        result
    }
}
