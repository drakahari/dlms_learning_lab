//! Bounded read-only virtual MIB operations. No packets, sockets, host commands or SNMP security.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Oid(#[serde(deserialize_with = "crate::cron_workspace::sequence::<_,u32,32>")] Vec<u32>);
impl Oid {
    pub fn parse(raw: &str) -> Result<Self, String> {
        let s = raw.trim().strip_prefix('.').unwrap_or(raw.trim());
        if s.is_empty() {
            return Err("Enter a dotted numeric object identifier.".into());
        }
        if s.len() > 384 {
            return Err("This simulator supports object identifiers up to 384 characters.".into());
        }
        let mut parts = Vec::new();
        for p in s.split('.') {
            if p.is_empty() || !p.bytes().all(|c| c.is_ascii_digit()) {
                return Err("Use whole non-negative numbers separated by dots.".into());
            }
            parts.push(
                p.parse::<u32>().map_err(
                    |_| "An object-identifier number exceeds the supported 32-bit range.",
                )?,
            );
            if parts.len() > 32 {
                return Err("This simulator supports at most 32 object-identifier numbers.".into());
            }
        }
        let oid = Self(parts);
        oid.validate()?;
        Ok(oid)
    }
    fn validate(&self) -> Result<(), String> {
        if self.0.len() < 2
            || self.0.len() > 32
            || self.0[0] > 2
            || (self.0[0] < 2 && self.0[1] > 39)
        {
            return Err("An object identifier needs valid first and second numbers (0–2, then 0–39 unless the first is 2).".into());
        }
        Ok(())
    }
    pub fn dotted(&self) -> String {
        self.0
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(".")
    }
    fn instance(base: &str, index: u32) -> Self {
        Self::parse(&format!("{base}.{index}")).expect("fixed source MIB base")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkState {
    Up,
    Down,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Text(String),
    Integer(i32),
    Gauge32(u32),
    Counter32(u32),
    Counter64(u64),
    TimeTicks(u32),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub index: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub description: String,
    pub admin: LinkState,
    pub oper: LinkState,
    pub speed_bits_per_second: u32,
    pub in_octets: u64,
    pub out_octets: u64,
    pub discontinuity_ticks: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agent {
    pub uptime_ticks: u32,
    pub available: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Interface,12>")]
    pub interfaces: Vec<Interface>,
}
const UPTIME: &str = "1.3.6.1.2.1.1.3";
const DESCR: &str = "1.3.6.1.2.1.2.2.1.2";
const SPEED: &str = "1.3.6.1.2.1.2.2.1.5";
const ADMIN: &str = "1.3.6.1.2.1.2.2.1.7";
const OPER: &str = "1.3.6.1.2.1.2.2.1.8";
const IN32: &str = "1.3.6.1.2.1.2.2.1.10";
const OUT32: &str = "1.3.6.1.2.1.2.2.1.16";
const IN64: &str = "1.3.6.1.2.1.31.1.1.1.6";
const OUT64: &str = "1.3.6.1.2.1.31.1.1.1.10";
const DISCONT: &str = "1.3.6.1.2.1.31.1.1.1.19";
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Value { oid: Oid, value: Value },
    NoSuchObject,
    NoSuchInstance,
    EndOfMibView,
}
impl Agent {
    pub fn validate(&self) -> Result<(), String> {
        if self.interfaces.is_empty() || self.interfaces.len() > 12 {
            return Err("Supply 1–12 virtual interfaces.".into());
        }
        let mut indices = std::collections::BTreeSet::new();
        for i in &self.interfaces {
            if i.index == 0 || i.index > i32::MAX as u32 || !indices.insert(i.index) {
                return Err("Interface indexes must be unique positive Integer32 values.".into());
            }
            if i.description.is_empty()
                || i.description.len() > 64
                || !i
                    .description
                    .bytes()
                    .all(|b| b.is_ascii_graphic() || b == b' ')
            {
                return Err("Use a 1–64 character printable ASCII interface description.".into());
            }
            if i.speed_bits_per_second == 0 {
                return Err("Supply a positive supported interface speed.".into());
            }
        }
        Ok(())
    }
    fn objects(&self) -> BTreeMap<Oid, Value> {
        let mut result = BTreeMap::from([(
            Oid::instance(UPTIME, 0),
            Value::TimeTicks(self.uptime_ticks),
        )]);
        for i in &self.interfaces {
            let state = |s| Value::Integer(if s == LinkState::Up { 1 } else { 2 });
            for (base, value) in [
                (DESCR, Value::Text(i.description.clone())),
                (SPEED, Value::Gauge32(i.speed_bits_per_second)),
                (ADMIN, state(i.admin)),
                (OPER, state(i.oper)),
                (IN32, Value::Counter32(i.in_octets as u32)),
                (OUT32, Value::Counter32(i.out_octets as u32)),
                (IN64, Value::Counter64(i.in_octets)),
                (OUT64, Value::Counter64(i.out_octets)),
                (DISCONT, Value::TimeTicks(i.discontinuity_ticks)),
            ] {
                result.insert(Oid::instance(base, i.index), value);
            }
        }
        result
    }
    pub fn get(&self, oids: &[Oid]) -> Result<Vec<Reply>, String> {
        self.query(oids, false)
    }
    pub fn get_next(&self, oids: &[Oid]) -> Result<Vec<Reply>, String> {
        self.query(oids, true)
    }
    fn query(&self, oids: &[Oid], next: bool) -> Result<Vec<Reply>, String> {
        self.validate()?;
        if oids.is_empty() || oids.len() > 16 {
            return Err("Poll 1–16 object identifiers at a time in this simulator.".into());
        }
        for oid in oids {
            oid.validate()?
        }
        if !self.available {
            return Err("The supplied virtual agent is unavailable; this is not an observed network timeout.".into());
        }
        let objects = self.objects();
        Ok(oids
            .iter()
            .map(|oid| {
                if next {
                    use std::ops::Bound::{Excluded, Unbounded};
                    objects
                        .range((Excluded(oid), Unbounded))
                        .next()
                        .map(|(o, v)| Reply::Value {
                            oid: o.clone(),
                            value: v.clone(),
                        })
                        .unwrap_or(Reply::EndOfMibView)
                } else if let Some(value) = objects.get(oid) {
                    Reply::Value {
                        oid: oid.clone(),
                        value: value.clone(),
                    }
                } else if [
                    UPTIME, DESCR, SPEED, ADMIN, OPER, IN32, OUT32, IN64, OUT64, DISCONT,
                ]
                .iter()
                .any(|base| {
                    let base = Oid::parse(base).expect("fixed MIB base");
                    oid.0.starts_with(&base.0)
                }) {
                    Reply::NoSuchInstance
                } else {
                    Reply::NoSuchObject
                }
            })
            .collect())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Width {
    Counter32,
    Counter64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub at_seconds: u64,
    pub uptime_ticks: u32,
    pub discontinuity_ticks: u32,
    pub value: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rate {
    pub octets: u64,
    pub seconds: u64,
}
impl Rate {
    pub fn at_least_bits_per_second(self, threshold: u64) -> bool {
        u128::from(self.octets) * 8 >= u128::from(threshold) * u128::from(self.seconds)
    }
}
pub fn rate(
    before: Sample,
    after: Sample,
    width: Width,
    speed_bits_per_second: u32,
) -> Result<Rate, String> {
    let seconds = after
        .at_seconds
        .checked_sub(before.at_seconds)
        .filter(|v| (1..=3600).contains(v))
        .ok_or("Use two ordered snapshots 1–3600 seconds apart.")?;
    if speed_bits_per_second == 0 {
        return Err("Supply the supported link capacity before calculating a rate.".into());
    }
    if after.uptime_ticks.wrapping_sub(before.uptime_ticks) != seconds as u32 * 100 {
        return Err("Agent uptime does not match the sample interval; a restart or incompatible samples makes this rate unknown.".into());
    }
    if before.discontinuity_ticks != after.discontinuity_ticks {
        return Err(
            "The counter discontinuity marker changed; start a new measurement pair.".into(),
        );
    }
    let modulus = match width {
        Width::Counter32 => 1u128 << 32,
        Width::Counter64 => 1u128 << 64,
    };
    if u128::from(before.value) >= modulus || u128::from(after.value) >= modulus {
        return Err("A supplied counter value exceeds its declared width.".into());
    }
    let max_bits = u128::from(speed_bits_per_second) * u128::from(seconds);
    if max_bits >= modulus * 8 {
        return Err("The interval can hide a whole counter wrap; choose the high-capacity counter or a shorter interval.".into());
    }
    let delta = (u128::from(after.value) + modulus - u128::from(before.value)) % modulus;
    if delta * 8 > max_bits {
        return Err("The observed counter change exceeds this supplied link capacity; these samples cannot provide a valid rate.".into());
    }
    Ok(Rate {
        octets: u64::try_from(delta).expect("delta is below Counter64 modulus"),
        seconds,
    })
}

#[cfg(test)]
mod tests;

/// State-derived bounded monitoring, using the virtual MIB operations above.
pub mod monitor;
