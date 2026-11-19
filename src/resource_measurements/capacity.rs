//! Bounded occupancy forecasts and supplied workload replay; no host changes.
//! A supplied capacity snapshot is a stock, not a CPU or I/O counter.
use serde::{Deserialize, Serialize};
pub const MAX_KIB: u64 = 1 << 42;
pub const MAX_DAY: u32 = 36_600;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub day: u32,
    pub used_kib: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Series {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub volume: String,
    pub capacity_kib: u64,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Sample,32>")]
    pub samples: Vec<Sample>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub first: usize,
    pub last: usize,
}
/// Exact signed growth. A falling/flat stock is representable without inventing
/// a negative cumulative-counter rate or asserting permanent future recovery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub change_kib: i128,
    pub days: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Window,
    Bound,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Invalid => {
                "The supplied capacity observations or plan are inconsistent. No plan was applied."
            }
            Self::Window => "Choose two different observations in increasing date order.",
            Self::Bound => {
                "The requested forecast exceeds the simulator's disclosed time or size bounds."
            }
        })
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Series {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > 32 * 1024 {
            return Err(Error::Bound);
        }
        let series: Self = serde_json::from_str(text).map_err(|_| Error::Invalid)?;
        series.validate()?;
        Ok(series)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !name(&self.volume)
            || self.capacity_kib == 0
            || self.capacity_kib > MAX_KIB
            || !(2..=32).contains(&self.samples.len())
            || self
                .samples
                .iter()
                .any(|p| p.day > MAX_DAY || p.used_kib > self.capacity_kib)
            || self.samples.windows(2).any(|p| p[0].day >= p[1].day)
        {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    pub fn growth(&self, window: Window) -> Result<Growth> {
        self.validate()?;
        if window.first >= window.last || window.last >= self.samples.len() {
            return Err(Error::Window);
        }
        let a = &self.samples[window.first];
        let b = &self.samples[window.last];
        Ok(Growth {
            change_kib: i128::from(b.used_kib) - i128::from(a.used_kib),
            days: b.day - a.day,
        })
    }
}
impl Growth {
    /// Round projected occupied KiB upward. Negative projections stop at zero,
    /// an explicit linear-model boundary rather than a deletion operation.
    pub fn at(self, anchor: &Sample, day: u32) -> Result<u64> {
        if self.days == 0
            || day < anchor.day
            || day > MAX_DAY
            || anchor.used_kib > MAX_KIB
            || self.change_kib.unsigned_abs() > u128::from(MAX_KIB)
        {
            return Err(Error::Bound);
        }
        let n = self.change_kib * i128::from(day - anchor.day);
        let d = i128::from(self.days);
        let rounded = n.div_euclid(d) + i128::from(n.rem_euclid(d) != 0);
        let value = (i128::from(anchor.used_kib) + rounded).max(0);
        if value > i128::from(MAX_KIB) {
            return Err(Error::Bound);
        }
        Ok(value as u64)
    }
    /// First whole day whose rounded occupancy exceeds the supplied safe limit.
    /// Equality retains the required reserve; it is not already a breach.
    pub fn first_breach(self, anchor: &Sample, limit: u64, through: u32) -> Result<Option<u32>> {
        self.at(anchor, through)?;
        if limit > MAX_KIB {
            return Err(Error::Bound);
        }
        if anchor.used_kib > limit {
            return Ok(Some(anchor.day));
        }
        if self.change_kib <= 0 {
            return Ok(None);
        }
        let days =
            i128::from(limit - anchor.used_kib) * i128::from(self.days) / self.change_kib + 1;
        let day = i128::from(anchor.day) + days;
        Ok((day <= i128::from(through)).then_some(day as u32))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    pub capacity_kib: u64,
    pub lead_days: u32,
    pub cost: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub offer: String,
    pub order_day: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub day: u32,
    pub added_kib: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Demand {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    pub daily_kib: u64,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Change,8>")]
    pub changes: Vec<Change>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Day {
    pub day: u32,
    pub used_kib: u64,
    pub capacity_kib: u64,
    pub reserve_met: bool,
}
/// Finite virtual plan replay. All demand is supplied; it is not a prediction
/// of the learner's computer and performs no actual storage action.
pub fn replay(
    series: &Series,
    offer: &Offer,
    plan: &Plan,
    demand: &Demand,
    reserve_kib: u64,
    horizon_days: u32,
) -> Result<Vec<Day>> {
    series.validate()?;
    let anchor = series.samples.last().unwrap();
    if !name(&offer.id)
        || plan.offer != offer.id
        || offer.capacity_kib < series.capacity_kib
        || offer.capacity_kib > MAX_KIB
        || offer.lead_days > 90
        || plan.order_day < anchor.day
        || reserve_kib == 0
        || reserve_kib > MAX_KIB
        || !(1..=90).contains(&horizon_days)
        || anchor
            .day
            .checked_add(horizon_days)
            .is_none_or(|d| d > MAX_DAY)
        || plan.order_day > anchor.day + horizon_days
        || !name(&demand.id)
        || demand.daily_kib > MAX_KIB
        || demand.changes.len() > 8
        || demand.changes.iter().any(|c| {
            c.added_kib > MAX_KIB || c.day <= anchor.day || c.day > anchor.day + horizon_days
        })
    {
        return Err(Error::Invalid);
    }
    let arrival = plan
        .order_day
        .checked_add(offer.lead_days)
        .ok_or(Error::Bound)?;
    let mut days = Vec::with_capacity(horizon_days as usize + 1);
    for offset in 0..=horizon_days {
        let day = anchor.day + offset;
        let used = u128::from(anchor.used_kib)
            + u128::from(demand.daily_kib) * u128::from(offset)
            + demand
                .changes
                .iter()
                .filter(|c| c.day <= day)
                .map(|c| u128::from(c.added_kib))
                .sum::<u128>();
        if used > u128::from(MAX_KIB) {
            return Err(Error::Bound);
        }
        let capacity = if day >= arrival {
            offer.capacity_kib
        } else {
            series.capacity_kib
        };
        days.push(Day {
            day,
            used_kib: used as u64,
            capacity_kib: capacity,
            reserve_met: used + u128::from(reserve_kib) <= u128::from(capacity),
        });
    }
    Ok(days)
}
