//! Bounded configuration inspection and mailbox administration on virtual facts.
//! No host shell, file access, socket, process or environment expansion.
use crate::{
    mailbox_access::{Facts, Failure, Flag, Issue, Message, Runtime},
    mailbox_config::Config,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub changed: bool,
}
fn error<T>(issue: Issue, text: impl Into<String>) -> Result<T, Failure> {
    Err(Failure {
        issue,
        message: text.into(),
    })
}
/// Literal command words with quoting. This is not a shell evaluator.
fn words(line: &str) -> Result<Vec<String>, Failure> {
    if line.len() > 2048 {
        return error(Issue::Limit, "Keep one administration command within 2 KiB.");
    }
    if !line.is_ascii() || line.bytes().any(|b| b.is_ascii_control() && b != b'\t') {
        return error(
            Issue::Unsupported,
            "This command field supports one ASCII command, without control characters or multiple lines.",
        );
    }
    let mut words = vec![];
    let mut value = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        match quote {
            Some('\'') => {
                if ch == '\'' {
                    quote = None;
                } else {
                    value.push(ch);
                }
            }
            Some('"') => match ch {
                '"' => quote = None,
                '$' | '`' => {
                    return error(
                        Issue::Unsupported,
                        "Variable and command expansion are not supported here; supply literal arguments.",
                    );
                }
                '\\' => match chars.next() {
                    Some(c @ ('$' | '`' | '"' | '\\')) => value.push(c),
                    Some(c) => {
                        value.push('\\');
                        value.push(c)
                    }
                    None => {
                        return error(
                            Issue::Syntax,
                            "Complete the quoted argument after the backslash.",
                        );
                    }
                },
                _ => value.push(ch),
            },
            None => match ch {
                '\'' | '"' => {
                    quote = Some(ch);
                    started = true;
                }
                '\\' => {
                    value.push(chars.next().ok_or_else(|| Failure {
                        issue: Issue::Syntax,
                        message: "Complete the escaped argument.".into(),
                    })?);
                    started = true;
                }
                ' ' | '\t' => {
                    if started {
                        words.push(std::mem::take(&mut value));
                        started = false;
                    }
                }
                '$' | '`' | '|' | '&' | ';' | '<' | '>' | '(' | ')' => {
                    return error(
                        Issue::Unsupported,
                        "Pipelines, redirection, expansion and command lists are outside this administration-command subset.",
                    );
                }
                c if c.is_ascii_control() => {
                    return error(
                        Issue::Unsupported,
                        "Control characters are outside this command subset.",
                    );
                }
                _ => {
                    value.push(ch);
                    started = true;
                }
            },
            _ => unreachable!(),
        }
        if words.len() > 64 {
            return error(Issue::Limit, "Use no more than 64 literal arguments.");
        }
    }
    if quote.is_some() {
        return error(
            Issue::Syntax,
            "Close the quoted argument before running the command.",
        );
    }
    if started {
        words.push(value);
    }
    if words.is_empty() {
        return error(
            Issue::Syntax,
            "Type a supported doveconf or doveadm command.",
        );
    }
    if words.len() > 64 {
        return error(Issue::Limit, "Use no more than 64 literal arguments.");
    }
    Ok(words)
}
fn setting(c: &Config, key: &str) -> Result<String, Failure> {
    Ok(match key {
        "dovecot_config_version" | "dovecot_storage_version" => "2.4.0".into(),
        "protocols" => c
            .protocols
            .iter()
            .map(|p| p.name())
            .collect::<Vec<_>>()
            .join(" "),
        "ssl" => c.ssl.name().into(),
        "auth_allow_cleartext" => if c.auth_allow_cleartext { "yes" } else { "no" }.into(),
        "pop3_no_flag_updates" => if c.pop3_no_flag_updates { "yes" } else { "no" }.into(),
        "auth_mechanisms" => c
            .auth_mechanisms
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" "),
        "ssl_server_cert_file" => c.certificate.clone(),
        "ssl_server_key_file" => c.key.clone(),
        "mail_driver" => "maildir".into(),
        "mail_path" => c.mail_path.clone(),
        _ => {
            return error(
                Issue::Unsupported,
                format!("Setting {key:?} is outside the explicitly modeled configuration profile."),
            );
        }
    })
}
fn doveconf(r: &Runtime, args: &[String]) -> Result<Output, Failure> {
    let hide = args.first().is_some_and(|s| s == "-h");
    let keys = &args[usize::from(hide)..];
    if keys.is_empty() || keys.iter().any(|s| s.starts_with('-') || s.contains('/')) {
        return error(
            Issue::Unsupported,
            "Supported: doveconf [-h] setting [setting ...]. Full/default dumps, filters, includes, alternate files and overrides are not modeled.",
        );
    }
    let c = Config::parse(&r.saved).map_err(Failure::from)?;
    let mut text = String::new();
    for key in keys {
        let value = setting(&c, key)?;
        if !hide {
            text.push_str(key);
            text.push_str(" = ");
        }
        text.push_str(&value);
        text.push('\n');
    }
    Ok(Output {
        text,
        changed: false,
    })
}
#[derive(Clone, Debug)]
enum Predicate {
    Uid(Vec<(u32, u32)>),
    Seen(bool),
    Deleted(bool),
    Subject(String),
}
#[derive(Clone, Debug)]
struct Query {
    mailbox: Option<String>,
    predicates: Vec<Predicate>,
}
fn uid_set(value: &str) -> Result<Vec<(u32, u32)>, Failure> {
    if value.contains('*') {
        return error(
            Issue::Unsupported,
            "The last-message * UID marker is not modeled yet; use explicit positive UID numbers/ranges.",
        );
    }
    let number = |s: &str| {
        s.parse::<u32>()
            .ok()
            .filter(|n| *n > 0 && s.bytes().all(|b| b.is_ascii_digit()))
            .ok_or_else(|| Failure {
                issue: Issue::Syntax,
                message: "UIDs are positive numbers; use commas or inclusive colon ranges.".into(),
            })
    };
    let mut result = vec![];
    for item in value.split(',') {
        let parts = item.split(':').collect::<Vec<_>>();
        let (a, b) = match parts.as_slice() {
            [one] => {
                let n = number(one)?;
                (n, n)
            }
            [a, b] => (number(a)?, number(b)?),
            _ => {
                return error(
                    Issue::Syntax,
                    "Use UID numbers or inclusive ranges such as101:109.",
                );
            }
        };
        result.push((a.min(b), a.max(b)));
        if result.len() > 16 {
            return error(Issue::Limit, "Use at most 16 UID ranges.");
        }
    }
    Ok(result)
}
fn query(args: &[String]) -> Result<Query, Failure> {
    if args.is_empty() {
        return error(
            Issue::Syntax,
            "Supply a search query, for example mailbox INBOX unseen.",
        );
    }
    let mut q = Query {
        mailbox: None,
        predicates: vec![],
    };
    let mut at = 0;
    while at < args.len() {
        let key = args[at].to_ascii_lowercase();
        at += 1;
        match key.as_str() {
            "all" => {}
            "seen" => q.predicates.push(Predicate::Seen(true)),
            "unseen" => q.predicates.push(Predicate::Seen(false)),
            "deleted" => q.predicates.push(Predicate::Deleted(true)),
            "undeleted" => q.predicates.push(Predicate::Deleted(false)),
            "mailbox" | "uid" | "subject" => {
                let value = args.get(at).ok_or_else(|| Failure {
                    issue: Issue::Syntax,
                    message: format!("Give {key} its argument."),
                })?;
                at += 1;
                match key.as_str() {
                    "mailbox" => {
                        if q.mailbox.is_some() {
                            return error(
                                Issue::Unsupported,
                                "This subset supports one exact mailbox per query.",
                            );
                        }
                        if value.contains(['*', '%', '?']) {
                            return error(
                                Issue::Unsupported,
                                "Mailbox patterns are outside this profile; name one exact supplied mailbox.",
                            );
                        }
                        q.mailbox = Some(if value.eq_ignore_ascii_case("INBOX") {
                            "INBOX".into()
                        } else {
                            value.clone()
                        });
                    }
                    "uid" => q.predicates.push(Predicate::Uid(uid_set(value)?)),
                    _ => {
                        if value.is_empty() {
                            return error(
                                Issue::Unsupported,
                                "Empty SUBJECT matching is outside this profile; provide nonempty literal text.",
                            );
                        }
                        q.predicates
                            .push(Predicate::Subject(value.to_ascii_lowercase()));
                    }
                }
            }
            _ => {
                return error(
                    Issue::Unsupported,
                    format!(
                        "Search term {key:?} is not modeled. Supported terms: ALL, MAILBOX exact-name, UID positive-set, SEEN/UNSEEN, DELETED/UNDELETED and SUBJECT literal-text; terms combine with AND."
                    ),
                );
            }
        }
        if q.predicates.len() > 16 {
            return error(Issue::Limit, "Use at most 16 search conditions.");
        }
    }
    Ok(q)
}
impl Query {
    fn matches(&self, name: &str, m: &Message) -> bool {
        self.mailbox.as_deref().is_none_or(|n| n == name)
            && self.predicates.iter().all(|p| match p {
                Predicate::Uid(ranges) => ranges.iter().any(|(a, b)| (*a..=*b).contains(&m.uid)),
                Predicate::Seen(yes) => m.flags.contains(&Flag::Seen) == *yes,
                Predicate::Deleted(yes) => m.flags.contains(&Flag::Deleted) == *yes,
                Predicate::Subject(text) => m.headers.iter().any(|(k, v)| {
                    k.eq_ignore_ascii_case("Subject") && v.to_ascii_lowercase().contains(text)
                }),
            })
    }
}
/// Stable synthetic GUID-shaped fixture identity, not a Dovecot GUID algorithm.
fn guid(user: &str, name: &str, validity: u32) -> String {
    let mut n = 0xcbf29ce484222325u64;
    for b in user.bytes().chain([0]).chain(name.bytes()) {
        n = (n ^ u64::from(b)).wrapping_mul(0x100000001b3);
    }
    format!("{n:016x}{validity:016x}")
}
fn doveadm(r: &mut Runtime, f: &Facts, args: &[String]) -> Result<Output, Failure> {
    let Some(verb) = args.first() else {
        return error(Issue::Syntax, "Supply a supported doveadm action.");
    };
    let (kind, mut at) = if verb == "mailbox" && args.get(1).is_some_and(|s| s == "list") {
        ("list", 2)
    } else {
        (verb.as_str(), 1)
    };
    if !matches!(kind, "list" | "search" | "fetch" | "expunge") {
        return error(
            Issue::Unsupported,
            "Supported doveadm actions: mailbox list, search, fetch and expunge. Other administration, sockets and authentication backends are not simulated.",
        );
    }
    if args.get(at).map(String::as_str) != Some("-u") {
        return error(
            Issue::Unsupported,
            "Choose one supplied account with -u USER; all-user, wildcard and external-user operations are outside this subset.",
        );
    }
    at += 1;
    let user = args.get(at).ok_or_else(|| Failure {
        issue: Issue::Syntax,
        message: "Give -u a supplied account name.".into(),
    })?;
    at += 1;
    if !f.accounts.get(user).is_some_and(|a| a.enabled) {
        return error(
            Issue::Authentication,
            "No enabled supplied account has that name.",
        );
    }
    let cfg = if r.running {
        r.active_config()?
    } else {
        Config::parse(&r.saved).map_err(Failure::from)?
    };
    f.resolved_maildir(&cfg, user)?;
    let boxes = r.store.get(user).ok_or_else(|| Failure {
        issue: Issue::Mailbox,
        message: "This supplied account has no represented mail store.".into(),
    })?;
    if boxes.values().any(|b| {
        b.messages.iter().any(|m| {
            m.headers
                .iter()
                .any(|(k, v)| !k.is_ascii() || !v.is_ascii())
        })
    }) {
        return error(
            Issue::Unsupported,
            "This administration profile supports supplied ASCII headers; encoded/MIME header matching is not modeled.",
        );
    }
    if kind == "list" {
        if at != args.len() {
            return error(
                Issue::Unsupported,
                "Mailbox list supports the exact supplied account only; patterns/subscriptions/encoding options are not modeled.",
            );
        }
        return Ok(Output {
            text: boxes.keys().map(|s| format!("{s}\n")).collect(),
            changed: false,
        });
    }
    let fields = if kind == "fetch" {
        let fields = args
            .get(at)
            .ok_or_else(|| Failure {
                issue: Issue::Syntax,
                message: "Supply quoted fetch fields, for example 'uid flags hdr.subject'.".into(),
            })?
            .split_ascii_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        at += 1;
        if fields.is_empty()
            || fields.len() > 8
            || fields.iter().any(|s| {
                !matches!(
                    s.as_str(),
                    "uid" | "flags" | "mailbox" | "hdr.subject" | "body"
                )
            })
        {
            return error(
                Issue::Unsupported,
                "Fetch fields modeled: uid, flags, mailbox, hdr.subject and plain body. MIME, indexes and other fields are outside this subset.",
            );
        }
        fields
    } else {
        vec![]
    };
    let q = query(&args[at..])?;
    if q.mailbox.as_ref().is_some_and(|n| !boxes.contains_key(n)) {
        return error(
            Issue::Mailbox,
            "No such exact supplied mailbox; paths and wildcard patterns are not modeled.",
        );
    }
    if kind == "expunge" && q.mailbox.is_none() {
        return error(
            Issue::Unsupported,
            "This simulator requires MAILBOX exact-name for expunge; unscoped deletion is not implemented.",
        );
    }
    let mut text = String::new();
    let mut matches = vec![];
    for (name, b) in boxes {
        for m in &b.messages {
            if q.matches(name, m) {
                matches.push((name.clone(), m.uid));
                match kind {
                    "search" => {
                        text.push_str(&format!("{} {}\n", guid(user, name, b.uid_validity), m.uid))
                    }
                    "fetch" => {
                        if !text.is_empty() {
                            text.push('\u{c}');
                            text.push('\n');
                        }
                        for field in &fields {
                            let v = match field.as_str() {
                                "uid" => m.uid.to_string(),
                                "flags" => m.flag_text(),
                                "mailbox" => name.clone(),
                                "hdr.subject" => m
                                    .headers
                                    .iter()
                                    .filter(|(k, _)| k.eq_ignore_ascii_case("Subject"))
                                    .map(|(_, v)| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join("\n"),
                                _ => m.body.clone(),
                            };
                            text.push_str(&format!("{field}: {v}\n"));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    if kind == "expunge" {
        for (name, b) in r.store.get_mut(user).unwrap() {
            b.messages
                .retain(|m| !matches.iter().any(|(n, uid)| n == name && *uid == m.uid));
        }
    }
    Ok(Output {
        text,
        changed: kind == "expunge" && !matches.is_empty(),
    })
}
/// Transactional result application: malformed/unsupported/oversized output changes nothing.
pub fn run(r: &mut Runtime, f: &Facts, line: &str) -> Result<Output, Failure> {
    r.validate(f)?;
    let args = words(line)?;
    let mut scratch = r.clone();
    let out = match args[0].as_str() {
        "doveconf" => doveconf(&scratch, &args[1..])?,
        "doveadm" => doveadm(&mut scratch, f, &args[1..])?,
        _ => {
            return error(
                Issue::Unsupported,
                "Only the documented doveconf and doveadm subset is supported in this administration field.",
            );
        }
    };
    if out.text.len() > 32 * 1024 {
        return error(
            Issue::Limit,
            "Administration output exceeds 32 KiB; no changes applied. Narrow the query or fetch fewer fields.",
        );
    }
    scratch.validate(f)?;
    *r = scratch;
    Ok(out)
}
