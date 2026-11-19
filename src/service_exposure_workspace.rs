//! Source-bound, replayable terminal work. No scenario is registered implicitly.
use crate::{
    linux_filter::Class,
    script_teaching::View,
    service_exposure::{self as core, Observation, World},
    service_exposure_task::Task,
};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
const CONSOLE: usize = 32 * 1024;
fn within<T: Serialize>(v: &T, max: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|s| s.len() <= max)
}
fn source_task(t: &Task) -> Result<String> {
    fn sort(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Array(items) => {
                for item in items {
                    sort(item);
                }
            }
            serde_json::Value::Object(map) => {
                let ordered: std::collections::BTreeMap<_, _> =
                    std::mem::take(map).into_iter().collect();
                for (key, mut item) in ordered {
                    sort(&mut item);
                    map.insert(key, item);
                }
            }
            _ => {}
        }
    }
    // Explicit ordering remains stable if serde_json's map backend changes.
    // This is source comparison, not a hash or an authentication token.
    let mut value = serde_json::to_value(t).map_err(|e| e.to_string())?;
    sort(&mut value);
    serde_json::to_string(&value).map_err(|e| e.to_string())
}

fn draft(s: &str, max: usize) -> bool {
    s.len() <= max && !s.contains('\0')
}
fn append(old: &str, input: &str, output: &str, prompt: &str) -> Result<String> {
    let next = format!("{old}{prompt} {input}\n{output}");
    if next.len() > CONSOLE {
        return Err("This console reached its32KiB limit; the action was not applied. Save a progress backup before resetting this selected workspace.".into());
    }
    Ok(next)
}
fn diagnostic(class: Class, message: &str) -> String {
    format!(
        "{}: {message}\n",
        match class {
            Class::Syntax => "Syntax error",
            Class::Unsupported => "Unsupported command or syntax",
            Class::Limit => "Simulation limit",
            Class::Context => "Workspace requirement",
        }
    )
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Admin {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        input: String,
    },
    Scan {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        input: String,
    },
    /// New runs use a distinct semantic version. Never reinterpret old Scan
    /// history: its unsupported command observations must remain unchanged.
    ProbeV2 {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        input: String,
    },
    /// Separate interpreter version: never reinterpret rejected old commands.
    ObserverV3 {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        input: String,
    },
    TelnetEscapeV3,
    Select {
        viewpoint: usize,
    },
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    AdminInput {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        text: String,
    },
    ScanInput {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        text: String,
    },
    Event {
        event: Event,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    pub viewpoint: usize,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Observation,32>")]
    pub rows: Vec<Observation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,262144>")]
    pub source_task: String,
    pub world: World,
    pub viewpoint: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub admin_input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub scan_input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub admin_output: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub scan_output: String,
    pub admin_status: Option<i32>,
    pub scan_status: Option<i32>,
    pub last_scan: Option<Scan>,
    /// Missing legacy state stays absent, including on serialization. Task/Goal
    /// snapshots and pristine source data therefore do not acquire new fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telnet: Option<crate::telnet_client::Mode>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,96>")]
    pub history: Vec<Event>,
}
impl Scope {
    fn seed(t: &Task) -> Result<Self> {
        Ok(Self {
            source_task: source_task(t)?,
            world: t.initial.clone(),
            viewpoint: 0,
            admin_input: String::new(),
            scan_input: String::new(),
            admin_output: String::new(),
            scan_output: String::new(),
            admin_status: None,
            scan_status: None,
            last_scan: None,
            telnet: None,
            checked: false,
            history: vec![],
        })
    }
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Self::seed(t)
    }
    pub fn shape(&self) -> Result<()> {
        if let Some(mode) = self.telnet {
            mode.validate().map_err(|e| e.message)?;
            if mode == crate::telnet_client::Mode::Idle {
                return Err("An idle observer has no stored Telnet client phase.".into());
            }
        }
        if !draft(&self.admin_input, 4096)
            || !draft(&self.scan_input, 1024)
            || self.source_task.len() > 256 * 1024
            || self.admin_output.len() > CONSOLE
            || self.scan_output.len() > CONSOLE
            || self.history.len() > 96
            || !within(&self.history, 256 * 1024)
            || !within(self, 768 * 1024)
        {
            return Err("Keep this exposure workspace within its disclosed input, console, history and memory limits.".into());
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        t.validate()?;
        self.shape()?;
        if self.source_task != source_task(t)? || !self.world.same_problem(&t.initial) {
            return Err("This saved workspace belongs to different source requirements. Its prior work must be retained for an explicit migration, not credited against new requirements.".into());
        }
        let mut replay = Self::seed(t)?;
        for event in &self.history {
            replay.event(event)?;
        }
        if self.world != replay.world
            || self.viewpoint != replay.viewpoint
            || self.admin_output != replay.admin_output
            || self.scan_output != replay.scan_output
            || self.admin_status != replay.admin_status
            || self.scan_status != replay.scan_status
            || self.last_scan != replay.last_scan
            || self.telnet != replay.telnet
            || self.checked != replay.checked
        {
            return Err("Saved results or state do not match this attempt's actual simulated actions; prior work is retained.".into());
        }
        Ok(())
    }
    fn clear_observation(&mut self) {
        self.last_scan = None;
        self.scan_output.clear();
        self.scan_status = None;
        self.telnet = None;
    }
    fn event(&mut self, event: &Event) -> Result<()> {
        use crate::telnet_client::Mode;
        let active = matches!(
            self.telnet,
            Some(Mode::Connected { .. } | Mode::Command { .. })
        );
        // Old histories never had an active client. Keep their interpreter arms
        // exact while preventing a new live client from becoming a shell channel.
        if active
            && (matches!(event, Event::Scan { .. } | Event::ProbeV2 { .. })
                || matches!(event, Event::Select { viewpoint } if *viewpoint != self.viewpoint))
        {
            return Err("Close the simulated Telnet client at its local prompt before changing observer or running a separate probe.".into());
        }
        match event {
            Event::Admin { input } => {
                if !draft(input, 4096) {
                    return Err("Server input exceeds4096bytes or contains NUL.".into());
                }
                let mut world = self.world.clone();
                let (text, status, changed) = match world.admin_command(input) {
                    Ok(o) => (o.text, o.status, o.changed),
                    Err(e) => (diagnostic(e.class, &e.message), 1, false),
                };
                let output = append(&self.admin_output, input, &text, "server #")?;
                self.world = world;
                self.admin_output = output;
                self.admin_status = Some(status);
                self.admin_input.clear();
                if changed {
                    self.clear_observation();
                    self.checked = false;
                }
            }
            Event::Scan { input } => {
                if !draft(input, 1024) {
                    return Err("Scan input exceeds1024bytes or contains NUL.".into());
                }
                let (text, status, last) = match self.world.scan_command(self.viewpoint, input) {
                    Ok(rows) => (
                        core::report(&self.world, self.viewpoint, &rows).map_err(|e| e.message)?,
                        0,
                        Some(Scan {
                            viewpoint: self.viewpoint,
                            rows,
                        }),
                    ),
                    Err(e) => (diagnostic(e.class, &e.message), 1, None),
                };
                self.scan_output = append(&self.scan_output, input, &text, "observer $")?;
                self.scan_status = Some(status);
                self.last_scan = last;
                self.scan_input.clear();
            }
            Event::ProbeV2 { input } => {
                if !draft(input, 1024) {
                    return Err("Probe input exceeds 1024 bytes or contains NUL.".into());
                }
                let (text, status, last) =
                    match crate::nc_probe::observer_v2(&self.world, self.viewpoint, input) {
                        Ok(report) => (
                            report.output,
                            report.status,
                            Some(Scan {
                                viewpoint: self.viewpoint,
                                rows: report.observations,
                            }),
                        ),
                        Err(error) => (diagnostic(error.class, &error.message), 1, None),
                    };
                self.scan_output = append(&self.scan_output, input, &text, "observer $")?;
                self.scan_status = Some(status);
                self.last_scan = last;
                self.scan_input.clear();
            }
            Event::ObserverV3 { input } => self.observer_v3(input)?,
            Event::TelnetEscapeV3 => {
                let next = crate::telnet_client::escape(self.telnet.unwrap_or_default())
                    .map_err(|e| e.message)?;
                self.scan_output = append(
                    &self.scan_output,
                    "Escape to local Telnet prompt",
                    &next.output,
                    "native action:",
                )?;
                self.telnet = Some(next.mode);
                self.scan_status = next.exit_status;
            }
            Event::Select { viewpoint } => {
                if *viewpoint >= self.world.viewpoints.len() {
                    return Err("Choose one of the supplied observer positions.".into());
                }
                if self.viewpoint != *viewpoint {
                    self.viewpoint = *viewpoint;
                    self.clear_observation();
                }
            }
            Event::Check => self.checked = true,
        }
        Ok(())
    }
    fn observer_v3(&mut self, input: &str) -> Result<()> {
        use crate::telnet_client::Mode;
        if !draft(input, 1024) {
            return Err("Keep observer input within1024bytes without NUL.".into());
        }
        let mode = self.telnet.unwrap_or_default();
        let first = input.split_whitespace().next();
        if mode != Mode::Idle || first == Some("telnet") {
            let prompt = if matches!(mode, Mode::Command { .. }) {
                "telnet>"
            } else {
                "observer $"
            };
            // Escape already displayed a pending local prompt. Consume that
            // prompt before echoing the submitted command exactly once. This
            // belongs only to the new client lifecycle; legacy events stay exact.
            let previous = if matches!(mode, Mode::Command { .. }) {
                self.scan_output
                    .strip_suffix("telnet> ")
                    .unwrap_or(&self.scan_output)
            } else {
                &self.scan_output
            };
            match crate::telnet_client::run(&self.world, self.viewpoint, mode, input) {
                Ok(next) => {
                    self.scan_output = append(previous, input, &next.output, prompt)?;
                    self.telnet = if next.mode == Mode::Idle {
                        None
                    } else {
                        Some(next.mode)
                    };
                    self.scan_status = next.exit_status;
                    if let Some(rows) = next.observations {
                        self.last_scan = Some(Scan {
                            viewpoint: self.viewpoint,
                            rows,
                        });
                    }
                }
                Err(error) => {
                    self.scan_output = append(
                        previous,
                        input,
                        &diagnostic(error.class, &error.message),
                        prompt,
                    )?;
                    if mode == Mode::Idle {
                        self.telnet = None;
                        self.scan_status = Some(1);
                        self.last_scan = None;
                    } else {
                        // An unsupported input did not end the running client.
                        self.scan_status = None;
                    }
                }
            }
        } else {
            let result = if matches!(first, Some("nc" | "nmap")) {
                crate::nc_probe::observer_v2(&self.world, self.viewpoint, input)
            } else {
                Err(crate::linux_filter::Fault { class: crate::linux_filter::Class::Unsupported, message: "Observer subset: numeric nmap TCP connect, nc -n -v -z SUPPLIED_IP PORT, or telnet -c SUPPLIED_IP NONDEFAULT_PORT. No shell, data exchange or real target runs.".into() })
            };
            let (output, status, rows) = match result {
                Ok(report) => (
                    report.output,
                    report.status,
                    Some(Scan {
                        viewpoint: self.viewpoint,
                        rows: report.observations,
                    }),
                ),
                Err(error) => (diagnostic(error.class, &error.message), 1, None),
            };
            self.scan_output = append(&self.scan_output, input, &output, "observer $")?;
            self.scan_status = Some(status);
            self.last_scan = rows;
            self.telnet = None;
        }
        self.scan_input.clear();
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        match a {
            Action::AdminInput { text } => {
                if !draft(text, 4096) {
                    return Err("Keep server input within4096bytes without NUL.".into());
                }
                next.admin_input = text.clone();
            }
            Action::ScanInput { text } => {
                if !draft(text, 1024) {
                    return Err("Keep scan input within1024bytes without NUL.".into());
                }
                next.scan_input = text.clone();
            }
            Action::Event { event } => {
                if next.history.len() >= 96 {
                    return Err("This attempt reached96actions. Save a progress backup before resetting only this workspace.".into());
                }
                next.event(event)?;
                next.history.push(event.clone());
            }
        }
        next.shape()?;
        *self = next;
        Ok(())
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.bound(t).is_ok() && self.checked && t.checks(&self.world).iter().all(|c| c.passed)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,48>")]
    pub worked: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.assignment.validate()?;
        self.example.validate()?;
        if self.version != 1
            || self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.worked.is_empty()
            || self.worked.len() > 48
            || !within(self, 768 * 1024)
            || self.assignment.initial.server == self.example.initial.server
        {
            return Err("Use a bounded version1 exposure lesson with a stable ID, separate example server and reachable worked actions.".into());
        }
        let mut example = Scope::new(&self.example)?;
        for a in &self.worked {
            example.apply(&self.example, a)?;
        }
        if !example.solved(&self.example) {
            return Err(
                "The actual worked actions must complete every practice requirement.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        if self.version != 1
            || self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !within(self, 768 * 1024)
            || s.version != 1
            || s.source_id != self.source_id
            || !within(s, 1536 * 1024)
        {
            return Err("The saved exposure lesson has a different identifier/version or exceeds its memory limit.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.example.bound(&self.example)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1
            || self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !within(self, 1536 * 1024)
        {
            return Err(
                "Keep a version1 bounded exposure session with its stable lesson ID.".into(),
            );
        }
        for s in [&self.assignment, &self.example] {
            s.shape()?;
            s.world.validate().map_err(|e| e.message)?;
            if s.viewpoint >= s.world.viewpoints.len() {
                return Err("The selected observer is not part of this lesson.".into());
            }
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            example: Scope::new(&d.example)?,
        })
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.example.apply(&d.example, a)?;
        } else {
            next.assignment.apply(&d.assignment, a)?;
        }
        if !within(&next, 1536 * 1024) {
            return Err(
                "The exposure lesson exceeds its memory limit; prior work retained.".into(),
            );
        }
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.example = Scope::new(&d.example)?;
        } else {
            self.assignment = Scope::new(&d.assignment)?;
        }
        Ok(())
    }
}
