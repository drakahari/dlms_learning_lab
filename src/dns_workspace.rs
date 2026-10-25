//! Bounded record-editor grammar, current DNS outcomes and pack-only adapters.
//! This module has no host file, process, resolver or socket access.
use crate::dns::{self, Data, QueryType, Record, Server, TargetStatus, Zone};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_TEXT: usize = 48_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Snapshot {
    Configured,
    Primary,
    Secondary,
}
impl Snapshot {
    pub fn label(self) -> &'static str {
        match self {
            Self::Configured => "Saved zone",
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
        }
    }
    pub fn zone(self, server: &Server) -> &Zone {
        match self {
            Self::Configured => &server.configured,
            Self::Primary => &server.primary,
            Self::Secondary => &server.secondary,
        }
    }
}
pub fn data_text(data: &Data) -> String {
    match data {
        Data::A { address } => address.to_string(),
        Data::Aaaa { address } => address.to_string(),
        Data::Cname { target } | Data::Ns { target } | Data::Ptr { target } => target.clone(),
        Data::Mx {
            preference,
            exchange,
        } => format!("{preference} {exchange}"),
    }
}
pub fn type_text(kind: QueryType) -> &'static str {
    match kind {
        QueryType::A => "A",
        QueryType::Aaaa => "AAAA",
        QueryType::Cname => "CNAME",
        QueryType::Mx => "MX",
        QueryType::Ns => "NS",
        QueryType::Ptr => "PTR",
    }
}
pub fn record_text(record: &Record) -> String {
    format!(
        "{} {} IN {} {}",
        record.owner,
        record.ttl,
        type_text(record.data.query_type()),
        data_text(&record.data)
    )
}
fn number(text: &str, max: u32, label: &str) -> Result<u32, String> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "{label}: enter a whole decimal number from 0 through {max}"
        ));
    }
    text.parse::<u32>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| format!("{label}: enter a whole decimal number from 0 through {max}"))
}
fn absolute(text: &str) -> Result<String, String> {
    if !text.ends_with('.') {
        return Err("Use a full domain name ending in a dot; relative names and origin expansion are not supported in this editor".into());
    }
    dns::canonical_name(text).map_err(|e| e.message)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub version: u32,
    pub serial: String,
    pub text: String,
    pub query_name: String,
    pub query_type: QueryType,
    pub query_copy: Snapshot,
    pub output: String,
    pub notice: Option<String>,
}
impl Draft {
    pub fn new(server: &Server) -> Self {
        Self {
            version: 1,
            serial: server.configured.serial.to_string(),
            text: server
                .configured
                .records
                .iter()
                .map(|r| {
                    let mut r = r.clone();
                    r.owner = dns::canonical_name(&r.owner).unwrap_or(r.owner);
                    match &mut r.data {
                        Data::Cname { target } | Data::Ns { target } | Data::Ptr { target } => {
                            *target = dns::canonical_name(target).unwrap_or(target.clone())
                        }
                        Data::Mx { exchange, .. } => {
                            *exchange = dns::canonical_name(exchange).unwrap_or(exchange.clone())
                        }
                        _ => {}
                    }
                    record_text(&r)
                })
                .collect::<Vec<_>>()
                .join("\n"),
            query_name: server.configured.origin.clone(),
            query_type: QueryType::A,
            query_copy: Snapshot::Primary,
            output: String::new(),
            notice: None,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.serial.len() <= 40
            && self.text.len() <= MAX_TEXT
            && self.query_name.len() <= 1024
            && self.query_copy != Snapshot::Configured
            && self.output.len() <= 40_000
            && self.notice.as_ref().is_none_or(|s| s.len() <= 2048)
    }
    pub fn parse(&self, origin: &str) -> Result<Zone, String> {
        if !self.valid() {
            return Err("Record editor draft exceeds the supported bounds".into());
        }
        let serial = number(&self.serial, u32::MAX, "SOA serial")?;
        let mut records = Vec::new();
        for (line_number, line) in self.text.lines().enumerate() {
            let line = line.split(';').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let parsed=(|| {
                let p:Vec<_>=line.split_whitespace().collect();
                if p.len()<5 {return Err("Use OWNER. TTL IN TYPE DATA, one complete record per line".into());}
                let owner=absolute(p[0])?;
                let ttl=number(p[1],dns::MAX_TTL,"TTL seconds")?;
                if !p[2].eq_ignore_ascii_case("IN") {return Err("Only Internet-class (IN) records are supported".into());}
                let data=match p[3].to_ascii_uppercase().as_str() {
                    "A" if p.len()==5=>Data::A{address:p[4].parse().map_err(|_|"A needs a valid IPv4 address")?},
                    "AAAA" if p.len()==5=>Data::Aaaa{address:p[4].parse().map_err(|_|"AAAA needs a valid IPv6 address")?},
                    "CNAME" if p.len()==5=>Data::Cname{target:absolute(p[4])?},
                    "NS" if p.len()==5=>Data::Ns{target:absolute(p[4])?},
                    "PTR" if p.len()==5=>Data::Ptr{target:absolute(p[4])?},
                    "MX" if p.len()==6=>Data::Mx{preference:number(p[4],u16::MAX as u32,"MX preference")? as u16,exchange:absolute(p[5])?},
                    "A"|"AAAA"|"CNAME"|"NS"|"MX"|"PTR"=>return Err("Wrong number of record fields; MX needs preference and full exchange name, other supported types need one data value".into()),
                    _=>return Err("Unsupported record type or syntax: this editor supports A, AAAA, CNAME, MX, PTR and apex NS only; no directives, includes or SOA lines".into()),
                }; Ok(Record{owner,ttl,data})
            })().map_err(|e:String|format!("Line {}: {e}",line_number+1))?;
            records.push(parsed);
            if records.len() > dns::MAX_RECORDS {
                return Err("A zone supports at most 64 records".into());
            }
        }
        let zone = Zone {
            origin: origin.into(),
            serial,
            records,
        };
        zone.validate_shape().map_err(|e| e.message)?;
        Ok(zone)
    }
    /// Parse first, then atomically change only the saved zone. Publication remains explicit.
    pub fn save(&self, server: &mut Server) -> Result<(), String> {
        server.validate().map_err(|e| e.message)?;
        let zone = self.parse(&server.configured.origin)?;
        server.configured = zone;
        Ok(())
    }
    pub fn matches(&self, server: &Server) -> bool {
        self.parse(&server.configured.origin).ok().is_some_and(|z| {
            z == server.configured || z.equivalent(&server.configured).unwrap_or(false)
        })
    }
    pub fn lookup(&mut self, server: &Server) -> Result<(), String> {
        if !self.valid() {
            return Err("Lookup draft exceeds supported bounds".into());
        }
        let answer = self
            .query_copy
            .zone(server)
            .query(&self.query_name, self.query_type)
            .map_err(|e| e.message)?;
        self.output = format!(
            "{} · {} {} · {:?}\n{}",
            self.query_copy.label(),
            self.query_name,
            type_text(self.query_type),
            answer.code,
            answer.note
        );
        for record in &answer.records {
            self.output.push('\n');
            self.output.push_str(&record_text(record));
        }
        Ok(())
    }
}
fn canonical_data(data: &[Data]) -> Result<BTreeSet<Data>, String> {
    let zone = Zone {
        origin: "validation.test.".into(),
        serial: 1,
        records: data
            .iter()
            .cloned()
            .map(|data| Record {
                owner: "validation.test.".into(),
                ttl: 0,
                data,
            })
            .collect(),
    };
    zone.validate_shape().map_err(|e| e.message)?;
    data.iter()
        .map(|data| {
            Ok(match data {
                Data::Cname { target } => Data::Cname {
                    target: dns::canonical_name(target).map_err(|e| e.message)?,
                },
                Data::Ns { target } => Data::Ns {
                    target: dns::canonical_name(target).map_err(|e| e.message)?,
                },
                Data::Ptr { target } => Data::Ptr {
                    target: dns::canonical_name(target).map_err(|e| e.message)?,
                },
                Data::Mx {
                    preference,
                    exchange,
                } => Data::Mx {
                    preference: *preference,
                    exchange: dns::canonical_name(exchange).map_err(|e| e.message)?,
                },
                _ => data.clone(),
            })
        })
        .collect()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Records {
        copy: Snapshot,
        owner: String,
        record_type: QueryType,
        data: Vec<Data>,
    },
    Resolves {
        copy: Snapshot,
        name: String,
        record_type: QueryType,
        data: Vec<Data>,
    },
    /// A completed authoritative query with no data of the requested type.
    /// This says nothing about whether another copy is synchronized.
    NoAnswer {
        copy: Snapshot,
        name: String,
        record_type: QueryType,
    },
    TargetAddress {
        copy: Snapshot,
        owner: String,
        record_type: QueryType,
        target: String,
    },
    AliasTo {
        copy: Snapshot,
        owner: String,
        record_type: QueryType,
        target: String,
    },
    Published {
        secondary: bool,
    },
}
impl Requirement {
    pub fn valid_for(&self, server: &Server) -> bool {
        let inside = |name: &str| {
            dns::canonical_name(name)
                .ok()
                .zip(dns::canonical_name(&server.configured.origin).ok())
                .is_some_and(|(n, o)| n == o || n.ends_with(&format!(".{o}")))
        };
        match self {
            Self::NoAnswer { copy, name, .. } => *copy != Snapshot::Configured && inside(name),
            Self::AliasTo {
                copy,
                owner,
                record_type,
                target,
            } => {
                *copy != Snapshot::Configured
                    && matches!(record_type, QueryType::A | QueryType::Aaaa)
                    && inside(owner)
                    && inside(target)
                    && dns::canonical_name(owner).ok() != dns::canonical_name(target).ok()
            }
            Self::Records {
                owner,
                record_type,
                data,
                ..
            }
            | Self::Resolves {
                name: owner,
                record_type,
                data,
                ..
            } => {
                inside(owner)
                    && !data.is_empty()
                    && data.len() <= 8
                    && data.iter().all(|r| r.query_type() == *record_type)
                    && canonical_data(data).is_ok()
                    && !matches!(
                        self,
                        Self::Resolves {
                            copy: Snapshot::Configured,
                            ..
                        }
                    )
            }
            Self::TargetAddress {
                owner,
                record_type,
                target,
                ..
            } => {
                inside(owner)
                    && matches!(record_type, QueryType::Ns | QueryType::Mx)
                    && inside(target)
            }
            Self::Published { .. } => true,
        }
    }
    pub fn check(&self, server: &Server) -> (bool, String) {
        if server.validate().is_err() || !self.valid_for(server) {
            return (
                false,
                "The DNS state or requested outcome is invalid".into(),
            );
        }
        match self {
            Self::NoAnswer {
                copy,
                name,
                record_type,
            } => {
                let zone = copy.zone(server);
                let origin = dns::canonical_name(&zone.origin).unwrap();
                // Query exactly once. Only CNAME targets are walked names;
                // an external PTR target is returned data, not recursion.
                let (pass, reason) = match zone.query(name, *record_type) {
                    Err(_) => (false, "could not complete a valid authoritative query"),
                    Ok(answer)
                        if !answer.authoritative
                            || !matches!(answer.code, dns::Code::Noerror | dns::Code::Nxdomain) =>
                    {
                        (
                            false,
                            "needs a completed authoritative answer, not a refusal or query failure",
                        )
                    }
                    Ok(answer) => {
                        let leaves_zone = answer.records.iter().any(|record| {
                            if let Data::Cname { target } = &record.data {
                                dns::canonical_name(target).is_ok_and(|target| {
                                    target != origin && !target.ends_with(&format!(".{origin}"))
                                })
                            } else {
                                false
                            }
                        });
                        if leaves_zone {
                            (
                                false,
                                "cannot prove absence through an alias outside this zone",
                            )
                        } else if answer
                            .records
                            .iter()
                            .any(|r| r.data.query_type() == *record_type)
                        {
                            (
                                false,
                                "still returns data of this type; inspect the served copy and any alias path",
                            )
                        } else {
                            (true, "authoritatively returns no data of this type")
                        }
                    }
                };
                (
                    pass,
                    format!(
                        "{}: {} {} {reason}",
                        copy.label(),
                        name,
                        type_text(*record_type)
                    ),
                )
            }
            Self::AliasTo {
                copy,
                owner,
                record_type,
                target,
            } => {
                let wanted_owner = dns::canonical_name(owner).unwrap();
                let wanted_target = dns::canonical_name(target).unwrap();
                let pass = copy.zone(server).query(owner, *record_type).is_ok_and(|a| {
                    a.code == dns::Code::Noerror
                        && a.records.first().is_some_and(|r| {
                            r.owner == wanted_owner && matches!(r.data, Data::Cname { .. })
                        })
                        && a.records.iter().any(|r| {
                            r.owner == wanted_target && r.data.query_type() == *record_type
                        })
                });
                (
                    pass,
                    format!(
                        "{}: alias {} {}",
                        copy.label(),
                        owner,
                        if pass {
                            format!("reaches the canonical host {target}")
                        } else {
                            format!(
                                "must follow a working CNAME path to {target}; a matching address on an unrelated host is not enough"
                            )
                        }
                    ),
                )
            }
            Self::Published { secondary } => {
                let primary = server
                    .configured
                    .equivalent(&server.primary)
                    .unwrap_or(false);
                let synced = server
                    .primary
                    .equivalent(&server.secondary)
                    .unwrap_or(false);
                (primary&&(!secondary||synced),if !primary {"Saved changes are not published on the primary. Reload the primary after correcting its configuration."}else if *secondary&&!synced {"The secondary still serves a different copy. Check the serial and update the secondary."}else if *secondary {"Saved, primary and secondary copies agree"}else {"The primary serves the saved zone"}.into())
            }
            Self::Records {
                copy,
                owner,
                record_type,
                data,
            } => {
                let zone = copy.zone(server);
                let owner = dns::canonical_name(owner).unwrap();
                let expected = canonical_data(data).unwrap();
                let actual = canonical_data(
                    &zone
                        .records
                        .iter()
                        .filter(|r| {
                            dns::canonical_name(&r.owner).ok().as_deref() == Some(&owner)
                                && r.data.query_type() == *record_type
                        })
                        .map(|r| r.data.clone())
                        .collect::<Vec<_>>(),
                );
                let pass =
                    zone.check_configuration().is_ok() && actual.is_ok_and(|a| a == expected);
                (
                    pass,
                    format!(
                        "{}: {} {} {}",
                        copy.label(),
                        owner,
                        type_text(*record_type),
                        if pass {
                            "records match the task"
                        } else {
                            "records differ from the task; inspect this name/type without deleting unrelated records"
                        }
                    ),
                )
            }
            Self::Resolves {
                copy,
                name,
                record_type,
                data,
            } => {
                let expected = canonical_data(data).unwrap();
                let answer = copy.zone(server).query(name, *record_type);
                let pass = answer.as_ref().is_ok_and(|a| {
                    a.code == dns::Code::Noerror
                        && canonical_data(
                            &a.records
                                .iter()
                                .filter(|r| r.data.query_type() == *record_type)
                                .map(|r| r.data.clone())
                                .collect::<Vec<_>>(),
                        )
                        .is_ok_and(|a| a == expected)
                });
                let reason = if pass {
                    "returns the required answer".into()
                } else {
                    answer.map_or_else(
                        |e| e.message,
                        |a| format!("does not return the required answer: {}", a.note),
                    )
                };
                (
                    pass,
                    format!(
                        "{}: {} {} {reason}",
                        copy.label(),
                        name,
                        type_text(*record_type)
                    ),
                )
            }
            Self::TargetAddress {
                copy,
                owner,
                record_type,
                target,
            } => {
                let wanted_owner = dns::canonical_name(owner).unwrap();
                let wanted_target = dns::canonical_name(target).unwrap();
                let pass = copy.zone(server).target_readiness().is_ok_and(|rows| {
                    rows.iter().any(|r| {
                        r.owner == wanted_owner
                            && r.kind == *record_type
                            && r.target == wanted_target
                            && r.status == TargetStatus::Addressed
                    })
                });
                (
                    pass,
                    format!(
                        "{}: {} target {} {}",
                        copy.label(),
                        type_text(*record_type),
                        target,
                        if pass {
                            "has an in-zone address record"
                        } else {
                            "needs the intended NS/MX record and its own A or AAAA address"
                        }
                    ),
                )
            }
        }
    }
}
pub fn valid_requirements(requirements: &[Requirement], server: &Server) -> bool {
    server.validate().is_ok()
        && !requirements.is_empty()
        && requirements.len() <= 20
        && requirements.iter().all(|r| r.valid_for(server))
        && requirements
            .iter()
            .any(|r| matches!(r, Requirement::Resolves { .. }))
        && requirements
            .iter()
            .any(|r| matches!(r, Requirement::Published { .. }))
}
pub fn same_problem(a: &Server, b: &Server) -> bool {
    a.version == b.version
        && dns::canonical_name(&a.configured.origin).ok()
            == dns::canonical_name(&b.configured.origin).ok()
        && a.validate().is_ok()
        && b.validate().is_ok()
}
/// Declarative reference adapter, not a learner Linux command or a shell.
pub fn adapter(server: &mut Server, input: &str) -> Result<String, String> {
    if input.len() > MAX_TEXT + 64 {
        return Err("DNS reference action exceeds the supported bounds".into());
    }
    if let Some(json) = input.strip_prefix("zone ") {
        let zone: Zone = serde_json::from_str(json).map_err(|_| "Malformed DNS reference zone")?;
        zone.validate_shape().map_err(|e| e.message)?;
        if dns::canonical_name(&zone.origin).map_err(|e| e.message)?
            != dns::canonical_name(&server.configured.origin).map_err(|e| e.message)?
        {
            return Err("Reference zone must retain the supplied origin".into());
        }
        server.configured = zone;
        return Ok("Saved simulated zone".into());
    }
    match input {"reload-primary"=>{server.reload_primary().map_err(|e|e.message)?;Ok("Primary published".into())},"update-secondary"=>server.update_secondary().map(|s|format!("{s:?}")).map_err(|e|e.message),_=>Err("Unsupported DNS reference action; use the native editor, publication and lookup controls".into())}
}
