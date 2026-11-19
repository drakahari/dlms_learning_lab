//! Pure paired observations over supplied synthetic counters; no host reads.
//! This is measurement arithmetic, not a scheduler or a Linux utility emulator.
pub mod capacity;
use serde::{Deserialize, Serialize};
pub const MAX_COUNTER: u64 = 1 << 52;
pub const MAX_INPUT: usize = 32 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cpu {
    /// user,nice,system,idle,iowait,irq,softirq,steal. User/nice include
    /// guest time if present; no separate guest columns are added again.
    pub ticks: [u64; 8],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub total_kib: u64,
    pub free_kib: u64,
    pub available_kib: u64,
    pub swap_total_kib: u64,
    pub swap_free_kib: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    pub read_ops: u64,
    pub write_ops: u64,
    pub read_sectors: u64,
    pub write_sectors: u64,
    pub read_ms: u64,
    pub write_ms: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub boot: String,
    pub at_ms: u64,
    pub cores: u8,
    pub cpu: Cpu,
    pub memory: Memory,
    pub disk: Disk,
    pub network: Network,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Series {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Snapshot,32>")]
    pub samples: Vec<Snapshot>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Bound,
    Identity,
    Time,
    Reset,
    NoCpuInterval,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self{
 Self::Invalid=>"Invalid supplied observation; no measurement was produced.",
 Self::Bound=>"Observation exceeds the disclosed input or counter bounds.",
 Self::Identity=>"Pair samples from the same boot, CPU inventory, disk and interface.",
 Self::Time=>"Select a later snapshot; the interval must be positive and at most one day.",
 Self::Reset=>"A cumulative counter decreased. Reset, reattachment, wrap or unsupported accounting change prevents this interval; choose compatible samples.",
 Self::NoCpuInterval=>"CPU counters did not advance; this pair has no measured CPU interval.",
 })
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
/// An exact nonnegative ratio. Derivation keeps units explicit; UI rounding is separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub numerator: u128,
    pub denominator: u128,
}
impl Ratio {
    fn new(n: u128, d: u128) -> Self {
        assert!(d > 0);
        Self {
            numerator: n,
            denominator: d,
        }
    }
    pub fn scaled_floor(self, scale: u64) -> Result<u128> {
        if self.denominator == 0 {
            return Err(Error::Invalid);
        }
        self.numerator
            .checked_mul(u128::from(scale))
            .map(|n| n / self.denominator)
            .ok_or(Error::Bound)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Window {
    pub elapsed_ms: u64,
    /// Fractions of aggregate interval ticks. Multiply by100 to display percent.
    pub cpu_shares: [Ratio; 8],
    /// Executed work excludes idle, iowait and steal; inspect those separately.
    pub cpu_work_share: Ratio,
    pub available_kib: u64,
    pub swap_used_kib: u64,
    pub read_ops_per_second: Ratio,
    pub write_ops_per_second: Ratio,
    pub read_bytes_per_second: Ratio,
    pub write_bytes_per_second: Ratio,
    /// Read/write completion accounting only; not a latency histogram or service time.
    pub mean_completion_ms: Option<Ratio>,
    pub rx_bytes_per_second: Ratio,
    pub tx_bytes_per_second: Ratio,
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}
impl Snapshot {
    pub fn validate(&self) -> Result<()> {
        if !identifier(&self.boot)
            || self.boot.len() > 48
            || !(1..=64).contains(&self.cores)
            || !identifier(&self.disk.name)
            || self.disk.name.len() > 32
            || !identifier(&self.network.name)
            || self.network.name.len() > 32
        {
            return Err(Error::Invalid);
        }
        let m = &self.memory;
        if m.total_kib == 0
            || m.free_kib > m.total_kib
            || m.available_kib > m.total_kib
            || m.swap_free_kib > m.swap_total_kib
        {
            return Err(Error::Invalid);
        }
        if self.at_ms > MAX_COUNTER
            || self.cpu.ticks.iter().any(|n| *n > MAX_COUNTER)
            || [
                m.total_kib,
                m.free_kib,
                m.available_kib,
                m.swap_total_kib,
                m.swap_free_kib,
                self.disk.read_ops,
                self.disk.write_ops,
                self.disk.read_sectors,
                self.disk.write_sectors,
                self.disk.read_ms,
                self.disk.write_ms,
                self.network.rx_bytes,
                self.network.tx_bytes,
            ]
            .iter()
            .any(|n| *n > MAX_COUNTER)
        {
            return Err(Error::Bound);
        }
        Ok(())
    }
}
impl Series {
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_INPUT {
            return Err(Error::Bound);
        }
        let value: Self = serde_json::from_str(text).map_err(|_| Error::Invalid)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || !(2..=32).contains(&self.samples.len()) {
            return Err(Error::Invalid);
        }
        for sample in &self.samples {
            sample.validate()?;
        }
        Ok(())
    }
    /// A stored series may deliberately include a reset to teach rejecting a pair.
    pub fn window(&self, start: usize, end: usize) -> Result<Window> {
        self.validate()?;
        let a = self.samples.get(start).ok_or(Error::Invalid)?;
        let b = self.samples.get(end).ok_or(Error::Invalid)?;
        if start >= end {
            return Err(Error::Time);
        }
        // Do not hide a known intervening reset by choosing distant endpoints.
        // A sub-tick CPU sample may be stationary; all other deltas must still
        // be checked before that diagnostic can be ignored here.
        for pair in self.samples[start..=end].windows(2) {
            match measure(&pair[0], &pair[1]) {
                Ok(_) | Err(Error::NoCpuInterval) => {}
                Err(error) => return Err(error),
            }
        }
        measure(a, b)
    }
}
fn delta(a: u64, b: u64) -> Result<u64> {
    b.checked_sub(a).ok_or(Error::Reset)
}
pub fn measure(a: &Snapshot, b: &Snapshot) -> Result<Window> {
    a.validate()?;
    b.validate()?;
    if a.boot != b.boot
        || a.cores != b.cores
        || a.disk.name != b.disk.name
        || a.network.name != b.network.name
    {
        return Err(Error::Identity);
    }
    let elapsed = b
        .at_ms
        .checked_sub(a.at_ms)
        .filter(|t| *t > 0 && *t <= 86_400_000)
        .ok_or(Error::Time)?;
    let mut ticks = [0u64; 8];
    for (i, t) in ticks.iter_mut().enumerate() {
        *t = delta(a.cpu.ticks[i], b.cpu.ticks[i])?;
    }
    let total: u128 = ticks.iter().map(|t| u128::from(*t)).sum();
    let read = delta(a.disk.read_ops, b.disk.read_ops)?;
    let write = delta(a.disk.write_ops, b.disk.write_ops)?;
    let read_sectors = delta(a.disk.read_sectors, b.disk.read_sectors)?;
    let write_sectors = delta(a.disk.write_sectors, b.disk.write_sectors)?;
    let read_ms = delta(a.disk.read_ms, b.disk.read_ms)?;
    let write_ms = delta(a.disk.write_ms, b.disk.write_ms)?;
    let rx = delta(a.network.rx_bytes, b.network.rx_bytes)?;
    let tx = delta(a.network.tx_bytes, b.network.tx_bytes)?;
    if total == 0 {
        return Err(Error::NoCpuInterval);
    }
    let shares = ticks.map(|t| Ratio::new(u128::from(t), total));
    let work = [0usize, 1, 2, 5, 6]
        .into_iter()
        .map(|i| u128::from(ticks[i]))
        .sum();
    let rate = |n: u64, unit: u64| {
        Ratio::new(u128::from(n) * u128::from(unit) * 1000, u128::from(elapsed))
    };
    let completions = u128::from(read) + u128::from(write);
    Ok(Window {
        elapsed_ms: elapsed,
        cpu_shares: shares,
        cpu_work_share: Ratio::new(work, total),
        available_kib: b.memory.available_kib,
        swap_used_kib: b.memory.swap_total_kib - b.memory.swap_free_kib,
        read_ops_per_second: rate(read, 1),
        write_ops_per_second: rate(write, 1),
        read_bytes_per_second: rate(read_sectors, 512),
        write_bytes_per_second: rate(write_sectors, 512),
        mean_completion_ms: (completions != 0)
            .then(|| Ratio::new(u128::from(read_ms) + u128::from(write_ms), completions)),
        rx_bytes_per_second: rate(rx, 1),
        tx_bytes_per_second: rate(tx, 1),
    })
}
