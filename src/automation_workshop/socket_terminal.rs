//! One fixed command subset for the socket retirement work order.
use super::socket_retirement::{Operation, Target, Task, Work};
use crate::unit_manager::Runtime;

pub const HELP: &str = "This virtual terminal supports systemctl status, is-active, is-enabled, start, stop, enable and disable on one supplied unit. Enable and disable also accept --now. Use the local test controls to send a sample request or preview the next simulated startup.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub status: u8,
    pub text: String,
}

fn status(text: impl Into<String>, status: u8) -> Output {
    Output {
        status,
        text: text.into(),
    }
}

impl Task {
    pub fn terminal(&self, work: &mut Work, input: &str) -> Result<Output, String> {
        if input.len() > 512 || input.chars().any(|c| c.is_control() && c != '\t') {
            return Err("Use a short command on one line in this simulated terminal.".into());
        }
        let mut tokens = input.split_ascii_whitespace();
        if tokens.next() != Some("systemctl") {
            return Err(HELP.into());
        }
        let verb = tokens.next().ok_or(HELP)?;
        let first = tokens
            .next()
            .ok_or("Specify one supplied service or socket unit.")?;
        let (name, now) = if first == "--now" {
            (tokens.next().ok_or("Specify one unit after --now.")?, true)
        } else {
            (first, false)
        };
        if tokens.next().is_some() || (now && !matches!(verb, "enable" | "disable")) {
            return Err("Supply one unit. --now is supported with enable and disable.".into());
        }
        let (target, flag, active) = if name == self.activating_socket {
            (
                Target::ObsoleteSocket,
                work.activation.enabled,
                work.activation.listening,
            )
        } else if name == self.obsolete_service || name == self.required_service {
            let enabled = work
                .services
                .boot_links()
                .values()
                .any(|names| names.contains(name));
            let active = work
                .services
                .runtime()
                .get(name)
                .is_some_and(Runtime::active);
            let target = if name == self.required_service {
                Target::RequiredService
            } else {
                Target::ObsoleteService
            };
            (target, enabled, active)
        } else {
            return Err("Choose a service or socket listed in this work order.".into());
        };
        let (enablement, active_text) = (
            if flag { "enabled" } else { "disabled" },
            if active {
                if target == Target::ObsoleteSocket {
                    "active (listening)"
                } else {
                    "active (running)"
                }
            } else {
                "inactive (dead)"
            },
        );
        let operation = match verb {
            "is-active" => {
                return Ok(status(
                    if active { "active" } else { "inactive" },
                    if active { 0 } else { 3 },
                ));
            }
            "is-enabled" => return Ok(status(enablement, if flag { 0 } else { 1 })),
            "status" => {
                return Ok(status(
                    format!(
                        "{name}\nLoaded: loaded (supplied virtual unit; {enablement})\nActive: {active_text}"
                    ),
                    if active { 0 } else { 3 },
                ));
            }
            "start" => Operation::Start,
            "stop" => Operation::Stop,
            "enable" => Operation::Enable,
            "disable" => Operation::Disable,
            _ => return Err(HELP.into()),
        };
        self.apply(work, target, operation)?;
        if now {
            self.apply(
                work,
                target,
                if operation == Operation::Enable {
                    Operation::Start
                } else {
                    Operation::Stop
                },
            )?;
        }
        Ok(status("", 0))
    }
}
