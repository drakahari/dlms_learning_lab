//! Bounded single-egress teaching model. Capability alone is not curriculum coverage.
//! No host clock, packet, file, subprocess or device API. Equal-size packets only.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, Visitor},
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
pub const MAX_JSON_BYTES: usize = 65536;
pub const MAX_FLOWS: usize = 4;
pub const MAX_PACKETS: usize = 128;
pub const MAX_ARRIVAL: u16 = 63;
pub const MAX_BUFFER: u16 = 64;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub flow: String,
    pub arrival: u16,
    pub dscp: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    #[serde(deserialize_with = "flows")]
    pub flows: BTreeSet<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Packet,MAX_PACKETS>")]
    pub packets: Vec<Packet>,
    pub buffer_budget: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mark {
    Keep,
    Set(u8),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scheduler {
    Fifo,
    StrictPriority,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(deserialize_with = "marks")]
    pub marks: BTreeMap<String, Mark>,
    pub scheduler: Scheduler,
    #[serde(deserialize_with = "priorities")]
    pub priority_dscp: BTreeSet<u8>,
    pub high_buffer: u16,
    pub standard_buffer: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Queue {
    Shared,
    High,
    Standard,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Delivery {
    Delivered { departure: u16, waiting_slots: u16 },
    TailDrop { at: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PacketTrace {
    pub id: String,
    pub flow: String,
    pub arrival: u16,
    pub incoming_dscp: u8,
    pub outgoing_dscp: u8,
    pub queue: Queue,
    pub delivery: Delivery,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Trace {
    pub packets: Vec<PacketTrace>,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
impl Fixture {
    pub fn validate(&self) -> Result<(), String> {
        if self.flows.is_empty()
            || self.flows.len() > MAX_FLOWS
            || self.flows.iter().any(|s| !identifier(s))
        {
            return Err("Supply one to four bounded flow identifiers".into());
        }
        if self.packets.is_empty()
            || self.packets.len() > MAX_PACKETS
            || self.buffer_budget == 0
            || self.buffer_budget > MAX_BUFFER
        {
            return Err("Packet or buffer budget limit".into());
        }
        let mut ids = BTreeSet::new();
        let mut previous = 0;
        for p in &self.packets {
            if !identifier(&p.id)
                || !ids.insert(p.id.as_str())
                || !self.flows.contains(&p.flow)
                || p.dscp > 63
                || p.arrival > MAX_ARRIVAL
                || p.arrival < previous
            {
                return Err("Invalid packet ID, flow, DSCP or arrival order".into());
            }
            previous = p.arrival;
        }
        Ok(())
    }
}
impl Policy {
    pub fn validate(&self, f: &Fixture) -> Result<(), String> {
        f.validate()?;
        if self.marks.len() != f.flows.len()
            || self
                .marks
                .iter()
                .any(|(flow, m)| !f.flows.contains(flow) || matches!(m,Mark::Set(n)if *n>63))
        {
            return Err("Each supplied flow needs one valid marking policy; DSCP is 0–63".into());
        }
        if self.priority_dscp.len() > 4
            || self.priority_dscp.iter().any(|n| *n > 63)
            || self.standard_buffer == 0
            || self.standard_buffer > MAX_BUFFER
            || self.high_buffer > MAX_BUFFER
        {
            return Err("Priority mapping or waiting-buffer limit".into());
        }
        if self
            .standard_buffer
            .checked_add(self.high_buffer)
            .is_none_or(|n| n > f.buffer_budget)
        {
            return Err("Waiting queues exceed the supplied shared buffer budget".into());
        }
        match self.scheduler {
            Scheduler::Fifo if self.high_buffer != 0 || !self.priority_dscp.is_empty() => {
                return Err(
                    "FIFO uses one shared waiting queue; no priority mapping or reserved queue"
                        .into(),
                );
            }
            Scheduler::StrictPriority if self.high_buffer == 0 || self.priority_dscp.is_empty() => {
                return Err(
                    "Priority scheduling needs a nonempty explicit DSCP mapping and a high queue"
                        .into(),
                );
            }
            _ => {}
        }
        Ok(())
    }
}
/// Every virtual slot: admit arrivals in supplied order; tail-drop a full waiting
/// queue; transmit one equal-size packet, high before standard when configured.
/// A selected packet departs at slot+1. Waiting excludes that serialization slot.
pub fn replay(f: &Fixture, p: &Policy) -> Result<Trace, String> {
    p.validate(f)?;
    let mut high = VecDeque::<usize>::new();
    let mut ordinary = VecDeque::<usize>::new();
    let mut records: Vec<Option<PacketTrace>> = vec![None; f.packets.len()];
    let mut outgoing = vec![0u8; f.packets.len()];
    let mut queues = vec![Queue::Shared; f.packets.len()];
    let mut next = 0;
    // Worst case: last arrival at63, followed by128 waiting packet transmissions.
    // No while loop can depend on learner-provided unbounded work or a wall clock.
    for slot in 0..=MAX_ARRIVAL + MAX_PACKETS as u16 {
        while next < f.packets.len() && f.packets[next].arrival == slot {
            let index = next;
            let packet = &f.packets[index];
            next += 1;
            let mark = match p.marks[&packet.flow] {
                Mark::Keep => packet.dscp,
                Mark::Set(n) => n,
            };
            outgoing[index] = mark;
            let is_high =
                p.scheduler == Scheduler::StrictPriority && p.priority_dscp.contains(&mark);
            let queue = if p.scheduler == Scheduler::Fifo {
                Queue::Shared
            } else if is_high {
                Queue::High
            } else {
                Queue::Standard
            };
            queues[index] = queue;
            let (waiting, capacity) = if is_high {
                (&mut high, p.high_buffer)
            } else {
                (&mut ordinary, p.standard_buffer)
            };
            if waiting.len() >= usize::from(capacity) {
                records[index] = Some(record(packet, mark, queue, Delivery::TailDrop { at: slot }));
            } else {
                waiting.push_back(index);
            }
        }
        if let Some(index) = high.pop_front().or_else(|| ordinary.pop_front()) {
            let packet = &f.packets[index];
            records[index] = Some(record(
                packet,
                outgoing[index],
                queues[index],
                Delivery::Delivered {
                    departure: slot + 1,
                    waiting_slots: slot - packet.arrival,
                },
            ));
        }
        if next == f.packets.len() && high.is_empty() && ordinary.is_empty() {
            break;
        }
    }
    let packets = records
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or("Bounded replay did not drain all packets")?;
    Ok(Trace { packets })
}
fn record(p: &Packet, outgoing_dscp: u8, queue: Queue, delivery: Delivery) -> PacketTrace {
    PacketTrace {
        id: p.id.clone(),
        flow: p.flow.clone(),
        arrival: p.arrival,
        incoming_dscp: p.dscp,
        outgoing_dscp,
        queue,
        delivery,
    }
}

#[derive(Deserialize)]
struct Name(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
fn flows<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<String>, D::Error> {
    let v = crate::cron_workspace::sequence::<D, Name, MAX_FLOWS>(d)?;
    let mut set = BTreeSet::new();
    for Name(n) in v {
        if !identifier(&n) || !set.insert(n) {
            return Err(D::Error::custom("Duplicate or invalid QoS flow"));
        }
    }
    Ok(set)
}
fn priorities<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeSet<u8>, D::Error> {
    let v = crate::cron_workspace::sequence::<D, u8, 4>(d)?;
    let mut set = BTreeSet::new();
    for n in v {
        if n > 63 || !set.insert(n) {
            return Err(D::Error::custom("Duplicate or invalid priority DSCP"));
        }
    }
    Ok(set)
}
fn marks<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Mark>, D::Error> {
    struct Marks;
    impl<'de> Visitor<'de> for Marks {
        type Value = BTreeMap<String, Mark>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most four distinct bounded flow policies")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Name(key)) = a.next_key()? {
                if !identifier(&key) || out.len() >= MAX_FLOWS || out.contains_key(&key) {
                    return Err(A::Error::custom(
                        "Invalid, duplicate or excessive QoS flow policy",
                    ));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Marks)
}
impl Fixture {
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_JSON_BYTES {
            return Err("QoS fixture JSON limit".into());
        }
        let f: Self = serde_json::from_str(text)
            .map_err(|_| "Invalid bounded QoS fixture JSON".to_owned())?;
        f.validate()?;
        Ok(f)
    }
}
impl Policy {
    pub fn from_json(text: &str, f: &Fixture) -> Result<Self, String> {
        if text.len() > MAX_JSON_BYTES {
            return Err("QoS policy JSON limit".into());
        }
        let p: Self =
            serde_json::from_str(text).map_err(|_| "Invalid bounded QoS policy JSON".to_owned())?;
        p.validate(f)?;
        Ok(p)
    }
}

pub const HELP: &str = "Native model: identify supplied named streams, keep or rewrite DSCP0–63, select FIFO or explicit two-queue priority, and share finite waiting space. A full queue rejects a new arrival (tail drop). All supplied packets have equal size; arrivals enter queues before each virtual slot selects one packet. Labels are not numeric priority ranks. No IOS, arbitrary header filters, real packets/time, fairness, policing, shaping, ECN/AQM or host network operations are implemented.";
