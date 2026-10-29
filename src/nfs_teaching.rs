//! Separate bounded NFS practice; inode metadata stays immutable.
use crate::{nfs as core, nfs_workspace as work, script_teaching::View};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
pub const MAX_LESSON: usize = 512 * 1024;
pub const MAX_LEARNING: usize = 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::string::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,2400>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionStep {
    #[serde(deserialize_with = "work::string::<_,160>")]
    pub title: String,
    #[serde(deserialize_with = "work::string::<_,800>")]
    pub explanation: String,
    pub action: core::Action,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "work::string::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "work::list::<_,core::Action,16>")]
    pub actions: Vec<core::Action>,
    #[serde(deserialize_with = "work::list::<_,bool,37>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lesson {
    pub definition: work::Definition,
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub example_actions: Vec<ActionStep>,
    pub try_change: Experiment,
    pub repair: Vec<core::Action>,
}
impl<'de> Deserialize<'de> for Lesson {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            definition: work::Definition,
            #[serde(deserialize_with = "work::string::<_,1600>")]
            prerequisite_note: String,
            #[serde(deserialize_with = "work::list::<_,Concept,6>")]
            concepts: Vec<Concept>,
            #[serde(deserialize_with = "work::string::<_,1600>")]
            example_description: String,
            #[serde(deserialize_with = "work::list::<_,ActionStep,16>")]
            example_actions: Vec<ActionStep>,
            try_change: Experiment,
            #[serde(deserialize_with = "work::list::<_,core::Action,16>")]
            repair: Vec<core::Action>,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            definition: r.definition,
            prerequisite_note: r.prerequisite_note,
            concepts: r.concepts,
            example_description: r.example_description,
            example_actions: r.example_actions,
            try_change: r.try_change,
            repair: r.repair,
        };
        if !l.bounded() {
            return Err(D::Error::custom("NFS Lesson admission differs"));
        }
        Ok(l)
    }
}
fn prose(s: &str, n: usize) -> bool {
    work::text(s, n) && !s.trim().is_empty()
}
/// Namespace separation is immutable context, not forbidden evolving values.
pub fn separate(a: &core::State, b: &core::State) -> bool {
    let a = a.inventory();
    let b = b.inventory();
    fn overlap(a: &str, b: &str) -> bool {
        a == b
            || a.strip_prefix(b).is_some_and(|s| s.starts_with('/'))
            || b.strip_prefix(a).is_some_and(|s| s.starts_with('/'))
    }
    let ids_a = a
        .servers
        .iter()
        .map(|s| s.id.as_str())
        .chain(a.clients.iter().map(|s| s.id.as_str()))
        .chain(a.profiles.iter().map(|s| s.id.as_str()));
    let ids_b = b
        .servers
        .iter()
        .map(|s| s.id.as_str())
        .chain(b.clients.iter().map(|s| s.id.as_str()))
        .chain(b.profiles.iter().map(|s| s.id.as_str()))
        .collect::<Vec<_>>();
    a.fixture != b.fixture
        && ids_a.into_iter().all(|s| !ids_b.contains(&s))
        && a.servers
            .iter()
            .map(|s| s.address)
            .chain(a.clients.iter().map(|c| c.address))
            .all(|ip| {
                !b.servers.iter().any(|s| s.address == ip)
                    && !b.clients.iter().any(|c| c.address == ip)
            })
        && a.roots.iter().all(|r| {
            !b.roots.iter().any(|q| {
                overlap(&q.path, &r.path)
                    || q.files
                        .iter()
                        .any(|f| r.files.iter().any(|g| f.path == g.path))
            })
        })
        && a.clients
            .iter()
            .flat_map(|c| &c.mountpoints)
            .all(|p| !b.clients.iter().any(|c| c.mountpoints.contains(p)))
}
impl Lesson {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LESSON {
            return Err("NFS Lesson raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NFS Lesson".into())
    }
    pub fn bounded(&self) -> bool {
        if self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.example_actions.is_empty()
            || self.example_actions.len() > 16
            || self.try_change.actions.is_empty()
            || self.try_change.actions.len() > 16
            || self.repair.is_empty()
            || self.repair.len() > 16
            || self.try_change.expected.len() > 37
            || self.try_change.expected.len() != self.definition.row_count()
        {
            return false;
        }
        if !prose(&self.prerequisite_note, 1600)
            || !prose(&self.example_description, 1600)
            || !prose(&self.try_change.description, 1600)
            || !self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 160) && prose(&c.text, 2400))
            || !self
                .example_actions
                .iter()
                .all(|a| prose(&a.title, 160) && prose(&a.explanation, 800))
        {
            return false;
        }
        let i = self.definition.initial.inventory();
        self.example_actions
            .iter()
            .map(|a| &a.action)
            .chain(&self.try_change.actions)
            .chain(&self.repair)
            .all(|a| work::action_shape(i, a) && work::within(a, core::MAX_ACTION))
            && self.definition.validate().is_ok()
            && work::within(self, MAX_LESSON)
    }
    pub fn separate_from(&self, s: &core::State) -> bool {
        self.bounded() && s.validate().is_ok() && separate(&self.definition.initial, s)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("NFS Lesson scalar/action/context/aggregate bound".into());
        }
        let d = &self.definition;
        let initial = d.checks(&d.initial);
        if initial.iter().all(|r| r.0) {
            return Err("NFS practice needs a substantive change".into());
        }
        let mut s = d.initial.clone();
        let mut session = work::Session::default();
        for a in &self.example_actions {
            let result = work::perform(d, &mut s, &mut session, &a.action)?;
            if matches!(result, core::Decision::Denied { .. }) {
                return Err("Worked NFS action was denied".into());
            }
        }
        if !d.checks(&s).iter().all(|r| r.0) {
            return Err("NFS example does not satisfy its independent goal".into());
        }
        // One bounded worked snapshot proves a real adverse change; no retained trial States.
        let worked = s.clone();
        for a in &self.try_change.actions {
            work::perform(d, &mut s, &mut session, a)?;
        }
        let rows = d.checks(&s).into_iter().map(|r| r.0).collect::<Vec<_>>();
        if s == worked || rows != self.try_change.expected || rows.iter().all(|v| *v) {
            return Err(
                "NFS experiment must change actual State and match every declared row".into(),
            );
        }
        for a in &self.repair {
            work::perform(d, &mut s, &mut session, a)?;
        }
        if !d.checks(&s).iter().all(|r| r.0) {
            return Err("NFS repair does not restore its stated outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub context: work::Definition,
    pub simulation: core::State,
    pub session: work::Session,
    pub selected_example: usize,
    pub checked: bool,
}
impl<'de> Deserialize<'de> for Learning {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            view: View,
            context: work::Definition,
            simulation: core::State,
            session: work::Session,
            selected_example: usize,
            checked: bool,
        }
        let r = Raw::deserialize(d)?;
        let l = Self {
            version: r.version,
            view: r.view,
            context: r.context,
            simulation: r.simulation,
            session: r.session,
            selected_example: r.selected_example,
            checked: r.checked,
        };
        if !l.valid() {
            return Err(D::Error::custom("NFS Learning admission differs"));
        }
        Ok(l)
    }
}
#[derive(Serialize)]
struct Proposed<'a> {
    version: u32,
    view: View,
    context: &'a work::Definition,
    simulation: &'a core::State,
    session: &'a work::Session,
    selected_example: usize,
    checked: bool,
}
#[cfg(test)]
std::thread_local! { static FILL_COPIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[derive(Serialize)]
struct FillSession<'a> {
    version: u32,
    server: &'a str,
    client: &'a str,
    mountpoint: &'a str,
    profile: &'a str,
    relative: &'a str,
    write_text: &'a str,
    entries: &'a [work::Entry],
}
impl<'a> From<&'a work::Session> for FillSession<'a> {
    fn from(s: &'a work::Session) -> Self {
        Self {
            version: s.version,
            server: &s.server,
            client: &s.client,
            mountpoint: &s.mountpoint,
            profile: &s.profile,
            relative: &s.relative,
            write_text: &s.write_text,
            entries: &s.entries,
        }
    }
}
impl FillSession<'_> {
    fn to_owned_session(&self) -> work::Session {
        work::Session {
            version: self.version,
            server: self.server.into(),
            client: self.client.into(),
            mountpoint: self.mountpoint.into(),
            profile: self.profile.into(),
            relative: self.relative.into(),
            write_text: self.write_text.into(),
            entries: self.entries.to_vec(),
        }
    }
}
fn compact_size<T: Serialize + ?Sized>(v: &T) -> Option<usize> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= MAX_LEARNING)
                .ok_or_else(|| std::io::Error::other("NFS fill count limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut c = Counter(0);
    serde_json::to_writer(&mut c, v).ok()?;
    Some(c.0)
}
impl Learning {
    pub fn from_json(raw: &[u8]) -> Result<Self, String> {
        if raw.len() > MAX_LEARNING {
            return Err("NFS Learning raw bound".into());
        }
        serde_json::from_slice(raw).map_err(|_| "Invalid bounded NFS Learning".into())
    }
    pub(crate) fn new_size(l: &Lesson, new: bool) -> Option<usize> {
        Self::new_size_view(
            l,
            if new {
                View::Introduction
            } else {
                View::Assignment
            },
        )
    }
    pub(crate) fn new_size_view(l: &Lesson, view: View) -> Option<usize> {
        if !l.bounded() {
            return None;
        }
        let session = work::Session::default();
        work::size(
            &Proposed {
                version: 1,
                view,
                context: &l.definition,
                simulation: &l.definition.initial,
                session: &session,
                selected_example: 0,
                checked: false,
            },
            MAX_LEARNING,
        )
    }
    pub(crate) fn presentation_size(&self, view: View, selected_example: usize) -> Option<usize> {
        work::size(
            &Proposed {
                version: self.version,
                view,
                context: &self.context,
                simulation: &self.simulation,
                session: &self.session,
                selected_example,
                checked: self.checked,
            },
            MAX_LEARNING,
        )
    }
    pub(crate) fn proposed_size(
        &self,
        simulation: &core::State,
        session: &work::Session,
    ) -> Option<usize> {
        work::size(
            &Proposed {
                version: self.version,
                view: self.view,
                context: &self.context,
                simulation,
                session,
                selected_example: self.selected_example,
                checked: self.checked && simulation == &self.simulation,
            },
            MAX_LEARNING,
        )
    }
    pub fn new(l: &Lesson) -> Result<Self, String> {
        if !l.bounded() {
            return Err("NFS Lesson differs".into());
        }
        let session = work::Session::default();
        if !work::within(
            &Proposed {
                version: 1,
                view: View::Introduction,
                context: &l.definition,
                simulation: &l.definition.initial,
                session: &session,
                selected_example: 0,
                checked: false,
            },
            MAX_LEARNING,
        ) {
            return Err("NFS new practice aggregate before copying".into());
        }
        Ok(Self {
            version: 1,
            view: View::Introduction,
            context: l.definition.clone(),
            simulation: l.definition.initial.clone(),
            session,
            selected_example: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.selected_example < 16
            && self.context.validate().is_ok()
            && self.context.same_assignment(&self.simulation)
            && self.simulation.validate().is_ok()
            && self.session.valid_for(&self.simulation)
            && work::within(self, MAX_LEARNING)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        l.bounded()
            && self.valid()
            && self.context == l.definition
            && self.selected_example < l.example_actions.len()
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(
                false,
                "NFS practice differs from its authored lesson".into(),
            )];
        }
        self.context.checks(&self.simulation)
    }
    pub(crate) fn commit(
        &mut self,
        simulation: core::State,
        session: work::Session,
    ) -> Result<(), String> {
        let changed = simulation != self.simulation;
        let checked = self.checked && !changed;
        if !work::within(
            &Proposed {
                version: self.version,
                view: self.view,
                context: &self.context,
                simulation: &simulation,
                session: &session,
                selected_example: self.selected_example,
                checked,
            },
            MAX_LEARNING,
        ) {
            return Err("NFS practice aggregate; old State and Session retained".into());
        }
        self.simulation = simulation;
        self.session = session;
        self.checked = checked;
        Ok(())
    }
    pub fn perform(&mut self, l: &Lesson, a: &core::Action) -> Result<core::Decision, String> {
        if !self.valid_for(l) {
            return Err("Foreign NFS practice".into());
        }
        let (s, t, d) = work::propose(&self.context, &self.simulation, &self.session, a)?;
        self.commit(s, t)?;
        Ok(d)
    }
    /// Fill editor/request fields only. Save/Apply/Mount/Read/Write still require their explicit control.
    pub fn fill_selected(&mut self, l: &Lesson) -> Result<(), String> {
        self.fill_with_budget(l, |_| true)
    }
    pub(crate) fn fill_with_budget(
        &mut self,
        l: &Lesson,
        mut admit: impl FnMut(usize) -> bool,
    ) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign NFS practice".into());
        }
        let a = &l.example_actions[self.selected_example].action;
        if !work::action_shape(self.simulation.inventory(), a) {
            return Err("NFS example action context differs".into());
        }
        if let core::Action::SetExportsDraft { server, text } = a {
            let old = &self
                .simulation
                .runtime()
                .servers
                .iter()
                .find(|s| s.server == *server)
                .expect("admitted editor server")
                .draft;
            let bytes = self.editor_fill_preflight(old, text)?;
            if !admit(bytes) {
                return Err(
                    "NFS containing Attempt exceeds2MiB before copying; previous fields retained"
                        .into(),
                );
            }
            #[cfg(test)]
            FILL_COPIES.with(|n| n.set(n.get() + 1));
            let mut simulation = self.simulation.clone();
            simulation.apply(a).map_err(|e| work::notice(e.message))?;
            return self.commit(simulation, self.session.clone());
        }
        if let core::Action::SetFstabDraft { client, text } = a {
            let old = &self
                .simulation
                .runtime()
                .clients
                .iter()
                .find(|s| s.client == *client)
                .expect("admitted editor client")
                .draft;
            let bytes = self.editor_fill_preflight(old, text)?;
            if !admit(bytes) {
                return Err(
                    "NFS containing Attempt exceeds2MiB before copying; previous fields retained"
                        .into(),
                );
            }
            #[cfg(test)]
            FILL_COPIES.with(|n| n.set(n.get() + 1));
            let mut simulation = self.simulation.clone();
            simulation.apply(a).map_err(|e| work::notice(e.message))?;
            return self.commit(simulation, self.session.clone());
        }
        let mut proposed = FillSession::from(&self.session);
        match a {
            core::Action::SaveExportsDraft { server }
            | core::Action::ApplySavedExports { server } => proposed.server = server,
            core::Action::SaveFstabDraft { client } => proposed.client = client,
            core::Action::MountExplicit {
                client,
                specification,
            } => {
                proposed.client = client;
                proposed.server = &specification.server;
                proposed.mountpoint = &specification.mountpoint;
            }
            core::Action::MountSaved { client, mountpoint }
            | core::Action::Unmount { client, mountpoint } => {
                proposed.client = client;
                proposed.mountpoint = mountpoint;
            }
            core::Action::ReadExisting {
                client,
                mountpoint,
                profile,
                relative,
            }
            | core::Action::WriteExisting {
                client,
                mountpoint,
                profile,
                relative,
                ..
            } => {
                proposed.client = client;
                proposed.mountpoint = mountpoint;
                proposed.profile = profile;
                proposed.relative = relative;
                if let core::Action::WriteExisting { text, .. } = a {
                    proposed.write_text = text;
                }
            }
            core::Action::SetExportsDraft { .. } | core::Action::SetFstabDraft { .. } => {
                unreachable!("editor paths preflighted above")
            }
        }
        // Reconcile only dependent VIEW selections. Action-owned fields win; raw request/write text stays exact.
        if proposed.client != self.session.client {
            let client = self
                .simulation
                .inventory()
                .clients
                .iter()
                .find(|c| c.id == proposed.client)
                .expect("admitted client selection");
            if !client.profiles.iter().any(|p| p == proposed.profile) {
                proposed.profile = "";
            }
            if !client.mountpoints.iter().any(|p| p == proposed.mountpoint) {
                proposed.mountpoint = "";
            }
        }
        #[derive(Serialize)]
        struct FillLearning<'a> {
            version: u32,
            view: View,
            context: &'a work::Definition,
            simulation: &'a core::State,
            session: &'a FillSession<'a>,
            selected_example: usize,
            checked: bool,
        }
        let learning = FillLearning {
            version: self.version,
            view: self.view,
            context: &self.context,
            simulation: &self.simulation,
            session: &proposed,
            selected_example: self.selected_example,
            checked: self.checked,
        };
        if !work::within(&proposed, work::MAX_SESSION)
            || work::size(&learning, MAX_LEARNING).is_none_or(|n| !admit(n))
        {
            return Err(
                "NFS filled Session or practice aggregate; previous fields retained".into(),
            );
        }
        #[cfg(test)]
        FILL_COPIES.with(|n| n.set(n.get() + 1));
        self.session = proposed.to_owned_session();
        Ok(())
    }
    fn editor_fill_preflight(&self, old: &str, new: &str) -> Result<usize, String> {
        let changed = old != new;
        let old = compact_size(old).ok_or("NFS previous editor aggregate")?;
        let new = compact_size(new).ok_or("NFS proposed editor aggregate")?;
        let state = compact_size(&self.simulation).ok_or("NFS current State aggregate")?;
        let learning = compact_size(self).ok_or("NFS current practice aggregate")?;
        let delta = usize::from(self.checked && changed);
        // Replacing exactly one JSON string changes no State structure. A changed checked true -> false adds one byte.
        let state = state.checked_sub(old).and_then(|n| n.checked_add(new));
        let learning = learning
            .checked_sub(old)
            .and_then(|n| n.checked_add(new))
            .and_then(|n| n.checked_add(delta));
        if state.is_none_or(|n| n > core::MAX_STATE) || learning.is_none_or(|n| n > MAX_LEARNING) {
            return Err(
                "NFS editor State or practice aggregate before copying; previous fields retained"
                    .into(),
            );
        }
        learning.ok_or_else(|| "NFS editor count overflow".into())
    }
    pub fn reset(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign NFS practice".into());
        }
        let mut fresh = Self::new(l)?;
        fresh.view = View::Example;
        *self = fresh;
        Ok(())
    }
}

