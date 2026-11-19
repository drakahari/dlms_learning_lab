//! Bounded connection-only Telnet client state; learner execution stays in-process.
//! TCP transport diagnostics only; no TELNET negotiation or application data.
use crate::{
    linux_filter::{Class, Fault},
    service_exposure::{Observation, PortState, World},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
type Result<T> = std::result::Result<T, Fault>;
fn fail<T>(class: Class, message: &str) -> Result<T> {
    Err(Fault {
        class,
        message: message.into(),
    })
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Mode {
    #[default]
    Idle,
    Connected {
        port: u16,
    },
    Command {
        port: u16,
    },
}
impl Mode {
    pub fn validate(self) -> Result<()> {
        match self {
            Self::Connected { port: 0 | 23 } | Self::Command { port: 0 | 23 } => fail(
                Class::Context,
                "This transport-only activity needs a nonzero, non-default Telnet port; negotiation on port 23 is not modeled.",
            ),
            _ => Ok(()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transition {
    pub mode: Mode,
    pub observations: Option<Vec<Observation>>,
    /// None means that the client is still running, not a fabricated exit0.
    pub exit_status: Option<i32>,
    pub output: String,
}
fn line(command: &str) -> Result<()> {
    if command.len() > 1024
        || !command.is_ascii()
        || command.bytes().any(|b| b.is_ascii_control() && b != b'\t')
    {
        return fail(
            Class::Limit,
            "Keep a connection command to one ASCII line within1KiB.",
        );
    }
    if command.bytes().any(|b| b";|&<>$`\\\"'".contains(&b)) {
        return fail(
            Class::Unsupported,
            "No shell operators, quoting, redirection, local shell or file commands run.",
        );
    }
    Ok(())
}
pub fn parse(command: &str) -> Result<(Ipv4Addr, u16)> {
    line(command)?;
    let words: Vec<_> = command.split_whitespace().collect();
    let operands = match words.as_slice() {
        ["telnet", "-c", rest @ ..] => rest,
        ["telnet", rest @ ..] => rest,
        _ => {
            return fail(
                Class::Unsupported,
                "Use telnet -c SUPPLIED_IP PORT; only a supplied numeric TCP transport is modeled.",
            );
        }
    };
    if operands.iter().any(|s| s.starts_with('-')) {
        return fail(
            Class::Unsupported,
            "Only optional -c is supported; no login, trace, custom escape, routing or other client options.",
        );
    }
    let [host, port] = operands else {
        return fail(
            Class::Syntax,
            "Supply both the numeric virtual-server address and one decimal TCP port.",
        );
    };
    let host = host.parse::<Ipv4Addr>().map_err(|_| Fault {
        class: Class::Unsupported,
        message: "Names and IPv6 are outside this connection-only subset.".into(),
    })?;
    if port.is_empty()
        || !port.bytes().all(|b| b.is_ascii_digit())
        || (port.len() > 1 && port.starts_with('0'))
    {
        return fail(
            Class::Unsupported,
            "Use one ordinary decimal port, not a service name, range or list.",
        );
    }
    let port = port.parse::<u16>().ok().filter(|p| *p > 0).ok_or(Fault {
        class: Class::Limit,
        message: "Use a port from1through65535.".into(),
    })?;
    if port == 23 {
        return fail(
            Class::Unsupported,
            "This activity tests TCP transport on explicit non-default ports. TELNET option negotiation on port 23 is outside the supported subset.",
        );
    }
    Ok((host, port))
}
/// Pure transitions: no mutation of the virtual server and no host API.
/// Call only from a NEW versioned history event; Scan/ProbeV2 stay fixed.
pub fn run(world: &World, viewpoint: usize, mode: Mode, command: &str) -> Result<Transition> {
    mode.validate()?;
    world.validate()?;
    if viewpoint >= world.viewpoints.len() {
        return fail(
            Class::Context,
            "Choose one of the supplied observer positions.",
        );
    }
    line(command)?;
    match mode {
        Mode::Connected { .. } => fail(
            Class::Unsupported,
            "Application data is not modeled. Use the native Escape action, then quit at the local Telnet prompt.",
        ),
        Mode::Command { .. } => {
            if command.split_whitespace().collect::<Vec<_>>().as_slice() != ["quit"] {
                return fail(
                    Class::Unsupported,
                    "This local prompt supports quit only. It closes the simulated connection; local shell, send, files and protocol commands are not supported.",
                );
            }
            Ok(Transition {
                mode: Mode::Idle,
                observations: None,
                exit_status: Some(0),
                output: "Simulated Telnet: connection closed; client exited.\n".into(),
            })
        }
        Mode::Idle => {
            let (host, port) = parse(command)?;
            if host != world.server {
                return fail(
                    Class::Context,
                    "Use only the supplied virtual server; no real target is contacted.",
                );
            }
            let rows = world.scan(viewpoint, &[port])?;
            let state = rows
                .first()
                .ok_or(Fault {
                    class: Class::Context,
                    message: "No supplied port observation was produced.".into(),
                })?
                .state;
            let (mode, exit_status, detail) = match state {
                PortState::Open => (
                    Mode::Connected { port },
                    None,
                    format!(
                        "Trying {host}...\nConnected to {host} port {port}.\nEscape character is '^]'.\nTransport only; TELNET negotiation, login and application data are not modeled.\n"
                    ),
                ),
                PortState::Closed => (
                    Mode::Idle,
                    Some(1),
                    format!(
                        "Trying {host}...\ntelnet: Unable to connect to remote host: Connection refused (port {port}).\n"
                    ),
                ),
                PortState::Filtered => (
                    Mode::Idle,
                    Some(1),
                    format!(
                        "Trying {host}...\ntelnet: Unable to connect: modeled timeout on port {port}; no real wait.\n"
                    ),
                ),
            };
            Ok(Transition {
                mode,
                observations: Some(rows),
                exit_status,
                output: format!("Simulated Telnet TCP transport diagnostic\n{detail}"),
            })
        }
    }
}
/// Native action equivalent to entering the local command context. The literal
/// characters ^] are NOT a shell command or accepted as magic learner input.
pub fn escape(mode: Mode) -> Result<Transition> {
    mode.validate()?;
    let Mode::Connected { port } = mode else {
        return fail(
            Class::Context,
            "Escape is available only while a simulated connection is open.",
        );
    };
    Ok(Transition {
        mode: Mode::Command { port },
        observations: None,
        exit_status: None,
        output: "Simulated Telnet local command mode\ntelnet> ".into(),
    })
}
