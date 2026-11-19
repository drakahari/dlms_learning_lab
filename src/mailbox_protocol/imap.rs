//! A deliberately bounded IMAP command subset, not a full IMAP server.
use super::*;
use crate::mailbox_access::{Flag, Mailbox};

#[derive(Clone, Debug)]
enum Value {
    Atom(String),
    Quoted(String),
    List(Vec<Value>),
}
impl Value {
    fn string(&self) -> Result<&str, Failure> {
        match self {
            Self::Atom(s) | Self::Quoted(s) => Ok(s),
            _ => syntax("Expected an atom or quoted string, not a list."),
        }
    }
    fn atom(&self) -> Result<&str, Failure> {
        match self {
            Self::Atom(s) => Ok(s),
            _ => syntax("This argument requires an unquoted protocol atom."),
        }
    }
}
fn syntax<T>(message: &str) -> Result<T, Failure> {
    Err(Failure {
        issue: Issue::Syntax,
        message: message.into(),
    })
}
fn unsupported<T>(message: &str) -> Result<T, Failure> {
    Err(Failure {
        issue: Issue::Unsupported,
        message: message.into(),
    })
}
fn parse(line: &str) -> Result<Vec<Value>, Failure> {
    fn values(
        bytes: &[u8],
        i: &mut usize,
        nested: bool,
        count: &mut usize,
    ) -> Result<Vec<Value>, Failure> {
        let mut out = Vec::new();
        while *i < bytes.len() {
            if bytes[*i].is_ascii_whitespace() {
                *i += 1;
                continue;
            }
            if bytes[*i] == b')' {
                if !nested {
                    return syntax("Unexpected closing parenthesis.");
                }
                *i += 1;
                return Ok(out);
            }
            *count += 1;
            if *count > 96 {
                return syntax("Use at most96 protocol tokens per command.");
            }
            let value = match bytes[*i] {
                b'(' => {
                    if nested {
                        return unsupported("Nested argument lists are outside this IMAP subset.");
                    }
                    *i += 1;
                    Value::List(values(bytes, i, true, count)?)
                }
                b'"' => {
                    *i += 1;
                    let mut s = String::new();
                    let mut closed = false;
                    while *i < bytes.len() {
                        let b = bytes[*i];
                        *i += 1;
                        if b == b'"' {
                            closed = true;
                            break;
                        }
                        if b == b'\\' {
                            if *i == bytes.len() || !matches!(bytes[*i], b'"' | b'\\') {
                                return syntax("Inside quotes, escape only a quote or backslash.");
                            }
                            s.push(bytes[*i] as char);
                            *i += 1;
                        } else {
                            s.push(b as char);
                        }
                    }
                    if !closed {
                        return syntax("Close the quoted string with a double quote.");
                    }
                    Value::Quoted(s)
                }
                b'{' | b'}' => {
                    return unsupported(
                        "IMAP literal input is not supported; use a quoted string for supplied names/passwords.",
                    );
                }
                _ => {
                    let start = *i;
                    while *i < bytes.len()
                        && !bytes[*i].is_ascii_whitespace()
                        && !b"()\"{}".contains(&bytes[*i])
                    {
                        *i += 1;
                    }
                    Value::Atom(
                        String::from_utf8(bytes[start..*i].to_vec()).expect("outer ASCII guard"),
                    )
                }
            };
            out.push(value);
            if *i < bytes.len() && !bytes[*i].is_ascii_whitespace() && bytes[*i] != b')' {
                return syntax("Separate protocol arguments with spaces.");
            }
        }
        if nested {
            syntax("Close the argument list with a parenthesis.")
        } else {
            Ok(out)
        }
    }
    values(line.as_bytes(), &mut 0, false, &mut 0)
}
fn tag_ok(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_graphic() && !b"()+{%*\"\\]}".contains(&b))
}
fn arity(args: &[Value], n: usize) -> Result<(), Failure> {
    if args.len() == n {
        Ok(())
    } else {
        syntax("Wrong number of arguments; see the supported-command reference.")
    }
}
fn ok(tag: &str, command: &str, prefix: String) -> Reply {
    let mut reply = Reply::imap(tag, Code::Ok, format!("{command} completed"));
    reply.text.insert_str(0, &prefix);
    reply
}
fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
fn selected(session: &Session) -> Result<(String, Selection), Failure> {
    if let Phase::Imap {
        user: Some(u),
        selected: Some(s),
    } = &session.phase
    {
        return Ok((u.clone(), s.clone()));
    }
    Err(Failure {
        issue: Issue::Mailbox,
        message: "Log in and SELECT or EXAMINE a supplied mailbox first.".into(),
    })
}
fn indices(set: &str, mailbox: &Mailbox, uid: bool) -> Result<Vec<usize>, Failure> {
    let top = if uid {
        mailbox.messages.last().map_or(0, |m| m.uid)
    } else {
        mailbox.messages.len() as u32
    };
    let number = |s: &str| -> Result<u32, Failure> {
        if s == "*" {
            return Ok(top);
        }
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return syntax(
                "Use positive message numbers, *, ranges such as1:3, or comma-separated sets.",
            );
        }
        let n = s
            .parse::<u32>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| Failure {
                issue: Issue::Syntax,
                message: "Message numbers must be positive32-bit integers.".into(),
            })?;
        if !uid && n > top {
            return syntax(
                "A sequence number is beyond the selected mailbox; inspect its EXISTS count first.",
            );
        }
        Ok(n)
    };
    let mut chosen = BTreeSet::new();
    for part in set.split(',') {
        let ends: Vec<_> = part.split(':').collect();
        if ends.len() > 2 {
            return syntax("A sequence range has only one colon.");
        }
        let a = number(ends[0])?;
        let b = if ends.len() == 2 { number(ends[1])? } else { a };
        for (i, m) in mailbox.messages.iter().enumerate() {
            let n = if uid { m.uid } else { i as u32 + 1 };
            if n >= a.min(b) && n <= a.max(b) {
                chosen.insert(i);
            }
        }
    }
    Ok(chosen.into_iter().collect())
}
fn atoms(value: &Value) -> Result<Vec<String>, Failure> {
    match value {
        Value::List(items) => items
            .iter()
            .map(|v| v.atom().map(str::to_ascii_uppercase))
            .collect(),
        _ => Ok(vec![value.atom()?.to_ascii_uppercase()]),
    }
}
impl Session {
    pub(super) fn imap(&mut self, r: &mut Runtime, f: &Facts, line: &str) -> Reply {
        // Retain a valid tag on parser errors, but never reflect an invalid tag.
        let tag = line
            .split_ascii_whitespace()
            .next()
            .filter(|s| tag_ok(s))
            .unwrap_or("*")
            .to_owned();
        match self.imap_eval(r, f, line) {
            Ok(reply) => reply,
            Err(e) => Reply::failure(Protocol::Imap, &tag, e),
        }
    }
    fn imap_eval(&mut self, r: &mut Runtime, f: &Facts, line: &str) -> Result<Reply, Failure> {
        let words = parse(line)?;
        if words.len() < 2 {
            return syntax("Start with a short tag, then a command; for example a1 CAPABILITY.");
        }
        let tag = words[0].atom()?;
        if !tag_ok(tag) {
            return syntax("Use a short protocol tag such as a1, without special characters.");
        }
        let command = words[1].atom()?.to_ascii_uppercase();
        let args = &words[2..];
        match command.as_str() {
            "CAPABILITY" => {
                arity(args, 0)?;
                let cfg = r.active_config()?;
                // Do not claim full IMAP4rev1 conformance for this partial model.
                let mut caps = "* CAPABILITY X-SIMULATOR".to_owned();
                if let Phase::Imap { user: None, .. } = self.phase {
                    if !self.link.tls && cfg.ssl != Ssl::No {
                        caps.push_str(" STARTTLS");
                    }
                    if !cfg.permits_password(self.link.tls, false) {
                        caps.push_str(" LOGINDISABLED");
                    }
                }
                caps.push_str("\r\n");
                return Ok(ok(tag, "CAPABILITY", caps));
            }
            "NOOP" => {
                arity(args, 0)?;
                return Ok(ok(tag, "NOOP", String::new()));
            }
            "LOGOUT" => {
                arity(args, 0)?;
                self.link.closed = true;
                return Ok(ok(tag, "LOGOUT", "* BYE Logging out\r\n".into()));
            }
            "STARTTLS" => {
                arity(args, 0)?;
                if !matches!(self.phase, Phase::Imap { user: None, .. }) {
                    return syntax("STARTTLS is available only before authentication.");
                }
                match self.link.start_tls(r, f) {
                    Ok(()) => {
                        let mut out = Reply::imap(tag, Code::Ok, "Begin TLS negotiation now");
                        out.client_note = Some("Virtual certificate checks passed. Recheck CAPABILITY on this logical protected connection.".into());
                        return Ok(out);
                    }
                    Err(e) if self.link.closed => {
                        let mut out = Reply::imap(tag, Code::Ok, "Begin TLS negotiation now");
                        out.code = Code::Closed;
                        out.issue = Some(e.issue);
                        out.client_note = Some(e.message);
                        return Ok(out);
                    }
                    Err(e) => return Err(e),
                }
            }
            "LOGIN" => {
                arity(args, 2)?;
                if !matches!(self.phase, Phase::Imap { user: None, .. }) {
                    return syntax("Already authenticated; reconnect to change accounts.");
                }
                let user = args[0].string()?;
                r.authenticate(f, &self.link, user, args[1].string()?)?;
                self.phase = Phase::Imap {
                    user: Some(user.into()),
                    selected: None,
                };
                return Ok(ok(tag, "LOGIN", String::new()));
            }
            _ => {}
        }
        if !matches!(
            command.as_str(),
            "LIST" | "SELECT" | "EXAMINE" | "CLOSE" | "EXPUNGE" | "FETCH" | "STORE" | "UID"
        ) {
            return unsupported(
                "This command is outside the documented IMAP subset; it was not executed.",
            );
        }
        let user = match &self.phase {
            Phase::Imap { user: Some(u), .. } => u.clone(),
            _ => {
                return Err(Failure {
                    issue: Issue::Authentication,
                    message: "Authenticate with LOGIN before accessing mailboxes.".into(),
                });
            }
        };
        match command.as_str() {
            "LIST" => {
                arity(args, 2)?;
                if !args[0].string()?.is_empty() {
                    return unsupported(
                        "LIST supports an empty reference and a flat supplied mailbox name, * or %.",
                    );
                }
                let pattern = args[1].string()?;
                if pattern.contains(['*', '%']) && !matches!(pattern, "*" | "%") {
                    return unsupported(
                        "Partial-name LIST wildcards are outside this flat-mailbox subset.",
                    );
                }
                let cfg = r.active_config()?;
                f.resolved_maildir(&cfg, &user)?;
                let mut text = String::new();
                for name in r.store[&user].keys() {
                    if matches!(pattern, "*" | "%")
                        || name == pattern
                        || name == "INBOX" && pattern.eq_ignore_ascii_case("INBOX")
                    {
                        text.push_str(&format!("* LIST () NIL {}\r\n", quote(name)));
                    }
                }
                Ok(ok(tag, "LIST", text))
            }
            "SELECT" | "EXAMINE" => {
                arity(args, 1)?;
                let name = args[0].string()?;
                if let Phase::Imap { selected, .. } = &mut self.phase {
                    *selected = None;
                }
                let mailbox = r.mailbox(f, &user, name)?;
                let read_only = command == "EXAMINE";
                let mut text = format!(
                    "* FLAGS (\\Seen \\Deleted)\r\n* {} EXISTS\r\n* 0 RECENT\r\n* OK [UIDVALIDITY {}] UIDs valid\r\n* OK [UIDNEXT {}] Predicted next UID\r\n",
                    mailbox.messages.len(),
                    mailbox.uid_validity,
                    mailbox.uid_next
                );
                if let Some(i) = mailbox
                    .messages
                    .iter()
                    .position(|m| !m.flags.contains(&Flag::Seen))
                {
                    text.push_str(&format!("* OK [UNSEEN {}] First unseen message\r\n", i + 1));
                }
                text.push_str(if read_only {
                    "* OK [PERMANENTFLAGS ()] Read-only mailbox\r\n"
                } else {
                    "* OK [PERMANENTFLAGS (\\Seen \\Deleted)] Supported persistent flags\r\n"
                });
                if let Phase::Imap { selected, .. } = &mut self.phase {
                    *selected = Some(Selection {
                        mailbox: if name.eq_ignore_ascii_case("INBOX") {
                            "INBOX".into()
                        } else {
                            name.into()
                        },
                        read_only,
                    });
                }
                let mut out = Reply::imap(
                    tag,
                    Code::Ok,
                    format!(
                        "[{}] {command} completed",
                        if read_only { "READ-ONLY" } else { "READ-WRITE" }
                    ),
                );
                out.text.insert_str(0, &text);
                Ok(out)
            }
            "CLOSE" | "EXPUNGE" => {
                arity(args, 0)?;
                let (user, selection) = selected(self)?;
                if command == "EXPUNGE" && selection.read_only {
                    return Err(Failure {issue:Issue::Mailbox,message:"EXAMINE selected a read-only mailbox; SELECT it before changing flags or expunging.".into()});
                }
                let mut text = String::new();
                if !selection.read_only {
                    for n in r.mailbox_mut(f, &user, &selection.mailbox)?.expunge() {
                        if command == "EXPUNGE" {
                            text.push_str(&format!("* {n} EXPUNGE\r\n"));
                        }
                    }
                }
                if command == "CLOSE"
                    && let Phase::Imap { selected, .. } = &mut self.phase
                {
                    *selected = None;
                }
                Ok(ok(tag, &command, text))
            }
            "FETCH" | "STORE" | "UID" => {
                let (operation, args, uid) = if command == "UID" {
                    if args.is_empty() {
                        return syntax("UID needs FETCH or STORE and their arguments.");
                    }
                    (args[0].atom()?.to_ascii_uppercase(), &args[1..], true)
                } else {
                    (command.clone(), args, false)
                };
                if !matches!(operation.as_str(), "FETCH" | "STORE") {
                    return unsupported("UID supports only FETCH and STORE in this subset.");
                }
                arity(args, if operation == "FETCH" { 2 } else { 3 })?;
                let (user, selection) = selected(self)?;
                let chosen = indices(
                    args[0].atom()?,
                    r.mailbox(f, &user, &selection.mailbox)?,
                    uid,
                )?;
                let mut text = String::new();
                if operation == "FETCH" {
                    let mut attrs = atoms(&args[1])?;
                    if attrs.is_empty() {
                        return syntax("FETCH needs at least one supported attribute.");
                    }
                    if attrs.iter().any(|s| {
                        !matches!(
                            s.as_str(),
                            "FLAGS" | "UID" | "RFC822.SIZE" | "BODY[]" | "BODY.PEEK[]"
                        )
                    }) {
                        return unsupported(
                            "FETCH supports FLAGS, UID, RFC822.SIZE, BODY[] and BODY.PEEK[] only; sections/partials/MIME are not modeled.",
                        );
                    }
                    if uid && !attrs.iter().any(|s| s == "UID") {
                        attrs.push("UID".into());
                    }
                    for i in chosen {
                        let message = &mut r.mailbox_mut(f, &user, &selection.mailbox)?.messages[i];
                        let changed = !selection.read_only
                            && attrs.iter().any(|s| s == "BODY[]")
                            && message.flags.insert(Flag::Seen);
                        let mut fields = Vec::new();
                        for attr in &attrs {
                            fields.push(match attr.as_str() {
                                "FLAGS" => format!("FLAGS ({})", message.flag_text()),
                                "UID" => format!("UID {}", message.uid),
                                "RFC822.SIZE" => format!("RFC822.SIZE {}", message.raw().len()),
                                _ => {
                                    let raw = message.raw();
                                    format!("BODY[] {{{}}}\r\n{raw}", raw.len())
                                }
                            });
                        }
                        if changed && !attrs.iter().any(|s| s == "FLAGS") {
                            fields.push(format!("FLAGS ({})", message.flag_text()));
                        }
                        text.push_str(&format!("* {} FETCH ({})\r\n", i + 1, fields.join(" ")));
                    }
                } else {
                    if selection.read_only {
                        return Err(Failure {issue:Issue::Mailbox,message:"This mailbox is read-only. SELECT it before STORE; EXAMINE is for inspection.".into()});
                    }
                    let item = args[1].atom()?.to_ascii_uppercase();
                    let (item, silent) = item
                        .strip_suffix(".SILENT")
                        .map_or((item.as_str(), false), |s| (s, true));
                    if !matches!(item, "FLAGS" | "+FLAGS" | "-FLAGS") {
                        return unsupported(
                            "STORE supports FLAGS, +FLAGS and -FLAGS, with optional .SILENT.",
                        );
                    }
                    let mut flags = BTreeSet::new();
                    for flag in atoms(&args[2])? {
                        flags.insert(match flag.as_str() {"\\SEEN"=>Flag::Seen,"\\DELETED"=>Flag::Deleted,_=>return unsupported("Only \\Seen and \\Deleted flags are modeled; no keywords or other system flags are silently discarded.")});
                    }
                    for i in chosen {
                        let message = &mut r.mailbox_mut(f, &user, &selection.mailbox)?.messages[i];
                        match item {
                            "FLAGS" => message.flags = flags.clone(),
                            "+FLAGS" => message.flags.extend(&flags),
                            _ => message.flags.retain(|f| !flags.contains(f)),
                        }
                        if !silent {
                            text.push_str(&format!(
                                "* {} FETCH ({}FLAGS ({}))\r\n",
                                i + 1,
                                if uid {
                                    format!("UID {} ", message.uid)
                                } else {
                                    String::new()
                                },
                                message.flag_text()
                            ));
                        }
                    }
                }
                Ok(ok(tag, if uid { "UID" } else { &operation }, text))
            }
            _ => unreachable!(),
        }
    }
}
