//! Monitoring uses the virtual agent's GET/GETNEXT operations, not fixture expected answers.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkRule {
    OperationalDown,
    UnplannedDown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Incoming,
    Outgoing,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Plan {
    Link {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        description: String,
        rule: LinkRule,
    },
    Traffic {
        #[serde(deserialize_with = "crate::cron::text::<_,64>")]
        description: String,
        direction: Direction,
        width: Width,
        threshold_percent: u8,
    },
}
impl Plan {
    pub fn validate(&self) -> Result<(), String> {
        let description = match self {
            Self::Link { description, .. } | Self::Traffic { description, .. } => description,
        };
        if description.is_empty()
            || description.len() > 64
            || !description
                .bytes()
                .all(|b| b.is_ascii_graphic() || b == b' ')
        {
            return Err(
                "Enter the exact printable interface description (1–64 characters).".into(),
            );
        }
        if let Self::Traffic {
            threshold_percent, ..
        } = self
            && !(1..=100).contains(threshold_percent)
        {
            return Err("Enter a warning threshold from 1 to 100 percent.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub at_seconds: u64,
    pub agent: Agent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    pub before: Option<Snapshot>,
    pub after: Snapshot,
}
impl Stream {
    pub fn validate(&self) -> Result<(), String> {
        self.after.agent.validate()?;
        if let Some(b) = &self.before {
            b.agent.validate()?
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownCause {
    UnavailableAgent,
    MissingInterface,
    AmbiguousDescription,
    ChangedInterface,
    MissingMeasurement,
    ChangedCapacity,
    UnusableCounter,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    content = "cause",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Outcome {
    Quiet,
    Alert,
    Unknown(UnknownCause),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Poll {
    pub before: bool,
    pub next: bool,
    pub requested: Oid,
    pub reply: Reply,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub outcome: Outcome,
    pub explanation: String,
    pub interface_index: Option<u32>,
    pub rate: Option<Rate>,
    pub polls: Vec<Poll>,
}
struct Reader<'a> {
    agent: &'a Agent,
    before: bool,
    polls: &'a mut Vec<Poll>,
}
impl Reader<'_> {
    fn read(&mut self, oid: Oid, next: bool) -> Result<Reply, String> {
        if self.polls.len() >= 40 {
            return Err("This simulator bounds a monitor run to 40 object reads.".into());
        }
        let reply = if next {
            self.agent.get_next(std::slice::from_ref(&oid))?
        } else {
            self.agent.get(std::slice::from_ref(&oid))?
        }
        .remove(0);
        self.polls.push(Poll {
            before: self.before,
            next,
            requested: oid,
            reply: reply.clone(),
        });
        Ok(reply)
    }
    fn value(&mut self, base: &str, index: u32) -> Result<Value, String> {
        match self.read(Oid::instance(base, index), false)? {
            Reply::Value { value, .. } => Ok(value),
            _ => Err(
                "A required supplied MIB instance is absent; this source cannot be evaluated."
                    .into(),
            ),
        }
    }
    fn index(&mut self, description: &str) -> Result<Result<u32, UnknownCause>, String> {
        let base = Oid::parse(DESCR)?;
        let mut cursor = base.clone();
        let mut matching = Vec::new();
        for _ in 0..=12 {
            match self.read(cursor.clone(), true)? {
                Reply::Value { oid, value } if oid.0.starts_with(&base.0) => {
                    if oid.0.len() != base.0.len() + 1 {
                        return Err("Unexpected virtual interface-instance shape.".into());
                    }
                    let Value::Text(text) = value else {
                        return Err("ifDescr must return a text value.".into());
                    };
                    if text == description {
                        matching.push(*oid.0.last().unwrap())
                    }
                    cursor = oid;
                }
                _ => break,
            }
        }
        Ok(match matching.as_slice() {
            [index] => Ok(*index),
            [] => Err(UnknownCause::MissingInterface),
            _ => Err(UnknownCause::AmbiguousDescription),
        })
    }
}
fn unknown(
    cause: UnknownCause,
    index: Option<u32>,
    message: impl Into<String>,
    polls: Vec<Poll>,
) -> Observation {
    Observation {
        outcome: Outcome::Unknown(cause),
        explanation: message.into(),
        interface_index: index,
        rate: None,
        polls,
    }
}
fn lookup_message(cause: UnknownCause) -> &'static str {
    match cause {
        UnknownCause::MissingInterface => {
            "The interface description was not found in this agent's table. Compare the saved monitor target with ifDescr."
        }
        UnknownCause::AmbiguousDescription => {
            "Several interfaces have this description; the monitor cannot safely choose one."
        }
        _ => "A supplied interface measurement is unavailable.",
    }
}
fn display_rate(bits: u128, seconds: u64) -> String {
    let (factor, prefix) = [
        (1_000_000_000u128, "G"),
        (1_000_000, "M"),
        (1_000, "k"),
        (1, ""),
    ]
    .into_iter()
    .find(|(factor, _)| bits >= u128::from(seconds) * factor)
    .unwrap_or((1, ""));
    let tenths = bits * 10 / (u128::from(seconds) * factor);
    format!("{}.{} {prefix}bit/s", tenths / 10, tenths % 10)
}
pub fn observe(plan: &Plan, stream: &Stream) -> Result<Observation, String> {
    plan.validate()?;
    stream.validate()?;
    let mut polls = Vec::new();
    if !stream.after.agent.available {
        return Ok(unknown(
            UnknownCause::UnavailableAgent,
            None,
            "The supplied virtual agent is unavailable. No alert/quiet conclusion is justified.",
            polls,
        ));
    }
    let description = match plan {
        Plan::Link { description, .. } | Plan::Traffic { description, .. } => description,
    };
    let index = {
        let mut r = Reader {
            agent: &stream.after.agent,
            before: false,
            polls: &mut polls,
        };
        match r.index(description)? {
            Ok(i) => i,
            Err(c) => return Ok(unknown(c, None, lookup_message(c), polls)),
        }
    };
    match plan {
        Plan::Link { rule, .. } => {
            let mut reader = Reader {
                agent: &stream.after.agent,
                before: false,
                polls: &mut polls,
            };
            let Value::Integer(admin) = reader.value(ADMIN, index)? else {
                return Err("ifAdminStatus must return an Integer.".into());
            };
            let Value::Integer(oper) = reader.value(OPER, index)? else {
                return Err("ifOperStatus must return an Integer.".into());
            };
            let alert = oper == 2 && (*rule == LinkRule::OperationalDown || admin == 1);
            Ok(Observation {
                outcome: if alert {
                    Outcome::Alert
                } else {
                    Outcome::Quiet
                },
                explanation: format!(
                    "Interface {index}: administrative {} and operational {}. {}",
                    if admin == 1 { "Up" } else { "Down" },
                    if oper == 1 { "Up" } else { "Down" },
                    if alert {
                        "This matches the configured alert rule."
                    } else {
                        "This does not match the configured alert rule; quiet does not itself mean the link is Up."
                    }
                ),
                interface_index: Some(index),
                rate: None,
                polls,
            })
        }
        Plan::Traffic {
            direction,
            width,
            threshold_percent,
            ..
        } => {
            let Some(before) = &stream.before else {
                return Ok(unknown(
                    UnknownCause::MissingMeasurement,
                    Some(index),
                    "A rate needs two ordered measurements; one counter is a total, not a speed.",
                    polls,
                ));
            };
            if !before.agent.available {
                return Ok(unknown(
                    UnknownCause::UnavailableAgent,
                    Some(index),
                    "The earlier supplied agent snapshot is unavailable; a rate cannot be inferred.",
                    polls,
                ));
            }
            let old_index = {
                let mut r = Reader {
                    agent: &before.agent,
                    before: true,
                    polls: &mut polls,
                };
                match r.index(description)? {
                    Ok(i) => i,
                    Err(c) => return Ok(unknown(c, Some(index), lookup_message(c), polls)),
                }
            };
            if old_index != index {
                return Ok(unknown(
                    UnknownCause::ChangedInterface,
                    Some(index),
                    "The interface instance changed between snapshots. Start a fresh pair rather than joining different counters.",
                    polls,
                ));
            }
            let counter = match (direction, width) {
                (Direction::Incoming, Width::Counter32) => IN32,
                (Direction::Outgoing, Width::Counter32) => OUT32,
                (Direction::Incoming, Width::Counter64) => IN64,
                (Direction::Outgoing, Width::Counter64) => OUT64,
            };
            let mut sample = |s: &Snapshot, old: bool| -> Result<(Sample, u32), String> {
                let mut reader = Reader {
                    agent: &s.agent,
                    before: old,
                    polls: &mut polls,
                };
                let value = match reader.value(counter, index)? {
                    Value::Counter32(v) => u64::from(v),
                    Value::Counter64(v) => v,
                    _ => return Err("The selected MIB object must return a counter.".into()),
                };
                let Value::TimeTicks(uptime_ticks) = reader.value(UPTIME, 0)? else {
                    return Err("sysUpTime must return TimeTicks.".into());
                };
                let Value::TimeTicks(discontinuity_ticks) = reader.value(DISCONT, index)? else {
                    return Err("The counter-discontinuity marker must return TimeTicks.".into());
                };
                let Value::Gauge32(speed) = reader.value(SPEED, index)? else {
                    return Err("ifSpeed must return Gauge32.".into());
                };
                Ok((
                    Sample {
                        at_seconds: s.at_seconds,
                        uptime_ticks,
                        discontinuity_ticks,
                        value,
                    },
                    speed,
                ))
            };
            let (b, old_speed) = sample(before, true)?;
            let (a, speed) = sample(&stream.after, false)?;
            if old_speed != speed {
                return Ok(unknown(
                    UnknownCause::ChangedCapacity,
                    Some(index),
                    "Link capacity changed between samples; this fixed-capacity measurement cannot be compared safely.",
                    polls,
                ));
            }
            let measured = match rate(b, a, *width, speed) {
                Ok(v) => v,
                Err(message) => {
                    return Ok(unknown(
                        UnknownCause::UnusableCounter,
                        Some(index),
                        message,
                        polls,
                    ));
                }
            };
            let alert = u128::from(measured.octets) * 8 * 100
                >= u128::from(speed)
                    * u128::from(measured.seconds)
                    * u128::from(*threshold_percent);
            let utilization_tenths = u128::from(measured.octets) * 8 * 1000
                / (u128::from(speed) * u128::from(measured.seconds));
            Ok(Observation {
                outcome: if alert {
                    Outcome::Alert
                } else {
                    Outcome::Quiet
                },
                explanation: format!(
                    "Interface {index}: {} in this sample, about {}.{}% of {} capacity; the {}% warning threshold is {}.",
                    display_rate(u128::from(measured.octets) * 8, measured.seconds),
                    utilization_tenths / 10,
                    utilization_tenths % 10,
                    display_rate(u128::from(speed), 1),
                    threshold_percent,
                    if alert { "reached" } else { "not reached" }
                ),
                interface_index: Some(index),
                rate: Some(measured),
                polls,
            })
        }
    }
}

#[cfg(test)]
mod tests;
