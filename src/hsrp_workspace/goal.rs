//! Source-owned policy and transient, settled fault probes. No fault edits persist.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "crate::cron::text::<_,80>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Event {
    Power { router: core::RouterId, up: bool },
    Lan { router: core::RouterId, up: bool },
    Wan { router: core::RouterId, up: bool },
}
impl Event {
    pub fn fault(&self) -> core::FaultEvent {
        match *self {
            Self::Power { router, up } => core::FaultEvent::RouterPower { router, up },
            Self::Lan { router, up } => core::FaultEvent::LanLink { router, up },
            Self::Wan { router, up } => core::FaultEvent::WanLink { router, up },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub title: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Event,8>")]
    pub events: Vec<Event>,
    // None accepts either current active identity; it never accepts no active.
    #[serde(deserialize_with = "bounded::sequence::<_,Option<core::RouterId>,8>")]
    pub expected_active: Vec<Option<core::RouterId>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub initial: core::State,
    #[serde(deserialize_with = "bounded::sequence::<_,core::RouterId,2>")]
    pub editable_routers: Vec<core::RouterId>,
    #[serde(deserialize_with = "bounded::sequence::<_,u8,8>")]
    pub editable_clients: Vec<u8>,
    pub normal_active: Option<core::RouterId>,
    #[serde(deserialize_with = "bounded::sequence::<_,Probe,8>")]
    pub probes: Vec<Probe>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Worked {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub title: String,
    #[serde(deserialize_with = "crate::cron::text::<_,800>")]
    pub why: String,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub description: String,
    pub policy: Policy,
    #[serde(deserialize_with = "bounded::sequence::<_,Worked,12>")]
    pub worked: Vec<Worked>,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub try_change: String,
    pub experiment: Action,
    pub repair: Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub policy: Policy,
    pub lesson: Lesson,
}
impl Policy {
    pub fn validate(&self) -> Result<(), String> {
        model(&self.initial)?;
        if self.initial.revision() > u64::MAX - (MAX_ACTIONS as u64 + 8) {
            return Err("Source revision lacks bounded configuration/probe headroom".into());
        }
        if self.editable_routers.is_empty()
            || self.editable_routers.len() > 2
            || self.editable_clients.len() > 8
            || self.editable_routers.iter().collect::<BTreeSet<_>>().len()
                != self.editable_routers.len()
            || self.editable_clients.iter().collect::<BTreeSet<_>>().len()
                != self.editable_clients.len()
            || self
                .editable_clients
                .iter()
                .any(|id| !self.initial.clients().iter().any(|c| c.id == *id))
            || self.initial.clients().iter().any(|c| !c.link_up)
            || core::RouterId::ALL.iter().any(|id| {
                let f = self.initial.router(*id).facts;
                !f.power_on || !f.lan_link_up || !f.wan_link_up || !f.route_available
            })
            || self.probes.len() < 2
            || self.probes.len() > 8
        {
            return Err("HSRP source needs healthy fixed facts, explicit editable identities and two-router failure coverage".into());
        }
        let mut restored = BTreeSet::new();
        let mut names = BTreeSet::new();
        for p in &self.probes {
            if p.title.trim().is_empty()
                || p.title.len() > 96
                || !names.insert(&p.title)
                || p.events.is_empty()
                || p.events.len() > 8
                || p.events.len() != p.expected_active.len()
            {
                return Err("Invalid bounded HSRP probe sequence".into());
            }
            let mut facts = core::RouterId::ALL.map(|id| self.initial.router(id).facts);
            // A return must restore the same failed fact in the same sequence.
            // An unrelated already-up link cannot stand in for powered-on recovery.
            let mut pending = BTreeSet::new();
            for e in &p.events {
                if let Event::Power { router, up } | Event::Lan { router, up } = e {
                    let fact = if matches!(e, Event::Power { .. }) {
                        0u8
                    } else {
                        1u8
                    };
                    if *up {
                        if pending.remove(&(*router, fact)) {
                            restored.insert(*router);
                        }
                    } else {
                        pending.insert((*router, fact));
                    }
                }
                let (id, up) = match e {
                    Event::Power { router, up }
                    | Event::Lan { router, up }
                    | Event::Wan { router, up } => (*router, *up),
                };
                let f = &mut facts[if id == core::RouterId::A { 0 } else { 1 }];
                match e {
                    Event::Power { .. } => f.power_on = up,
                    Event::Lan { .. } => f.lan_link_up = up,
                    Event::Wan { .. } => f.wan_link_up = up,
                };
                if !facts
                    .iter()
                    .any(|f| f.power_on && f.lan_link_up && f.wan_link_up && f.route_available)
                {
                    return Err(
                        "A forwarding resilience probe cannot remove every usable supplied router"
                            .into(),
                    );
                }
            }
            if !pending.is_empty() {
                return Err("Each power/LAN failure probe must explicitly restore the same failed fact before ending".into());
            }
        }
        if restored.len() != 2 {
            return Err(
                "Both routers need an explicit power/LAN failure and same-fact return in their probe sequences".into(),
            );
        }
        Ok(())
    }
    pub fn authorize(&self, a: &Action) -> Result<(), String> {
        match a {
            Action::Router { router, config } => {
                if !self.editable_routers.contains(router) {
                    return Err("That router configuration is protected by the task".into());
                }
                config.validate().map_err(|e| e.to_string())?;
            }
            Action::Gateway { client, .. } => {
                if !self.editable_clients.contains(client) {
                    return Err("That client gateway is protected by the task".into());
                }
            }
        }
        Ok(())
    }
    pub fn replay(&self, history: &[Action]) -> Result<core::State, String> {
        if history.len() > MAX_ACTIONS {
            return Err(
                "HSRP configuration-history bound; selected reset starts a fresh attempt".into(),
            );
        }
        let mut state = self.initial.clone();
        for a in history {
            self.authorize(a)?;
            if !changed(&state, a) {
                return Err("HSRP history contains a no-op that was never published".into());
            }
            apply(&mut state, a)?;
        }
        Ok(state)
    }
    pub fn bind(
        &self,
        initial: &core::State,
        current: &core::State,
        history: &[Action],
    ) -> Result<(), String> {
        if initial != &self.initial
            || !same_fixed_facts(&self.initial, current)?
            || &self.replay(history)? != current
        {
            return Err(
                "HSRP state/history differs from the authored source; previous work retained"
                    .into(),
            );
        }
        Ok(())
    }
    pub fn rows(&self, current: &core::State) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|()| model(current)) {
            return vec![(false, e)];
        }
        let mut rows = Vec::new();
        for id in core::RouterId::ALL {
            let r = current.router(id);
            rows.push((
                r.config.participating,
                format!(
                    "{}: {} HSRP candidate; both peers must participate",
                    r.label,
                    if r.config.participating {
                        "participating"
                    } else {
                        "disabled"
                    }
                ),
            ));
        }
        rows.push((
            current.active().is_some()
                && current.standby().is_some()
                && self
                    .normal_active
                    .is_none_or(|id| current.active() == Some(id)),
            format!(
                "Current settled roles: Active {:?}, Standby {:?}{}",
                current.active(),
                current.standby(),
                self.normal_active
                    .map(|id| format!("; required Active {id:?}"))
                    .unwrap_or_default()
            ),
        ));
        clients_rows(current, "Current", &mut rows);
        for p in &self.probes {
            let mut probe = current.clone();
            for (step, e) in p.events.iter().enumerate() {
                if let Err(err) = probe.fault(e.fault()) {
                    rows.push((
                        false,
                        format!("{} step{} refused: {err}", p.title, step + 1),
                    ));
                    break;
                }
                rows.push((
                    probe.active().is_some()
                        && p.expected_active[step].is_none_or(|id| probe.active() == Some(id)),
                    format!(
                        "{} step{}: {e:?}; settled Active {:?}{}",
                        p.title,
                        step + 1,
                        probe.active(),
                        p.expected_active[step]
                            .map(|id| format!("; required {id:?}"))
                            .unwrap_or_default()
                    ),
                ));
                clients_rows(&probe, &format!("{} step{}", p.title, step + 1), &mut rows);
            }
        }
        rows
    }
}
fn clients_rows(s: &core::State, at: &str, rows: &mut Vec<(bool, String)>) {
    for c in s.clients() {
        match s.client_status(c.id){Ok(r)=>rows.push((r.uses_virtual_ip&&r.forwarding_available(),format!("{at} · {} gateway {}: {:?} via {:?}; required virtual gateway {} and modeled forwarding",c.label,c.gateway,r.outcome,r.via,s.virtual_ip()))),Err(e)=>rows.push((false,format!("{at} · {}: {e}",c.label)))}
    }
}
impl Definition {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_DEFINITION {
            return Err("HSRP raw definition exceeds128KiB".into());
        }
        let d: Self = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
        d.shape()?;
        Ok(d)
    }
    pub fn to_json(&self) -> Result<Vec<u8>, String> {
        self.shape()?;
        serde_json::to_vec(self).map_err(|e| e.to_string())
    }
    pub fn shape(&self) -> Result<(), String> {
        self.policy.validate()?;
        self.lesson.policy.validate()?;
        let l = &self.lesson;
        let addresses = |s: &core::State| {
            let mut a = BTreeSet::from([
                s.virtual_ip(),
                s.router(core::RouterId::A).address,
                s.router(core::RouterId::B).address,
            ]);
            a.extend(s.clients().iter().map(|c| c.address));
            a
        };
        if !addresses(&self.policy.initial).is_disjoint(&addresses(&l.policy.initial))
            || l.prerequisite_note.trim().is_empty()
            || l.prerequisite_note.len() > 1600
            || l.description.trim().is_empty()
            || l.description.len() > 1600
            || l.try_change.trim().is_empty()
            || l.try_change.len() > 1600
            || l.concepts.len() < 3
            || l.concepts.len() > 6
            || l.worked.is_empty()
            || l.worked.len() > 12
            || !bounded::within(self, MAX_DEFINITION)
        {
            return Err(
                "HSRP needs bounded prerequisite teaching and a separate practice world".into(),
            );
        }
        for c in &l.concepts {
            if c.title.trim().is_empty()
                || c.title.len() > 80
                || c.text.trim().is_empty()
                || c.text.len() > 1600
            {
                return Err("Invalid HSRP concept text".into());
            }
        }
        for w in &l.worked {
            if w.title.trim().is_empty()
                || w.title.len() > 96
                || w.why.trim().is_empty()
                || w.why.len() > 800
            {
                return Err("Invalid worked configuration explanation".into());
            }
            l.policy.authorize(&w.action)?;
        }
        l.policy.authorize(&l.experiment)?;
        l.policy.authorize(&l.repair)?;
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.shape()?;
        s.validate()?;
        self.policy.bind(&s.initial, &s.current, &s.history)?;
        self.lesson
            .policy
            .bind(&s.example_initial, &s.example, &s.example_history)?;
        s.draft.shape(&self.policy.initial)?;
        s.example_draft.shape(&self.lesson.policy.initial)?;
        if s.selected_example >= self.lesson.worked.len() {
            return Err("Unknown worked HSRP action".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let mut rows = draft_row(&s.draft, &self.policy, &s.current);
        rows.extend(self.policy.rows(&s.current));
        rows
    }
    pub fn practice_checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let mut rows = draft_row(&s.example_draft, &self.lesson.policy, &s.example);
        rows.extend(self.lesson.policy.rows(&s.example));
        rows
    }
    /// Declarative authoring admission additionally verifies the separate worked example.
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        let mut s = Session::new(self)?;
        for w in &self.lesson.worked {
            s.apply_action(self, &w.action, true)?;
        }
        if self.practice_checks(&s).iter().any(|r| !r.0) {
            return Err(
                "Worked HSRP practice must satisfy all current and failure outcomes".into(),
            );
        }
        s.apply_action(self, &self.lesson.experiment, true)?;
        if self.practice_checks(&s).iter().all(|r| r.0) {
            return Err("HSRP practice experiment must cause observable failure".into());
        }
        s.apply_action(self, &self.lesson.repair, true)?;
        if self.practice_checks(&s).iter().any(|r| !r.0) {
            return Err("HSRP practice repair must restore every outcome".into());
        }
        Ok(())
    }
}

fn draft_row(d: &Draft, p: &Policy, state: &core::State) -> Vec<(bool, String)> {
    match d.changes(p, state) {
        Ok(a) => vec![(
            a.is_empty(),
            if a.is_empty() {
                "Displayed configuration is applied".into()
            } else {
                format!(
                    "{} pending configuration changes; Apply or repair the draft",
                    a.len()
                )
            },
        )],
        Err(e) => vec![(false, format!("Unapplied configuration: {e}"))],
    }
}
