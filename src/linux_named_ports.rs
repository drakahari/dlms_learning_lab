//! Opt-in, virtual service-name resolution at filter-rule creation time.
//! Existing numeric profiles remain unchanged. No NSS or host file/command is used.
use crate::{
    linux_filter::{self as filter, Class, Fault},
    service_names as names,
    transport_core::Protocol,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub option: String,
    pub name: String,
    pub protocol: Protocol,
    pub port: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub output: filter::Output,
    pub resolutions: Vec<Resolution>,
}
fn fault(class: Class, message: impl Into<String>) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn database_fault(e: names::Fault) -> Fault {
    let class = match e.kind {
        names::FaultKind::Syntax => Class::Syntax,
        names::FaultKind::Unsupported => Class::Unsupported,
        names::FaultKind::Limit => Class::Limit,
        names::FaultKind::Invalid => Class::Context,
    };
    fault(class, format!("Virtual /etc/services: {e}"))
}
fn quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
fn encode(w: &[String]) -> String {
    w.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ")
}

// Only this adapter interprets names. The existing numeric table still validates
// the complete command and performs its scratch-copy/commit operation afterward.
fn prepare(database: &str, line: &str, program: &str) -> Result<(String, Vec<Resolution>), Fault> {
    let mut w = filter::words(line)?;
    if w.first().map(String::as_str) != Some(program) {
        return Err(fault(
            Class::Unsupported,
            format!("Use the simulated {program} filter subset; no host command ran."),
        ));
    }
    let mut start = 1;
    if matches!(w.get(start).map(String::as_str), Some("-t" | "--table")) {
        if w.get(start + 1).map(String::as_str) != Some("filter") {
            return Err(fault(
                Class::Unsupported,
                "Named service ports are supported only in the filter table.",
            ));
        }
        start += 2;
    }
    let op = w.get(start).map(String::as_str).unwrap_or("");
    if !matches!(op, "-A" | "--append" | "-I" | "--insert") {
        return Ok((encode(&w), vec![]));
    }
    let insert = matches!(op, "-I" | "--insert");
    let mut at = start + 2;
    if insert
        && w.get(at)
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
    {
        at += 1;
    }
    let mut protocol = None;
    let mut protocol_value = None;
    let mut ports = vec![];
    while at < w.len() {
        let key = w[at].as_str();
        let Some(value) = w.get(at + 1) else {
            return Err(fault(
                Class::Syntax,
                "Supply a value after each filter option.",
            ));
        };
        match key {
            "-p" | "--protocol" => {
                if protocol.is_some() {
                    return Err(fault(Class::Syntax, "Specify the protocol only once."));
                }
                protocol = Some(match value.as_str() {
                    "tcp" | "6" => Some(Protocol::Tcp),
                    "udp" | "17" => Some(Protocol::Udp),
                    _ => None,
                });
                protocol_value = Some(at + 1);
            }
            "--sport" | "--source-port" | "--dport" | "--destination-port" => ports.push(at),
            "-s" | "--source" | "-d" | "--destination" | "-i" | "--in-interface" | "-o"
            | "--out-interface" | "-j" | "--jump" | "-m" | "--match" | "--ctstate"
            | "--icmp-type" | "--icmpv6-type" => {}
            "-t" | "--table" if value == "filter" => {}
            _ => {
                return Err(fault(
                    Class::Unsupported,
                    "This option is outside the named-port filter subset; no rule changed.",
                ));
            }
        }
        // Consume the complete literal value. A quoted interface named '--dport'
        // is one operand, never a new option or a name lookup request.
        at += 2;
    }
    // The legacy IPv4 numeric parser names transports textually. Normalize the
    // two explicitly supported protocol-number aliases only in this opt-in adapter.
    if let Some(p) = protocol.flatten() {
        w[protocol_value.expect("an explicit protocol supplied its operand")] =
            names::protocol_name(p).into();
    }
    let mut parsed = None;
    let mut resolutions = vec![];
    for at in ports {
        let name = &w[at + 1];
        if !name.is_empty() && name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let proto = protocol.flatten().ok_or_else(|| {
            fault(
                Class::Unsupported,
                "Named ports require explicit -p tcp (6) or -p udp (17).",
            )
        })?;
        if name.contains('/') {
            return Err(fault(
                Class::Syntax,
                "Use just the name or alias in a port option. The separate -p option selects TCP or UDP.",
            ));
        }
        let query = names::Query::parse(&format!("{name}/{}", names::protocol_name(proto)))
            .map_err(database_fault)?;
        if parsed.is_none() {
            parsed = Some(names::Database::parse(database).map_err(database_fault)?);
        }
        let record=parsed.as_ref().and_then(|d|d.lookup(&query)).ok_or_else(||fault(Class::Context,format!("No supplied /etc/services entry matches {name}/{}. Check the saved file, spelling and protocol; existing rules were retained.",names::protocol_name(proto))))?;
        resolutions.push(Resolution {
            option: w[at].clone(),
            name: name.clone(),
            protocol: proto,
            port: record.port,
        });
        w[at + 1] = record.port.to_string();
    }
    Ok((encode(&w), resolutions))
}

