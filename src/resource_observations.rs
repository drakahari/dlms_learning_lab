//! Finite supplied observations, never host /proc or sleeping processes.
use crate::resource_measurements::{self as core, MAX_COUNTER, Snapshot};
use serde::{Deserialize, Serialize};
pub const MAX_INPUT: usize = 64 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// at_ms is supplied milliseconds since this boot, not an epoch timestamp.
    pub counters: Snapshot,
    pub runnable: u16,
    pub blocked: u16,
    pub buffers_kib: u64,
    /// Procps cache combines Cached and reclaimable slab.
    pub cache_kib: u64,
    pub reclaimable_kib: u64,
    /// Guest counters are also included in user/nice CPU ticks.
    pub guest_ticks: [u64; 2],
    /// Explicit supplied page inventory; never inferred from the development host.
    pub page_kib: u16,
    pub swap_in_pages: u64,
    pub swap_out_pages: u64,
    /// Cumulative vmstat block accounting in KiB, not diskstats sectors.
    pub block_in_kib: u64,
    pub block_out_kib: u64,
    pub interrupts: u64,
    pub context_switches: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recording {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Observation,32>")]
    pub observations: Vec<Observation>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Bound,
    Unsupported,
    MissingInterval,
    ChangedSource,
    Decreased,
    NoCpuInterval,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
   Self::Invalid=>"Invalid supplied observation or selection; nothing was run.",
   Self::Bound=>"The supplied recording or command exceeds the simulator's bounds.",
   Self::Unsupported=>"Supported here: vmstat, vmstat DELAY COUNT, or vmstat -S K DELAY COUNT (positive whole seconds; at most8 rows). Other options and shell syntax are not implemented. No host command was executed.",
   Self::MissingInterval=>"The recording has no compatible observation at that interval. Choose a supplied interval; no readings were invented.",
   Self::ChangedSource=>"Boot or device inventory changes inside the requested interval; select one continuous source.",
   Self::Decreased=>"A cumulative counter decreases inside this interval. This recording cannot provide that measurement; no decrease was silently replaced by zero.",
   Self::NoCpuInterval=>"CPU counters did not advance in the selected interval. Choose a measurable interval.",
  })
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
impl Observation {
    pub fn validate(&self) -> Result<()> {
        self.counters.validate().map_err(|_| Error::Invalid)?;
        if !matches!(self.page_kib, 4 | 8 | 16 | 32 | 64)
            || self.counters.at_ms == 0
            || self.runnable > 32768
            || self.blocked > 32768
        {
            return Err(Error::Invalid);
        }
        if self.guest_ticks[0] > self.counters.cpu.ticks[0]
            || self.guest_ticks[1] > self.counters.cpu.ticks[1]
        {
            return Err(Error::Invalid);
        }
        let m = &self.counters.memory;
        if self.reclaimable_kib > self.cache_kib {
            return Err(Error::Invalid);
        }
        if u128::from(m.free_kib) + u128::from(self.buffers_kib) + u128::from(self.cache_kib)
            > u128::from(m.total_kib)
        {
            return Err(Error::Invalid);
        }
        if self
            .cumulative()
            .iter()
            .chain([self.buffers_kib, self.cache_kib].iter())
            .any(|v| *v > MAX_COUNTER)
        {
            return Err(Error::Bound);
        }
        Ok(())
    }
    fn cumulative(&self) -> [u64; 8] {
        [
            self.guest_ticks[0],
            self.guest_ticks[1],
            self.swap_in_pages,
            self.swap_out_pages,
            self.block_in_kib,
            self.block_out_kib,
            self.interrupts,
            self.context_switches,
        ]
    }
}
/// Typed observed fields keep later grading independent of display strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmstatReading {
    pub from: Option<usize>,
    pub to: usize,
    pub fields: [u128; 18],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VmstatRun {
    pub rows: Vec<VmstatReading>,
}
impl VmstatRun {
    pub fn text(&self) -> String {
        let labels = [
            "r", "b", "swpd", "free", "buff", "cache", "si", "so", "bi", "bo", "in", "cs", "us",
            "sy", "id", "wa", "st", "gu",
        ];
        let minimum = [2, 2, 6, 6, 6, 6, 4, 4, 5, 5, 4, 4, 2, 2, 2, 2, 2, 2];
        let widths: Vec<_> = labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                self.rows
                    .iter()
                    .map(|r| r.fields[i].to_string().len())
                    .max()
                    .unwrap_or(0)
                    .max(label.len())
                    .max(minimum[i])
            })
            .collect();
        let groups = [
            (0, 2, "procs"),
            (2, 6, "memory"),
            (6, 8, "swap"),
            (8, 10, "io"),
            (10, 12, "system"),
            (12, 18, "cpu"),
        ];
        let mut out = groups
            .into_iter()
            .map(|(a, b, name)| {
                let width = widths[a..b].iter().sum::<usize>() + b - a - 1;
                format!("{name:-^width$}")
            })
            .collect::<Vec<_>>()
            .join(" ");
        out.push('\n');
        out.push_str(
            &labels
                .iter()
                .zip(&widths)
                .map(|(label, width)| format!("{label:>width$}"))
                .collect::<Vec<_>>()
                .join(" "),
        );
        out.push('\n');
        for row in &self.rows {
            out.push_str(
                &row.fields
                    .iter()
                    .zip(&widths)
                    .map(|(value, width)| format!("{value:>width$}"))
                    .collect::<Vec<_>>()
                    .join(" "),
            );
            out.push('\n');
        }
        out
    }
}
impl Recording {
    pub fn parse(text: &str) -> Result<Self> {
        if text.len() > MAX_INPUT {
            return Err(Error::Bound);
        }
        let value: Self = serde_json::from_str(text).map_err(|_| Error::Invalid)?;
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || !(2..=32).contains(&self.observations.len()) {
            return Err(Error::Invalid);
        }
        for o in &self.observations {
            o.validate()?
        }
        Ok(())
    }
    /// Finite selected readings; no wall-time waits, host access or state mutation.
    /// Disclosed virtual /proc/meminfo subset; no filesystem access.
    pub fn meminfo(&self, index: usize) -> Result<String> {
        self.validate()?;
        let o = self.observations.get(index).ok_or(Error::Invalid)?;
        let m = &o.counters.memory;
        let fields = [
            ("MemTotal", m.total_kib),
            ("MemFree", m.free_kib),
            ("MemAvailable", m.available_kib),
            ("Buffers", o.buffers_kib),
            ("Cached", o.cache_kib - o.reclaimable_kib),
            ("SReclaimable", o.reclaimable_kib),
            ("SwapTotal", m.swap_total_kib),
            ("SwapFree", m.swap_free_kib),
        ];
        Ok(fields
            .into_iter()
            .map(|(name, value)| format!("{name}: {value:>12} kB\n"))
            .collect())
    }
    pub fn vmstat(&self, start: usize, command: &str) -> Result<String> {
        Ok(self.vmstat_run(start, command)?.text())
    }
    pub fn vmstat_run(&self, start: usize, command: &str) -> Result<VmstatRun> {
        self.validate()?;
        if command.len() > 128 {
            return Err(Error::Bound);
        }
        if command.chars().any(|c| c.is_control() && c != '\t') {
            return Err(Error::Unsupported);
        }
        let words: Vec<_> = command.split_ascii_whitespace().collect();
        let args = match words.as_slice() {
            ["vmstat", rest @ ..] => rest,
            _ => return Err(Error::Unsupported),
        };
        let args = match args {
            ["-S", "K", rest @ ..] => rest,
            _ => args,
        };
        let positive = |s: &str| -> Result<u64> {
            if s.is_empty() || s.len() > 4 || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(Error::Unsupported);
            }
            s.parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or(Error::Unsupported)
        };
        let (delay, count) = match args {
            [] => (0, 1),
            [d, n] => {
                let d = positive(d)?;
                let n = positive(n)?;
                if d > 3600 || n > 8 {
                    return Err(Error::Bound);
                }
                (d, n as usize)
            }
            _ => return Err(Error::Unsupported),
        };
        let first = self.observations.get(start).ok_or(Error::Invalid)?;
        // Validate the complete request before returning any rows.
        let mut indices = vec![start];
        for n in 1..count {
            let target = first
                .counters
                .at_ms
                .checked_add(delay * 1000 * n as u64)
                .ok_or(Error::Bound)?;
            let last = *indices.last().unwrap();
            let end = self
                .observations
                .iter()
                .enumerate()
                .skip(last + 1)
                .find(|(_, o)| o.counters.at_ms == target)
                .map(|(i, _)| i)
                .ok_or(Error::MissingInterval)?;
            self.continuity(last, end)?;
            indices.push(end);
        }
        let mut rows = Vec::with_capacity(indices.len());
        for (n, index) in indices.iter().enumerate() {
            let from = if n == 0 { None } else { Some(indices[n - 1]) };
            let before = from.map(|i| &self.observations[i]);
            rows.push(VmstatReading {
                from,
                to: *index,
                fields: self.observations[*index].fields(before)?,
            });
        }
        let output = VmstatRun { rows };
        if output.text().len() > 16384 {
            return Err(Error::Bound);
        }
        Ok(output)
    }
    fn continuity(&self, start: usize, end: usize) -> Result<()> {
        for pair in self.observations[start..=end].windows(2) {
            if pair[0].page_kib != pair[1].page_kib {
                return Err(Error::ChangedSource);
            }
            match core::measure(&pair[0].counters, &pair[1].counters) {
                Ok(_) | Err(core::Error::NoCpuInterval) => {}
                Err(core::Error::Identity) => return Err(Error::ChangedSource),
                Err(core::Error::Reset) => return Err(Error::Decreased),
                _ => return Err(Error::MissingInterval),
            }
            pair[1].deltas(Some(&pair[0]))?;
        }
        Ok(())
    }
}
impl Observation {
    fn deltas(&self, before: Option<&Self>) -> Result<([u64; 8], [u64; 8], u64)> {
        let mut cpu = self.counters.cpu.ticks;
        let mut ancillary = self.cumulative();
        let mut elapsed = self.counters.at_ms;
        if let Some(old) = before {
            elapsed = elapsed
                .checked_sub(old.counters.at_ms)
                .filter(|n| *n > 0)
                .ok_or(Error::MissingInterval)?;
            for (i, v) in cpu.iter_mut().enumerate() {
                *v = v
                    .checked_sub(old.counters.cpu.ticks[i])
                    .ok_or(Error::Decreased)?
            }
            for (i, v) in ancillary.iter_mut().enumerate() {
                *v = v.checked_sub(old.cumulative()[i]).ok_or(Error::Decreased)?
            }
        }
        if ancillary[0] > cpu[0] || ancillary[1] > cpu[1] {
            return Err(Error::Invalid);
        }
        Ok((cpu, ancillary, elapsed))
    }
    fn fields(&self, before: Option<&Self>) -> Result<[u128; 18]> {
        let (cpu, extra, elapsed) = self.deltas(before)?;
        let total: u128 = cpu.iter().map(|n| u128::from(*n)).sum();
        if total == 0 {
            return Err(Error::NoCpuInterval);
        }
        let percent = |n: u128| (n * 100 + total / 2) / total;
        let guest = u128::from(extra[0]) + u128::from(extra[1]);
        let user = u128::from(cpu[0]) + u128::from(cpu[1]) - guest;
        let system = u128::from(cpu[2]) + u128::from(cpu[5]) + u128::from(cpu[6]);
        // procps first row truncates rate averages; interval rows round to nearest.
        let rate = |n: u64| {
            let numerator = u128::from(n) * 1000;
            let d = u128::from(elapsed);
            (numerator + if before.is_some() { d / 2 } else { 0 }) / d
        };
        let m = &self.counters.memory;
        Ok([
            u128::from(self.runnable),
            u128::from(self.blocked),
            u128::from(m.swap_total_kib - m.swap_free_kib),
            u128::from(m.free_kib),
            u128::from(self.buffers_kib),
            u128::from(self.cache_kib),
            rate(extra[2] * u64::from(self.page_kib)),
            rate(extra[3] * u64::from(self.page_kib)),
            rate(extra[4]),
            rate(extra[5]),
            rate(extra[6]),
            rate(extra[7]),
            percent(user),
            percent(system),
            percent(u128::from(cpu[3])),
            percent(u128::from(cpu[4])),
            percent(u128::from(cpu[7])),
            percent(guest),
        ])
    }
}
