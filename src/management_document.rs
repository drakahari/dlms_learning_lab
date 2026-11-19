//! Complete bounded management-policy documents. Never an IOS or host session.
use crate::{acl, acl_editor as editor, acl_routing, management_path, management_policy};
use std::collections::{BTreeMap, BTreeSet};

type Result<T> = std::result::Result<T, editor::Diagnostic>;
fn err(category: editor::Category, line: usize, message: &str) -> editor::Diagnostic {
    editor::Diagnostic {
        category,
        line: (line != 0).then_some(line),
        message: message.into(),
    }
}
fn kw(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
fn number(s: &str, line: usize) -> Result<u16> {
    if s.is_empty() || s.len() > 4 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(
            editor::Category::Syntax,
            line,
            "Use an ordinary decimal line or access-list number.",
        ));
    }
    let n = s.parse::<u16>().map_err(|_| {
        err(
            editor::Category::Limit,
            line,
            "The supplied number is outside this activity's bounds.",
        )
    })?;
    if n.to_string() != s {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "Use decimal numbers without leading zeros in this document.",
        ));
    }
    Ok(n)
}
fn list_number(s: &str, line: usize) -> Result<String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "VTY source access classes must be numbered in this activity; named lists are excluded.",
        ));
    }
    let n = number(s, line)?;
    if !(1..=99).contains(&n) && !(1300..=1999).contains(&n) {
        return Err(err(
            editor::Category::Unsupported,
            line,
            "Use a numbered standard list: 1–99 or 1300–1999. Named or extended VTY lists are outside this activity.",
        ));
    }
    Ok(n.to_string())
}
#[derive(Clone)]
struct LineDraft {
    protocol: Option<(bool, bool)>,
    list: Option<(String, usize)>,
    header: usize,
}
enum Mode {
    Global,
    List,
    Lines(Vec<u8>),
}

