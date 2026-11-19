//! Bounded IPv6 filter save/restore data, reusing the existing tracked-rule parser.
//! Pure in-process values. No file I/O, executable scripts, kernel or subprocess.
use crate::{
    linux_v6_filter::{Chain, Class, Fault, Verdict},
    linux_v6_state_filter::Table,
};
pub const MAX_BYTES: usize = 32768;
pub const MAX_LINES: usize = 256;
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn context(message: String) -> Fault {
    Fault {
        class: Class::Syntax,
        message,
    }
}
fn text_ok(s: &str, max: usize) -> bool {
    s.len() <= max
        && s.bytes()
            .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
}
pub fn save(table: &Table) -> Result<String, Fault> {
    table.validate()?;
    let mut out = String::from("*filter\n");
    for c in [Chain::Input, Chain::Forward, Chain::Output] {
        let policy = match table.base.chain(c).policy {
            Verdict::Accept => "ACCEPT",
            Verdict::Drop => "DROP",
        };
        out.push_str(&format!(":{} {policy} [0:0]\n", c.name()));
    }
    for line in table
        .listing(None)?
        .lines()
        .filter(|l| l.starts_with("-A "))
    {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("COMMIT\n");
    if out.len() > MAX_BYTES {
        return Err(fault(
            Class::Limit,
            "The exported virtual IPv6 filter exceeded 32 KiB.",
        ));
    }
    Ok(out)
}
pub fn restore(text: &str) -> Result<Table, Fault> {
    if !text_ok(text, MAX_BYTES) || text.lines().count() > MAX_LINES {
        return Err(fault(
            Class::Limit,
            "Use at most 256 lines and 32 KiB of ASCII filter data.",
        ));
    }
    let mut table = Table::new(Verdict::Accept);
    let mut started = false;
    let mut committed = false;
    let mut policies = vec![];
    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if committed {
            return Err(fault(
                Class::Unsupported,
                "This profile restores one complete filter table; extra tables or commands after COMMIT are unmodeled.",
            ));
        }
        if !started {
            if line != "*filter" {
                return Err(fault(
                    Class::Unsupported,
                    "Begin this virtual snapshot with *filter; other tables and executable scripts are not supported.",
                ));
            }
            started = true;
            continue;
        }
        if line == "COMMIT" {
            if policies.len() != 3 {
                return Err(fault(
                    Class::Unsupported,
                    "This snapshot profile requires explicit INPUT, FORWARD and OUTPUT policy declarations.",
                ));
            }
            committed = true;
            continue;
        }
        if let Some(header) = line.strip_prefix(':') {
            if !table.base.input.rules.is_empty()
                || !table.base.forward.rules.is_empty()
                || !table.base.output.rules.is_empty()
            {
                return Err(fault(
                    Class::Unsupported,
                    "Declare all three policies before rules in this snapshot profile.",
                ));
            }
            let parts = header.split_whitespace().collect::<Vec<_>>();
            if parts.len() != 3 {
                return Err(context(format!(
                    "Line{}: use :CHAIN POLICY [0:0].",
                    line_no + 1
                )));
            }
            if !matches!(parts[0], "INPUT" | "FORWARD" | "OUTPUT") {
                return Err(fault(
                    Class::Unsupported,
                    "Custom chains are outside this IPv6 filter snapshot profile.",
                ));
            }
            if policies.contains(&parts[0]) {
                return Err(fault(
                    Class::Unsupported,
                    "Use one policy declaration per built-in chain in this profile.",
                ));
            }
            if parts[2] != "[0:0]" {
                return Err(fault(
                    Class::Unsupported,
                    "This read-only policy-analysis profile does not model packet/byte counters; only [0:0] declarations are supported.",
                ));
            }
            table
                .command(&format!("ip6tables -P {} {}", parts[0], parts[1]))
                .map_err(|e| Fault {
                    class: e.class,
                    message: format!("Line{}: {}", line_no + 1, e.message),
                })?;
            policies.push(parts[0]);
            continue;
        }
        if line.starts_with("-A ") {
            if policies.len() != 3 {
                return Err(fault(
                    Class::Unsupported,
                    "Declare all three policies before adding rules in this snapshot profile.",
                ));
            }
            table
                .command(&format!("ip6tables {line}"))
                .map_err(|e| Fault {
                    class: e.class,
                    message: format!("Line{}: {}", line_no + 1, e.message),
                })?;
            continue;
        }
        return Err(context(format!(
            "Line{}: expected a policy declaration, supported -A rule or COMMIT; no host command ran.",
            line_no + 1
        )));
    }
    if !started || !committed {
        return Err(fault(
            Class::Syntax,
            "Complete the snapshot with *filter, all three policies, supported rules and COMMIT.",
        ));
    }
    table.validate()?;
    Ok(table)
}

/// A validated complete table is committed only after all parsing succeeds.
/// check_only corresponds to restore --test; connection observations are separate.
pub fn apply_restore(current: &mut Table, text: &str, check_only: bool) -> Result<(), Fault> {
    let next = restore(text)?;
    if !check_only {
        *current = next;
    }
    Ok(())
}
