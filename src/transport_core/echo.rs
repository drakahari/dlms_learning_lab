//! Finite IPv4 unicast Echo query/reply, not a ping command or ICMP error engine.
//! All endpoints, payloads and classifiers are supplied virtual teaching data.
use super::{
    Decision, Disposition, Error, ErrorKind, Host, MAX_DETAIL_BYTES, MAX_HOSTS, fail, identifier,
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const MAX_CASES: usize = 8;
pub const MAX_PAYLOAD: usize = 1024;
pub const MAX_CONFIG: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub destination: String,
    pub identifier: u16,
    pub sequence: u16,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u8,1024>")]
    pub payload: Vec<u8>,
}

/// Validate borrowed source data before cloning it into a reducer/report.
pub fn validate(hosts: &[Host], queries: &[Query]) -> Result<(), Error> {
    if !(2..=MAX_HOSTS).contains(&hosts.len()) || queries.len() > MAX_CASES {
        return Err(fail(
            ErrorKind::Limit,
            "Echo host/query count exceeds the bounded activity scope.",
        ));
    }
    for (i, host) in hosts.iter().enumerate() {
        if !identifier(&host.id)
            || host.address.is_unspecified()
            || host.address.is_multicast()
            || host.address == Ipv4Addr::BROADCAST
        {
            return Err(fail(
                ErrorKind::Invalid,
                "Echo requires supplied named IPv4 unicast hosts.",
            ));
        }
        if hosts[..i]
            .iter()
            .any(|h| h.id == host.id || h.address == host.address)
        {
            return Err(fail(
                ErrorKind::Invalid,
                "Echo host names and addresses must be unique.",
            ));
        }
    }
    for (i, query) in queries.iter().enumerate() {
        if !identifier(&query.id)
            || !identifier(&query.source)
            || !identifier(&query.destination)
            || !hosts.iter().any(|h| h.id == query.source)
            || !hosts.iter().any(|h| h.id == query.destination)
            || queries[..i].iter().any(|q| q.id == query.id)
        {
            return Err(fail(
                ErrorKind::Invalid,
                "Every echo query needs a unique identifier and two supplied hosts.",
            ));
        }
        if query.source == query.destination {
            return Err(fail(
                ErrorKind::Unsupported,
                "This echo activity requires two distinct virtual endpoint hosts.",
            ));
        }
        if query.payload.len() > MAX_PAYLOAD {
            return Err(fail(ErrorKind::Limit, "Echo payload exceeds 1024 bytes."));
        }
    }
    Ok(())
}

