//! Literal explicit-device smartd.conf rules and inspection of supplied layouts.
use super::{Disk, Inventory, Layout, Result, valid_selector};
use std::collections::BTreeSet;

pub const LIMIT: usize = 4096;
const MAX_RULES: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub selector: String,
    pub health: bool,
    pub selftest: bool,
}

pub fn parse(text: &str) -> Result<Vec<Rule>> {
    if text.len() > LIMIT
        || text.lines().count() > 32
        || !text.is_ascii()
        || text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
        || text
            .as_bytes()
            .iter()
            .enumerate()
            .any(|(i, b)| *b == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n'))
    {
        return Err("Keep the supplied configuration within 4 KiB and 32 plain-text lines.".into());
    }
    let mut rules = Vec::new();
    let mut selectors = BTreeSet::new();
    for (i, line) in text.lines().enumerate() {
        let statement = line.split_once('#').map_or(line, |(left, _)| left).trim();
        if statement.is_empty() {
            continue;
        }
        let line_error = |reason: &str| format!("Line {}. {reason}", i + 1);
        let mut words = statement.split_ascii_whitespace();
        let selector = words.next().expect("Nonempty statement");
        if selector == "DEVICESCAN" {
            return Err(line_error(
                "Use explicit devices in this exercise. DEVICESCAN is outside this subset.",
            ));
        }
        if !valid_selector(selector) {
            return Err(line_error(
                "Start with a supplied /dev/sdX or /dev/disk/by-id/ata-… selector.",
            ));
        }
        if rules.len() == MAX_RULES || !selectors.insert(selector) {
            return Err(line_error(
                "Enter up to eight rules, one for each distinct selector.",
            ));
        }
        let mut rule = Rule {
            selector: selector.into(),
            health: false,
            selftest: false,
        };
        while let Some(word) = words.next() {
            match word {
                "-H" if !rule.health => rule.health = true,
                "-l" if !rule.selftest && words.next() == Some("selftest") => rule.selftest = true,
                _ => {
                    return Err(line_error(
                        "Use -H, -l selftest, or both, once each. Other smartd directives and line continuations are outside this subset.",
                    ));
                }
            }
        }
        if !rule.health && !rule.selftest {
            return Err(line_error(
                "State the monitoring checks explicitly with -H or -l selftest.",
            ));
        }
        rules.push(rule);
    }
    Ok(rules)
}

#[derive(Clone, Debug)]
pub struct Watched<'a> {
    pub rule: &'a Rule,
    pub disk: Option<&'a Disk>,
}

pub fn preview<'a>(
    rules: &'a [Rule],
    inventory: &'a Inventory,
    layout: Layout,
) -> Vec<Watched<'a>> {
    rules
        .iter()
        .map(|rule| Watched {
            rule,
            disk: inventory.find(&rule.selector, layout).map(|(_, disk)| disk),
        })
        .collect()
}
