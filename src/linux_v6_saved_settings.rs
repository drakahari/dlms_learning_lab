//! One validated virtual sysctl file; no real file, process or startup integration.
use crate::{linux_v6_admin as core, linux_v6_settings as settings};
use std::fmt;
pub const FILE: &str = "/etc/sysctl.d/70-learning-lab-ipv6.conf";
pub const MAX_TEXT: usize = 4096;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub key: String,
    pub value: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fault {
    pub line: Option<usize>,
    pub detail: String,
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(n) = self.line {
            write!(f, "Line {n}: {}", self.detail)
        } else {
            f.write_str(&self.detail)
        }
    }
}
fn fault(line: Option<usize>, detail: impl Into<String>) -> Fault {
    Fault {
        line,
        detail: detail.into(),
    }
}
fn trim(s: &str) -> &str {
    s.trim_matches([' ', '\t', '\r'])
}
pub fn text_ok(text: &str) -> bool {
    text.len() <= MAX_TEXT
        && !text
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
/// The native Save action validates this narrow subset first. This is not a
/// claim that real procps rolls back a partially failing unrestricted file.
pub fn parse(model: &settings::State, text: &str) -> Result<Vec<Assignment>, Fault> {
    model.validate().map_err(|e| fault(None, e.message))?;
    if !text_ok(text) {
        return Err(fault(
            None,
            "Keep the virtual settings file within 4 KiB, using ordinary text.",
        ));
    }
    let mut probe = model.clone();
    let mut assignments = vec![];
    for (n, line) in text.lines().enumerate() {
        let number = Some(n + 1);
        if n >= 24 || line.len() > 256 {
            return Err(fault(
                number,
                "This activity supports at most 24 lines of 256 bytes each.",
            ));
        }
        let line = trim(line);
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| fault(number, "Use KEY = VALUE, or a whole-line # or ; comment."))?;
        let key = trim(key);
        let value = trim(value);
        if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(fault(
                number,
                "This simulator subset uses one bare decimal setting value; quotes, inline comments, lists and expansions are not supported.",
            ));
        }
        let value = value.parse::<u8>().map_err(|_| {
            fault(
                number,
                "Use a supported forwarding/accept_ra value: 0, 1, or 2 as appropriate.",
            )
        })?;
        probe
            .command(&format!("sysctl -w {key}={value}"))
            .map_err(|e| fault(number, e.message))?;
        assignments.push(Assignment {
            key: key.into(),
            value,
        });
    }
    Ok(assignments)
}
pub fn apply(world: &mut core::State, text: &str) -> Result<String, Fault> {
    let assignments = parse(&world.settings, text)?;
    let mut next = world.clone();
    let mut output = String::new();
    for a in assignments {
        let out = next
            .command(&format!("sysctl -w {}={}", a.key, a.value))
            .map_err(|e| fault(None, e))?;
        output.push_str(&out.text);
    }
    *world = next;
    Ok(output)
}
/// Fresh settings/learned routes on the supplied already-persistent network.
/// This is a scoped startup-settings test, not a complete Linux reboot model.
pub fn fresh(world: &core::State, text: &str) -> Result<core::State, Fault> {
    world.validate().map_err(|e| fault(None, e))?;
    let mut next = core::State::new(world.host.clone()).map_err(|e| fault(None, e))?;
    apply(&mut next, text)?;
    Ok(next)
}
