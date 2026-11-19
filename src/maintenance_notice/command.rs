//! Literal command adapter for the finite notice model. No subprocess or host I/O.
use super::{Action, Document, Facts, World};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Syntax,
    Unsupported,
    Limit,
    State,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub kind: Kind,
    pub message: String,
}
impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Diagnostic {}
type Result<T> = std::result::Result<T, Diagnostic>;
fn fail(kind: Kind, text: &str) -> Diagnostic {
    Diagnostic {
        kind,
        message: text.into(),
    }
}
pub const HELP: &str = "Supplied local-notice terminal: who; date +%H:%M; cat /etc/issue or /etc/motd; wall (reads the visible notice composer as its simulated standard input); shutdown -r +MINUTES [literal message]; shutdown -c [literal message]. Quotes and literal escapes are supported. Only explicit future intervals from 1 through 1440 minutes are supported; cancel an existing schedule before replacing it. No wall argv/file/options, instant/absolute shutdown times, other shutdown/systemctl options, shell expansion, pipes, redirects or host access. Advance simulated clock and login previews are native simulator actions, not Linux commands. All supplied terminals are writable by the simulated administrator. Recurring countdown broadcasts, root/PAM login exceptions, getty escapes, SSH banners and real restarts are outside this subset.";
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Empty,
    Who,
    Clock,
    Read(Document),
    Change(Action),
}
/// Reuse only the existing finite literal lexer; retain its length/word guards.
/// The BIOS command dispatcher is never called.
pub fn parse(raw: &str, stdin: &str) -> Result<Request> {
    let words=crate::bios_loader::words(raw).map_err(|e| {
        use crate::boot_entry::ErrorKind as E;
        match e.kind {
            E::Syntax=>fail(Kind::Syntax,"Syntax error: close quoted words and finish literal escapes before running this command."),
            E::Limit=>fail(Kind::Limit,"Keep one command within 1024 bytes and 24 words."),
            _=>fail(Kind::Unsupported,"Unsupported syntax: use literal ASCII command words. Expansions, globbing, operators, comments and multiline command text are outside this terminal subset."),
        }
    })?;
    let args = words.iter().map(String::as_str).collect::<Vec<_>>();
    match args.as_slice() {
        [] => Ok(Request::Empty),
        ["who"] => Ok(Request::Who),
        ["date", "+%H:%M"] => Ok(Request::Clock),
        ["cat", "/etc/issue"] => Ok(Request::Read(Document::Issue)),
        ["cat", "/etc/motd"] => Ok(Request::Read(Document::Motd)),
        ["wall"] => {
            if !super::text(stdin) || stdin.trim().is_empty() {
                return Err(fail(
                    Kind::Syntax,
                    "Write the message in the visible notice composer first; this wall subset reads that text as its simulated standard input.",
                ));
            }
            Ok(Request::Change(Action::Broadcast { text: stdin.into() }))
        }
        ["wall", ..] => Err(fail(
            Kind::Unsupported,
            "This wall subset reads only the visible simulated stdin composer. Message/file operands and options are not supported; nothing was sent.",
        )),
        ["shutdown", "-c", message @ ..] => {
            if message.iter().any(|word| word.starts_with('-')) {
                return Err(fail(
                    Kind::Unsupported,
                    "Unknown shutdown option; cancellation supports -c and optional literal message words.",
                ));
            }
            Ok(Request::Change(Action::Cancel {
                text: if message.is_empty() {
                    "Scheduled shutdown cancelled.".into()
                } else {
                    message.join(" ")
                },
            }))
        }
        ["shutdown", "-r", time, message @ ..] => {
            let Some(n) = time.strip_prefix('+') else {
                return Err(fail(
                    Kind::Unsupported,
                    "Use an explicit future +MINUTES interval in this subset; now and absolute clock times are not implemented.",
                ));
            };
            if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                return Err(fail(
                    Kind::Syntax,
                    "Write decimal minutes after +, for example +15.",
                ));
            }
            let delay=n.parse::<u32>().ok().filter(|d|(1..=1440).contains(d)).ok_or_else(||fail(Kind::Limit,"Use 1 through 1440 future simulated minutes; immediate or larger schedules are outside this subset."))?;
            if message.iter().any(|word| word.starts_with('-')) {
                return Err(fail(
                    Kind::Unsupported,
                    "Additional shutdown options are outside this subset; use literal message words.",
                ));
            }
            Ok(Request::Change(Action::Schedule {
                after_minutes: delay,
                text: if message.is_empty() {
                    "A system restart has been scheduled.".into()
                } else {
                    message.join(" ")
                },
            }))
        }
        _ => Err(fail(
            Kind::Unsupported,
            "Unsupported command or options. Open Supported commands for this finite local-notice terminal; nothing ran on the host.",
        )),
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub action: Option<Action>,
}
/// Interpret once on a clone. Only successful actions are returned for persistence;
/// query output is derived from the current virtual world and supplied source.
pub fn run(facts: &Facts, world: &World, raw: &str, stdin: &str) -> Result<Output> {
    facts.validate().map_err(|message| Diagnostic {
        kind: Kind::State,
        message,
    })?;
    if world.stopped {
        return Err(fail(
            Kind::State,
            "The recorded host reached its simulated restart; reset this attempt to continue. No actual system restarted.",
        ));
    }
    let request = parse(raw, stdin)?;
    let stdout = match request {
        Request::Empty => String::new(),
        Request::Clock => format!("{:02}:{:02}\n", (world.minute / 60) % 24, world.minute % 60),
        Request::Who => {
            let mut text = String::new();
            for login in &facts.logins {
                let stamp = facts
                    .date
                    .after(login.since_minute)
                    .map_err(|e| fail(Kind::State, &e.to_string()))?;
                text.push_str(&format!(
                    "{:<12} {:<10} {}\n",
                    login.user,
                    login.tty,
                    stamp.text()
                ));
            }
            text
        }
        Request::Read(document) => world.files.get(document).to_owned(),
        Request::Change(action) => {
            let mut next = world.clone();
            next.apply(facts, &action).map_err(|message| Diagnostic {
                kind: Kind::State,
                message,
            })?;
            let stdout = if matches!(action, Action::Schedule { .. }) {
                let at = next
                    .restart
                    .as_ref()
                    .expect("schedule established by the typed transition")
                    .at;
                let stamp = facts
                    .date
                    .after(at)
                    .map_err(|e| fail(Kind::State, &e.to_string()))?;
                format!(
                    "Reboot scheduled for {} UTC; use 'shutdown -c' to cancel.\n",
                    stamp.text()
                )
            } else {
                String::new()
            };
            return Ok(Output {
                stdout,
                action: Some(action),
            });
        }
    };
    Ok(Output {
        stdout,
        action: None,
    })
}