/// Full Scenario/version-aware preparation; no work is copied before cumulative admission.
pub fn prepare(
    a: &mut crate::progress::Attempt,
    s: &crate::pack::Scenario,
    new_attempt: bool,
) -> Result<(), String> {
    if crate::workspace::is_one_off_workspace(a) {
        return Err("One-off workspace belongs to its own activity; previous work retained".into());
    }

    work::prepare_attempt(a, s, new_attempt)
}

#[cfg(test)]
mod fill_preflight_tests {
    use super::*;
    fn source(case: usize) -> Lesson {
        let v: serde_json::Value =
            serde_json::from_str(include_str!("../tests/fixtures/nfs-workbench-v1.json")).unwrap();
        serde_json::from_value(v["cases"][case]["lesson"].clone()).unwrap()
    }
    fn replace(v: &mut serde_json::Value, old: &str, new: &str) {
        match v {
            serde_json::Value::String(s) if s == old => *s = new.into(),
            serde_json::Value::Array(a) => {
                for x in a {
                    replace(x, old, new)
                }
            }
            serde_json::Value::Object(o) => {
                for x in o.values_mut() {
                    replace(x, old, new)
                }
            }
            _ => {}
        }
    }
    #[test]
    fn actual_fill_counts_no_owned_copies_on_escaped_session_refusal() {
        let mut v = serde_json::to_value(source(1)).unwrap();
        replace(&mut v, "gallery1 published result\n", &"\u{1}".repeat(4096));
        let l: Lesson = serde_json::from_value(v).unwrap();
        let mut p = Learning::new(&l).unwrap();
        p.selected_example = l
            .example_actions
            .iter()
            .position(|a| matches!(a.action, core::Action::WriteExisting { .. }))
            .unwrap();
        p.session.entries = (0..16)
            .map(|n| work::Entry {
                operation: format!("Imported display {n}"),
                detail: "x".repeat(3900),
            })
            .collect();
        assert!(p.valid_for(&l));
        let before = serde_json::to_value(&p).unwrap();
        FILL_COPIES.with(|n| n.set(0));
        assert!(p.fill_selected(&l).is_err());
        assert_eq!(FILL_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&p).unwrap() == before);
        p.session.entries.clear();
        FILL_COPIES.with(|n| n.set(0));
        p.fill_selected(&l).unwrap();
        assert_eq!(FILL_COPIES.with(|n| n.get()), 1);
        assert_eq!(p.session.write_text, "\u{1}".repeat(4096));
    }
    #[test]
    fn actual_editor_fill_admits_exact_replacement_bytes_before_state_copy() {
        let mut l = source(0);
        let core::Action::SetExportsDraft { text, .. } = &mut l.example_actions[0].action else {
            panic!()
        };
        while text.len() < core::MAX_DOCUMENT {
            let remaining = core::MAX_DOCUMENT - text.len();
            if remaining == 1 {
                text.push('#');
            } else {
                let tabs = (remaining - 2).min(510);
                text.push('#');
                text.extend(std::iter::repeat_n('\t', tabs));
                text.push('\n');
            }
        }
        l.validate().unwrap();
        let mut p = Learning::new(&l).unwrap();
        let mut runtime = p.simulation.runtime().clone();
        for f in &mut runtime.contents {
            f.text = "\u{1}".repeat(core::MAX_FILE);
        }
        let inventory = p.simulation.inventory().clone();
        let mut last = core::State::new(inventory.clone(), runtime.clone()).unwrap();
        let document = ("\t".repeat(511) + "\n").repeat(16);
        let mut refused = false;
        for n in 0..9 {
            if n < 8 {
                if n % 2 == 0 {
                    runtime.clients[n / 2].draft = document.clone();
                } else {
                    runtime.clients[n / 2].saved = document.clone();
                }
            } else {
                runtime.servers[0].saved = document.clone();
            }
            match core::State::new(inventory.clone(), runtime.clone()) {
                Ok(next) => last = next,
                Err(e) => {
                    assert_eq!(e.kind, core::FaultKind::Limit);
                    refused = true;
                    break;
                }
            }
        }
        assert!(refused, "fixture reaches the actual escaped State frontier");
        p.simulation = last;
        assert!(p.valid_for(&l));
        let before = serde_json::to_value(&p).unwrap();
        FILL_COPIES.with(|n| n.set(0));
        let e = p.fill_selected(&l).unwrap_err();
        assert!(e.contains("before copying"));
        assert_eq!(FILL_COPIES.with(|n| n.get()), 0);
        assert!(serde_json::to_value(&p).unwrap() == before);
        let mut low = Learning::new(&l).unwrap();
        let old_applied = low.simulation.runtime().servers[0].applied.clone();
        FILL_COPIES.with(|n| n.set(0));
        low.fill_selected(&l).unwrap();
        assert_eq!(FILL_COPIES.with(|n| n.get()), 1);
        assert_eq!(low.simulation.runtime().servers[0].applied, old_applied);
        assert_eq!(
            low.simulation.runtime().servers[0].draft.len(),
            core::MAX_DOCUMENT
        );
    }
}
