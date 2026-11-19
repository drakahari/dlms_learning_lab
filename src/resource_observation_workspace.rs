//! Recorded-host investigation. Requests and report claims are simulated data.
use crate::{resource_observations as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const HELP: &str = "Recorded-host simulator: vmstat, vmstat DELAY COUNT, vmstat -S K DELAY COUNT and cat /proc/meminfo. Counts are finite, at most8 rows, from supplied observations. No host file, process, command or clock is used. The first vmstat row is since boot; later rows measure intervals. Processes/memory are instantaneous. The virtual meminfo contains only eight disclosed fields. Other commands, flags, paths, shell operators and continuous sampling are unsupported.";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    WorkPercent,
    IdlePercent,
    WaitPercent,
    StealPercent,
    Runnable,
    Blocked,
    SwapUsedKib,
    FreeKib,
    CacheKib,
    SwapInKibPerSecond,
    SwapOutKibPerSecond,
    BlockInKibPerSecond,
    BlockOutKibPerSecond,
    AvailableKib,
}
impl Metric {
    pub fn label(self) -> &'static str {
        match self {
            Self::WorkPercent => "Executing work (us + sy + gu)",
            Self::IdlePercent => "Idle (id)",
            Self::WaitPercent => "I/O wait (wa)",
            Self::StealPercent => "Steal (st)",
            Self::Runnable => "Runnable processes (r)",
            Self::Blocked => "Blocked processes (b)",
            Self::SwapUsedKib => "Allocated swap (swpd)",
            Self::FreeKib => "Free memory (free)",
            Self::CacheKib => "Cache including reclaimable slab (cache)",
            Self::SwapInKibPerSecond => "Swap in (si)",
            Self::SwapOutKibPerSecond => "Swap out (so)",
            Self::BlockInKibPerSecond => "Block input (bi)",
            Self::BlockOutKibPerSecond => "Block output (bo)",
            Self::AvailableKib => "Available memory (MemAvailable)",
        }
    }
    pub fn units(self) -> &'static str {
        match self {
            Self::WorkPercent | Self::IdlePercent | Self::WaitPercent | Self::StealPercent => "%",
            Self::Runnable | Self::Blocked => "processes",
            Self::SwapInKibPerSecond
            | Self::SwapOutKibPerSecond
            | Self::BlockInKibPerSecond
            | Self::BlockOutKibPerSecond => "KiB/s",
            _ => "KiB",
        }
    }
    fn value(self, reading: &Reading, row: usize, task: &Task) -> Result<u128, String> {
        match reading {
            Reading::Vmstat(run) => {
                let v = &run
                    .rows
                    .get(row)
                    .ok_or("Choose an actual result row")?
                    .fields;
                Ok(match self{Self::WorkPercent=>v[12]+v[13]+v[17],Self::IdlePercent=>v[14],Self::WaitPercent=>v[15],Self::StealPercent=>v[16],Self::Runnable=>v[0],Self::Blocked=>v[1],Self::SwapUsedKib=>v[2],Self::FreeKib=>v[3],Self::CacheKib=>v[5],Self::SwapInKibPerSecond=>v[6],Self::SwapOutKibPerSecond=>v[7],Self::BlockInKibPerSecond=>v[8],Self::BlockOutKibPerSecond=>v[9],Self::AvailableKib=>return Err("vmstat free is not MemAvailable. Read the virtual /proc/meminfo at this observation.".into())})
            }
            Reading::Memory(index) => {
                if row != 0 {
                    return Err("A meminfo reading has one snapshot".into());
                }
                let m = &task.recording.observations[*index].counters.memory;
                match self{Self::AvailableKib=>Ok(u128::from(m.available_kib)),Self::FreeKib=>Ok(u128::from(m.free_kib)),Self::SwapUsedKib=>Ok(u128::from(m.swap_total_kib-m.swap_free_kib)),Self::CacheKib=>Ok(u128::from(task.recording.observations[*index].cache_kib)),_=>Err("A memory snapshot does not measure an activity rate. Run vmstat with an interval and finite count.".into())}
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selection {
    Interval {
        from_ms: u64,
        to_ms: u64,
        min_ms: u64,
    },
    Snapshot {
        from_ms: u64,
        to_ms: u64,
    },
    Extremum {
        from_ms: u64,
        to_ms: u64,
        step_ms: u64,
        highest: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub boot: String,
    pub metric: Metric,
    pub selection: Selection,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    pub recording: core::Recording,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Rule,8>")]
    pub rules: Vec<Rule>,
}
fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub start: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub command: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Vmstat(core::VmstatRun),
    Memory(usize),
}
impl Reading {
    pub fn text(&self, task: &Task) -> Result<String, String> {
        match self {
            Self::Vmstat(v) => Ok(v.text()),
            Self::Memory(i) => task.recording.meminfo(*i).map_err(|e| e.to_string()),
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Vmstat(r) => r.rows.len(),
            Self::Memory(_) => 1,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn span(&self, row: usize, task: &Task) -> Result<(Option<usize>, usize), String> {
        match self {
            Self::Vmstat(r) => {
                let row = r.rows.get(row).ok_or("Choose an actual result row")?;
                Ok((row.from, row.to))
            }
            Self::Memory(i) if row == 0 && *i < task.recording.observations.len() => Ok((None, *i)),
            _ => Err("Choose an actual result row".into()),
        }
    }
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        self.recording.validate().map_err(|e| e.to_string())?;
        if !slug(&self.id)
            || self.rules.is_empty()
            || self.rules.len() > 8
            || self
                .rules
                .iter()
                .map(|r| &r.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.rules.len()
        {
            return Err("Supply a named investigation with distinct report requirements".into());
        }
        for r in &self.rules {
            if !slug(&r.id)
                || r.label.trim().is_empty()
                || r.label.len() > 160
                || r.label.chars().any(char::is_control)
                || !self
                    .recording
                    .observations
                    .iter()
                    .any(|o| o.counters.boot == r.boot)
            {
                return Err("Invalid report requirement or source boot".into());
            }
            match r.selection {
                Selection::Interval {
                    from_ms,
                    to_ms,
                    min_ms,
                } if from_ms >= to_ms || min_ms == 0 || min_ms > to_ms - from_ms => {
                    return Err("Use a positive, bounded report interval".into());
                }
                Selection::Extremum {
                    from_ms,
                    to_ms,
                    step_ms,
                    ..
                } if from_ms >= to_ms
                    || step_ms == 0
                    || step_ms % 1000 != 0
                    || step_ms > 3_600_000
                    || step_ms > to_ms - from_ms =>
                {
                    return Err(
                        "Peak/trough selection needs a supplied whole-second interval".into(),
                    );
                }
                Selection::Snapshot { from_ms, to_ms } if from_ms > to_ms => {
                    return Err("Invalid snapshot range".into());
                }
                _ => {}
            }
            if r.metric == Metric::AvailableKib
                && !matches!(r.selection, Selection::Snapshot { .. })
            {
                return Err("Available memory is a snapshot, not an interval counter".into());
            }
            if matches!(r.selection, Selection::Snapshot { .. })
                && !matches!(
                    r.metric,
                    Metric::Runnable
                        | Metric::Blocked
                        | Metric::SwapUsedKib
                        | Metric::FreeKib
                        | Metric::CacheKib
                        | Metric::AvailableKib
                )
            {
                return Err("An activity rate is not an instantaneous snapshot".into());
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 96 * 1024 {
            return Err("Investigation exceeds its source limit".into());
        }
        Ok(())
    }
    pub fn probe(&self, p: &Probe) -> Result<Reading, String> {
        if p.command.len() > 128
            || p.start >= self.recording.observations.len()
            || p.command.chars().any(|c| c.is_control() && c != '\t')
        {
            return Err("Choose a supplied observation and one bounded command line".into());
        }
        let words: Vec<_> = p.command.split_ascii_whitespace().collect();
        if words == ["cat", "/proc/meminfo"] {
            self.recording.meminfo(p.start).map_err(|e| e.to_string())?;
            return Ok(Reading::Memory(p.start));
        }
        self.recording
            .vmstat_run(p.start, &p.command)
            .map(Reading::Vmstat)
            .map_err(|e| {
                format!("{e} Also supported: cat /proc/meminfo (virtual eight-field subset).")
            })
    }
    fn rule_value(&self, r: &Rule, e: &Evidence) -> Result<u128, String> {
        let reading = self.probe(&e.probe)?;
        let (from, to) = reading.span(e.row, self)?;
        let end = &self.recording.observations[to].counters;
        if end.boot != r.boot {
            return Err(
                "This evidence belongs to another boot; select the requested source.".into(),
            );
        }
        match r.selection {
            Selection::Interval {
                from_ms,
                to_ms,
                min_ms,
            } => {
                let Some(start) = from else {
                    return Err("The first vmstat row is a since-boot average, not this interval. Attach a later interval row.".into());
                };
                let start = &self.recording.observations[start].counters;
                if start.at_ms < from_ms || end.at_ms > to_ms || end.at_ms - start.at_ms < min_ms {
                    return Err(format!(
                        "Use an interval wholly inside {from_ms}–{to_ms} ms, at least {min_ms} ms long. This row describes {}–{} ms.",
                        start.at_ms, end.at_ms
                    ));
                }
            }
            Selection::Extremum {
                from_ms,
                to_ms,
                step_ms,
                highest,
            } => {
                let Some(start) = from else {
                    return Err("Choose an interval row, not the first since-boot average".into());
                };
                let begin = &self.recording.observations[start].counters;
                if begin.at_ms < from_ms || end.at_ms > to_ms || end.at_ms - begin.at_ms != step_ms
                {
                    return Err(format!(
                        "Compare supplied {step_ms} ms intervals wholly inside {from_ms}–{to_ms} ms."
                    ));
                }
                let chosen = r.metric.value(&reading, e.row, self)?;
                let mut best = None;
                for (i, o) in self.recording.observations.iter().enumerate() {
                    if o.counters.boot != r.boot
                        || o.counters.at_ms < from_ms
                        || o.counters
                            .at_ms
                            .checked_add(step_ms)
                            .is_none_or(|t| t > to_ms)
                    {
                        continue;
                    }
                    let p = Probe {
                        start: i,
                        command: format!("vmstat {} 2", step_ms / 1000),
                    };
                    if let Ok(candidate) = self.probe(&p) {
                        let value = r.metric.value(&candidate, 1, self)?;
                        best = Some(best.map_or(value, |old: u128| {
                            if highest {
                                old.max(value)
                            } else {
                                old.min(value)
                            }
                        }));
                    }
                }
                if best != Some(chosen) {
                    return Err(format!(
                        "Another eligible interval has a {} displayed {}. Compare every supplied interval before attaching the evidence.",
                        if highest { "higher" } else { "lower" },
                        r.metric.label()
                    ));
                }
            }
            Selection::Snapshot { from_ms, to_ms } => {
                if end.at_ms < from_ms || end.at_ms > to_ms {
                    return Err(format!(
                        "Use a snapshot inside {from_ms}–{to_ms} ms. This one is at {} ms.",
                        end.at_ms
                    ));
                }
            }
        }
        r.metric.value(&reading, e.row, self)
    }
    pub fn check(&self, d: &Draft) -> Vec<(bool, String)> {
        if let Err(e) = d.bound(self) {
            return vec![(false, e)];
        }
        self.rules.iter().map(|r|{
  let Some(c)=d.claims.iter().find(|c|c.rule==r.id)else{return(false,format!("{}: report row is missing.",r.label))};
  let Some(e)=&c.evidence else{return(false,format!("{}: run a measurement, choose its relevant row and attach it as evidence.",r.label))};
  let expected=match self.rule_value(r,e){Ok(v)=>v,Err(error)=>return(false,format!("{}: {error}",r.label))};
  let value=c.value.trim();if value.is_empty(){return(false,format!("{}: enter the measured whole number in {}.",r.label,r.metric.units()))}
  let actual=if value.bytes().all(|b|b.is_ascii_digit()){value.parse::<u128>().ok()}else{None};
  let Some(actual)=actual else{return(false,format!("{}: use digits only; the {} unit is already labeled.",r.label,r.metric.units()))};
  if actual==expected{return(true,format!("{}: reported value matches the attached observation ({}).",r.label,r.metric.units()))}
  let reason=if r.metric==Metric::WorkPercent{"Add us + sy + gu from the attached interval row. Keep id, wa and st separate; do not substitute the first since-boot row."}else if matches!(r.metric,Metric::SwapInKibPerSecond|Metric::SwapOutKibPerSecond){"si/so are current KiB per second. swpd is allocated swap, not a transfer rate."}else if r.metric==Metric::AvailableKib{"Use MemAvailable, not MemFree or total minus free. These supplied values have different meanings."}else{"Read the labeled column from the attached measurement. An interval rate differs from a cumulative total or a different sample."};
  (false,format!("{}: {actual} does not match that measurement. {reason}",r.label))
 }).collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub probe: Probe,
    pub row: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub rule: String,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub value: String,
    pub evidence: Option<Evidence>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Check,
    Run {
        probe: Probe,
    },
    Attach {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        rule: String,
        row: usize,
    },
    Value {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        rule: String,
        #[serde(deserialize_with = "crate::cron::text::<_,24>")]
        value: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub task_id: String,
    pub cursor: usize,
    pub result_row: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,8>")]
    pub history: Vec<Probe>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Claim,8>")]
    pub claims: Vec<Claim>,
    pub checked: bool,
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            cursor: 0,
            result_row: 0,
            input: String::new(),
            history: vec![],
            claims: t
                .rules
                .iter()
                .map(|r| Claim {
                    rule: r.id.clone(),
                    value: String::new(),
                    evidence: None,
                })
                .collect(),
            checked: false,
        }
    }
    pub fn shape(&self) -> Result<(), String> {
        if !slug(&self.task_id)
            || self.cursor >= 32
            || self.result_row >= 8
            || self.input.len() > 128
            || self.input.chars().any(|c| c.is_control() && c != '\t')
            || self.history.len() > 8
            || self.claims.len() > 8
            || self.claims.is_empty()
        {
            return Err("Invalid saved observation draft shape".into());
        }
        for p in &self.history {
            if p.start >= 32
                || p.command.len() > 128
                || p.command.chars().any(|c| c.is_control() && c != '\t')
            {
                return Err("Invalid saved probe".into());
            }
        }
        let mut ids = BTreeSet::new();
        for c in &self.claims {
            if !slug(&c.rule)
                || !ids.insert(&c.rule)
                || c.value.len() > 24
                || c.value.chars().any(char::is_control)
            {
                return Err("Invalid or duplicate saved report row".into());
            }
            if let Some(e) = &c.evidence
                && (e.row >= 8
                    || e.probe.start >= 32
                    || e.probe.command.len() > 128
                    || e.probe.command.chars().any(|c| c.is_control() && c != '\t'))
            {
                return Err("Invalid saved report evidence".into());
            }
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        self.shape()?;
        if self.task_id != t.id
            || self.cursor >= t.recording.observations.len()
            || self.input.len() > 128
            || self.input.chars().any(|c| c.is_control() && c != '\t')
            || self.history.len() > 8
            || self.claims.len() != t.rules.len()
            || self
                .claims
                .iter()
                .any(|c| !t.rules.iter().any(|r| r.id == c.rule))
            || self
                .claims
                .iter()
                .map(|c| &c.rule)
                .collect::<BTreeSet<_>>()
                .len()
                != self.claims.len()
        {
            return Err("Saved observation workspace does not match its authored task".into());
        }
        for p in &self.history {
            if p.start >= t.recording.observations.len()
                || p.command.len() > 128
                || p.command.chars().any(|c| c.is_control() && c != '\t')
            {
                return Err("Invalid saved measurement request".into());
            }
        }
        match self.history.last().and_then(|p| t.probe(p).ok()) {
            Some(output) if self.result_row >= output.len() => {
                return Err("Saved result selection is outside the actual output".into());
            }
            None if self.result_row != 0 => return Err("No measured row is selected".into()),
            _ => {}
        }
        for c in &self.claims {
            if c.value.len() > 24 || c.value.chars().any(char::is_control) {
                return Err("Report value exceeds its input limit".into());
            }
            if let Some(e) = &c.evidence {
                let output = t.probe(&e.probe)?;
                if e.row >= output.len() {
                    return Err("Saved evidence references a nonexistent output row".into());
                }
            }
        }
        Ok(())
    }
    pub fn select_row(&mut self, t: &Task, row: usize) -> Result<(), String> {
        let p = self.history.last().ok_or("Run a measurement first")?;
        let output = t.probe(p)?;
        if row >= output.len() {
            return Err("Select an actual output row".into());
        }
        self.result_row = row;
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<(), String> {
        self.bound(t)?;
        let mut next = self.clone();
        match a {
            Action::Check => {
                let _ = next.check(t)?;
            }
            Action::Run { probe } => {
                if probe.start >= t.recording.observations.len()
                    || probe.command.len() > 128
                    || probe.command.chars().any(|c| c.is_control() && c != '\t')
                {
                    return Err(
                        "Choose one supplied observation and one bounded command line".into(),
                    );
                }
                if next.history.len() == 8 {
                    next.history.remove(0);
                }
                next.cursor = probe.start;
                next.result_row = 0;
                next.input = probe.command.clone();
                next.history.push(probe.clone());
            }
            Action::Attach { rule, row } => {
                let p = next
                    .history
                    .last()
                    .ok_or("Run a measurement first")?
                    .clone();
                let result = t.probe(&p)?;
                if *row >= result.len() {
                    return Err("Choose an actual output row".into());
                }
                next.claims
                    .iter_mut()
                    .find(|c| c.rule == *rule)
                    .ok_or("Unknown report row")?
                    .evidence = Some(Evidence {
                    probe: p,
                    row: *row,
                });
                next.checked = false;
            }
            Action::Value { rule, value } => {
                if value.len() > 24 || value.chars().any(char::is_control) {
                    return Err("Use a bounded whole-number report value".into());
                }
                next.claims
                    .iter_mut()
                    .find(|c| c.rule == *rule)
                    .ok_or("Unknown report row")?
                    .value = value.clone();
                next.checked = false;
            }
        }
        next.bound(t)?;
        *self = next;
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<(bool, String)>, String> {
        self.bound(t)?;
        self.checked = true;
        Ok(t.check(self))
    }
    pub fn report(&self, t: &Task) -> Result<String, String> {
        self.bound(t)?;
        let verified = self.checked && t.check(self).iter().all(|r| r.0);
        let mut out = format!(
            "Recorded-host measurement report · {}\n",
            if verified {
                "verified"
            } else {
                "unchecked or needs correction"
            }
        );
        for r in &t.rules {
            let c = self.claims.iter().find(|c| c.rule == r.id).unwrap();
            out.push_str(&format!("{}: {} {}\n", r.label, c.value, r.metric.units()));
            if let Some(e) = &c.evidence {
                let output = t.probe(&e.probe)?;
                let (from, to) = output.span(e.row, t)?;
                let end = &t.recording.observations[to].counters;
                let span = from.map_or_else(
                    || format!("snapshot/boot row at {} ms", end.at_ms),
                    |i| {
                        format!(
                            "{}–{} ms interval",
                            t.recording.observations[i].counters.at_ms, end.at_ms
                        )
                    },
                );
                out.push_str(&format!(
                    "  Evidence: {} · {span} · {} · row {}\n",
                    end.boot,
                    e.probe.command,
                    e.row + 1
                ));
            } else {
                out.push_str("  Evidence: not attached\n");
            }
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,40>")]
    pub reference: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,40>")]
    pub example_reference: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || self.initial.recording == self.example.recording {
            return Err("Keep worked recording and independent assignment separate".into());
        }
        for (t, actions) in [
            (&self.initial, &self.reference),
            (&self.example, &self.example_reference),
        ] {
            if actions.is_empty() || actions.len() > 40 {
                return Err("Provide a bounded reachable worked investigation".into());
            }
            let mut d = Draft::new(t);
            for a in actions {
                d.apply(t, a)?;
            }
            if !d.checked || !t.check(&d).iter().all(|r| r.0) {
                return Err(
                    "Worked investigation does not satisfy every report requirement".into(),
                );
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        s.validate()?;
        if s.version != 1 || s.example_action > self.example_reference.len() {
            return Err("Unsupported or invalid saved observation session".into());
        }
        s.assignment.bound(&self.initial)?;
        s.practice.bound(&self.example)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub example_action: usize,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.example_action > 40 {
            return Err("Unsupported saved observation version or example position".into());
        }
        self.assignment.shape()?;
        self.practice.shape()?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 48 * 1024 {
            return Err("Saved investigation exceeds its size limit".into());
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            example_action: 0,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
}

impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (t, d) = if practice {
            (&self.example, &s.practice)
        } else {
            (&self.initial, &s.assignment)
        };
        if !d.checked {
            return vec![(false,"Report is unchecked. Attach measured evidence, enter the values, then choose Check report.".into())];
        }
        t.check(d)
    }
}
impl Session {
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String, String> {
        d.bound(self)?;
        let raw=text.strip_prefix("resource-action ").ok_or("Use the native recorded-host terminal and report controls. resource-action is a simulator-only content reference, not a Linux command.")?;
        if raw.len() > 8192 {
            return Err("Observation reference exceeds limit".into());
        }
        let action: Action = serde_json::from_str(raw)
            .map_err(|_| "Invalid simulator-only observation reference")?;
        if let Action::Run { probe } = &action {
            d.initial.probe(probe)?;
        }
        let mut next = self.clone();
        next.assignment.apply(&d.initial, &action)?;
        d.bound(&next)?;
        *self = next;
        Ok("Applied simulated investigation operation".into())
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::ResourceObservations { simulation },
        crate::engine::Goal::ResourceObservations { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Observation state and objective must agree".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Investigation must begin with separate, unanswered report drafts".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::ResourceObservations { .. })
        || matches!(s.initial, crate::engine::State::ResourceObservations { .. })
        || matches!(s.goal, crate::engine::Goal::ResourceObservations { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<(), String> {
    let d = source(s)?;
    let crate::engine::State::ResourceObservations { simulation } = &a.state else {
        return Err("Saved investigation belongs to another task".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible investigation metadata; original work retained".into());
    }
    d.bound(simulation)
}
