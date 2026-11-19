//! Closed, bounded Postfix command subset. All state is supplied virtual mail data.
use crate::{
    smtp_config::{Document, ErrorKind},
    smtp_queue::{self as queue, Status},
    smtp_workspace,
};
pub const HELP: &str = "Simulated commands on the selected server: postconf -n; postfix reload; postqueue -p; postqueue -f. No shell syntax, sudo, paths, pipes or other flags. Edit main.cf in the native editor and Save first. Flush advances one bounded sample attempt immediately; real Postfix schedules asynchronous delivery. Queue sizes count represented subject/body bytes, not RFC message sizes. No clock, daemon or host files are represented. postfix check is not a route validator: its real file-ownership/directory checks are outside this model.";
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Result {
    pub status: u8,
    pub stdout: String,
    pub stderr: String,
    /// Simulator observations, separate from the utility's own output.
    pub observations: Vec<String>,
    pub changed: bool,
}
fn error(status: u8, message: impl Into<String>) -> Result {
    Result {
        status,
        stdout: String::new(),
        stderr: message.into(),
        observations: vec![],
        changed: false,
    }
}
fn fault(e: crate::smtp_config::Fault) -> Result {
    let class = match e.kind {
        ErrorKind::Invalid => "Invalid configuration",
        ErrorKind::Unsupported => "Outside the supported configuration subset",
        ErrorKind::Limit => "Configuration limit",
    };
    error(1, format!("{class}: {}\n", e.message))
}
pub fn run(f: &queue::Facts, r: &mut queue::Runtime, node: &str, input: &str) -> Result {
    if let Err(e) = r.validate(f) {
        return fault(e);
    }
    if !f.world.nodes.contains_key(node) {
        return error(2, "Choose a supplied mail server.\n");
    }
    if input.len() > 4096 {
        return error(2, "Command exceeds4KiB. Nothing was run.\n");
    }
    if input.chars().any(|c| c.is_control() && c != '\t')
        || input.contains([';', '|', '&', '$', '`', '>', '<', '\\', '"', '\''])
    {
        return error(
            2,
            "Unsupported shell syntax. This console accepts one supported command, never a host shell.\n",
        );
    }
    let tokens = input.split_ascii_whitespace().collect::<Vec<_>>();
    let mut result = Result {
        status: 0,
        stdout: String::new(),
        stderr: String::new(),
        observations: vec![],
        changed: false,
    };
    match tokens.as_slice() {
        ["postconf", "-n"] => {
            let d = match Document::parse(&r.files[node]) {
                Ok(d) => d,
                Err(e) => return fault(e),
            };
            result.stdout = d
                .parameters
                .iter()
                .map(|(k, v)| format!("{k} = {v}\n"))
                .collect();
            result.stderr = d
                .warnings
                .iter()
                .map(|w| format!("postconf: warning: {w}\n"))
                .collect();
        }
        ["postfix", "reload"] => {
            if let Err(e) = r.reload(f, node) {
                return fault(e);
            }
            result.stderr = "postfix/postfix-script: refreshing the Postfix mail system\n".into();
            result.changed = true;
            result.observations.push(format!("{node}: the represented running settings now match its saved file. Existing sample mail was not retried."));
        }
        ["postqueue", "-p"] => {
            let pending = f
                .samples
                .iter()
                .filter(|(id, s)| s.entry == node && r.messages[*id].status.pending())
                .collect::<Vec<_>>();
            if pending.is_empty() {
                result.stdout = "Mail queue is empty\n".into();
            } else {
                result.stdout = "-Queue ID- --Sample bytes-- -Sender/Recipient-------\n".into();
                let mut bytes = 0;
                for (id, s) in &pending {
                    let m = &r.messages[*id];
                    let size = s.subject.len() + s.body.len();
                    bytes += size;
                    result.stdout.push_str(&format!(
                        "{:<10} {:>16}  {}\n",
                        m.queue_id,
                        size,
                        s.sender.as_deref().unwrap_or("MAILER-DAEMON")
                    ));
                    if let Status::Deferred { reason } = &m.status {
                        result.stdout.push_str(&format!("({reason})\n"));
                    }
                    result
                        .stdout
                        .push_str(&format!("                            {}\n\n", s.recipient));
                }
                result.stdout.push_str(&format!(
                    "-- {bytes} represented bytes in {} Request(s).\n",
                    pending.len()
                ));
            }
        }
        ["postqueue", "-f"] => {
            let observations = match r.flush(f, node) {
                Ok(o) => o,
                Err(e) => return fault(e),
            };
            result.changed = !observations.is_empty();
            result.observations.push(if observations.is_empty() {
                format!("No pending sample mail on {node}; nothing was retried.")
            } else {
                format!(
                    "One bounded retry pass handled {} pending sample(s) on {node}.",
                    observations.len()
                )
            });
            result.observations.push(
                "Real postqueue -f requests asynchronous delivery; exit0 does not prove delivery."
                    .into(),
            );
            for (id, t) in observations {
                result.observations.push(format!(
                    "{}: {}",
                    r.messages[&id].queue_id,
                    smtp_workspace::observed(&t)
                ));
            }
        }
        ["postfix", "check"] => {
            return error(
                2,
                "Not simulated: real postfix check examines file ownership/permissions and missing directories. It is not a proof of correct routing. Use the editor and Check configuration for this task's behavioral tests.\n",
            );
        }
        _ => return error(2, format!("Unsupported command or arguments. {HELP}\n")),
    }
    // No command ever imports a host default or interprets parameter values as code.
    result
}
/// Keep console history on a UTF-8 boundary, retaining the latest observation.
pub fn append(history: &mut String, text: &str) {
    history.push_str(text);
    if history.len() > 32768 {
        let mut first = history.len() - 32768;
        while !history.is_char_boundary(first) {
            first += 1;
        }
        history.drain(..first);
    }
}
