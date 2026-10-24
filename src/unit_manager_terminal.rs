//! Closed literal service commands over supplied virtual data. Never host execution.
pub mod session;
use crate::{
    shell,
    unit_manager::{self as core, Action, JobResult, Runtime, State},
    unit_manager_workspace::Definition,
};
use serde::{Deserialize, Serialize};
pub use session::Session;

pub const MAX_COMMAND_BYTES: usize = 2048;
pub const MAX_OUTPUT_BYTES: usize = 32_768;
pub const HELP: &str = "Virtual service terminal: systemctl status/cat NAME; systemctl show NAME -p Description,Requires,Wants,After,Before,ActiveState; is-active/is-enabled NAME; start/stop/enable/disable NAME; daemon-reload; get-default/set-default TARGET. journalctl -u NAME -o cat [-n N] [--no-pager]. Literal supplied units only; no shell, host services, restart/reload/mask/isolate, remote commands or arbitrary properties. Start includes one disclosed simulated program-initialization event after its start job; exit0 is not readiness or assignment credit.";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticKind {
    Syntax,
    Unsupported,
    Context,
    Limit,
    Model,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
}
fn diagnostic(kind: DiagnosticKind, message: impl Into<String>) -> Diagnostic {
    let text = message.into();
    let mut end = text.len().min(512);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Diagnostic {
        kind,
        message: text[..end].into(),
    }
}
type Result<T> = std::result::Result<T, Diagnostic>;
#[derive(Clone, Debug)]
pub enum Query {
    Status {
        unit: String,
    },
    Cat {
        unit: String,
    },
    Show {
        unit: String,
        properties: Vec<String>,
    },
    IsActive {
        unit: String,
        quiet: bool,
    },
    IsEnabled {
        unit: String,
        quiet: bool,
    },
    GetDefault,
    Journal {
        unit: String,
        lines: Option<usize>,
    },
    Help,
}
#[derive(Clone, Debug)]
pub enum Command {
    Query(Query),
    Mutation(Action),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub stdout: String,
    pub stderr: String,
    pub status: u8,
    pub changed: bool,
    pub display_escaped: bool,
}
#[derive(Default)]
struct Options {
    no_pager: bool,
    quiet: bool,
    property: Option<String>,
    unit: Option<String>,
    lines: Option<String>,
    output: Option<String>,
    operands: Vec<String>,
}
fn put(slot: &mut Option<String>, value: String) -> Result<()> {
    if slot.is_some() {
        return Err(diagnostic(
            DiagnosticKind::Syntax,
            "Repeated command option",
        ));
    }
    *slot = Some(value);
    Ok(())
}
fn options(words: &[String]) -> Result<Options> {
    let mut result = Options::default();
    let mut i = 0;
    while i < words.len() {
        let word = &words[i];
        if word == "--no-pager" {
            if result.no_pager {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    "Repeated --no-pager option",
                ));
            }
            result.no_pager = true;
        } else if matches!(word.as_str(), "--quiet" | "-q") {
            if result.quiet {
                return Err(diagnostic(DiagnosticKind::Syntax, "Repeated quiet option"));
            }
            result.quiet = true;
        } else {
            let selected = match word.as_str() {
                "-p" | "--property" => Some(0),
                "-u" | "--unit" => Some(1),
                "-n" | "--lines" => Some(2),
                "-o" | "--output" => Some(3),
                _ => None,
            };
            let (slot, value) = if let Some(slot) = selected {
                i += 1;
                let value = words.get(i).ok_or_else(|| {
                    diagnostic(DiagnosticKind::Syntax, "Option requires an explicit value")
                })?;
                (Some(slot), Some(value.clone()))
            } else if let Some((name, value)) = word.split_once('=') {
                let slot = match name {
                    "--property" => Some(0),
                    "--unit" => Some(1),
                    "--lines" => Some(2),
                    "--output" => Some(3),
                    _ => None,
                };
                (slot, slot.map(|_| value.to_owned()))
            } else {
                (None, None)
            };
            if let (Some(slot), Some(value)) = (slot, value) {
                match slot {
                    0 => put(&mut result.property, value)?,
                    1 => put(&mut result.unit, value)?,
                    2 => put(&mut result.lines, value)?,
                    _ => put(&mut result.output, value)?,
                }
            } else if word.starts_with('-') {
                return Err(diagnostic(
                    DiagnosticKind::Unsupported,
                    "That option is not supported in this virtual terminal; see Help",
                ));
            } else {
                result.operands.push(word.clone());
            }
        }
        i += 1;
    }
    Ok(result)
}
fn known(inventory: &core::Inventory, name: &str) -> Result<String> {
    if !inventory.units.contains(name) {
        return Err(diagnostic(
            DiagnosticKind::Context,
            "Use an exact full unit name supplied by this activity; no host lookup is performed",
        ));
    }
    Ok(name.into())
}
fn one(inventory: &core::Inventory, values: &[String]) -> Result<String> {
    if values.len() != 1 {
        return Err(diagnostic(
            DiagnosticKind::Syntax,
            "This command takes one supplied full unit name",
        ));
    }
    known(inventory, &values[0])
}
fn property(name: &str) -> bool {
    matches!(
        name,
        "Description" | "Requires" | "Wants" | "After" | "Before" | "ActiveState"
    )
}
/// Parse an entire bounded argument vector before any mutation. Direct callers
/// must not use a constructed Command to bypass execute/perform validation.
pub fn parse(input: &str, inventory: &core::Inventory) -> Result<Command> {
    if input.len() > MAX_COMMAND_BYTES {
        return Err(diagnostic(
            DiagnosticKind::Limit,
            "Command exceeds2048 UTF-8 bytes",
        ));
    }
    inventory
        .validate()
        .map_err(|_| diagnostic(DiagnosticKind::Context, "Invalid virtual unit inventory"))?;
    let words = shell::literal_command(input).map_err(|e| {
        let kind = if matches!(
            e.as_str(),
            "Unfinished escape" | "Close the quoted string before running"
        ) {
            DiagnosticKind::Syntax
        } else {
            DiagnosticKind::Unsupported
        };
        diagnostic(kind, e)
    })?;
    if words.is_empty() {
        return Err(diagnostic(
            DiagnosticKind::Syntax,
            "Enter one supported command; see Help",
        ));
    }
    if words.len() > 32 {
        return Err(diagnostic(
            DiagnosticKind::Limit,
            "Enter one supported command, at most32 arguments",
        ));
    }
    if words == ["help"] {
        return Ok(Command::Query(Query::Help));
    }
    let options = options(&words[1..])?;
    if words[0] == "journalctl" {
        if !options.operands.is_empty() || options.quiet || options.property.is_some() {
            return Err(diagnostic(
                DiagnosticKind::Unsupported,
                "Only an exact unit filter, message-only output, an optional explicit line count and --no-pager are supported",
            ));
        }
        let unit = known(
            inventory,
            options.unit.as_deref().ok_or_else(|| {
                diagnostic(DiagnosticKind::Syntax, "Use journalctl -u NAME -o cat")
            })?,
        )?;
        if options.output.as_deref() != Some("cat") {
            return Err(diagnostic(
                DiagnosticKind::Unsupported,
                "Use -o cat for the supported message-only journal view; real timestamp/PID output formats are not modeled",
            ));
        }
        let lines = if let Some(value) = options.lines {
            if value.is_empty() || !value.bytes().all(|c| c.is_ascii_digit()) {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    "Journal line count must be a whole decimal number1 through256",
                ));
            }
            Some(
                value
                    .parse::<usize>()
                    .ok()
                    .filter(|n| (1..=core::journal::MAX_EVENTS).contains(n))
                    .ok_or_else(|| {
                        diagnostic(
                            DiagnosticKind::Limit,
                            "Supported journal line count is1 through256",
                        )
                    })?,
            )
        } else {
            None
        };
        return Ok(Command::Query(Query::Journal { unit, lines }));
    }
    if words[0] != "systemctl" {
        return Err(diagnostic(
            DiagnosticKind::Unsupported,
            "This terminal supports the stated systemctl/journalctl subset only; no host or shell command runs",
        ));
    }
    if options.unit.is_some() || options.lines.is_some() || options.output.is_some() {
        return Err(diagnostic(
            DiagnosticKind::Unsupported,
            "Those options belong to the supported journal query, not this systemctl command",
        ));
    }
    let Some(verb) = options.operands.first() else {
        return Err(diagnostic(
            DiagnosticKind::Syntax,
            "Specify the supported systemctl operation",
        ));
    };
    if options.quiet && !matches!(verb.as_str(), "is-active" | "is-enabled") {
        return Err(diagnostic(
            DiagnosticKind::Unsupported,
            "Quiet output is supported only for is-active and is-enabled",
        ));
    }
    if options.property.is_some() && verb != "show" {
        return Err(diagnostic(
            DiagnosticKind::Unsupported,
            "Property selection is supported only by systemctl show",
        ));
    }
    let args = &options.operands[1..];
    let query = match verb.as_str() {
        "status" => Some(Query::Status {
            unit: one(inventory, args)?,
        }),
        "cat" => Some(Query::Cat {
            unit: one(inventory, args)?,
        }),
        "is-active" => Some(Query::IsActive {
            unit: one(inventory, args)?,
            quiet: options.quiet,
        }),
        "is-enabled" => Some(Query::IsEnabled {
            unit: one(inventory, args)?,
            quiet: options.quiet,
        }),
        "get-default" => {
            if !args.is_empty() {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    "get-default takes no unit argument",
                ));
            }
            Some(Query::GetDefault)
        }
        "show" => {
            let unit = one(inventory, args)?;
            let selected=options.property.ok_or_else(||diagnostic(DiagnosticKind::Unsupported,"Select the modeled properties explicitly with -p; full systemctl property structures are not supported"))?;
            let mut properties = Vec::new();
            for item in selected.split(',') {
                if properties.len() >= 8 || !property(item) || properties.iter().any(|s| s == item)
                {
                    return Err(diagnostic(
                        DiagnosticKind::Unsupported,
                        "Choose distinct supported properties: Description, Requires, Wants, After, Before, ActiveState",
                    ));
                }
                properties.push(item.into());
            }
            Some(Query::Show { unit, properties })
        }
        _ => None,
    };
    if let Some(query) = query {
        return Ok(Command::Query(query));
    }
    let action = match verb.as_str() {
        "daemon-reload" => {
            if !args.is_empty() {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    "daemon-reload takes no unit argument",
                ));
            }
            Action::DaemonReload {}
        }
        "start" => Action::Start {
            unit: one(inventory, args)?,
        },
        "stop" => Action::Stop {
            unit: one(inventory, args)?,
        },
        "enable" => Action::Enable {
            unit: one(inventory, args)?,
        },
        "disable" => Action::Disable {
            unit: one(inventory, args)?,
        },
        "set-default" => {
            let target = one(inventory, args)?;
            if !target.ends_with(".target") {
                return Err(diagnostic(
                    DiagnosticKind::Syntax,
                    "set-default needs a supplied .target name",
                ));
            }
            Action::SetDefault { target }
        }
        _ => {
            return Err(diagnostic(
                DiagnosticKind::Unsupported,
                "That systemctl operation is outside this simulation; restart/reload/mask/isolate/--now and host operations are not implemented",
            ));
        }
    };
    Ok(Command::Mutation(action))
}
#[derive(Default)]
struct Output {
    body: String,
    escaped: bool,
}
impl Output {
    fn push(&mut self, text: &str) -> Result<()> {
        for c in text.chars() {
            let mut bytes = [0; 4];
            if c.is_control() && !matches!(c, '\n' | '\t') {
                let escaped = format!("\\u{{{:x}}}", c as u32);
                if self.body.len() + escaped.len() > MAX_OUTPUT_BYTES {
                    return Err(diagnostic(
                        DiagnosticKind::Limit,
                        "Output exceeds32KiB; narrow the request or inspect the native editor",
                    ));
                }
                self.body.push_str(&escaped);
                self.escaped = true;
            } else {
                let s = c.encode_utf8(&mut bytes);
                if self.body.len() + s.len() > MAX_OUTPUT_BYTES {
                    return Err(diagnostic(
                        DiagnosticKind::Limit,
                        "Output exceeds32KiB; use a smaller journal -n value or inspect the editor",
                    ));
                }
                self.body.push_str(s);
            }
        }
        Ok(())
    }
    fn line(&mut self, text: &str) -> Result<()> {
        self.push(text)?;
        self.push("\n")
    }
    fn finish(self, status: u8) -> Outcome {
        Outcome {
            stdout: self.body,
            stderr: String::new(),
            status,
            changed: false,
            display_escaped: self.escaped,
        }
    }
}
fn active(state: &State, unit: &str) -> (&'static str, u8) {
    match state.runtime().get(unit) {
        Some(
            Runtime::Running { .. }
            | Runtime::Exited { .. }
            | Runtime::TargetActive {}
            | Runtime::Dispatched { .. },
        ) => ("active", 0),
        Some(Runtime::Failed { .. }) => ("failed", 3),
        _ => ("inactive", 3),
    }
}
fn enabled(state: &State, unit: &str) -> Result<(&'static str, u8)> {
    let source = &state.documents()[unit];
    let definition = core::parse_unit(state.inventory(), unit, source).map_err(|e| {
        diagnostic(
            DiagnosticKind::Model,
            format!("Saved install information cannot be read: {e}"),
        )
    })?;
    if state.boot_links().values().any(|v| v.contains(unit)) {
        Ok(("enabled", 0))
    } else if definition.wanted_by.is_empty() {
        Ok(("static", 0))
    } else {
        Ok(("disabled", 1))
    }
}
fn show_query(state: &State, query: &Query) -> Result<Outcome> {
    let mut output = Output::default();
    match query {
        Query::Help => output.line(HELP)?,
        Query::GetDefault => output.line(state.default_target())?,
        Query::IsActive { unit, quiet } => {
            let (label, status) = active(state, unit);
            if !quiet {
                output.line(label)?;
            }
            return Ok(output.finish(status));
        }
        Query::IsEnabled { unit, quiet } => {
            let (label, status) = enabled(state, unit)?;
            if !quiet {
                output.line(label)?;
            }
            return Ok(output.finish(status));
        }
        Query::Cat { unit } => {
            output.line(&format!("# Virtual backing source: {unit}"))?;
            output.push(&state.documents()[unit])?;
        }
        Query::Show { unit, properties } => {
            let d = state
                .effective_properties(unit)
                .map_err(|e| diagnostic(DiagnosticKind::Model, e.to_string()))?;
            for p in properties {
                let value = match p.as_str() {
                    "Description" => d.description.clone(),
                    "Requires" => d.requires.iter().cloned().collect::<Vec<_>>().join(" "),
                    "Wants" => d.wants.iter().cloned().collect::<Vec<_>>().join(" "),
                    "After" => d.after.iter().cloned().collect::<Vec<_>>().join(" "),
                    "Before" => d.before.iter().cloned().collect::<Vec<_>>().join(" "),
                    "ActiveState" => active(state, unit).0.into(),
                    _ => {
                        return Err(diagnostic(
                            DiagnosticKind::Unsupported,
                            "Unsupported virtual property",
                        ));
                    }
                };
                output.line(&format!("{p}={value}"))?;
            }
        }
        Query::Status { unit } => {
            let defs = state
                .definitions()
                .map_err(|e| diagnostic(DiagnosticKind::Model, e.to_string()))?;
            output.line(&format!(
                "{unit} - {}",
                defs.get(unit)
                    .map_or("Virtual unit (not loaded)", |d| d.description.as_str())
            ))?;
            output.line(if defs.contains_key(unit) {
                "     Loaded: loaded (supplied virtual unit source)"
            } else {
                "     Loaded: not loaded (simulator; manager reload required)"
            })?;
            let (label, status) = active(state, unit);
            let suffix = match state.runtime().get(unit) {
                Some(Runtime::Running { .. }) => " (running)",
                Some(Runtime::Exited { .. }) => " (exited)",
                _ => "",
            };
            output.line(&format!("     Active: {label}{suffix}"))?;
            if let Ok((policy, _)) = enabled(state, unit) {
                output.line(&format!("     Future install policy: {policy}"))?;
            } else {
                output.line("     Saved install information: invalid; inspect backing source")?;
            }
            if state.needs_daemon_reload(unit).is_ok_and(|v| v) {
                output.line("     Simulator: saved and loaded unit text differ")?;
            }
            if matches!(state.runtime().get(unit), Some(Runtime::Dispatched { .. })) {
                output.line("     Simulator: program initialization pending; manager-active is not readiness")?;
            }
            if let Some(journal) = state.journal().filter(|j| j.dropped() > 0) {
                output.line(&format!(
                    "     Simulator: retained journal window dropped {} older events",
                    journal.dropped()
                ))?;
            }
            if matches!(state.runtime().get(unit), Some(Runtime::Failed { .. })) {
                output.line("     Simulator: fictional program failed; inspect its retained virtual journal and configuration")?;
            }
            return Ok(output.finish(status));
        }
        Query::Journal { unit, lines } => {
            if let Some(journal) = state.journal() {
                let entries = journal
                    .events()
                    .iter()
                    .filter(|e| e.unit() == unit)
                    .collect::<Vec<_>>();
                let skip = entries.len().saturating_sub(lines.unwrap_or(entries.len()));
                for event in entries.into_iter().skip(skip) {
                    output.line(&event.message())?;
                }
            }
        }
    }
    Ok(output.finish(0))
}
fn allowed(action: &Action, inventory: &core::Inventory) -> bool {
    match action {
        Action::Start { unit }
        | Action::Stop { unit }
        | Action::Enable { unit }
        | Action::Disable { unit } => inventory.units.contains(unit),
        Action::DaemonReload {} => true,
        Action::SetDefault { target } => {
            inventory.units.contains(target) && target.ends_with(".target")
        }
        _ => false,
    }
}
/// Shared native Start policy: job status first, then one bounded pending-program
/// event. Validation failure at either stage publishes no state or journal.
pub fn perform(definition: &Definition, state: &mut State, action: &Action) -> Result<Outcome> {
    if !definition.same_assignment(state) || !allowed(action, state.inventory()) {
        return Err(diagnostic(
            DiagnosticKind::Context,
            "Action or state differs from this fixed virtual assignment",
        ));
    }
    let mut candidate = state.clone();
    let report = candidate
        .apply(action)
        .map_err(|e| diagnostic(DiagnosticKind::Model, e.to_string()))?;
    let requested_job = if matches!(action, Action::Start { .. } | Action::Stop { .. }) {
        let requested = report.requested.as_ref().ok_or_else(|| {
            diagnostic(
                DiagnosticKind::Model,
                "The virtual manager did not identify the requested job",
            )
        })?;
        Some(
            report
                .jobs
                .iter()
                .find(|j| &j.unit == requested)
                .ok_or_else(|| {
                    diagnostic(
                        DiagnosticKind::Model,
                        "The virtual manager did not report the requested job",
                    )
                })?,
        )
    } else {
        None
    };
    let status = u8::from(requested_job.is_some_and(|j| !j.result.succeeded()));
    if matches!(action, Action::Start { .. }) {
        candidate
            .apply(&Action::AdvancePrograms {})
            .map_err(|e| diagnostic(DiagnosticKind::Model, e.to_string()))?;
    }
    let changed = candidate != *state;
    *state = candidate;
    let stderr = if status != 0 {
        requested_job
            .and_then(|j| match &j.result {
                JobResult::Failed { message, .. } => Some(format!(
                    "Simulated requested job failed for {}: {message}",
                    j.unit
                )),
                JobResult::DependencyFailed { unit } => Some(format!(
                    "Simulated requested job blocked by failed ordered requirement {unit}"
                )),
                _ => None,
            })
            .unwrap_or_else(|| "Simulated requested job failed".into())
    } else {
        String::new()
    };
    Ok(Outcome {
        stdout: String::new(),
        stderr,
        status,
        changed,
        display_escaped: false,
    })
}
/// Goal-aware command execution; only parsed mutations can change supplied data.
pub fn execute(definition: &Definition, state: &mut State, input: &str) -> Result<Outcome> {
    if !definition.same_assignment(state) {
        return Err(diagnostic(
            DiagnosticKind::Context,
            "Foreign or invalid virtual unit assignment",
        ));
    }
    match parse(input, state.inventory())? {
        Command::Query(query) => show_query(state, &query),
        Command::Mutation(action) => perform(definition, state, &action),
    }
}
/// Raw engine/native inspection offers queries only, never a mutation fallback.
pub fn read_only(state: &State, input: &str) -> Result<Outcome> {
    state
        .validate()
        .map_err(|_| diagnostic(DiagnosticKind::Context, "Invalid virtual unit state"))?;
    match parse(input, state.inventory())? {
        Command::Query(query) => show_query(state, &query),
        Command::Mutation(_) => Err(diagnostic(
            DiagnosticKind::Context,
            "Service mutations require the fixed assignment context",
        )),
    }
}
