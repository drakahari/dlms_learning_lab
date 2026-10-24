//! Bounded authoritative DNS data, local publication and secondary serial decisions.
//! No sockets, resolver calls, processes, clocks or host files.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};
pub const MAX_RECORDS: usize = 64;
pub const MAX_ALIASES: usize = 16;
pub const MAX_BYTES: usize = 131_072;
pub const MAX_TTL: u32 = 0x7fff_ffff;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Configuration,
    Limit,
    Serial,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
type Result<T> = std::result::Result<T, Error>;
/// Host-style ASCII names only. Every input is absolute, with an optional final dot.
pub fn canonical_name(name: &str) -> Result<String> {
    if name.len() > 254 {
        return Err(error(
            ErrorKind::Limit,
            "Domain name exceeds the supported DNS wire length",
        ));
    }
    let text = name.strip_suffix('.').unwrap_or(name);
    if text.is_empty() {
        return Err(error(
            ErrorKind::Unsupported,
            "Root-zone names are not supported in this activity",
        ));
    }
    if text.len() > 253 {
        return Err(error(
            ErrorKind::Limit,
            "Domain name exceeds 255 wire bytes",
        ));
    }
    for label in text.split('.') {
        if label.is_empty() {
            return Err(error(
                ErrorKind::Invalid,
                "A domain name cannot contain an empty label",
            ));
        }
        if label.len() > 63 {
            return Err(error(
                ErrorKind::Limit,
                "A DNS label can contain at most 63 bytes",
            ));
        }
        if !label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || label.starts_with('-')
            || label.ends_with('-')
        {
            return Err(error(
                ErrorKind::Unsupported,
                "This simulator supports ASCII host-style labels: letters, digits and internal hyphens; no wildcards, escapes or internationalized names",
            ));
        }
    }
    Ok(format!("{}.", text.to_ascii_lowercase()))
}
fn within(name: &str, origin: &str) -> bool {
    name == origin || name.ends_with(&format!(".{origin}"))
}
/// Construct a reverse-DNS question name; performs no lookup or host operation.
pub fn reverse_name(address: IpAddr) -> String {
    match address {
        IpAddr::V4(address) => {
            let [a, b, c, d] = address.octets();
            format!("{d}.{c}.{b}.{a}.in-addr.arpa.")
        }
        IpAddr::V6(address) => {
            let mut result = String::with_capacity(73);
            const HEX: &[u8; 16] = b"0123456789abcdef";
            for octet in address.octets().into_iter().rev() {
                for nibble in [octet & 15, octet >> 4] {
                    result.push(HEX[usize::from(nibble)] as char);
                    result.push('.');
                }
            }
            result.push_str("ip6.arpa.");
            result
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QueryType {
    A,
    Aaaa,
    Cname,
    Mx,
    Ns,
    Ptr,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "UPPERCASE", deny_unknown_fields)]
pub enum Data {
    A { address: Ipv4Addr },
    Aaaa { address: Ipv6Addr },
    Cname { target: String },
    Mx { preference: u16, exchange: String },
    Ns { target: String },
    Ptr { target: String },
}
impl Data {
    pub fn query_type(&self) -> QueryType {
        match self {
            Self::A { .. } => QueryType::A,
            Self::Aaaa { .. } => QueryType::Aaaa,
            Self::Cname { .. } => QueryType::Cname,
            Self::Mx { .. } => QueryType::Mx,
            Self::Ns { .. } => QueryType::Ns,
            Self::Ptr { .. } => QueryType::Ptr,
        }
    }
    fn canonical(&self) -> Result<Self> {
        Ok(match self {
            Self::Cname { target } => Self::Cname {
                target: canonical_name(target)?,
            },
            Self::Ns { target } => Self::Ns {
                target: canonical_name(target)?,
            },
            Self::Ptr { target } => Self::Ptr {
                target: canonical_name(target)?,
            },
            Self::Mx {
                preference,
                exchange,
            } => Self::Mx {
                preference: *preference,
                exchange: canonical_name(exchange)?,
            },
            _ => self.clone(),
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub owner: String,
    pub ttl: u32,
    pub data: Data,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    pub origin: String,
    pub serial: u32,
    pub records: Vec<Record>,
}
impl Zone {
    /// Bounds/syntax, independent of the intentionally repairable configuration errors.
    pub fn validate_shape(&self) -> Result<()> {
        canonical_name(&self.origin)?;
        if self.records.len() > MAX_RECORDS {
            return Err(error(
                ErrorKind::Limit,
                "A simulated zone supports at most 64 records",
            ));
        }
        for r in &self.records {
            canonical_name(&r.owner)?;
            r.data.canonical()?;
            if r.ttl > MAX_TTL {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Supported TTL values range from 0 through 2147483647 seconds",
                ));
            }
        }
        Ok(())
    }
    /// Structural and known-alias checks, not a complete operational-readiness check.
    /// In-zone NS/MX address availability is reported separately by target_readiness.
    pub fn check_configuration(&self) -> Result<()> {
        self.validate_shape()?;
        let origin = canonical_name(&self.origin)?;
        let mut owners: BTreeMap<String, Vec<Data>> = BTreeMap::new();
        for r in &self.records {
            let name = canonical_name(&r.owner)?;
            if !within(&name, &origin) {
                return Err(error(
                    ErrorKind::Configuration,
                    format!("Record {name} lies outside the {origin} zone"),
                ));
            }
            if matches!(r.data, Data::Ns { .. }) && name != origin {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Subzone delegation is not simulated; NS records belong at this zone's apex",
                ));
            }
            owners.entry(name).or_default().push(r.data.canonical()?);
        }
        if !owners
            .get(&origin)
            .is_some_and(|rows| rows.iter().any(|r| matches!(r, Data::Ns { .. })))
        {
            return Err(error(
                ErrorKind::Configuration,
                "The zone needs an NS record at its origin",
            ));
        }
        for (name, rows) in &owners {
            let aliases: BTreeSet<_> = rows
                .iter()
                .filter_map(|r| {
                    if let Data::Cname { target } = r {
                        Some(target)
                    } else {
                        None
                    }
                })
                .collect();
            if !aliases.is_empty()
                && (name == &origin || rows.iter().any(|r| !matches!(r, Data::Cname { .. })))
            {
                return Err(error(
                    ErrorKind::Configuration,
                    format!(
                        "{name}: a CNAME alias cannot also hold other data (including the apex's implicit SOA)"
                    ),
                ));
            }
            if aliases.len() > 1 {
                return Err(error(
                    ErrorKind::Configuration,
                    format!("{name}: a CNAME can point to only one name"),
                ));
            }
        }
        for rows in owners.values() {
            for data in rows {
                let target = match data {
                    Data::Ns { target } | Data::Ptr { target } => Some(target),
                    Data::Mx { exchange, .. } => Some(exchange),
                    _ => None,
                };
                if let Some(target) = target
                    && owners
                        .get(target)
                        .is_some_and(|rows| rows.iter().any(|r| matches!(r, Data::Cname { .. })))
                {
                    let kinds = if matches!(data, Data::Ptr { .. }) {
                        "PTR"
                    } else {
                        "MX and NS"
                    };
                    return Err(error(
                        ErrorKind::Configuration,
                        format!(
                            "{target}: {kinds} targets must name a canonical host, not a CNAME alias"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }
    fn canonical_records(&self) -> Result<Vec<Record>> {
        self.check_configuration()?;
        let mut groups: BTreeMap<(String, QueryType), (u32, BTreeSet<Data>)> = BTreeMap::new();
        for r in &self.records {
            let data = r.data.canonical()?;
            let group = groups
                .entry((canonical_name(&r.owner)?, data.query_type()))
                .or_insert_with(|| (r.ttl, BTreeSet::new()));
            group.0 = group.0.min(r.ttl);
            group.1.insert(data);
        }
        Ok(groups
            .into_iter()
            .flat_map(|((owner, _), (ttl, rows))| {
                rows.into_iter().map(move |data| Record {
                    owner: owner.clone(),
                    ttl,
                    data,
                })
            })
            .collect())
    }
    pub fn equivalent(&self, other: &Self) -> Result<bool> {
        Ok(
            canonical_name(&self.origin)? == canonical_name(&other.origin)?
                && self.serial == other.serial
                && self.canonical_records()? == other.canonical_records()?,
        )
    }
    /// Address evidence for NS/MX targets. External hosts remain unknown without recursion.
    pub fn target_readiness(&self) -> Result<Vec<TargetCheck>> {
        let rows = self.canonical_records()?;
        let origin = canonical_name(&self.origin)?;
        let mut targets = BTreeSet::new();
        for row in &rows {
            let target = match &row.data {
                Data::Ns { target } => target,
                Data::Mx { exchange, .. } => exchange,
                _ => continue,
            };
            targets.insert((row.owner.clone(), row.data.query_type(), target.clone()));
        }
        Ok(targets
            .into_iter()
            .map(|(owner, kind, target)| {
                let status = if !within(&target, &origin) {
                    TargetStatus::ExternalUnknown
                } else if rows.iter().any(|r| {
                    r.owner == target && matches!(r.data, Data::A { .. } | Data::Aaaa { .. })
                }) {
                    TargetStatus::Addressed
                } else {
                    TargetStatus::MissingAddress
                };
                TargetCheck {
                    owner,
                    kind,
                    target,
                    status,
                }
            })
            .collect())
    }
    /// Query one authoritative snapshot. No recursive resolution or packet exchange.
    pub fn query(&self, name: &str, kind: QueryType) -> Result<Answer> {
        let original = canonical_name(name)?;
        let origin = canonical_name(&self.origin)?;
        let rows = self.canonical_records()?;
        if !within(&original, &origin) {
            return Ok(Answer{code:Code::Refused,authoritative:false,records:vec![],note:"This name is outside the simulated authoritative zone; no recursive lookup was attempted".into()});
        }
        let mut current = original;
        let mut visited = BTreeSet::new();
        let mut answers = Vec::new();
        for depth in 0..=MAX_ALIASES {
            if !visited.insert(current.clone()) {
                return Ok(Answer {
                    code: Code::Servfail,
                    authoritative: true,
                    records: vec![],
                    note: "A CNAME loop prevents this lookup; repair the alias targets".into(),
                });
            }
            let matching: Vec<_> = rows
                .iter()
                .filter(|r| r.owner == current && r.data.query_type() == kind)
                .cloned()
                .collect();
            if !matching.is_empty() {
                answers.extend(matching);
                return Ok(Answer {
                    code: Code::Noerror,
                    authoritative: true,
                    records: answers,
                    note: "Answer from the selected in-memory authoritative copy".into(),
                });
            }
            if kind != QueryType::Cname
                && let Some(alias) = rows
                    .iter()
                    .find(|r| r.owner == current && matches!(r.data, Data::Cname { .. }))
            {
                if depth == MAX_ALIASES {
                    return Ok(Answer {
                        code: Code::Servfail,
                        authoritative: true,
                        records: vec![],
                        note: "CNAME chain exceeds the simulator's 16-link limit".into(),
                    });
                }
                answers.push(alias.clone());
                let Data::Cname { target } = &alias.data else {
                    unreachable!()
                };
                if !within(target, &origin) {
                    return Ok(Answer{code:Code::Noerror,authoritative:true,records:answers,note:"The authoritative alias points outside this zone; resolving its target would need a separate resolver, which is not simulated".into()});
                }
                current = target.clone();
                continue;
            }
            let exists = current == origin
                || rows
                    .iter()
                    .any(|r| r.owner == current || r.owner.ends_with(&format!(".{current}")));
            return Ok(Answer {
                code: if exists {
                    Code::Noerror
                } else {
                    Code::Nxdomain
                },
                authoritative: true,
                records: answers,
                note: if exists {
                    "This name exists, but has no record of the requested type"
                } else {
                    "This name does not exist in the selected zone copy"
                }
                .into(),
            });
        }
        unreachable!("bounded alias walk always returns")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Code {
    Noerror,
    Nxdomain,
    Refused,
    Servfail,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub code: Code,
    /// Authority for the original query name, including failed in-zone alias walks.
    /// This is never a success flag: callers must inspect code and requested-type records.
    pub authoritative: bool,
    pub records: Vec<Record>,
    pub note: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetStatus {
    Addressed,
    MissingAddress,
    ExternalUnknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetCheck {
    pub owner: String,
    pub kind: QueryType,
    pub target: String,
    pub status: TargetStatus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SerialOrder {
    Equal,
    Newer,
    Older,
    Undefined,
}
pub fn serial_order(candidate: u32, current: u32) -> SerialOrder {
    match candidate.wrapping_sub(current) {
        0 => SerialOrder::Equal,
        0x8000_0000 => SerialOrder::Undefined,
        1..=0x7fff_ffff => SerialOrder::Newer,
        _ => SerialOrder::Older,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transfer {
    Updated,
    UnchangedEqual,
    UnchangedOlder,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub version: u32,
    pub configured: Zone,
    pub primary: Zone,
    pub secondary: Zone,
}
impl Server {
    pub fn new(zone: Zone) -> Result<Self> {
        zone.check_configuration()?;
        Ok(Self {
            version: 1,
            configured: zone.clone(),
            primary: zone.clone(),
            secondary: zone,
        })
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(error(
                ErrorKind::Unsupported,
                "Unsupported DNS state version",
            ));
        }
        self.configured.validate_shape()?;
        self.primary.check_configuration()?;
        self.secondary.check_configuration()?;
        let origin = canonical_name(&self.configured.origin)?;
        if canonical_name(&self.primary.origin)? != origin
            || canonical_name(&self.secondary.origin)? != origin
        {
            return Err(error(
                ErrorKind::Configuration,
                "Configured and served copies must belong to the same zone",
            ));
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "DNS state exceeds the bounded import size",
            ));
        }
        let model: Self = serde_json::from_slice(bytes).map_err(|e| {
            let unknown = e.to_string().contains("unknown variant")
                || e.to_string().contains("unknown field");
            error(
                if unknown {
                    ErrorKind::Unsupported
                } else {
                    ErrorKind::Invalid
                },
                if unknown {
                    "Unsupported DNS record fields or types"
                } else {
                    "Malformed DNS state"
                },
            )
        })?;
        model.validate()?;
        Ok(model)
    }
    pub fn reload_primary(&mut self) -> Result<()> {
        self.validate()?;
        self.configured.check_configuration()?;
        // Primary local publication is distinct from a secondary's serial decision.
        self.primary = self.configured.clone();
        Ok(())
    }
    pub fn update_secondary(&mut self) -> Result<Transfer> {
        self.validate()?;
        match serial_order(self.primary.serial, self.secondary.serial) {
            SerialOrder::Newer => {
                self.secondary = self.primary.clone();
                Ok(Transfer::Updated)
            }
            SerialOrder::Equal => Ok(Transfer::UnchangedEqual),
            SerialOrder::Older => Ok(Transfer::UnchangedOlder),
            SerialOrder::Undefined => Err(error(
                ErrorKind::Serial,
                "Serial values differ by exactly 2^31; their order is undefined. Choose a supported newer serial before updating the secondary",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn record(owner: &str, data: Data) -> Record {
        Record {
            owner: owner.into(),
            ttl: 300,
            data,
        }
    }
    fn a(owner: &str, ip: &str) -> Record {
        record(
            owner,
            Data::A {
                address: ip.parse().unwrap(),
            },
        )
    }
    fn alias(owner: &str, target: &str) -> Record {
        record(
            owner,
            Data::Cname {
                target: target.into(),
            },
        )
    }
    fn zone() -> Zone {
        Zone {
            origin: "lab.test.".into(),
            serial: 10,
            records: vec![
                record(
                    "lab.test",
                    Data::Ns {
                        target: "ns.lab.test".into(),
                    },
                ),
                a("ns.lab.test", "192.0.2.53"),
                a("www.lab.test", "192.0.2.10"),
            ],
        }
    }
    #[test]
    fn address_rrsets_normalize_names_duplicates_and_ttls_without_claiming_delivery_order() {
        let mut z = zone();
        z.records.push(a("WWW.LAB.TEST.", "192.0.2.11"));
        let mut duplicate = a("www.lab.test", "192.0.2.10");
        duplicate.ttl = 20;
        z.records.push(duplicate);
        z.records.push(record(
            "www.lab.test",
            Data::Aaaa {
                address: "2001:db8::10".parse().unwrap(),
            },
        ));
        let reply = z.query("WwW.LaB.Test.", QueryType::A).unwrap();
        assert_eq!(reply.code, Code::Noerror);
        assert!(reply.authoritative);
        assert_eq!(reply.records.len(), 2);
        assert!(
            reply
                .records
                .iter()
                .all(|r| r.owner == "www.lab.test." && r.ttl == 20)
        );
        let other = z.query("www.lab.test", QueryType::Aaaa).unwrap();
        assert_eq!(other.records.len(), 1);
        assert_eq!(other.records[0].ttl, 300);
        let mut reordered = z.clone();
        reordered.records.reverse();
        assert!(z.equivalent(&reordered).unwrap());
        assert_eq!(
            reply,
            reordered.query("www.lab.test", QueryType::A).unwrap()
        );
    }
    #[test]
    fn name_absence_nodata_empty_nonterminals_and_refusal_are_distinct() {
        let mut z = zone();
        z.records.push(a("host.floor.lab.test", "192.0.2.20"));
        for name in ["floor.lab.test", "lab.test", "www.lab.test"] {
            let r = z.query(name, QueryType::Mx).unwrap();
            assert_eq!(r.code, Code::Noerror);
            assert!(r.records.is_empty());
            assert!(r.authoritative);
        }
        assert_eq!(
            z.query("missing.lab.test", QueryType::A).unwrap().code,
            Code::Nxdomain
        );
        let outside = z.query("lab.test.attacker.test", QueryType::A).unwrap();
        assert_eq!(outside.code, Code::Refused);
        assert!(!outside.authoritative);
        assert!(outside.note.contains("no recursive"));
        assert_eq!(
            z.query("foo.lab.test.", QueryType::A).unwrap().code,
            Code::Nxdomain
        );
    }
    #[test]
    fn aliases_follow_chains_but_cname_queries_stop_and_external_targets_remain_unresolved() {
        let mut z = zone();
        z.records.extend([
            alias("portal.lab.test", "stage.lab.test"),
            alias("stage.lab.test", "www.lab.test"),
        ]);
        let r = z.query("portal.lab.test", QueryType::A).unwrap();
        assert_eq!(r.code, Code::Noerror);
        assert_eq!(r.records.len(), 3);
        assert!(matches!(r.records.last().unwrap().data, Data::A { .. }));
        let direct = z.query("portal.lab.test", QueryType::Cname).unwrap();
        assert_eq!(direct.records.len(), 1);
        z.records.push(alias("away.lab.test", "outside.example"));
        let r = z.query("away.lab.test", QueryType::A).unwrap();
        assert_eq!(r.code, Code::Noerror);
        assert_eq!(r.records.len(), 1);
        assert!(r.note.contains("not simulated"));
        z.records
            .push(alias("dangling.lab.test", "absent.lab.test"));
        let r = z.query("dangling.lab.test", QueryType::A).unwrap();
        assert_eq!(r.code, Code::Nxdomain);
        assert_eq!(r.records.len(), 1);
    }
    #[test]
    fn alias_loops_and_resource_limits_fail_and_repair_is_rechecked_from_current_state() {
        let mut z = zone();
        z.records.extend([
            alias("one.lab.test", "two.lab.test"),
            alias("two.lab.test", "one.lab.test"),
        ]);
        z.check_configuration().unwrap();
        assert_eq!(
            z.query("one.lab.test", QueryType::A).unwrap().code,
            Code::Servfail
        );
        z.records.last_mut().unwrap().data = Data::Cname {
            target: "www.lab.test".into(),
        };
        assert_eq!(
            z.query("one.lab.test", QueryType::A).unwrap().code,
            Code::Noerror
        );
        z.records.last_mut().unwrap().data = Data::Cname {
            target: "one.lab.test".into(),
        };
        assert_eq!(
            z.query("one.lab.test", QueryType::A).unwrap().code,
            Code::Servfail
        );
        let mut chain = zone();
        for n in 0..17 {
            chain.records.push(alias(
                &format!("n{n}.lab.test"),
                &format!("n{}.lab.test", n + 1),
            ));
        }
        chain.records.push(a("n17.lab.test", "192.0.2.17"));
        assert_eq!(
            chain.query("n0.lab.test", QueryType::A).unwrap().code,
            Code::Servfail
        );
        assert_eq!(
            chain.query("n1.lab.test", QueryType::A).unwrap().code,
            Code::Noerror
        );
    }
    #[test]
    fn conflicting_aliases_and_alias_mx_ns_targets_are_rejected_without_mutating_served_data() {
        for invalid in [
            alias("www.lab.test", "ns.lab.test"),
            alias("lab.test", "www.lab.test"),
            record(
                "lab.test",
                Data::Mx {
                    preference: 10,
                    exchange: "portal.lab.test".into(),
                },
            ),
            record(
                "lab.test",
                Data::Ns {
                    target: "portal.lab.test".into(),
                },
            ),
        ] {
            let mut server = Server::new(zone()).unwrap();
            server
                .configured
                .records
                .push(alias("portal.lab.test", "www.lab.test"));
            server.configured.records.push(invalid);
            server.validate().unwrap();
            let before = server.clone();
            assert_eq!(
                server.reload_primary().unwrap_err().kind,
                ErrorKind::Configuration
            );
            assert_eq!(server, before);
        }
        let mut z = zone();
        z.records.extend([
            alias("x.lab.test", "www.lab.test"),
            alias("X.LAB.TEST.", "ns.lab.test"),
        ]);
        assert!(z.check_configuration().is_err());
        let mut z = zone();
        z.records.push(record(
            "child.lab.test",
            Data::Ns {
                target: "ns.lab.test".into(),
            },
        ));
        assert_eq!(
            z.check_configuration().unwrap_err().kind,
            ErrorKind::Unsupported
        );
    }
    #[test]
    fn operational_target_readiness_distinguishes_missing_addresses_from_external_unknowns() {
        let mut z = zone();
        z.records.extend([
            record(
                "lab.test",
                Data::Mx {
                    preference: 10,
                    exchange: "mail.lab.test".into(),
                },
            ),
            record(
                "lab.test",
                Data::Mx {
                    preference: 20,
                    exchange: "mail.other.test".into(),
                },
            ),
        ]);
        z.records.retain(|r| r.owner != "ns.lab.test");
        z.check_configuration().unwrap();
        let checks = z.target_readiness().unwrap();
        assert!(
            checks
                .iter()
                .any(|r| r.kind == QueryType::Ns && r.status == TargetStatus::MissingAddress)
        );
        assert!(
            checks
                .iter()
                .any(|r| r.target == "mail.lab.test." && r.status == TargetStatus::MissingAddress)
        );
        assert!(
            checks.iter().any(
                |r| r.target == "mail.other.test." && r.status == TargetStatus::ExternalUnknown
            )
        );
        z.records.extend([
            a("ns.lab.test", "192.0.2.53"),
            record(
                "mail.lab.test",
                Data::Aaaa {
                    address: "2001:db8::25".parse().unwrap(),
                },
            ),
        ]);
        let checks = z.target_readiness().unwrap();
        assert_eq!(
            checks
                .iter()
                .filter(|r| r.status == TargetStatus::Addressed)
                .count(),
            2
        );
        z.records.retain(|r| r.owner != "mail.lab.test");
        assert!(
            z.target_readiness()
                .unwrap()
                .iter()
                .any(|r| r.target == "mail.lab.test." && r.status == TargetStatus::MissingAddress)
        );
    }
    #[test]
    fn authoritative_status_is_not_success_and_errors_retain_no_successful_answer() {
        let mut z = zone();
        z.records.extend([
            alias("one.lab.test", "two.lab.test"),
            alias("two.lab.test", "one.lab.test"),
        ]);
        let failure = z.query("one.lab.test", QueryType::A).unwrap();
        assert_eq!(failure.code, Code::Servfail);
        assert!(failure.authoritative);
        assert!(failure.records.is_empty());
        let missing = z.query("missing.lab.test", QueryType::A).unwrap();
        assert!(missing.authoritative);
        assert_eq!(missing.code, Code::Nxdomain);
        assert!(missing.records.is_empty());
    }
    #[test]
    fn mx_priorities_preserve_equal_preference_alternatives_and_addresses_are_separate_records() {
        let mut z = zone();
        z.records.extend([
            record(
                "lab.test",
                Data::Mx {
                    preference: 20,
                    exchange: "backup.lab.test".into(),
                },
            ),
            record(
                "lab.test",
                Data::Mx {
                    preference: 10,
                    exchange: "mail-b.lab.test".into(),
                },
            ),
            record(
                "lab.test",
                Data::Mx {
                    preference: 10,
                    exchange: "mail-a.lab.test".into(),
                },
            ),
        ]);
        let r = z.query("lab.test", QueryType::Mx).unwrap();
        assert_eq!(r.records.len(), 3);
        assert!(matches!(r.records[0].data, Data::Mx { preference: 10, .. }));
        assert!(matches!(r.records[1].data, Data::Mx { preference: 10, .. }));
        assert!(matches!(r.records[2].data, Data::Mx { preference: 20, .. }));
        assert_eq!(
            z.query("mail-a.lab.test", QueryType::A).unwrap().code,
            Code::Nxdomain
        );
    }
    #[test]
    fn primary_reload_allows_equal_serial_but_a_secondary_needs_a_newer_serial() {
        let mut s = Server::new(zone()).unwrap();
        s.configured.records.last_mut().unwrap().data = Data::A {
            address: "192.0.2.99".parse().unwrap(),
        };
        assert!(!s.configured.equivalent(&s.primary).unwrap());
        s.reload_primary().unwrap();
        assert!(s.configured.equivalent(&s.primary).unwrap());
        assert!(!s.primary.equivalent(&s.secondary).unwrap());
        assert_eq!(s.update_secondary().unwrap(), Transfer::UnchangedEqual);
        assert!(!s.primary.equivalent(&s.secondary).unwrap());
        s.configured.serial += 1;
        s.reload_primary().unwrap();
        assert_eq!(s.update_secondary().unwrap(), Transfer::Updated);
        assert!(s.primary.equivalent(&s.secondary).unwrap());
        s.configured.serial = 9;
        s.reload_primary().unwrap();
        assert_eq!(s.update_secondary().unwrap(), Transfer::UnchangedOlder);
        assert_eq!(s.secondary.serial, 11);
    }
    #[test]
    fn serial_wrap_and_half_range_comparisons_follow_defined_sequence_space() {
        for (candidate, current, want) in [
            (0, u32::MAX, SerialOrder::Newer),
            (u32::MAX, 0, SerialOrder::Older),
            (0, 0, SerialOrder::Equal),
            (0x7fff_ffff, 0, SerialOrder::Newer),
            (0x8000_0000, 0, SerialOrder::Undefined),
            (0, 0x8000_0000, SerialOrder::Undefined),
        ] {
            assert_eq!(serial_order(candidate, current), want);
        }
        let mut z = zone();
        z.serial = u32::MAX;
        let mut s = Server::new(z).unwrap();
        s.configured.serial = 0;
        s.reload_primary().unwrap();
        assert_eq!(s.update_secondary().unwrap(), Transfer::Updated);
        s.configured.serial = 0x8000_0000;
        s.reload_primary().unwrap();
        let before = s.clone();
        assert_eq!(s.update_secondary().unwrap_err().kind, ErrorKind::Serial);
        assert_eq!(s, before);
    }
    #[test]
    fn imports_versions_unknown_types_and_maximum_structural_bounds_are_checked() {
        let s = Server::new(zone()).unwrap();
        let bytes = serde_json::to_vec(&s).unwrap();
        assert_eq!(Server::decode(&bytes).unwrap(), s);
        let mut value = serde_json::to_value(&s).unwrap();
        value["version"] = 2.into();
        assert_eq!(
            Server::decode(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
        value["version"] = 1.into();
        value["primary"]["records"][0]["data"]["type"] = "TXT".into();
        assert_eq!(
            Server::decode(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
        assert_eq!(
            Server::decode(&vec![b' '; MAX_BYTES + 1]).unwrap_err().kind,
            ErrorKind::Limit
        );
        assert_eq!(
            Server::decode(b"{broken").unwrap_err().kind,
            ErrorKind::Invalid
        );
        let mut z = zone();
        while z.records.len() < MAX_RECORDS {
            z.records
                .push(a(&format!("n{}.lab.test", z.records.len()), "192.0.2.1"));
        }
        z.check_configuration().unwrap();
        assert_eq!(
            z.query("n63.lab.test", QueryType::A).unwrap().records.len(),
            1
        );
        z.records.push(a("overflow.lab.test", "192.0.2.1"));
        assert_eq!(z.validate_shape().unwrap_err().kind, ErrorKind::Limit);
    }
    #[test]
    fn names_enforce_wire_bounds_and_honestly_reject_unimplemented_label_syntax() {
        let maximum = format!(
            "{}.{}.{}.{}.",
            "a".repeat(63),
            "b".repeat(63),
            "c".repeat(63),
            "d".repeat(61)
        );
        assert_eq!(maximum.len(), 254);
        assert!(canonical_name(&maximum).is_ok());
        assert_eq!(
            canonical_name(&format!("{}.test", "a".repeat(64)))
                .unwrap_err()
                .kind,
            ErrorKind::Limit
        );
        for bad in ["a..test", "a.test.."] {
            assert_eq!(canonical_name(bad).unwrap_err().kind, ErrorKind::Invalid);
        }
        for unsupported in [
            "*.lab.test",
            "_service.lab.test",
            "é.lab.test",
            "a\\046b.lab.test",
            "/etc/passwd",
            ".",
            "-host.test",
        ] {
            assert_eq!(
                canonical_name(unsupported).unwrap_err().kind,
                ErrorKind::Unsupported
            );
        }
        let mut z = zone();
        z.records[0].ttl = MAX_TTL;
        z.check_configuration().unwrap();
        z.records[0].ttl = MAX_TTL + 1;
        assert_eq!(z.validate_shape().unwrap_err().kind, ErrorKind::Unsupported);
    }
}