/// Resolution and numeric validation both finish before the existing table commits.
pub fn v4(
    table: &mut crate::linux_state_filter::Table,
    database: &str,
    line: &str,
) -> Result<Applied, Fault> {
    let (numeric, resolutions) = prepare(database, line, "iptables")?;
    let output = table.command(&numeric)?;
    Ok(Applied {
        output,
        resolutions,
    })
}
pub fn v6(
    table: &mut crate::linux_v6_state_filter::Table,
    database: &str,
    line: &str,
) -> Result<Applied, Fault> {
    let (numeric, resolutions) = prepare(database, line, "ip6tables")?;
    let output = table.command(&numeric)?;
    Ok(Applied {
        output,
        resolutions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        linux_filter::Verdict, linux_state_filter as v4table, linux_v6_state_filter as v6table,
    };
    const DB: &str = "report 7443/tcp reports\nreport 7553/udp reports\nclock 123/udp time\n";
    #[test]
    fn protocol_specific_names_aliases_and_numeric_protocols_resolve_to_numeric_rules() {
        for (protocol, port) in [("tcp", 7443), ("6", 7443), ("udp", 7553), ("17", 7553)] {
            let mut t = v4table::Table::new(Verdict::Drop);
            let a = v4(
                &mut t,
                DB,
                &format!("iptables -A FORWARD -p {protocol} --dport reports -j ACCEPT"),
            )
            .unwrap();
            assert_eq!(a.resolutions[0].port, port);
            assert_eq!(t.base.forward.rules[0].destination_port, Some(port));
            assert!(
                t.listing(None)
                    .unwrap()
                    .contains(&format!("--dport {port}"))
            );
        }
    }
    #[test]
    fn resolving_both_ports_preserves_state_match_and_insert_position() {
        let mut t = v6table::Table::new(Verdict::Drop);
        v6(&mut t, DB, "ip6tables -A FORWARD -p tcp --dport 22 -j DROP").unwrap();
        let a=v6(&mut t,DB,"ip6tables -I FORWARD 1 -p tcp -m conntrack --ctstate ESTABLISHED --sport reports --dport report -j ACCEPT").unwrap();
        assert_eq!(a.resolutions.len(), 2);
        assert_eq!(t.base.forward.rules[0].source_port, Some(7443));
        assert_eq!(t.base.forward.rules[0].destination_port, Some(7443));
        assert_eq!(
            t.forward[0],
            Some(crate::linux_v6_state_filter::Match::Established)
        );
        assert_eq!(t.base.forward.rules[1].destination_port, Some(22));
    }
    #[test]
    fn file_edits_do_not_rewrite_already_created_rules() {
        let mut t = v6table::Table::new(Verdict::Drop);
        let line = "ip6tables -A FORWARD -p tcp --dport reports -j ACCEPT";
        v6(&mut t, DB, line).unwrap();
        let changed = DB.replace("7443/tcp", "8443/tcp");
        assert_eq!(t.base.forward.rules[0].destination_port, Some(7443));
        v6(&mut t, &changed, line).unwrap();
        assert_eq!(t.base.forward.rules[0].destination_port, Some(7443));
        assert_eq!(t.base.forward.rules[1].destination_port, Some(8443));
    }
    #[test]
    fn numeric_commands_do_not_need_a_valid_name_database() {
        let mut t = v6table::Table::new(Verdict::Drop);
        for port in [0, 65535] {
            let result = v6(
                &mut t,
                "broken file",
                &format!("ip6tables -A FORWARD -p udp --dport {port} -j DROP"),
            )
            .unwrap();
            assert!(result.resolutions.is_empty());
        }
        assert_eq!(t.base.forward.rules[0].destination_port, Some(0));
        v6(&mut t, "broken file", "ip6tables -S").unwrap();
    }
    #[test]
    fn literal_option_like_interface_does_not_become_a_port_or_state_option() {
        let mut t = v6table::Table::new(Verdict::Drop);
        v6(
            &mut t,
            DB,
            "ip6tables -A FORWARD -i '--dport' -p tcp --dport reports -j ACCEPT",
        )
        .unwrap();
        assert_eq!(t.base.forward.rules[0].ingress.as_deref(), Some("--dport"));
        assert_eq!(t.base.forward.rules[0].destination_port, Some(7443));
    }
    #[test]
    fn malformed_unknown_unsupported_or_expanding_input_retains_the_entire_table() {
        let mut t = v6table::Table::new(Verdict::Drop);
        v6(
            &mut t,
            DB,
            "ip6tables -A FORWARD -p tcp --dport reports -j ACCEPT",
        )
        .unwrap();
        for line in [
            "ip6tables -A FORWARD --dport reports -j ACCEPT",
            "ip6tables -A FORWARD -p icmpv6 --dport reports -j ACCEPT",
            "ip6tables -A FORWARD -p tcp --dport REPORTS -j ACCEPT",
            "ip6tables -A FORWARD -p tcp --dport reports/udp -j ACCEPT",
            "ip6tables -A FORWARD -p tcp --dport reports --dport 22 -j ACCEPT",
            "ip6tables -A FORWARD -p tcp --dport reports -j ACCEPT; id",
            "ip6tables -A FORWARD -p tcp --dport $(id) -j ACCEPT",
            "ip6tables -A FORWARD -p tcp --dport",
            "ip6tables -A FORWARD -p tcp -p udp --dport reports -j ACCEPT",
            "ip6tables -t nat -A FORWARD -p tcp --dport reports -j ACCEPT",
        ] {
            let before = t.clone();
            assert!(v6(&mut t, DB, line).is_err(), "{line}");
            assert_eq!(t, before);
        }
        for db in [
            "broken file",
            "report 7443/tcp reports\nother 7444/tcp reports\n",
        ] {
            let before = t.clone();
            assert!(
                v6(
                    &mut t,
                    db,
                    "ip6tables -A FORWARD -p tcp --dport reports -j ACCEPT"
                )
                .is_err()
            );
            assert_eq!(t, before);
        }
        let before = t.clone();
        assert!(v6(&mut t, DB, &"x".repeat(2049)).is_err());
        assert_eq!(t, before);
    }
    #[test]
    fn wrong_mapping_needs_a_replaced_rule_to_change_actual_header_results() {
        use crate::{
            linux_connections::Class as State,
            linux_filter::{Chain, Packet, Protocol},
        };
        let mut t = v4table::Table::new(Verdict::Drop);
        let line =
            "iptables -A FORWARD -i lan0 -o wan0 -s 10.20.0.0/16 -p tcp --dport reports -j ACCEPT";
        let broken = DB.replace("7443/tcp", "8443/tcp");
        v4(&mut t, &broken, line).unwrap();
        let mut p = Packet {
            source: "10.20.5.9".parse().unwrap(),
            destination: "203.0.113.9".parse().unwrap(),
            ingress: Some("lan0".into()),
            egress: Some("wan0".into()),
            protocol: Protocol::Tcp,
            source_port: Some(40000),
            destination_port: Some(7443),
        };
        assert_eq!(
            t.evaluate(Chain::Forward, &p, State::New).unwrap().verdict,
            Verdict::Drop
        );
        // Merely supplying the repaired file cannot mutate an installed match.
        v4(&mut t, DB, "iptables -S").unwrap();
        assert_eq!(
            t.evaluate(Chain::Forward, &p, State::New).unwrap().verdict,
            Verdict::Drop
        );
        v4(&mut t, DB, "iptables -D FORWARD 1").unwrap();
        v4(&mut t, DB, line).unwrap();
        assert_eq!(
            t.evaluate(Chain::Forward, &p, State::New).unwrap().verdict,
            Verdict::Accept
        );
        for (source, port, proto) in [
            ("10.20.5.9", 8443, Protocol::Tcp),
            ("10.21.5.9", 7443, Protocol::Tcp),
            ("10.20.5.9", 7443, Protocol::Udp),
        ] {
            p.source = source.parse().unwrap();
            p.destination_port = Some(port);
            p.protocol = proto;
            assert_eq!(
                t.evaluate(Chain::Forward, &p, State::New).unwrap().verdict,
                Verdict::Drop
            );
        }
    }
    #[test]
    fn failed_second_name_and_bad_target_do_not_commit_a_partly_resolved_rule() {
        let mut a = v4table::Table::new(Verdict::Drop);
        let mut b = v6table::Table::new(Verdict::Drop);
        for suffix in [
            "--sport reports --dport missing -j ACCEPT",
            "--sport reports --dport report -j REJECT",
        ] {
            let before_a = a.clone();
            let before_b = b.clone();
            assert!(v4(&mut a, DB, &format!("iptables -A FORWARD -p tcp {suffix}")).is_err());
            assert!(v6(&mut b, DB, &format!("ip6tables -A FORWARD -p tcp {suffix}")).is_err());
            assert_eq!(a, before_a);
            assert_eq!(b, before_b);
        }
    }
}
