//! Literal numeric nat-table command subset. No host execution.
use crate::{
    linux_filter::{self as base, Class, Fault},
    linux_flow_nat::{self as nat, Ports, Rule, Table, Target},
};
use std::{collections::BTreeSet, net::Ipv4Addr};
fn error(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn pre(s: &str) -> Result<bool, Fault> {
    match s {
        "PREROUTING" => Ok(true),
        "POSTROUTING" => Ok(false),
        _ => Err(error(
            Class::Unsupported,
            "This NAT profile supports PREROUTING DNAT and POSTROUTING SNAT/MASQUERADE; other hooks/custom chains are unmodeled.",
        )),
    }
}
fn hook(pre: bool) -> &'static str {
    if pre { "PREROUTING" } else { "POSTROUTING" }
}
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
fn number(s: &str) -> Result<usize, Fault> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(
            Class::Syntax,
            "Use a positive one-based rule number.",
        ));
    }
    s.parse()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| error(Class::Syntax, "Use a positive one-based rule number."))
}
fn port(s: &str) -> Result<u16, Fault> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(Class::Syntax, "Use a numeric port from0to65535."));
    }
    s.parse()
        .map_err(|_| error(Class::Syntax, "Use a numeric port from0to65535."))
}
fn ports(s: &str) -> Result<Ports, Fault> {
    let (first, last) = s.split_once('-').unwrap_or((s, s));
    let range = Ports {
        first: port(first)?,
        last: port(last)?,
    };
    if range.first > range.last {
        return Err(error(
            Class::Syntax,
            "The first port cannot exceed the last port.",
        ));
    }
    Ok(range)
}
fn address(s: &str) -> Result<Ipv4Addr, Fault> {
    s.parse().map_err(|_|error(Class::Unsupported,"Use one numeric IPv4 address; address ranges, prefixes, IPv6 and name lookup are not supported NAT targets."))
}
fn target(name: &str, value: Option<(&str, &str)>) -> Result<Target, Fault> {
    match (name, value) {
        ("DNAT", Some(("--to-destination", v))) => {
            let (a, p) = v.split_once(':').map_or((v, None), |(a, p)| (a, Some(p)));
            Ok(Target::Dnat {
                address: address(a)?,
                port: p.map(port).transpose()?,
            })
        }
        ("SNAT", Some(("--to-source", v))) => {
            let (a, p) = v.split_once(':').map_or((v, None), |(a, p)| (a, Some(p)));
            Ok(Target::Snat {
                address: address(a)?,
                ports: p.map(ports).transpose()?,
            })
        }
        ("MASQUERADE", None) => Ok(Target::Masquerade { ports: None }),
        ("MASQUERADE", Some(("--to-ports", v))) => Ok(Target::Masquerade {
            ports: Some(ports(v)?),
        }),
        ("DNAT" | "SNAT" | "MASQUERADE", _) => Err(error(
            Class::Syntax,
            "Use DNAT with --to-destination, SNAT with --to-source, or MASQUERADE with optional --to-ports.",
        )),
        _ => Err(error(
            Class::Unsupported,
            "Only DNAT, SNAT and MASQUERADE targets are supported; no other target was applied.",
        )),
    }
}
fn parse_rule(args: &[String], is_pre: bool) -> Result<Rule, Fault> {
    let (mut jump, mut translation) = (None, None);
    let mut predicate = vec![];
    let mut seen = BTreeSet::new();
    let mut i = 0;
    while i < args.len() {
        let key = match args[i].as_str() {
            "-j" | "--jump" => "jump",
            "--to-source" | "--to-destination" | "--to-ports" => "translation",
            _ => "predicate",
        };
        if key == "predicate" {
            if !matches!(
                args[i].as_str(),
                "-s" | "--source"
                    | "-d"
                    | "--destination"
                    | "-i"
                    | "--in-interface"
                    | "-o"
                    | "--out-interface"
                    | "-p"
                    | "--protocol"
                    | "--sport"
                    | "--source-port"
                    | "--dport"
                    | "--destination-port"
            ) {
                return Err(error(
                    Class::Unsupported,
                    "NAT matches support the documented numeric addresses, interfaces, protocols and ports only; no unknown option was applied.",
                ));
            }
            let value = args
                .get(i + 1)
                .ok_or_else(|| error(Class::Syntax, "Supply a value after the match option."))?;
            predicate.push(args[i].clone());
            predicate.push(value.clone());
            i += 2;
            continue;
        }
        if !seen.insert(key) {
            return Err(error(
                Class::Syntax,
                "Specify one target and at most one translation option.",
            ));
        }
        let value = args.get(i + 1).ok_or_else(|| {
            error(
                Class::Syntax,
                "Supply a value after the target or translation option.",
            )
        })?;
        if key == "jump" {
            jump = Some((i, value.as_str()));
        } else {
            translation = Some((i, args[i].as_str(), value.as_str()));
        }
        i += 2;
    }
    let (jump_index, name) =
        jump.ok_or_else(|| error(Class::Syntax, "Choose a supported -j target."))?;
    if translation.is_some_and(|(i, _, _)| i < jump_index) {
        return Err(error(
            Class::Unsupported,
            "Put the target's translation option after -j DNAT, SNAT or MASQUERADE.",
        ));
    }
    let target = target(name, translation.map(|(_, k, v)| (k, v)))?;
    let chain = if is_pre { "INPUT" } else { "OUTPUT" };
    let command = format!(
        "iptables -A {chain} {} -j ACCEPT",
        predicate
            .iter()
            .map(|s| quote(s))
            .collect::<Vec<_>>()
            .join(" ")
    );
    let mut parsed = base::Table::new(base::Verdict::Drop);
    parsed.command(&command)?;
    let predicate = if is_pre {
        parsed.input.rules.into_iter().next()
    } else {
        parsed.output.rules.into_iter().next()
    }
    .ok_or_else(|| error(Class::Context, "Parsed NAT predicate is missing"))?;
    let r = Rule { predicate, target };
    let t = if is_pre {
        Table {
            prerouting: vec![r.clone()],
            postrouting: vec![],
        }
    } else {
        Table {
            prerouting: vec![],
            postrouting: vec![r.clone()],
        }
    };
    t.validate().map_err(|m| error(Class::Unsupported, &m))?;
    Ok(r)
}
fn range_text(p: Ports) -> String {
    if p.first == p.last {
        p.first.to_string()
    } else {
        format!("{}-{}", p.first, p.last)
    }
}
fn rule_text(r: &Rule, is_pre: bool) -> Result<String, Fault> {
    let chain = if is_pre {
        base::Chain::Input
    } else {
        base::Chain::Output
    };
    let mut t = base::Table::new(base::Verdict::Drop);
    if is_pre {
        t.input.rules.push(r.predicate.clone());
    } else {
        t.output.rules.push(r.predicate.clone());
    }
    let listing = t.listing(Some(chain));
    let prefix = format!("-A {}", chain.name());
    let body = listing
        .lines()
        .find(|s| s.starts_with("-A "))
        .and_then(|s| s.strip_prefix(&prefix))
        .and_then(|s| s.strip_suffix(" -j ACCEPT"))
        .ok_or_else(|| error(Class::Context, "NAT predicate listing is inconsistent."))?;
    let tail = match r.target {
        Target::Dnat { address, port } => format!(
            "DNAT --to-destination {address}{}",
            port.map_or(String::new(), |p| format!(":{p}"))
        ),
        Target::Snat { address, ports } => format!(
            "SNAT --to-source {address}{}",
            ports.map_or(String::new(), |p| format!(":{}", range_text(p)))
        ),
        Target::Masquerade { ports } => format!(
            "MASQUERADE{}",
            ports.map_or(String::new(), |p| format!(" --to-ports {}", range_text(p)))
        ),
    };
    Ok(format!("-A {}{body} -j {tail}", hook(is_pre)))
}
impl Table {
    pub fn listing(&self, selected: Option<bool>) -> Result<String, Fault> {
        self.validate().map_err(|m| error(Class::Context, &m))?;
        let mut out = String::new();
        for is_pre in [true, false]
            .into_iter()
            .filter(|p| selected.is_none_or(|s| *p == s))
        {
            out.push_str(&format!("-P {} ACCEPT\n", hook(is_pre)));
        }
        for is_pre in [true, false]
            .into_iter()
            .filter(|p| selected.is_none_or(|s| *p == s))
        {
            for r in if is_pre {
                &self.prerouting
            } else {
                &self.postrouting
            } {
                out.push_str(&rule_text(r, is_pre)?);
                out.push('\n');
            }
        }
        if out.len() > 32768 {
            return Err(error(
                Class::Limit,
                "The supported NAT listing exceeds32KiB.",
            ));
        }
        Ok(out)
    }
    pub fn command(&mut self, line: &str) -> Result<base::Output, Fault> {
        self.validate().map_err(|m| error(Class::Context, &m))?;
        let mut words = base::words(line)?;
        if words.first().map(String::as_str) != Some("iptables") {
            return Err(error(
                Class::Unsupported,
                "Use the documented simulated iptables NAT commands; no host command ran.",
            ));
        }
        words.remove(0);
        let tables = words
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.as_str(), "-t" | "--table"))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if tables.len() != 1 {
            return Err(error(
                Class::Syntax,
                "Specify -t nat exactly once for this NAT command.",
            ));
        }
        let i = tables[0];
        if words.get(i + 1).map(String::as_str) != Some("nat") {
            return Err(error(
                Class::Unsupported,
                "This parser handles -t nat only; filter commands use their separate supported parser.",
            ));
        }
        words.drain(i..i + 2);
        let op = words.first().map(String::as_str).ok_or_else(|| {
            error(
                Class::Syntax,
                "Choose -S, -A, -I, -D by number, -F or an ACCEPT policy.",
            )
        })?;
        let mut next = self.clone();
        let mut text = String::new();
        match op {
            "-S" | "--list-rules" | "-F" | "--flush" => {
                if words.len() > 2 {
                    return Err(error(
                        Class::Unsupported,
                        "This query/flush takes only an optional supported chain.",
                    ));
                }
                let selected = words.get(1).map(|s| pre(s)).transpose()?;
                if matches!(op, "-S" | "--list-rules") {
                    text = next.listing(selected)?;
                } else {
                    if selected.is_none_or(|p| p) {
                        next.prerouting.clear();
                    }
                    if selected.is_none_or(|p| !p) {
                        next.postrouting.clear();
                    }
                }
            }
            "-P" | "--policy" => {
                if words.len() != 3 {
                    return Err(error(
                        Class::Syntax,
                        "Use -P CHAIN ACCEPT in this translation-only profile.",
                    ));
                }
                pre(&words[1])?;
                if words[2] != "ACCEPT" {
                    return Err(error(
                        Class::Unsupported,
                        "This NAT subset keeps ACCEPT policies; filtering belongs to the filter table.",
                    ));
                }
            }
            "-A" | "--append" | "-I" | "--insert" | "-D" | "--delete" => {
                let is_pre = words
                    .get(1)
                    .ok_or_else(|| error(Class::Syntax, "Name PREROUTING or POSTROUTING."))?;
                let is_pre = pre(is_pre)?;
                if matches!(op, "-D" | "--delete") {
                    if words.len() != 3 {
                        return Err(error(
                            Class::Unsupported,
                            "Delete a NAT rule by its positive one-based number.",
                        ));
                    }
                    let n = number(&words[2])?;
                    let rules = if is_pre {
                        &mut next.prerouting
                    } else {
                        &mut next.postrouting
                    };
                    if n > rules.len() {
                        return Err(error(Class::Syntax, "That NAT rule number does not exist."));
                    }
                    rules.remove(n - 1);
                } else {
                    if next.prerouting.len() + next.postrouting.len() >= nat::MAX_RULES {
                        return Err(error(
                            Class::Limit,
                            "Use at most32NAT rules across both supported chains.",
                        ));
                    }
                    let insert = matches!(op, "-I" | "--insert");
                    let (n, start) = if insert
                        && words
                            .get(2)
                            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
                    {
                        (number(&words[2])?, 3)
                    } else {
                        (1, 2)
                    };
                    let rule = parse_rule(&words[start..], is_pre)?;
                    let rules = if is_pre {
                        &mut next.prerouting
                    } else {
                        &mut next.postrouting
                    };
                    if insert {
                        if n > rules.len() + 1 {
                            return Err(error(
                                Class::Syntax,
                                "Insert at an existing position or immediately after the last NAT rule.",
                            ));
                        }
                        rules.insert(n - 1, rule);
                    } else {
                        rules.push(rule);
                    }
                }
            }
            _ => {
                return Err(error(
                    Class::Unsupported,
                    "Only -S, -A, -I, -D by number, -F and ACCEPT policies are supported here.",
                ));
            }
        }
        next.validate().map_err(|m| error(Class::Context, &m))?;
        let changed = next != *self;
        *self = next;
        Ok(base::Output { text, changed })
    }
}
