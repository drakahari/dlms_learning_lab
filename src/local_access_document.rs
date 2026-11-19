//! Complete, bounded local-AAA training documents; no IOS or cryptographic engine.
use crate::{
    acl_editor as editor, local_access as local, management_document, management_path,
    management_policy,
};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, editor::Diagnostic>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub incoming: management_policy::Policy,
    pub local: local::Policy,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Global,
    Lines,
    List,
}
fn kw(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
fn err(category: editor::Category, line: usize, message: &str) -> editor::Diagnostic {
    editor::Diagnostic {
        category,
        line: (line != 0).then_some(line),
        message: message.into(),
    }
}
fn once(slot: &mut bool, line: usize) -> Result<()> {
    if std::mem::replace(slot, true) {
        Err(err(
            editor::Category::Unsupported,
            line,
            "State each AAA setting once in this complete document; incremental overrides are not modeled.",
        ))
    } else {
        Ok(())
    }
}
fn user(tokens: &[&str], line: usize) -> Result<local::User> {
    if tokens
        .get(2)
        .is_some_and(|v| kw(v, "algorithm-type") || kw(v, "password") || kw(v, "nopassword"))
        || tokens
            .get(4)
            .is_some_and(|v| kw(v, "password") || kw(v, "nopassword"))
    {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "This subset uses explicit secret training words. Password encodings, algorithm settings and no-password bypass are not implemented.",
        ));
    }
    if tokens.len() < 6 {
        return Err(err(
            editor::Category::Syntax,
            line,
            "A local record needs username NAME privilege 1-or-15 secret TRAINING_WORD.",
        ));
    }
    if tokens.len() != 6 || !kw(tokens[2], "privilege") || !kw(tokens[4], "secret") {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "Use one explicit privilege and one plain training word with secret. Encoded hashes, algorithm settings, password/nopassword and quoted values are outside this simulator.",
        ));
    }
    let privilege = match tokens[3] {
        "1" => local::Privilege::User,
        "15" => local::Privilege::Administrator,
        n if n.bytes().all(|c| c.is_ascii_digit()) => {
            return Err(err(
                editor::Category::Unsupported,
                line,
                "This subset models user level 1 and privileged level 15; other command privilege levels are not implemented.",
            ));
        }
        _ => {
            return Err(err(
                editor::Category::Syntax,
                line,
                "Write the privilege as the ordinary decimal value 1 or 15.",
            ));
        }
    };
    if !tokens[5]
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
    {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "Use the supplied single-word training value (letters, digits, dot, underscore or dash). Quoting, escaping, expansion and encoded hashes are not supported.",
        ));
    }
    let u = local::User {
        name: tokens[1].into(),
        training_token: tokens[5].into(),
        privilege,
    };
    u.validate().map_err(|_| {
        err(
            editor::Category::Syntax,
            line,
            "Use a 1–32 character supported account name and a 1–128 character training word.",
        )
    })?;
    Ok(u)
}
pub fn parse(
    network: &management_path::Network,
    supplied: &management_policy::Policy,
    text: &str,
) -> Result<Settings> {
    if text.len() > editor::MAX_TEXT {
        return Err(err(
            editor::Category::Limit,
            0,
            "The complete local-access document supports at most 64 KiB.",
        ));
    }
    let mut mode = Mode::Global;
    let mut aaa = false;
    let mut method = false;
    let mut authorization = false;
    let mut selected = false;
    let mut users = Vec::new();
    let mut names = BTreeSet::new();
    let mut remaining = Vec::new();
    for (index, raw) in text.split_terminator('\n').enumerate() {
        let at = index + 1;
        if at > editor::MAX_LINES || raw.len() > editor::MAX_LINE {
            return Err(err(
                editor::Category::Limit,
                at,
                "Use at most 1024 lines and 512 bytes per line.",
            ));
        }
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        if raw.chars().any(|c| c.is_control() && c != '\t') {
            return Err(err(
                editor::Category::Unsupported,
                at,
                "Use ordinary text, tabs and line endings; other control characters are unsupported.",
            ));
        }
        remaining.push(raw);
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('!') {
            continue;
        }
        let t = raw.split_whitespace().collect::<Vec<_>>();
        if t.len() > editor::MAX_TOKENS {
            return Err(err(
                editor::Category::Limit,
                at,
                "A policy line supports at most 16 words.",
            ));
        }
        if t.len() == 1 && kw(t[0], "exit") {
            mode = Mode::Global;
            continue;
        }
        match mode {
            Mode::Global if kw(t[0], "aaa") => {
                if t.len() == 1
                    || (t.len() == 4
                        && kw(t[3], "default")
                        && (kw(t[1], "authentication") || kw(t[1], "authorization")))
                {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "Complete the AAA setting: new-model, authentication login default local-case, or authorization exec default local.",
                    ));
                }
                if t.len() == 2 && kw(t[1], "new-model") {
                    once(&mut aaa, at)?;
                } else if t.len() == 5
                    && kw(t[1], "authentication")
                    && kw(t[2], "login")
                    && kw(t[3], "default")
                    && kw(t[4], "local-case")
                {
                    once(&mut method, at)?;
                } else if t.len() == 5
                    && kw(t[1], "authorization")
                    && kw(t[2], "exec")
                    && kw(t[3], "default")
                    && kw(t[4], "local")
                {
                    once(&mut authorization, at)?;
                } else {
                    return Err(err(
                        editor::Category::Unsupported,
                        at,
                        "This document supports aaa new-model, default local-case login authentication and default local EXEC authorization. Named/remote methods, fallback, none and inferred defaults are excluded.",
                    ));
                }
                *remaining.last_mut().unwrap() = "";
            }
            Mode::Global if kw(t[0], "username") => {
                let u = user(&t, at)?;
                if users.len() == local::MAX_RECORDS {
                    return Err(err(
                        editor::Category::Limit,
                        at,
                        "The simulator supports at most 16 local account records.",
                    ));
                }
                if !names.insert(u.name.clone()) {
                    return Err(err(
                        editor::Category::Unsupported,
                        at,
                        "Declare each case-sensitive account name once; replace the record instead of adding an incremental override.",
                    ));
                }
                users.push(u);
                *remaining.last_mut().unwrap() = "";
            }
            Mode::Global if t.len() >= 2 && kw(t[0], "line") && kw(t[1], "vty") => {
                mode = Mode::Lines;
                selected = false;
            }
            Mode::Global if t.len() >= 3 && kw(t[0], "ip") && kw(t[1], "access-list") => {
                mode = Mode::List;
            }
            Mode::Lines if kw(t[0], "login") => {
                if t.len() == 1 || (t.len() == 2 && kw(t[1], "authentication")) {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "Complete the line selection as login authentication default.",
                    ));
                }
                if t.len() != 3 || !kw(t[1], "authentication") || !kw(t[2], "default") {
                    return Err(err(
                        editor::Category::Unsupported,
                        at,
                        "Only login authentication default is supported on supplied terminal lines. Local/non-AAA, named and bypass selections are outside this subset.",
                    ));
                }
                once(&mut selected, at)?;
                *remaining.last_mut().unwrap() = "";
            }
            _ => {}
        }
    }
    // Delegate every other setting to the unchanged management-document parser.
    // Blank filtered rows retain its original diagnostic line numbers.
    let incoming = management_document::parse(network, supplied, &remaining.join("\n"))?;
    if !aaa || !method || !authorization {
        return Err(err(
            editor::Category::Syntax,
            0,
            "State aaa new-model, default local-case login authentication and default local EXEC authorization explicitly. No device defaults are inferred.",
        ));
    }
    let local = local::Policy {
        version: 1,
        method: local::Method::LocalCase,
        exec_authorization: local::Authorization::Local,
        users,
    };
    local.validate().map_err(|_| {
        err(
            editor::Category::InvalidData,
            0,
            "The local account policy is outside the declared simulator limits.",
        )
    })?;
    Ok(Settings { incoming, local })
}