/// Standalone authoring boundary; app tasks additionally bind their host topology.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Host,6>")]
    pub hosts: Vec<Host>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Query,8>")]
    pub queries: Vec<Query>,
}
impl Config {
    pub fn decode(raw: &[u8]) -> Result<Self, Error> {
        if raw.len() > MAX_CONFIG {
            return Err(fail(ErrorKind::Limit, "Echo configuration exceeds 64 KiB."));
        }
        let config: Self = serde_json::from_slice(raw).map_err(|_| {
            fail(
                ErrorKind::Invalid,
                "Malformed or unsupported echo configuration fields.",
            )
        })?;
        if config.version != 1 {
            return Err(fail(
                ErrorKind::Unsupported,
                "Unsupported echo configuration version.",
            ));
        }
        validate(&config.hosts, &config.queries)?;
        Ok(config)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Request,
    Reply,
}
impl Kind {
    pub fn type_number(self) -> u8 {
        match self {
            Self::Request => 8,
            Self::Reply => 0,
        }
    }
    pub fn code(self) -> u8 {
        0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Packet {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub kind: Kind,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Ready,
    ReplyReady,
    Complete,
    RequestDropped,
    ReplyDropped,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Event {
    pub packet: Packet,
    pub decision: Decision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub query: String,
    pub phase: Phase,
    pub events: Vec<Event>,
    /// Present only after a matching reflected reply actually returns.
    pub received: Option<Vec<u8>>,
}
impl Report {
    pub fn round_trip(&self) -> bool {
        self.phase == Phase::Complete && self.received.is_some()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    SendRequest,
    SendReply,
}
pub struct Session {
    query: Query,
    source: Ipv4Addr,
    destination: Ipv4Addr,
    report: Report,
}
impl Session {
    pub fn new(hosts: &[Host], query: &Query) -> Result<Self, Error> {
        validate(hosts, std::slice::from_ref(query))?;
        let address = |id: &str| {
            hosts
                .iter()
                .find(|h| h.id == id)
                .map(|h| h.address)
                .ok_or_else(|| fail(ErrorKind::Invalid, "Unknown echo endpoint."))
        };
        Ok(Self {
            query: query.clone(),
            source: address(&query.source)?,
            destination: address(&query.destination)?,
            report: Report {
                query: query.id.clone(),
                phase: Phase::Ready,
                events: vec![],
                received: None,
            },
        })
    }
    pub fn report(&self) -> &Report {
        &self.report
    }
    pub fn into_report(self) -> Report {
        self.report
    }
    /// Only trusted, bounded in-process routing code supplies this callback.
    /// Invalid transitions and classifier failures leave this session unchanged.
    pub fn apply<F>(&mut self, action: Action, classify: &mut F) -> Result<(), Error>
    where
        F: FnMut(&Packet) -> Result<Decision, Error>,
    {
        let kind = match (self.report.phase, action) {
            (Phase::Ready, Action::SendRequest) => Kind::Request,
            (Phase::ReplyReady, Action::SendReply) => Kind::Reply,
            _ => {
                return Err(fail(
                    ErrorKind::Transition,
                    "Echo action is unavailable in the current phase.",
                ));
            }
        };
        let reply = kind == Kind::Reply;
        let packet = Packet {
            source: if reply { self.destination } else { self.source },
            destination: if reply { self.source } else { self.destination },
            kind,
            identifier: self.query.identifier,
            sequence: self.query.sequence,
            payload: self.query.payload.clone(),
        };
        let decision = classify(&packet).map_err(|error| {
            if error.message.len() > MAX_DETAIL_BYTES || error.message.contains('\0') {
                fail(
                    ErrorKind::Limit,
                    "Echo classifier diagnostic exceeds its text bound.",
                )
            } else {
                error
            }
        })?;
        decision.validate()?;
        let delivered = decision.disposition == Disposition::Delivered;
        self.report.phase = match (kind, delivered) {
            (Kind::Request, true) => Phase::ReplyReady,
            (Kind::Request, false) => Phase::RequestDropped,
            (Kind::Reply, true) => Phase::Complete,
            (Kind::Reply, false) => Phase::ReplyDropped,
        };
        if self.report.phase == Phase::Complete {
            self.report.received = Some(packet.payload.clone());
        }
        self.report.events.push(Event { packet, decision });
        Ok(())
    }
}
pub fn run<F>(hosts: &[Host], query: &Query, classify: &mut F) -> Result<Report, Error>
where
    F: FnMut(&Packet) -> Result<Decision, Error>,
{
    let mut session = Session::new(hosts, query)?;
    session.apply(Action::SendRequest, classify)?;
    if session.report.phase == Phase::ReplyReady {
        session.apply(Action::SendReply, classify)?;
    }
    Ok(session.into_report())
}

/// Existing stateless policy sees ICMP protocol1; this does not invent type filters.
pub fn acl_packet(packet: &Packet) -> crate::acl::Packet {
    crate::acl::Packet {
        source: packet.source,
        destination: packet.destination,
        protocol: crate::acl::PacketProtocol::Icmp {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Vec<Host>, Query) {
        (
            vec![
                Host {
                    id: "client".into(),
                    address: "192.0.2.10".parse().unwrap(),
                },
                Host {
                    id: "server".into(),
                    address: "198.51.100.20".parse().unwrap(),
                },
            ],
            Query {
                id: "echo-evidence".into(),
                source: "client".into(),
                destination: "server".into(),
                identifier: 0,
                sequence: u16::MAX,
                payload: vec![0, 10, 255, 82, 101, 97, 99, 104],
            },
        )
    }
    fn deliver(_: &Packet) -> Result<Decision, Error> {
        Ok(Decision::delivered(
            "Supplied virtual path delivered this packet.",
        ))
    }
    #[test]
    fn reply_reflects_each_query_field_and_reverses_the_ip_addresses_without_ports() {
        let (hosts, query) = fixture();
        let r = run(&hosts, &query, &mut deliver).unwrap();
        assert!(r.round_trip());
        assert_eq!(r.events.len(), 2);
        let a = &r.events[0].packet;
        let b = &r.events[1].packet;
        assert_eq!(
            (
                a.kind.type_number(),
                b.kind.type_number(),
                a.kind.code(),
                b.kind.code()
            ),
            (8, 0, 0, 0)
        );
        assert_eq!((a.source, a.destination), (b.destination, b.source));
        for p in [a, b] {
            assert_eq!(p.identifier, 0);
            assert_eq!(p.sequence, u16::MAX);
            assert_eq!(p.payload, query.payload);
            let projected = acl_packet(p);
            projected.validate().unwrap();
            assert_eq!(projected.protocol, crate::acl::PacketProtocol::Icmp {});
            let value = serde_json::to_value(p).unwrap();
            assert!(value.get("source_port").is_none() && value.get("destination_port").is_none());
        }
        assert_eq!(r.received, Some(query.payload));
    }
    #[test]
    fn request_receipt_alone_is_not_a_round_trip_and_illegal_actions_are_atomic() {
        let (hosts, query) = fixture();
        let mut s = Session::new(&hosts, &query).unwrap();
        let before = s.report().clone();
        assert!(s.apply(Action::SendReply, &mut deliver).is_err());
        assert_eq!(s.report(), &before);
        s.apply(Action::SendRequest, &mut deliver).unwrap();
        assert_eq!(s.report().phase, Phase::ReplyReady);
        assert!(!s.report().round_trip());
        assert!(s.report().received.is_none());
        let before = s.report().clone();
        assert!(s.apply(Action::SendRequest, &mut deliver).is_err());
        assert_eq!(s.report(), &before);
        s.apply(Action::SendReply, &mut deliver).unwrap();
        let before = s.report().clone();
        assert!(s.apply(Action::SendReply, &mut deliver).is_err());
        assert_eq!(s.report(), &before);
        assert_eq!(s.report().events.len(), 2);
    }
    #[test]
    fn outward_loss_never_generates_a_reply_and_return_loss_never_earns_success() {
        let (hosts, query) = fixture();
        for lost in [Kind::Request, Kind::Reply] {
            let mut count = 0;
            let r = run(&hosts, &query, &mut |p| {
                count += 1;
                Ok(if p.kind == lost {
                    Decision::dropped("Policy denied this direction.")
                } else {
                    Decision::delivered("Delivered.")
                })
            })
            .unwrap();
            assert!(!r.round_trip());
            assert!(r.received.is_none());
            assert_eq!(count, if lost == Kind::Request { 1 } else { 2 });
            assert_eq!(
                r.phase,
                if lost == Kind::Request {
                    Phase::RequestDropped
                } else {
                    Phase::ReplyDropped
                }
            );
        }
    }
    #[test]
    fn classifier_errors_and_oversized_diagnostics_do_not_commit_partial_transitions() {
        let (hosts, query) = fixture();
        for after_request in [false, true] {
            let mut s = Session::new(&hosts, &query).unwrap();
            if after_request {
                s.apply(Action::SendRequest, &mut deliver).unwrap();
            }
            let action = if after_request {
                Action::SendReply
            } else {
                Action::SendRequest
            };
            let before = s.report().clone();
            assert_eq!(
                s.apply(action, &mut |_| Err(fail(
                    ErrorKind::Classifier,
                    "Unknown virtual route."
                )))
                .unwrap_err()
                .kind,
                ErrorKind::Classifier
            );
            assert_eq!(s.report(), &before);
            for detail in ["x".repeat(MAX_DETAIL_BYTES + 1), "bad\0diagnostic".into()] {
                assert_eq!(
                    s.apply(action, &mut |_| Ok(Decision::delivered(&detail)))
                        .unwrap_err()
                        .kind,
                    ErrorKind::Limit
                );
                assert_eq!(s.report(), &before);
                assert_eq!(
                    s.apply(action, &mut |_| Err(Error::new(
                        ErrorKind::Classifier,
                        detail.clone()
                    )))
                    .unwrap_err()
                    .kind,
                    ErrorKind::Limit
                );
                assert_eq!(s.report(), &before);
            }
        }
    }
    #[test]
    fn zero_length_and_full_binary_payloads_are_valid_but_over_limit_is_rejected_before_callback() {
        let (hosts, mut query) = fixture();
        for n in [0, MAX_PAYLOAD] {
            query.payload = vec![255; n];
            let r = run(&hosts, &query, &mut deliver).unwrap();
            assert!(r.round_trip());
            assert_eq!(r.received, Some(vec![255; n]));
        }
        query.payload.push(0);
        let mut calls = 0;
        assert_eq!(
            run(&hosts, &query, &mut |_| {
                calls += 1;
                Ok(Decision::delivered("Not reached."))
            })
            .unwrap_err()
            .kind,
            ErrorKind::Limit
        );
        assert_eq!(calls, 0);
    }
    #[test]
    fn borrowed_row_validation_refuses_duplicate_unknown_and_same_host_inputs() {
        let (hosts, query) = fixture();
        assert!(validate(&hosts, &vec![query.clone(); MAX_CASES + 1]).is_err());
        assert!(validate(&hosts, &[query.clone(), query.clone()]).is_err());
        let mut bad = hosts.clone();
        bad.push(hosts[0].clone());
        assert!(validate(&bad, std::slice::from_ref(&query)).is_err());
        for name in ["missing", "../client", ""] {
            let mut bad = query.clone();
            bad.source = name.into();
            assert!(validate(&hosts, &[bad]).is_err());
        }
        let mut bad = query.clone();
        bad.destination = bad.source.clone();
        assert_eq!(
            validate(&hosts, &[bad]).unwrap_err().kind,
            ErrorKind::Unsupported
        );
        let mut bad = hosts.clone();
        bad[0].address = Ipv4Addr::BROADCAST;
        assert!(validate(&bad, &[query]).is_err());
    }
    #[test]
    fn case_limit_and_independent_sequences_are_bounded_without_restricting_same_path_queries() {
        let (hosts, query) = fixture();
        let cases = (0..MAX_CASES)
            .map(|i| {
                let mut q = query.clone();
                q.id = format!("case-{i}");
                q.sequence = i as u16;
                q
            })
            .collect::<Vec<_>>();
        validate(&hosts, &cases).unwrap();
        for q in &cases {
            let r = run(&hosts, q, &mut deliver).unwrap();
            assert_eq!(r.events[1].packet.sequence, q.sequence);
        }
        // A source-owned task can omit echo queries. It gains no ICMP coverage.
        validate(&hosts, &[]).unwrap();
    }
    #[test]
    fn bounded_json_rejects_unknown_fields_bad_versions_addresses_and_oversized_content() {
        let (hosts, query) = fixture();
        let valid = serde_json::json!({"version":1,"hosts":hosts,"queries":[query]});
        Config::decode(&serde_json::to_vec(&valid).unwrap()).unwrap();
        let mut bad = valid.clone();
        bad["queries"][0]["port"] = serde_json::json!(80);
        assert!(Config::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = valid.clone();
        bad["version"] = serde_json::json!(2);
        assert_eq!(
            Config::decode(&serde_json::to_vec(&bad).unwrap())
                .unwrap_err()
                .kind,
            ErrorKind::Unsupported
        );
        let mut bad = valid.clone();
        bad["hosts"][0]["address"] = serde_json::json!("2001:db8::1");
        assert!(Config::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert_eq!(
            Config::decode(&vec![b' '; MAX_CONFIG + 1])
                .unwrap_err()
                .kind,
            ErrorKind::Limit
        );
        assert!(Config::decode(b"not JSON").is_err());
    }
    #[test]
    fn serde_bounds_new_rows_payloads_and_identifiers_before_growing_them() {
        let (hosts, query) = fixture();
        let value = serde_json::json!({"version": 1, "hosts": hosts, "queries": [query]});
        let mut bad = value.clone();
        bad["queries"][0]["payload"] = serde_json::json!(vec![0_u8; MAX_PAYLOAD + 1]);
        assert!(Config::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = value.clone();
        bad["queries"][0]["id"] = serde_json::json!("x".repeat(65));
        assert!(Config::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = value.clone();
        bad["queries"] = serde_json::json!(vec![value["queries"][0].clone(); MAX_CASES + 1]);
        assert!(Config::decode(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}