pub fn parse(
    network: &management_path::Network,
    supplied: &management_policy::Policy,
    text: &str,
) -> Result<management_policy::Policy> {
    network.validate().map_err(|_| {
        err(
            editor::Category::FixedContext,
            0,
            "The supplied management network is invalid.",
        )
    })?;
    supplied.validate().map_err(|_| {
        err(
            editor::Category::FixedContext,
            0,
            "The supplied management policy context is invalid.",
        )
    })?;
    if text.len() > editor::MAX_TEXT {
        return Err(err(
            editor::Category::Limit,
            0,
            "The policy document supports at most 64 KiB.",
        ));
    }
    let mut mode = Mode::Global;
    let mut versions = None;
    let mut lines = BTreeMap::<u8, LineDraft>::new();
    let mut acl_lines = Vec::<String>::new();
    let mut list_names = BTreeSet::new();
    for (index, raw) in text.split_terminator('\n').enumerate() {
        let at = index + 1;
        if at > editor::MAX_LINES || raw.len() > editor::MAX_LINE {
            return Err(err(
                editor::Category::Limit,
                at,
                "Use at most 1024 lines and 512 bytes on each line.",
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
        let source = raw.trim_matches([' ', '\t']);
        // Keep original line positions for the shared ACL parser's diagnostics.
        acl_lines.push(String::new());
        if source.is_empty() || source.starts_with('!') {
            continue;
        }
        let tokens = source
            .split_ascii_whitespace()
            .take(editor::MAX_TOKENS + 1)
            .collect::<Vec<_>>();
        if tokens.len() > editor::MAX_TOKENS {
            return Err(err(
                editor::Category::Limit,
                at,
                "A policy line supports at most 16 words.",
            ));
        }
        if tokens.len() == 1 && kw(tokens[0], "exit") {
            match mode {
                Mode::Global => {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "There is no open section to exit.",
                    ));
                }
                Mode::List => acl_lines[index] = source.into(),
                Mode::Lines(_) => {}
            }
            mode = Mode::Global;
            continue;
        }
        match &mode {
            Mode::List => {
                acl_lines[index] = source.into();
            }
            Mode::Lines(selected) => {
                if tokens.len() == 2 && kw(tokens[0], "transport") && kw(tokens[1], "input") {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "transport input needs a protocol choice: ssh, telnet, both, or none.",
                    ));
                }
                if tokens.len() <= 2 && kw(tokens[0], "access-class") {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "An incoming attachment needs access-class NUMBER in.",
                    ));
                }
                if tokens.len() >= 3 && kw(tokens[0], "transport") && kw(tokens[1], "input") {
                    let settings = match &tokens[2..] {
                        [one] if kw(one, "ssh") => (true, false),
                        [one] if kw(one, "telnet") => (false, true),
                        [one] if kw(one, "none") => (false, false),
                        [one, two]
                            if (kw(one, "ssh") && kw(two, "telnet"))
                                || (kw(one, "telnet") && kw(two, "ssh")) =>
                        {
                            (true, true)
                        }
                        _ => {
                            return Err(err(
                                editor::Category::Unsupported,
                                at,
                                "Supported incoming choices are ssh, telnet, both, or none. The all keyword includes protocols this activity does not model.",
                            ));
                        }
                    };
                    for n in selected {
                        let row = lines
                            .get_mut(n)
                            .expect("selected rows were inserted together");
                        if row.protocol.replace(settings).is_some() {
                            return Err(err(
                                editor::Category::Unsupported,
                                at,
                                "Use one incoming-protocol setting per group in this complete document.",
                            ));
                        }
                    }
                } else if tokens.len() == 3 && kw(tokens[0], "access-class") {
                    if !kw(tokens[2], "in") {
                        return Err(err(
                            editor::Category::Unsupported,
                            at,
                            "Only an incoming source access class is supported here.",
                        ));
                    }
                    let name = list_number(tokens[1], at)?;
                    for n in selected {
                        let row = lines
                            .get_mut(n)
                            .expect("selected rows were inserted together");
                        if row.list.replace((name.clone(), at)).is_some() {
                            return Err(err(
                                editor::Category::Unsupported,
                                at,
                                "Use one incoming access class per group in this complete document.",
                            ));
                        }
                    }
                } else {
                    return Err(err(
                        editor::Category::Unsupported,
                        at,
                        "This line section supports transport input and optional access-class NUMBER in. Use exit before global settings.",
                    ));
                }
            }
            Mode::Global => {
                if tokens.len() == 3
                    && kw(tokens[0], "ip")
                    && kw(tokens[1], "ssh")
                    && kw(tokens[2], "version")
                {
                    return Err(err(
                        editor::Category::Syntax,
                        at,
                        "ip ssh version needs a version number. This activity supports version 2.",
                    ));
                }
                if tokens.len() == 4
                    && kw(tokens[0], "ip")
                    && kw(tokens[1], "ssh")
                    && kw(tokens[2], "version")
                {
                    if tokens[3] != "2" {
                        return Err(err(
                            editor::Category::Unsupported,
                            at,
                            "This activity supports explicit SSH version 2 or the compatibility selection no ip ssh version; version-1-only is excluded.",
                        ));
                    }
                    if versions
                        .replace(management_policy::Versions::Version2Only)
                        .is_some()
                    {
                        return Err(err(
                            editor::Category::Unsupported,
                            at,
                            "State the SSH version policy once in this complete document.",
                        ));
                    }
                } else if tokens.len() == 4
                    && kw(tokens[0], "no")
                    && kw(tokens[1], "ip")
                    && kw(tokens[2], "ssh")
                    && kw(tokens[3], "version")
                {
                    if versions
                        .replace(management_policy::Versions::Compatibility)
                        .is_some()
                    {
                        return Err(err(
                            editor::Category::Unsupported,
                            at,
                            "State the SSH version policy once in this complete document.",
                        ));
                    }
                } else if (tokens.len() == 3 || tokens.len() == 4)
                    && kw(tokens[0], "line")
                    && kw(tokens[1], "vty")
                {
                    let first = number(tokens[2], at)?;
                    let last = if tokens.len() == 4 {
                        number(tokens[3], at)?
                    } else {
                        first
                    };
                    if first > last
                        || last > 98
                        || last - first + 1 > management_policy::MAX_LINES as u16
                    {
                        return Err(err(
                            editor::Category::Limit,
                            at,
                            "Choose an ordered range of at most 16 supplied policy lines, within 0–98.",
                        ));
                    }
                    let mut selected = vec![];
                    for n in first..=last {
                        let n = n as u8;
                        if !supplied.lines.iter().any(|l| l.number == n) {
                            return Err(err(
                                editor::Category::FixedContext,
                                at,
                                "This range includes a policy line not supplied by the lab.",
                            ));
                        }
                        if lines
                            .insert(
                                n,
                                LineDraft {
                                    protocol: None,
                                    list: None,
                                    header: at,
                                },
                            )
                            .is_some()
                        {
                            return Err(err(
                                editor::Category::Unsupported,
                                at,
                                "Line groups must not overlap or repeat in this complete document.",
                            ));
                        }
                        selected.push(n);
                    }
                    mode = Mode::Lines(selected);
                } else if tokens.len() == 4
                    && kw(tokens[0], "ip")
                    && kw(tokens[1], "access-list")
                    && kw(tokens[2], "standard")
                {
                    let name = list_number(tokens[3], at)?;
                    if list_names.len() >= acl::MAX_LISTS {
                        return Err(err(
                            editor::Category::Limit,
                            at,
                            "This document supports at most eight source-list sections.",
                        ));
                    }
                    if !list_names.insert(name) {
                        return Err(err(
                            editor::Category::Unsupported,
                            at,
                            "Combine each access list into one section rather than repeating it.",
                        ));
                    }
                    acl_lines[index] = source.into();
                    mode = Mode::List;
                } else {
                    return Err(err(
                        editor::Category::Unsupported,
                        at,
                        "This document supports SSH version policy, supplied line vty sections and numbered standard source lists. Session commands, key/identity changes and other configuration are excluded.",
                    ));
                }
            }
        }
    }
    let versions=versions.ok_or_else(||err(editor::Category::Syntax,0,"State the SSH version policy explicitly; automatic device defaults are outside this document."))?;
    if lines.len() != supplied.lines.len() {
        return Err(err(
            editor::Category::FixedContext,
            0,
            "Include each supplied policy line exactly once; an omitted group must not disappear from checking.",
        ));
    }
    // Reuse the unchanged ACL document parser without inventing host endpoints or
    // applying a transit ACL to the management path. Only ACL sections reach it.
    let context = acl_routing::Network {
        schema_version: 1,
        topology: network.topology.clone(),
        host_endpoints: BTreeMap::new(),
        devices: BTreeMap::new(),
    };
    let parsed = editor::parse_router(&context, &network.management.device, &acl_lines.join("\n"))?;
    let mut result = management_policy::Policy {
        version: management_policy::VERSION,
        host_identity: supplied.host_identity.clone(),
        versions,
        lines: vec![],
    };
    let mut total_rules = 0;
    for original in &supplied.lines {
        let row = &lines[&original.number];
        let (ssh, telnet) = row.protocol.ok_or_else(|| {
            err(
                editor::Category::Syntax,
                row.header,
                "Choose incoming protocols explicitly for this line group.",
            )
        })?;
        let source_list = if let Some((name, at)) = &row.list {
            let list = parsed.policy.lists.get(name).ok_or_else(|| {
                err(
                    editor::Category::Resolution,
                    *at,
                    "The incoming access class refers to a source list missing from this document.",
                )
            })?;
            total_rules += list.rules.len();
            if total_rules > management_policy::MAX_TOTAL_RULES {
                return Err(err(
                    editor::Category::Limit,
                    *at,
                    "These line attachments exceed the 128 total source-rule budget.",
                ));
            }
            Some(list.clone())
        } else {
            None
        };
        result.lines.push(management_policy::Line {
            number: original.number,
            ssh,
            telnet,
            source_list,
        });
    }
    result.validate().map_err(|_| {
        err(
            editor::Category::InvalidData,
            0,
            "The resulting policy is outside the supported bounds.",
        )
    })?;
    Ok(result)
}
