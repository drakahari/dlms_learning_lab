//! In-process classic syslog selection. No sockets, files, processes or clock.
use serde::{Deserialize, Serialize};
pub const SEVERITIES: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
];
pub const FACILITIES: [&str; 24] = [
    "kern",
    "user",
    "mail",
    "daemon",
    "auth",
    "syslog",
    "lpr",
    "news",
    "uucp",
    "cron",
    "authpriv",
    "ftp",
    "code12 (NTP)",
    "code13 (audit)",
    "code14 (alert)",
    "code15 (clock)",
    "local0",
    "local1",
    "local2",
    "local3",
    "local4",
    "local5",
    "local6",
    "local7",
];
pub const HELP: &str = "Supported classic selector subset: facility.level DESTINATION, facility,facility.level, *.level, facility.*, facility.=level, followed by optional ;facility.none exclusions. One selector and one supplied virtual log path per line; blank lines and whole-line # comments are allowed. Severity names: emerg(0), alert(1), crit(2), err(3), warning(4), notice(5), info(6), debug(7). A threshold includes equal or more urgent levels (smaller numbers). Separate matching lines each deliver a copy. Only supplied virtual destinations exist. Remote actions, ! filters, later positive semicolon selectors, templates, stop/discard actions, property filters, includes, modules, journald and log rotation are unsupported; they fail explicitly. Replay replaces the previous sample output; it never writes host files. Canonical named facilities are supported for codes0–11 and16–23; other facility codes are observable only via wildcard matching. mark, numerical facility names and deprecated aliases are unsupported.";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub facility: u8,
    pub severity: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    pub message: String,
}
impl Event {
    pub fn validate(&self) -> Result<(), String> {
        if self.facility >= 24
            || self.severity >= 8
            || self.message.len() > 256
            || self.message.chars().any(char::is_control)
        {
            return Err("Invalid bounded syslog sample event".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub selected: [[bool; 8]; 24],
    pub destination: String,
}
fn facilities(text: &str) -> Result<Vec<usize>, String> {
    if text == "*" {
        return Ok((0..24).collect());
    }
    let mut result = vec![];
    for name in text.split(',') {
        // Canonical facilities supported by this instructional subset; no numerical aliases.
        let index=FACILITIES.iter().position(|s|*s==name).filter(|n|*n<12||*n>=16).ok_or_else(||format!("Unsupported facility ‘{}’. Use a canonical facility name or *; numerical aliases and unmodeled names are not supported.",name.chars().take(60).collect::<String>()))?;
        if result.contains(&index) {
            return Err("Repeated facility in one selector".into());
        }
        result.push(index);
    }
    Ok(result)
}
pub fn valid_destination(path: &str) -> bool {
    path.starts_with("/var/log/")
        && path.len() <= 128
        && path.split('/').skip(1).all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
pub fn parse(text: &str, destinations: &[String]) -> Result<Vec<Rule>, String> {
    if destinations.is_empty()
        || destinations.len() > 4
        || destinations.iter().any(|p| !valid_destination(p))
        || destinations
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != destinations.len()
    {
        return Err("Use one to four unique supplied virtual /var/log destinations".into());
    }
    if text.chars().any(|c| c.is_whitespace() && !c.is_ascii()) {
        return Err("Selector configuration supports ASCII whitespace only; replace unusual spaces before retrying".into());
    }

    if text.len() > 8192 || text.lines().count() > 32 {
        return Err("Configuration exceeds8192 bytes or32 lines".into());
    }
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err("Configuration contains unsupported control characters".into());
    }
    let mut rules = vec![];
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let error = |message: String| format!("Line {}: {message}", n + 1);
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.len() != 2 {
            return Err(error("Use one selector and one supplied virtual destination, separated by spaces. Other rsyslog syntax is unsupported.".into()));
        }
        let destination = words[1];
        if !destinations.iter().any(|d| d == destination) || !valid_destination(destination) {
            return Err(error("Destination must be one of the supplied virtual log paths; no host paths or remote actions are opened.".into()));
        }
        let selector = words[0].to_ascii_lowercase();
        let mut pieces = selector.split(';');
        let first = pieces.next().unwrap();
        let Some((names, level)) = first.split_once('.') else {
            return Err(error(
                "A selector needs facility.level, for example daemon.warning".into(),
            ));
        };
        let names = facilities(names).map_err(error)?;
        let mut selected = [[false; 8]; 24];
        let levels: Vec<usize> = if level == "*" {
            (0..8).collect()
        } else {
            let exact = level.starts_with('=');
            let name = level.strip_prefix('=').unwrap_or(level);
            let severity=SEVERITIES.iter().position(|s|*s==name).ok_or_else(||error(format!("Unsupported level ‘{}’. Use a named severity, =severity for exactly one level, or *.",level.chars().take(60).collect::<String>())))?;
            if exact {
                vec![severity]
            } else {
                (0..=severity).collect()
            }
        };
        for facility in names {
            for &severity in &levels {
                selected[facility][severity] = true;
            }
        }
        for exclusion in pieces {
            let Some((names, level)) = exclusion.split_once('.') else {
                return Err(error("Each exclusion needs facility.none".into()));
            };
            if level != "none" {
                return Err(error("Only ;facility.none exclusions are supported after the first selector. Later positive selectors and ! filters have additional rsyslog rules and are not approximated here.".into()));
            }
            for facility in facilities(names).map_err(error)? {
                selected[facility] = [false; 8];
            }
        }
        rules.push(Rule {
            selected,
            destination: destination.into(),
        });
    }
    Ok(rules)
}
pub fn deliveries(rules: &[Rule], facility: u8, severity: u8) -> Vec<String> {
    if facility >= 24 || severity >= 8 {
        return vec![];
    }
    let mut result = rules
        .iter()
        .filter(|r| r.selected[facility as usize][severity as usize])
        .map(|r| r.destination.clone())
        .collect::<Vec<_>>();
    result.sort();
    result
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub destination: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u8,24>")]
    pub facilities: Vec<u8>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u8,8>")]
    pub severities: Vec<u8>,
    pub copies: u8,
}
pub fn expected(policies: &[Policy], facility: u8, severity: u8) -> Vec<String> {
    let mut result = vec![];
    for p in policies {
        if p.facilities.contains(&facility) && p.severities.contains(&severity) {
            for _ in 0..p.copies {
                result.push(p.destination.clone());
            }
        }
    }
    result.sort();
    result
}
pub fn compare(rules: &[Rule], policies: &[Policy]) -> Vec<(bool, String)> {
    let mut result = vec![];
    for facility in 0..24 {
        let mut failures = vec![];
        for severity in 0..8 {
            let actual = deliveries(rules, facility, severity);
            let required = expected(policies, facility, severity);
            if actual != required {
                failures.push(format!(
                    "{}({severity}): required {}; actual {}",
                    SEVERITIES[severity as usize],
                    describe_destinations(&required),
                    describe_destinations(&actual)
                ));
            }
        }
        if !failures.is_empty() {
            result.push((
                false,
                format!(
                    "{} — {}.",
                    FACILITIES[facility as usize],
                    failures.into_iter().take(2).collect::<Vec<_>>().join("; ")
                ),
            ));
        }
    }
    if result.is_empty() {
        result.push((true,"All192 source/severity combinations deliver exactly the required copies, with no extra records in other destinations.".into()));
    }
    result
}
pub fn describe_destinations(destinations: &[String]) -> String {
    if destinations.is_empty() {
        "no delivery".into()
    } else {
        destinations.join(", ")
    }
}
