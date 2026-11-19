//! In-process router administration; files below are strings inside the simulated state.
use crate::{
    linux_filter::{self as filter, Chain, Class, Fault, Table, Verdict},
    linux_host as host, linux_router_path as path,
};
use serde::{Deserialize, Serialize};
pub const FILTER_FILE: &str = "/etc/lab-filter.rules";
pub const FORWARD_FILE: &str = "/etc/sysctl.d/lab-forwarding.conf";
pub const FORWARD_KEY: &str = "net.ipv4.ip_forward";
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub host: host::State,
    pub filter: Table,
    pub forwarding: bool,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub filter_file: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub forward_file: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub changed: bool,
}
fn fault(class: Class, message: &str) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn context(message: String) -> Fault {
    Fault {
        class: Class::Context,
        message,
    }
}
fn text_ok(s: &str, max: usize) -> bool {
    s.len() <= max
        && s.is_ascii()
        && !s
            .bytes()
            .any(|b| b.is_ascii_control() && !b"\t\r\n".contains(&b))
}
impl State {
    pub fn validate(&self) -> Result<(), Fault> {
        self.host.validate().map_err(context)?;
        self.filter.validate()?;
        if self.host.profile.ipv6.is_some() || self.host.configuration.ipv6.is_some() {
            return Err(fault(
                Class::Unsupported,
                "This router administration profile is IPv4-only.",
            ));
        }
        if !text_ok(&self.filter_file, 32768) || !text_ok(&self.forward_file, 4096) {
            return Err(fault(
                Class::Limit,
                "Keep virtual rules within 32 KiB and forwarding settings within 4 KiB, using ASCII text.",
            ));
        }
        if serde_json::to_vec(self)
            .map_err(|e| context(e.to_string()))?
            .len()
            > 128 * 1024
        {
            return Err(fault(
                Class::Limit,
                "The virtual router state exceeded 128 KiB.",
            ));
        }
        self.facts()?.validate()?;
        Ok(())
    }
    pub fn facts(&self) -> Result<path::Facts, Fault> {
        let mut links = vec![];
        for i in &self.host.configuration.interfaces {
            let Some(a) = &i.address else {
                return Err(fault(
                    Class::Context,
                    "This router profile keeps one supplied address on each interface; address removal is not modeled here.",
                ));
            };
            let Some(link) = self.host.profile.links.iter().find(|l| l.name == i.name) else {
                return Err(fault(
                    Class::Context,
                    "Unknown interface in virtual router configuration.",
                ));
            };
            links.push(path::Link {
                name: i.name.clone(),
                address: a.address,
                prefix: a.prefix,
                up: i.up && link.carrier,
            });
        }
        Ok(path::Facts {
            links,
            routes: self
                .host
                .configuration
                .routes
                .iter()
                .map(|r| path::Route {
                    destination: filter::Cidr {
                        network: r.destination.address,
                        prefix: r.destination.length,
                    },
                    device: r.device.clone(),
                    via: Some(r.via),
                    metric: r.metric,
                })
                .collect(),
        })
    }
    pub fn evaluate(&self, origin: &path::Origin, h: &path::Header) -> Result<path::Trace, Fault> {
        self.validate()?;
        let mut trace = path::Runtime {
            filter: self.filter.clone(),
            forwarding: self.forwarding,
        }
        .evaluate(&self.facts()?, origin, h)?;
        if let Some(route) = &trace.route {
            let selected = self.host.select(h.destination).map_err(context)?;
            // Reuse the host's route choice, never its source-address selection for transit traffic.
            if selected.device != route.device
                || selected.via != route.via
                || selected.metric != route.metric
                || selected.destination.address != route.destination.network
                || selected.destination.length != route.destination.prefix
            {
                return Err(fault(
                    Class::Context,
                    "The route adapter disagreed with the existing host selector; no trace was accepted.",
                ));
            }
        }
        if trace.gate == path::Gate::Allowed
            && trace.route.is_some()
            && !self.host.probe(h.destination).pass
        {
            trace.gate = path::Gate::PeerUnavailable;
        }
        Ok(trace)
    }
    pub fn read_file(&self, name: &str) -> Result<String, Fault> {
        match name {
            FILTER_FILE => Ok(self.filter_file.clone()),
            FORWARD_FILE => Ok(self.forward_file.clone()),
            "/proc/sys/net/ipv4/ip_forward" => Ok(format!("{}\n", u8::from(self.forwarding))),
            _ => Err(fault(
                Class::Unsupported,
                "Only the supplied virtual rules/forwarding files and ip_forward readout are available. No host file was opened.",
            )),
        }
    }
    pub fn write_file(&mut self, name: &str, text: &str) -> Result<(), Fault> {
        self.validate()?;
        let max = match name {
            FILTER_FILE => 32768,
            FORWARD_FILE => 4096,
            _ => {
                return Err(fault(
                    Class::Unsupported,
                    "Choose one of this router's two editable virtual files.",
                ));
            }
        };
        if !text_ok(text, max) {
            return Err(fault(
                Class::Limit,
                "Use bounded ASCII configuration text; the previous virtual file was retained.",
            ));
        }
        let mut next = self.clone();
        if name == FILTER_FILE {
            next.filter_file = text.into();
        } else {
            next.forward_file = text.into();
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
    pub fn command(&mut self, line: &str) -> Result<Output, Fault> {
        self.validate()?;
        let mut next = self.clone();
        let text = next.apply(line)?;
        next.validate()?;
        if text.len() > 32 * 1024 {
            return Err(fault(
                Class::Limit,
                "The simulated command output exceeded 32 KiB; previous state retained.",
            ));
        }
        let changed = next != *self;
        *self = next;
        Ok(Output { text, changed })
    }
    fn apply(&mut self, line: &str) -> Result<String, Fault> {
        let owned = filter::words(line)?;
        let v = owned.iter().map(String::as_str).collect::<Vec<_>>();
        match v.as_slice() {
            ["iptables", ..] => Ok(self.filter.command(line)?.text),
            ["sysctl", FORWARD_KEY] => {
                Ok(format!("{FORWARD_KEY} = {}\n", u8::from(self.forwarding)))
            }
            ["sysctl", "-n", FORWARD_KEY] => Ok(format!("{}\n", u8::from(self.forwarding))),
            ["sysctl", "-w", assignment] => {
                let Some((key, value)) = assignment.split_once('=') else {
                    return Err(fault(
                        Class::Syntax,
                        "Use sysctl -w net.ipv4.ip_forward=0 or=1.",
                    ));
                };
                if key != FORWARD_KEY {
                    return Err(fault(
                        Class::Unsupported,
                        "Only net.ipv4.ip_forward is modeled; other sysctl settings were not changed.",
                    ));
                }
                self.forwarding = boolean(value)?;
                Ok(format!("{FORWARD_KEY} = {}\n", u8::from(self.forwarding)))
            }
            ["sysctl", "-p", FORWARD_FILE] => {
                if let Some(value) = parse_forwarding(&self.forward_file)? {
                    self.forwarding = value;
                    Ok(format!("{FORWARD_KEY} = {}\n", u8::from(value)))
                } else {
                    Ok(String::new())
                }
            }
            ["cat", name] => self.read_file(name),
            ["iptables-save", ..] => {
                let mut table = false;
                let mut file = None;
                let mut at = 1;
                while at < v.len() {
                    match v[at] {
                        "-t" | "--table" if !table && v.get(at + 1) == Some(&"filter") => {
                            table = true;
                            at += 2;
                        }
                        "-f" | "--file"
                            if file.is_none() && v.get(at + 1) == Some(&FILTER_FILE) =>
                        {
                            file = Some(FILTER_FILE);
                            at += 2;
                        }
                        _ => {
                            return Err(fault(
                                Class::Unsupported,
                                "Use iptables-save [-t filter] [-f /etc/lab-filter.rules]. No counter dumps, module loading or other files/tables are modeled.",
                            ));
                        }
                    }
                }
                let text = save_filter(&self.filter)?;
                if file.is_some() {
                    self.filter_file = text;
                    Ok(String::new())
                } else {
                    Ok(text)
                }
            }
            ["iptables-restore", FILTER_FILE] => {
                self.filter = restore_filter(&self.filter_file)?;
                Ok(String::new())
            }
            ["iptables-restore", "--test", FILTER_FILE] => {
                restore_filter(&self.filter_file)?;
                Ok(String::new())
            }
            ["ip", ..] => {
                let allowed = matches!(
                    v.as_slice(),
                    ["ip", "link", "show"]
                        | ["ip", "link", "set", "dev", _, "up" | "down"]
                        | ["ip", "-4", "address", "show"]
                        | ["ip", "-4", "address", "show", "dev", _]
                        | ["ip", "-4", "route", ..]
                );
                if !allowed
                    || v.iter()
                        .any(|s| s.is_empty() || s.chars().any(char::is_whitespace))
                {
                    return Err(fault(
                        Class::Unsupported,
                        "Use the existing literal ip link, IPv4 address show or IPv4 route subset. Address mutation and spaced arguments are not modeled in this router profile.",
                    ));
                }
                self.host.execute(&v.join(" ")).map_err(|e| {
                    fault(
                        if e.contains("Unsupported")
                            || e.contains("outside")
                            || e.contains("limitation")
                        {
                            Class::Unsupported
                        } else {
                            Class::Context
                        },
                        &e,
                    )
                })
            }
            _ => Err(fault(
                Class::Unsupported,
                "Supported here: the IPv4 filter subset, named forwarding sysctl, virtual cat/save/restore and existing ip link/IPv4 route commands. No command ran on the host.",
            )),
        }
    }
}
fn boolean(s: &str) -> Result<bool, Fault> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(fault(
            Class::Unsupported,
            "Only the documented 0 and 1 forwarding values are modeled.",
        )),
    }
}
pub fn parse_forwarding(text: &str) -> Result<Option<bool>, Fault> {
    if !text_ok(text, 4096) {
        return Err(fault(
            Class::Limit,
            "Keep the forwarding file within 4 KiB of ASCII text.",
        ));
    }
    let mut value = None;
    for (line_no, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, v)) = line.split_once('=') else {
            return Err(context(format!(
                "Line{}: use net.ipv4.ip_forward = 0 or 1.",
                line_no + 1
            )));
        };
        if key.trim() != FORWARD_KEY {
            return Err(fault(
                Class::Unsupported,
                "Only the forwarding setting is supported in this virtual sysctl file.",
            ));
        }
        if value.is_some() {
            return Err(fault(
                Class::Unsupported,
                "This small file profile supports one explicit forwarding assignment, not repeated-setting precedence.",
            ));
        }
        value = Some(boolean(v.trim())?);
    }
    Ok(value)
}
pub fn save_filter(table: &Table) -> Result<String, Fault> {
    table.validate()?;
    let mut out = String::from("*filter\n");
    for c in [Chain::Input, Chain::Forward, Chain::Output] {
        let policy = match table.chain(c).policy {
            Verdict::Accept => "ACCEPT",
            Verdict::Drop => "DROP",
        };
        out.push_str(&format!(":{} {policy} [0:0]\n", c.name()));
    }
    for line in table.listing(None).lines().filter(|l| l.starts_with("-A ")) {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("COMMIT\n");
    if out.len() > 32768 {
        return Err(fault(
            Class::Limit,
            "The exported virtual filter exceeded 32 KiB.",
        ));
    }
    Ok(out)
}
pub fn restore_filter(text: &str) -> Result<Table, Fault> {
    if !text_ok(text, 32768) || text.lines().count() > 256 {
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
            if !table.input.rules.is_empty()
                || !table.forward.rules.is_empty()
                || !table.output.rules.is_empty()
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
                    "Custom chains are outside this filter snapshot profile.",
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
                .command(&format!("iptables -P {} {}", parts[0], parts[1]))
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
                .command(&format!("iptables {line}"))
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
