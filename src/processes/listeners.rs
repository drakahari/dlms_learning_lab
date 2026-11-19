//! Bounded supplied IPv4 socket metadata. No socket/syscall/host execution.
use super::{Output, Phase, Table};
use serde::{Deserialize, Deserializer, Serialize, de};
use std::{collections::BTreeSet, fmt, net::Ipv4Addr};

pub const MAX_LISTENERS: usize = 64;
const MAX_FD: u32 = 1_048_575;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Tcp,
    Udp,
}
impl Protocol {
    fn name(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
        }
    }
    fn state(self) -> &'static str {
        match self {
            Self::Tcp => "LISTEN",
            Self::Udp => "UNCONN",
        }
    }
}
/// Immutable source facts, not a learner-created socket. TCP backlog is supplied;
/// this slice fixes queued inbound data to zero. UDP is unconnected with sendQ0.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub protocol: Protocol,
    #[serde(deserialize_with = "address")]
    pub address: Ipv4Addr,
    pub port: u16,
    pub pid: u32,
    pub fd: u32,
    pub backlog: u16,
}
impl Listener {
    fn fields_valid(&self) -> bool {
        self.port != 0
            && super::bounded_id(self.pid)
            && self.fd <= MAX_FD
            && !self.address.is_multicast()
            && !self.address.is_broadcast()
            && (self.protocol != Protocol::Udp || self.backlog == 0)
    }
    fn endpoint(&self) -> String {
        format!(
            "{} {}:{} (process {}, FD{})",
            self.protocol.name(),
            self.address,
            self.port,
            self.pid,
            self.fd
        )
    }
}
fn address<'de, D: Deserializer<'de>>(d: D) -> Result<Ipv4Addr, D::Error> {
    struct Address;
    impl de::Visitor<'_> for Address {
        type Value = Ipv4Addr;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a canonical dotted IPv4 address of at most15bytes")
        }
        fn visit_str<E: de::Error>(self, s: &str) -> Result<Ipv4Addr, E> {
            if s.len() > 15 {
                return Err(E::custom("Socket IPv4 address exceeds15bytes"));
            }
            let ip = s
                .parse::<Ipv4Addr>()
                .map_err(|_| E::custom("Invalid socket IPv4 address"))?;
            if ip.to_string() != s {
                return Err(E::custom("Use canonical dotted IPv4 socket address"));
            }
            Ok(ip)
        }
    }
    d.deserialize_str(Address)
}
fn roster_valid(rows: &[Listener]) -> bool {
    if rows.len() > MAX_LISTENERS {
        return false;
    }
    let mut owners = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        if !row.fields_valid() || !owners.insert((row.pid, row.fd)) {
            return false;
        }
        // No shared/reuse sockets in this supplied model. A wildcard occupies
        // this protocol/port on every supplied local IPv4 address.
        if rows[..index].iter().any(|other| {
            row.protocol == other.protocol
                && row.port == other.port
                && (row.address == other.address
                    || row.address.is_unspecified()
                    || other.address.is_unspecified())
        }) {
            return false;
        }
    }
    true
}
pub(super) fn valid(table: &Table) -> bool {
    roster_valid(&table.listeners)
        && table
            .listeners
            .iter()
            .all(|row| table.processes.contains_key(&row.pid))
}
pub(super) fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Listener>, D::Error> {
    struct Rows;
    impl<'de> de::Visitor<'de> for Rows {
        type Value = Vec<Listener>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most64 supplied distinct bounded IPv4 socket rows")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut rows = Vec::new();
            while rows.len() < MAX_LISTENERS {
                let Some(row) = seq.next_element::<Listener>()? else {
                    return Ok(rows);
                };
                rows.push(row);
                if !roster_valid(&rows) {
                    return Err(de::Error::custom(
                        "Invalid socket fields, duplicate PID/FD or conflicting bind tuple",
                    ));
                }
            }
            struct Excess;
            impl<'de> Deserialize<'de> for Excess {
                fn deserialize<D: Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
                    Err(de::Error::custom("Socket row count exceeds64"))
                }
            }
            if seq.next_element::<Excess>()?.is_some() {
                unreachable!("Excess always refuses");
            }
            Ok(rows)
        }
    }
    d.deserialize_seq(Rows)
}
impl Table {
    /// Read-only projection of supplied endpoints and current owner phases.
    pub(super) fn ss(&self, args: &[&str]) -> Result<Output, String> {
        let mut flags = BTreeSet::new();
        for arg in args {
            let Some(chars) = arg.strip_prefix('-').filter(|s| !s.is_empty()) else {
                return Err(super::unsupported());
            };
            for flag in chars.chars() {
                if !matches!(flag, 'l' | 'n' | 't' | 'u' | 'p') {
                    return Err(super::unsupported());
                }
                flags.insert(flag);
            }
        }
        if !flags.contains(&'l')
            || !flags.contains(&'n')
            || !(flags.contains(&'t') || flags.contains(&'u'))
        {
            return Err(super::unsupported());
        }
        let combined = flags.contains(&'t') && flags.contains(&'u');
        let show_process = flags.contains(&'p');
        let mut header = Vec::new();
        if combined {
            header.push("Netid".to_owned());
        }
        header.extend(
            [
                "State",
                "Recv-Q",
                "Send-Q",
                "Local Address:Port",
                "Peer Address:Port",
            ]
            .map(str::to_owned),
        );
        if show_process {
            header.push("Process".to_owned());
        }
        let mut cells = vec![header];
        let mut rows: Vec<_> = self
            .listeners
            .iter()
            .filter(|row| {
                self.live(row.pid)
                    && flags.contains(&match row.protocol {
                        Protocol::Tcp => 't',
                        Protocol::Udp => 'u',
                    })
            })
            .collect();
        rows.sort_by_key(|row| (row.protocol, row.address, row.port, row.pid, row.fd));
        for row in rows {
            let mut values = Vec::new();
            if combined {
                values.push(row.protocol.name().to_owned());
            }
            let send = if row.protocol == Protocol::Tcp {
                row.backlog
            } else {
                0
            };
            values.extend([
                row.protocol.state().to_owned(),
                "0".into(),
                send.to_string(),
                format!("{}:{}", row.address, row.port),
                "0.0.0.0:*".into(),
            ]);
            let owner = &self.processes[&row.pid];
            if show_process {
                values.push(if self.user.uid == 0 || self.user.uid == owner.owner.uid {
                    format!("users:((\"{}\",pid={},fd={}))", owner.comm, row.pid, row.fd)
                } else {
                    String::new()
                });
            }
            cells.push(values);
        }
        // All admitted field values are ASCII. Calculate widths from the real
        // headers and complete rows once; the native terminal paints monospace.
        let widths: Vec<_> = (0..cells[0].len())
            .map(|column| cells.iter().map(|row| row[column].len()).max().unwrap_or(0))
            .collect();
        let mut text = String::new();
        for row in cells {
            let mut line = String::new();
            for (column, value) in row.iter().enumerate() {
                if column != 0 {
                    line.push(' ');
                }
                if column + 1 == row.len() {
                    line.push_str(value);
                } else {
                    line.push_str(&format!("{value:<width$}", width = widths[column]));
                }
            }
            text.push_str(line.trim_end());
            text.push('\n');
        }
        Ok(Output::success(text))
    }
}
pub(super) fn check(table: &Table, endpoint: &Listener, present: bool) -> (bool, String) {
    let description = endpoint.endpoint();
    if !table.listeners.contains(endpoint) {
        return (
            false,
            format!(
                "Known socket metadata for {description} is missing or changed; restore the supplied attempt. This is not a closed endpoint."
            ),
        );
    }
    let phase = &table.processes[&endpoint.pid].phase;
    let bound = !matches!(phase, Phase::Exited);
    if bound == present {
        return (
            true,
            if bound {
                format!("{description} remains bound")
            } else {
                format!("{description} is closed because its owner has ended")
            },
        );
    }
    match phase {
        Phase::Stopped {
            pending_termination,
            ..
        } => (
            false,
            format!(
                "{description} is still bound: its owner is paused{}. Pausing does not close a socket; inspect the owner's process state before retrying.",
                if *pending_termination {
                    " with a termination request waiting"
                } else {
                    ""
                }
            ),
        ),
        Phase::Active { .. } => (
            false,
            format!(
                "{description} is still bound by its active owner; identify the intended process before ending it."
            ),
        ),
        Phase::Exited => (
            false,
            format!(
                "{description} is closed because its owner has ended. This endpoint must remain available; Reset this attempt to restore the required process."
            ),
        ),
    }
}

pub(super) fn preservation_feedback(table: &Table, pid: u32, expected: &super::Process) -> String {
    let Some(current) = table.observed_process(pid) else {
        return ": the required owner is missing; restore the supplied attempt.".into();
    };
    if current.phase != expected.phase {
        match current.phase {
            Phase::Exited => return ": its owner has ended. Reset this attempt to restore the required process; ending it cannot be undone with CONT.".into(),
            Phase::Stopped { pending_termination: true, .. } => return ": its owner is paused with a termination request waiting. CONT would end it; Reset this attempt to keep the required process available.".into(),
            Phase::Stopped { pending_termination: false, .. } => return ": its owner is paused. A bound socket alone does not preserve its active state; restore the required process state.".into(),
            _ => return ": the required owner has a different process state; inspect it with ps.".into(),
        }
    }
    if current.nice != expected.nice {
        return format!(
            ": its nice value is {}, expected {}. Preserve its priority as well as its endpoint; an ordinary user may need Reset to undo a priority reduction.",
            current.nice, expected.nice
        );
    }
    ": the required owner's parent or identity changed; inspect ps and restore the supplied context.".into()
}
