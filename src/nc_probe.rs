//! Bounded numeric zero-I/O probes over the existing virtual TCP model.
//! No socket, filesystem, process or application-payload API.
use crate::linux_filter::{Class, Fault};
use std::{collections::BTreeSet, net::Ipv4Addr};
type Result<T> = std::result::Result<T, Fault>;
fn fail<T>(class: Class, message: &str) -> Result<T> {
    Err(Fault {
        class,
        message: message.into(),
    })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub target: Ipv4Addr,
    pub port: u16,
}
pub fn parse(input: &str) -> Result<Query> {
    if input.len() > 1024
        || !input.is_ascii()
        || input.bytes().any(|b| b.is_ascii_control() && b != b'\t')
    {
        return fail(
            Class::Limit,
            "Keep the port-probe command to one ASCII line within 1 KiB.",
        );
    }
    if input.bytes().any(|b| b";|&<>$`\\\"'".contains(&b)) {
        return fail(
            Class::Unsupported,
            "No shell operators, quoting, redirection or execution is supported by this numeric port-probe input.",
        );
    }
    let words: Vec<_> = input.split_whitespace().collect();
    if words.first() != Some(&"nc") {
        return fail(
            Class::Unsupported,
            "Use the disclosed nc -n -v -z numeric-host port form.",
        );
    }
    let mut flags = BTreeSet::new();
    let mut at = 1;
    while at < words.len() && words[at].starts_with('-') {
        let token = words[at];
        if token.len() < 2 || token.starts_with("--") {
            return fail(
                Class::Unsupported,
                "Only short n, v and z flags are modeled; no other options run.",
            );
        }
        for c in token[1..].chars() {
            if !['n', 'v', 'z'].contains(&c) {
                return fail(
                    Class::Unsupported,
                    "Only numeric (-n), verbose (-v), zero-I/O TCP (-z) probing is modeled. No listen, UDP, data, TLS, proxy or execution mode.",
                );
            }
            if !flags.insert(c) {
                return fail(
                    Class::Unsupported,
                    "Repeated flags are outside this bounded form; use each of -n, -v and -z once.",
                );
            }
        }
        at += 1;
    }
    if flags != BTreeSet::from(['n', 'v', 'z']) {
        return fail(
            Class::Unsupported,
            "This offline subset requires -n -v -z: numeric lookup-free, visible, zero-I/O TCP probing.",
        );
    }
    let [target, port] = &words[at..] else {
        return fail(
            Class::Syntax,
            "Supply one numeric server address and one decimal TCP port after the flags.",
        );
    };
    let target = target.parse::<Ipv4Addr>().map_err(|_| Fault {
        class: Class::Unsupported,
        message:
            "Use only the supplied numeric IPv4 address; names and IPv6 are outside this subset."
                .into(),
    })?;
    if port.is_empty()
        || !port.bytes().all(|b| b.is_ascii_digit())
        || (port.len() > 1 && port.starts_with('0'))
    {
        return fail(
            Class::Unsupported,
            "Use one ordinary decimal port, not a service name, list or range.",
        );
    }
    let port = port.parse::<u16>().ok().filter(|p| *p > 0).ok_or(Fault {
        class: Class::Limit,
        message: "Use a TCP port from 1 through 65535.".into(),
    })?;
    Ok(Query { target, port })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub observations: Vec<crate::service_exposure::Observation>,
    pub output: String,
    pub status: i32,
}
/// Query existing listener/filter facts; never change world state or contact a host.
pub fn probe(
    world: &crate::service_exposure::World,
    viewpoint: usize,
    command: &str,
) -> Result<Report> {
    let query = parse(command)?;
    if query.target != world.server {
        return fail(
            Class::Context,
            "Use only the supplied virtual server address; no real target is contacted.",
        );
    }
    let observations = world.scan(viewpoint, &[query.port])?;
    let observer = world.viewpoints.get(viewpoint).ok_or(Fault {
        class: Class::Context,
        message: "Choose a supplied observer.".into(),
    })?;
    let row = observations.first().ok_or(Fault {
        class: Class::Context,
        message: "No virtual port observation was produced.".into(),
    })?;
    use crate::service_exposure::PortState;
    let (status, line) = match row.state {
        PortState::Open => (
            0,
            format!(
                "Connection to {} {} port [tcp/*] succeeded!",
                world.server, row.port
            ),
        ),
        PortState::Closed => (
            1,
            format!(
                "nc: connect to {} port {} (tcp) failed: Connection refused",
                world.server, row.port
            ),
        ),
        PortState::Filtered => (
            1,
            format!(
                "nc: connect to {} port {} (tcp) failed: Operation timed out (modeled drop, not measured time)",
                world.server, row.port
            ),
        ),
    };
    let output = format!(
        "Simulated nc · numeric zero-I/O TCP probe · {} ({})\n{}\nNo application data, name lookup or host connection is performed.\n",
        observer.id, observer.address, line
    );
    Ok(Report {
        observations,
        output,
        status,
    })
}

/// Version-two observer semantics. Recorded legacy Scan events deliberately
/// keep the nmap-only interpreter, including former unsupported nc attempts.
pub fn observer_v2(
    world: &crate::service_exposure::World,
    viewpoint: usize,
    command: &str,
) -> Result<Report> {
    if command.len() > 1024
        || !command.is_ascii()
        || command.bytes().any(|b| b.is_ascii_control() && b != b'\t')
    {
        return fail(
            Class::Limit,
            "Keep the observer command to one ASCII line within 1 KiB.",
        );
    }
    match command.split_whitespace().next() {
        Some("nc") => probe(world, viewpoint, command),
        Some("nmap") => {
            let observations = world.scan_command(viewpoint, command)?;
            let output = crate::service_exposure::report(world, viewpoint, &observations)?;
            Ok(Report {
                observations,
                output,
                status: 0,
            })
        }
        _ => fail(
            Class::Unsupported,
            "Observer subset: nmap -n -Pn -sT -p PORT[,PORT...] SUPPLIED_IP or nc -n -v -z SUPPLIED_IP PORT. No real command, data session, listener or host target runs.",
        ),
    }
}
