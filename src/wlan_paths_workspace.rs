//! Draft task/draft binding for bounded AP paths; not integrated or counted yet.
use crate::{
    script_teaching::View,
    switching_workspace,
    wlan_paths::{ApMode, Control, Forwarding, Outcome, World},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
mod bindings;
pub use bindings::{HELP, context, relevant, source};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WlanField {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub forwarding: Forwarding,
    #[serde(deserialize_with = "crate::cron::text::<_,4>")]
    pub local_vlan: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4>")]
    pub controller_vlan: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fields {
    pub mode: ApMode,
    #[serde(deserialize_with = "crate::cron::text::<_,4>")]
    pub native: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,WlanField,4>")]
    pub wlans: Vec<WlanField>,
    pub ports: switching_workspace::Draft,
}
impl Fields {
    pub fn new(w: &World) -> Self {
        Self {
            mode: w.mode,
            native: w.ap_native.to_string(),
            ports: switching_workspace::Draft::new(&w.fabric),
            wlans: w
                .wlans
                .iter()
                .map(|m| WlanField {
                    id: m.id.clone(),
                    forwarding: m.forwarding,
                    local_vlan: m.local_vlan.to_string(),
                    controller_vlan: m.controller_vlan.to_string(),
                })
                .collect(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WlanEdit {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub forwarding: bool,
    pub local_vlan: bool,
    pub controller_vlan: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortEdit {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub switch: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub port: String,
    pub up: bool,
    pub mode: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Path { forwarding: Forwarding, vlan: u16 },
    Isolated,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,100>")]
    pub label: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub wlan: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub destination: String,
    pub expected: Expected,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub initial: World,
    pub edit_mode: bool,
    pub edit_native: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,WlanEdit,4>")]
    pub edit_wlans: Vec<WlanEdit>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PortEdit,16>")]
    pub edit_ports: Vec<PortEdit>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
    pub reference: Fields,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub id: String,
    pub label: String,
    pub correct: bool,
    pub evaluated: bool,
    pub feedback: String,
}
fn number(text: &str, label: &str) -> Result<u16> {
    if text.is_empty() {
        return Err(format!("{label}: enter a VLAN number."));
    }
    if !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{label}: use a whole VLAN number from 1 to 4094."));
    }
    text.parse::<u16>()
        .ok()
        .filter(|n| (1..=4094).contains(n))
        .ok_or_else(|| format!("{label}: VLAN numbers must be 1–4094."))
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn initial_fields(&self) -> Fields {
        Fields::new(&self.initial)
    }
    fn shape(&self) -> Result<()> {
        self.initial.validate()?;
        if !identifier(&self.id)
            || self.cases.is_empty()
            || self.cases.len() > 8
            || self.edit_wlans.len() > 4
            || self.edit_ports.len() > 16
        {
            return Err("Supply a stable AP-path task,1–8 cases and bounded editing scope.".into());
        }
        let keys: BTreeSet<_> = self.initial.wlans.iter().map(|w| w.id.as_str()).collect();
        if self
            .edit_wlans
            .iter()
            .map(|e| e.id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != self.edit_wlans.len()
            || self.edit_wlans.iter().any(|e| {
                !keys.contains(e.id.as_str())
                    || !(e.forwarding || e.local_vlan || e.controller_vlan)
            })
            || self
                .edit_ports
                .iter()
                .map(|e| (&e.switch, &e.port))
                .collect::<BTreeSet<_>>()
                .len()
                != self.edit_ports.len()
            || self.edit_ports.iter().any(|e| {
                !(e.up || e.mode)
                    || self
                        .initial
                        .fabric
                        .switches
                        .get(&e.switch)
                        .is_none_or(|s| !s.ports.contains_key(&e.port))
            })
        {
            return Err("Editing scope must name distinct supplied WLANs/ports and actual editable properties.".into());
        }
        if !(self.edit_mode
            || self.edit_native
            || !self.edit_wlans.is_empty()
            || !self.edit_ports.is_empty())
        {
            return Err("A practical task needs an editable setting.".into());
        }
        let endpoints = [&self.initial.ap, &self.initial.controller];
        if self
            .cases
            .iter()
            .map(|c| c.id.as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != self.cases.len()
            || self.cases.iter().any(|c| {
                !identifier(&c.id)
                    || c.label.trim().is_empty()
                    || c.label.len() > 100
                    || c.label.chars().any(char::is_control)
                    || !keys.contains(c.wlan.as_str())
                    || !self.initial.fabric.hosts.contains_key(&c.destination)
                    || endpoints.contains(&&c.destination)
                    || matches!(c.expected,Expected::Path{vlan,..}if !(1..=4094).contains(&vlan))
            })
        {
            return Err("Every case needs a distinct identifier, supplied WLAN/service and supported expected path.".into());
        }
        // No negative-only task: every supplied WLAN needs an actual positive service case.
        if keys.iter().any(|w| {
            !self
                .cases
                .iter()
                .any(|c| c.wlan == *w && matches!(c.expected, Expected::Path { .. }))
        }) {
            return Err("Verify an actual positive client-service path for every WLAN, including protected traffic.".into());
        }
        Ok(())
    }
    fn authorized(&self, f: &Fields) -> Result<()> {
        let old = self.initial_fields();
        if !f.ports.same_shape(&self.initial.fabric)
            || f.native.len() > 4
            || f.wlans.len() != old.wlans.len()
            || f.wlans
                .iter()
                .zip(&old.wlans)
                .any(|(a, b)| a.id != b.id || a.local_vlan.len() > 4 || a.controller_vlan.len() > 4)
            || (!self.edit_mode && f.mode != old.mode)
            || (!self.edit_native && f.native != old.native)
        {
            return Err(
                "Keep this activity's supplied identities and read-only AP settings.".into(),
            );
        }
        for (a, b) in f.wlans.iter().zip(&old.wlans) {
            let edit = self.edit_wlans.iter().find(|e| e.id == a.id);
            if (edit.is_none_or(|e| !e.forwarding) && a.forwarding != b.forwarding)
                || (edit.is_none_or(|e| !e.local_vlan) && a.local_vlan != b.local_vlan)
                || (edit.is_none_or(|e| !e.controller_vlan)
                    && a.controller_vlan != b.controller_vlan)
            {
                return Err(format!(
                    "{}: keep the supplied read-only WLAN settings.",
                    a.id
                ));
            }
        }
        for (switch, draft) in &f.ports.switches {
            let base = &old.ports.switches[switch];
            if draft.vlans != base.vlans {
                return Err(format!(
                    "{switch}: active VLAN inventory is supplied, not editable in this task."
                ));
            }
            for (port, a) in &draft.ports {
                let b = &base.ports[port];
                let edit = self
                    .edit_ports
                    .iter()
                    .find(|e| e.switch == *switch && e.port == *port);
                if (edit.is_none_or(|e| !e.up) && a.up != b.up)
                    || (edit.is_none_or(|e| !e.mode)
                        && (a.trunk != b.trunk
                            || a.access != b.access
                            || a.native != b.native
                            || a.allowed != b.allowed))
                {
                    return Err(format!(
                        "{switch}/{port}: preserve the read-only switch settings."
                    ));
                }
            }
        }
        Ok(())
    }
    pub fn world(&self, f: &Fields) -> Result<World> {
        self.authorized(f)?;
        let mut world = self.initial.clone();
        f.ports.apply(&mut world.fabric)?;
        world.mode = f.mode;
        world.ap_native = number(&f.native, "AP native VLAN")?;
        for (w, value) in world.wlans.iter_mut().zip(&f.wlans) {
            w.forwarding = value.forwarding;
            w.local_vlan = number(&value.local_vlan, &format!("{} local VLAN", w.label))?;
            w.controller_vlan = number(
                &value.controller_vlan,
                &format!("{} controller VLAN", w.label),
            )?;
        }
        world.validate()?;
        Ok(world)
    }
    pub fn checks(&self, f: &Fields) -> Vec<Check> {
        if let Err(e) = self.shape() {
            return vec![Check {
                id: "definition".into(),
                label: "Activity requirements".into(),
                correct: false,
                evaluated: false,
                feedback: e,
            }];
        }
        let world = match self.world(f) {
            Ok(w) => w,
            Err(e) => {
                return std::iter::once(Check {
                    id: "configuration".into(),
                    label: "Configuration".into(),
                    correct: false,
                    evaluated: true,
                    feedback: e,
                })
                .chain(self.cases.iter().map(|c| Check {
                    id: c.id.clone(),
                    label: c.label.clone(),
                    correct: false,
                    evaluated: false,
                    feedback: "Fix the configuration before evaluating this path.".into(),
                }))
                .collect();
            }
        };
        let control = world.control();
        let (ready,feedback)=match control {
            Ok(Control::Ready)=>(true,"The supplied management VLAN carries frames in both directions. This is not a controller-join test.".into()),
            Ok(Control::NativeMismatch{configured,attached})=>(false,format!("AP native VLAN {configured} differs from its switch attachment VLAN {attached}. Untagged management traffic must agree.")),
            Ok(Control::ControllerNativeMismatch{configured,attached})=>(false,format!("Controller native VLAN {configured} differs from its switch attachment VLAN {attached}. Check the supplied management/native mapping.")),
            Ok(Control::NoBidirectionalPath)=>(false,"The supplied management VLAN path is unavailable. Check device, cable and switch-port states and the management VLAN.".into()),
            Err(e)=>(false,e),
        };
        let mut checks = vec![Check {
            id: "management".into(),
            label: "AP/controller management path".into(),
            correct: ready,
            evaluated: true,
            feedback,
        }];
        for c in &self.cases {
            let (correct,evaluated,feedback)=match world.data_path(&c.wlan,&c.destination) {
                Ok(Outcome::Data(p))=>{
                    let origin = match p.forwarding {
                        Forwarding::Central => &world.controller,
                        Forwarding::Local => &world.ap,
                    };
                    if !world.fabric.live_path(origin, &c.destination).unwrap_or(false) {
                        (false, false, "Restore the service device, switch attachment and cable. Physical disconnection does not prove VLAN separation.".into())
                    } else {
                    let changed_vlan = match c.expected {
                        Expected::Path { vlan, .. } => p.forward.iter().chain(p.reverse.iter()).flatten()
                            .find(|hop| hop.vlan != vlan).map(|hop| (&hop.switch, hop.vlan, vlan)),
                        Expected::Isolated => None,
                    };
                    let correct=match c.expected {
                        Expected::Path{forwarding,vlan}=>p.bidirectional()&&p.forwarding==forwarding&&p.vlan==vlan&&changed_vlan.is_none(),
                        Expected::Isolated=>p.forward.is_none()&&p.reverse.is_none(),
                    };
                    let feedback=if correct {"The actual forwarding path meets this requirement.".into()}
                    else if let Some((switch, actual, required)) = changed_vlan {
                        format!("The path changes VLAN: {switch} carries VLAN {actual}, but this service requires VLAN {required} throughout. Check the native VLANs on the connecting trunk ports; bidirectional reachability alone does not satisfy the VLAN requirement.")
                    } else {format!("Actual {:?} switching uses VLAN {}. Forward path: {}; return path: {}. Compare the required forwarding location and VLAN, then check the relevant switch attachment.",p.forwarding,p.vlan,if p.forward.is_some(){"available"}else{"unavailable"},if p.reverse.is_some(){"available"}else{"unavailable"})};
                    (correct,true,feedback)
                    }
                },
                Ok(Outcome::Deferred(_))=>(false,false,"Restore the AP/controller management path before checking client-data forwarding. This is not a successful isolation test.".into()),
                Err(e)=>(false,false,e),
            };
            checks.push(Check {
                id: c.id.clone(),
                label: c.label.clone(),
                correct,
                evaluated,
                feedback,
            });
        }
        checks
    }
    pub fn validate(&self) -> Result<()> {
        self.shape()?;
        if !self
            .checks(&self.reference)
            .iter()
            .all(|c| c.evaluated && c.correct)
        {
            return Err("The AP-path reference cannot reach every declared outcome.".into());
        }
        if self
            .checks(&self.initial_fields())
            .iter()
            .all(|c| c.evaluated && c.correct)
        {
            return Err("An independent AP-path task must start with a meaningful problem.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || serde_json::to_value(&self.initial.initial).map_err(|e| e.to_string())?
                == serde_json::to_value(&self.example.initial).map_err(|e| e.to_string())?
        {
            return Err(
                "Use separate task identities and different worked-example settings.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task_id: String,
    pub fields: Fields,
    pub checked_fields: Option<Fields>,
}
impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            task_id: task.id.clone(),
            fields: task.initial_fields(),
            checked_fields: None,
        }
    }
    fn bounded_fields(f: &Fields, task: &Task) -> bool {
        let old = task.initial_fields();
        f.native.len() <= 4
            && f.ports.same_shape(&task.initial.fabric)
            && f.wlans.len() == old.wlans.len()
            && f.wlans.iter().zip(&old.wlans).all(|(a, b)| {
                a.id == b.id && a.local_vlan.len() <= 4 && a.controller_vlan.len() <= 4
            })
    }
    pub fn valid(&self, task: &Task) -> bool {
        self.shape()
            && self.task_id == task.id
            && Self::bounded_fields(&self.fields, task)
            && task.authorized(&self.fields).is_ok()
            && self
                .checked_fields
                .as_ref()
                .is_none_or(|f| Self::bounded_fields(f, task) && task.authorized(f).is_ok())
    }
    pub fn checked(&self) -> bool {
        self.checked_fields.as_ref().is_some_and(|f| {
            f.mode == self.fields.mode
                && f.native == self.fields.native
                && f.wlans == self.fields.wlans
                && f.ports.version == self.fields.ports.version
                && f.ports.switches == self.fields.ports.switches
        })
    }
    pub fn edited(&mut self) {
        self.checked_fields = None;
    }
    pub fn fill(&mut self, task: &Task) -> Result<()> {
        task.validate()?;
        if self.task_id != task.id {
            return Err("Open the matching AP-path task.".into());
        }
        self.fields = task.reference.clone();
        self.edited();
        Ok(())
    }
    pub fn check(&mut self, task: &Task) -> Result<Vec<Check>> {
        task.validate()?;
        if !self.valid(task) {
            return Err("This saved AP-path draft does not match the supplied activity.".into());
        }
        let checks = task.checks(&self.fields);
        self.checked_fields = Some(self.fields.clone());
        Ok(checks)
    }
    pub fn solved(&self, task: &Task) -> bool {
        task.validate().is_ok()
            && self.valid(task)
            && self.checked()
            && task
                .checks(&self.fields)
                .iter()
                .all(|c| c.correct && c.evaluated)
    }
    pub fn reset(&mut self, task: &Task) {
        *self = Self::new(task);
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(def: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&def.initial),
            practice: Draft::new(&def.example),
        }
    }
    pub fn valid(&self, def: &Definition) -> bool {
        self.shape().is_ok()
            && self.assignment.valid(&def.initial)
            && self.practice.valid(&def.example)
    }
    pub fn solved(&self, def: &Definition) -> bool {
        def.validate().is_ok() && self.valid(def) && self.assignment.solved(&def.initial)
    }
}

impl Fields {
    fn shape(&self) -> bool {
        let text = |s: &str, limit| s.len() <= limit && !s.chars().any(char::is_control);
        text(&self.native, 4)
            && self.ports.valid()
            && !self.wlans.is_empty()
            && self.wlans.len() <= 4
            && self
                .wlans
                .iter()
                .map(|w| &w.id)
                .collect::<BTreeSet<_>>()
                .len()
                == self.wlans.len()
            && self
                .wlans
                .iter()
                .all(|w| identifier(&w.id) && text(&w.local_vlan, 4) && text(&w.controller_vlan, 4))
    }
    fn same_keys(&self, other: &Self) -> bool {
        self.wlans
            .iter()
            .map(|w| &w.id)
            .eq(other.wlans.iter().map(|w| &w.id))
            && self.ports.switches.keys().eq(other.ports.switches.keys())
            && self
                .ports
                .switches
                .iter()
                .all(|(id, row)| row.ports.keys().eq(other.ports.switches[id].ports.keys()))
    }
}
impl Draft {
    pub fn shape(&self) -> bool {
        identifier(&self.task_id)
            && self.fields.shape()
            && self
                .checked_fields
                .as_ref()
                .is_none_or(|f| f.shape() && f.same_keys(&self.fields))
    }
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1
            || !self.assignment.shape()
            || !self.practice.shape()
            || self.assignment.task_id == self.practice.task_id
        {
            return Err("Invalid or unsupported AP-path draft format; original retained.".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
