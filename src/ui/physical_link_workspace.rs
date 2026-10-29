//! Proposed focused native physical-uplink workspace. Source-only; no host I/O.
use super::*;
use dlms_learning_lab::{
    engine::{Goal, State},
    physical_link as core, physical_link_assignment as assignment, physical_link_workspace as work,
    script_teaching::View,
};
pub(super) mod memo;
#[derive(Default)]
pub(super) struct Metrics {
    pub tabs: Vec<(View, egui::Rect)>,
    pub title: Option<egui::Rect>,
    pub goal: Option<egui::Rect>,
    pub expanders: Vec<(String, egui::Rect, bool, egui::Rect)>,
    pub first_failure: Option<egui::Rect>,
    pub notice: Option<egui::Rect>,
    pub controls: Vec<(String, egui::Rect, egui::Id)>,
    pub choices: Vec<(String, String, egui::Rect, bool)>,
    pub choice_clips: Vec<(String, String, egui::Rect)>,
    pub numbers: Vec<(usize, egui::Rect, egui::Id)>,
    pub topology: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
    pub rows: Vec<(usize, bool, egui::Rect)>,
    pub detail_rows: Vec<(usize, bool, egui::Rect)>,
    pub details_header: Option<egui::Rect>,
    pub complete: bool,
    pub rejected: bool,
}
#[derive(Default)]
pub(super) struct Presentation {
    memo: memo::Memo,
    owner: Option<(String, u32, bool, u64)>,
    draft: Option<assignment::Draft>,
    numbers: [String; 2],
    fields: [Option<egui::Id>; 2],
    retired_fields: [Option<egui::Id>; 2],
    input_generation: u64,
    scope_generation: u64,
    blocked_gesture: bool,
    veto_frame: Option<u64>,
    notice: Option<String>,
    logs: Vec<String>,
    hint: usize,
}
impl Presentation {
    pub(super) fn displayed_draft(&self) -> Option<&assignment::Draft> {
        self.draft.as_ref()
    }
    pub(super) fn sample_values(&self) -> &[String; 2] {
        &self.numbers
    }
    pub(super) fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }
    pub(super) fn rows(
        &self,
        a: &Attempt,
        s: &Scenario,
        practice: bool,
    ) -> Option<&[(bool, String)]> {
        self.memo.rows(a, s, practice)
    }
    pub(super) fn native_counts(&self) -> (usize, usize, usize) {
        (
            self.memo.counters.admissions,
            self.memo.counters.copies,
            self.memo.counters.grades,
        )
    }
    #[cfg(test)]
    pub(in crate::ui) fn observed(&self) -> (usize, usize, usize, usize) {
        (
            self.memo.counters.admissions,
            self.memo.counters.copies,
            self.memo.counters.grades,
            self.memo.entries_count(),
        )
    }
    pub(super) fn with_index(certificates: completion_index::Handle) -> Self {
        let mut p = Self::default();
        p.memo.certificates = certificates;
        p
    }
    pub fn clear(&mut self) {
        self.owner = None;
        self.draft = None;
        for (n, id) in self.fields.iter().enumerate() {
            if id.is_some() {
                self.retired_fields[n] = *id;
            }
        }
        self.fields = [None, None];
        self.input_generation = self.input_generation.saturating_add(1);
        self.scope_generation = self.scope_generation.saturating_add(1);
        self.notice = None;
        self.hint = 0;
        self.logs.clear();
    }
    pub fn reset_all(&mut self) {
        self.clear();
        self.memo.clear();
    }
    fn append(&mut self, text: String) {
        let mut text = text;
        while text.len() > 600 {
            text.pop();
        }
        self.logs.push(text);
        while self.logs.len() > 80 || self.logs.iter().map(String::len).sum::<usize>() > 48 * 1024 {
            self.logs.remove(0);
        }
    }
    fn bind(
        &mut self,
        ctx: &egui::Context,
        s: &Scenario,
        practice: bool,
        draft: &assignment::Draft,
    ) {
        for id in self.retired_fields.iter_mut() {
            if let Some(old) = id.take() {
                ctx.data_mut(|d| {
                    d.remove::<egui::text_edit::TextEditState>(old);
                });
            }
        }
        let owner = (
            s.id.clone(),
            s.version,
            practice,
            self.memo.source_generation(s).unwrap_or(0),
        );
        if self.owner.as_ref() != Some(&owner) {
            for id in self.fields.iter().flatten() {
                ctx.data_mut(|d| {
                    d.remove::<egui::text_edit::TextEditState>(*id);
                });
            }
            self.input_generation = self.input_generation.saturating_add(1);
            self.owner = Some(owner);
            self.numbers = ["12".into(), "18".into()];
            self.fields = [None, None];
            self.notice = None;
            self.hint = 0;
            self.logs.clear();
            self.draft = Some(draft.clone());
        } else if self.draft.as_ref() != Some(draft) {
            self.draft = Some(draft.clone());
        }
    }
}
fn status(report: &Result<core::LinkReport, String>) -> String {
    match report {
        Err(_) => "Outside the supported model".into(),
        Ok(r) => match r.state {
            core::LinkState::Up => "Up · compatible mode".into(),
            core::LinkState::DegradedDuplex => "Connected · duplex differs".into(),
            core::LinkState::AdministrativelyDown => "Stopped at an interface".into(),
            core::LinkState::Disconnected => "Declared disconnected".into(),
            core::LinkState::NoCommonAbility => "No shared operating rate".into(),
            core::LinkState::OutsideModel => match r.qualification {
                core::Qualification::TooShortForRoute => "Cable is shorter than the route".into(),
                core::Qualification::IncompatibleProfile => "The end profiles differ".into(),
                core::Qualification::IncompatibleCable => "Cable and profiles differ".into(),
                core::Qualification::OutsideReach => "Outside the supplied profile reach".into(),
                core::Qualification::Qualified => "Unsupported operating setting".into(),
            },
        },
    }
}
fn operating(value: Option<core::Operational>) -> String {
    value.map_or_else(
        || "No current operating mode".into(),
        |v| format!("{} · {}", rate(v.ability.rate), duplex(v.ability.duplex)),
    )
}
pub(super) fn profile(v: core::Profile) -> &'static str {
    match v {
        core::Profile::CopperRj45 => "Copper RJ45",
        core::Profile::SxOm2 => "SX over OM2 fiber",
        core::Profile::LxLhOs2 => "LX/LH over OS2 fiber",
    }
}
pub(super) fn rate(v: core::Rate) -> &'static str {
    match v {
        core::Rate::M10 => "10 Mb/s",
        core::Rate::M100 => "100 Mb/s",
        core::Rate::M1000 => "1 Gb/s",
    }
}
pub(super) fn duplex(v: core::Duplex) -> &'static str {
    match v {
        core::Duplex::Half => "Half duplex",
        core::Duplex::Full => "Full duplex",
    }
}
fn action(ui: &mut egui::Ui, m: &mut Metrics, key: &str, label: &str, main: bool) -> bool {
    let r = ui.add(if main {
        primary(ui, label)
    } else {
        button(label)
    });
    name_control(&r, label);
    m.controls.push((key.into(), r.rect, r.id));
    r.clicked()
}
fn chooser<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    id: &str,
    current: &mut T,
    values: &[(T, String, bool)],
    width: f32,
) -> bool {
    let text = values
        .iter()
        .find(|(v, _, _)| v == current)
        .map(|(_, s, _)| s.as_str())
        .unwrap_or("Unsupported draft value");
    let mut changed = false;
    let response = controls::dropdown(ui, id, width)
        .selected_text(text)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (v, label, enabled) in values {
                let r = ui.add_enabled(*enabled, egui::Button::selectable(*current == *v, label));
                m.choices.push((id.into(), label.clone(), r.rect, *enabled));
                m.choice_clips
                    .push((id.into(), label.clone(), ui.clip_rect()));
                if r.clicked() {
                    *current = *v;
                    changed = true;
                }
            }
        });
    name_control(&response.response, id);
    m.controls
        .push((id.into(), response.response.rect, response.response.id));
    changed
}
fn port_editor(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    id: &str,
    spec: &core::PortSpec,
    value: &mut core::PortConfig,
) -> bool {
    let mut changed = false;
    ui.strong(&spec.label);
    controls::form_row(ui, "Interface enabled", |ui, _| {
        let r = ui.checkbox(&mut value.admin_up, "Up");
        name_control(&r, format!("{} enabled", spec.label));
        m.controls.push((format!("up:{id}"), r.rect, r.id));
        changed |= r.changed();
    });
    controls::form_row(ui, "Media profile", |ui, width| {
        let options = spec
            .profiles
            .iter()
            .map(|p| (*p, profile(*p).into(), true))
            .collect::<Vec<_>>();
        changed |= chooser(
            ui,
            m,
            &format!("profile:{id}"),
            &mut value.profile,
            &options,
            width,
        );
    });
    let fixed = spec
        .abilities
        .iter()
        .copied()
        .filter(|a| value.profile != core::Profile::CopperRj45 || a.rate != core::Rate::M1000)
        .filter(|a| {
            value.profile == core::Profile::CopperRj45
                || (a.rate == core::Rate::M1000 && a.duplex == core::Duplex::Full)
        })
        .collect::<Vec<_>>();
    let mut mode = usize::from(matches!(value.negotiation, core::Negotiation::Fixed { .. }));
    controls::form_row(ui, "Choose operating mode", |ui, width| {
        if chooser(
            ui,
            m,
            &format!("mode:{id}"),
            &mut mode,
            &[
                (0, "Auto".into(), value.profile == core::Profile::CopperRj45),
                (1, "Fixed".into(), !fixed.is_empty()),
            ],
            width,
        ) {
            changed = true;
            value.negotiation = if mode == 0 {
                core::Negotiation::Auto {
                    advertise: spec.abilities.clone(),
                }
            } else {
                core::Negotiation::Fixed {
                    ability: *fixed
                        .last()
                        .expect("Enabled fixed choice has a supported ability"),
                }
            };
        }
    });
    match &mut value.negotiation {
        core::Negotiation::Fixed { ability } => {
            controls::form_row(ui, "Fixed rate", |ui, width| {
                let options = [core::Rate::M10, core::Rate::M100, core::Rate::M1000]
                    .into_iter()
                    .map(|r| (r, rate(r).into(), fixed.iter().any(|a| a.rate == r)))
                    .collect::<Vec<_>>();
                changed |= chooser(
                    ui,
                    m,
                    &format!("rate:{id}"),
                    &mut ability.rate,
                    &options,
                    width,
                );
            });
            controls::form_row(ui, "Fixed duplex", |ui, width| {
                let options = [core::Duplex::Half, core::Duplex::Full]
                    .into_iter()
                    .map(|d| {
                        (
                            d,
                            duplex(d).into(),
                            fixed
                                .iter()
                                .any(|a| a.rate == ability.rate && a.duplex == d),
                        )
                    })
                    .collect::<Vec<_>>();
                changed |= chooser(
                    ui,
                    m,
                    &format!("duplex:{id}"),
                    &mut ability.duplex,
                    &options,
                    width,
                );
            });
        }
        core::Negotiation::Auto { advertise } => {
            if value.profile != core::Profile::CopperRj45 {
                ui.label("This supplied optical profile uses Fixed 1 Gb/s full duplex.");
            }
            let expander=controls::collapsing_header(ui,"Optional: advertised Auto abilities").id_salt(("advertise",id)).show(ui,|ui|{ui.add(egui::Label::new("Auto normally offers the supported modes. Both Auto ends need a shared choice.").wrap());let choices=spec.abilities.union(advertise).copied().collect::<Vec<_>>();for a in &choices{let mut selected=advertise.contains(a);let r=ui.checkbox(&mut selected,format!("{} · {}",rate(a.rate),duplex(a.duplex)));m.controls.push((format!("advertise:{id}:{:?}:{:?}",a.rate,a.duplex),r.rect,r.id));if r.changed(){changed=true;if selected{advertise.insert(*a);}else{advertise.remove(a);}}}});
            m.expanders.push((
                format!("advertisement:{id}"),
                expander.header_response.rect,
                expander.header_response.enabled(),
                ui.clip_rect(),
            ));
        }
    }
    changed
}
struct SmallBuffer<'a> {
    text: &'a mut String,
    before: Option<String>,
    received: bool,
    rejected: bool,
}
impl SmallBuffer<'_> {
    fn remember(&mut self) {
        if self.before.is_none() {
            self.before = Some(self.text.clone());
        }
    }
    fn rollback(&mut self) {
        if let Some(s) = self.before.take() {
            *self.text = s;
        }
    }
}
impl egui::TextBuffer for SmallBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<SmallBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.text
    }
    fn insert_text(&mut self, s: &str, at: usize) -> usize {
        if s.is_empty() || self.rejected {
            return 0;
        }
        self.received = true;
        if self.text.len().saturating_add(s.len()) > 4 || !s.bytes().all(|b| b.is_ascii_digit()) {
            self.rejected = true;
            return 0;
        }
        self.remember();
        let byte = self
            .text
            .char_indices()
            .nth(at)
            .map_or(self.text.len(), |(n, _)| n);
        self.text.insert_str(byte, s);
        s.chars().count()
    }
    fn delete_char_range(&mut self, r: std::ops::Range<usize>) {
        let at = |n| {
            self.text
                .char_indices()
                .nth(n)
                .map_or(self.text.len(), |(p, _)| p)
        };
        let (start, end) = (at(r.start), at(r.end));
        if !self.rejected && start < end {
            self.remember();
            self.text.replace_range(start..end, "");
        }
    }
}
fn pending(ctx: &egui::Context, fields: &[Option<egui::Id>; 2], focus: Option<egui::Id>) -> bool {
    fields.iter().any(|id|id.is_some()&&*id==focus)&&ctx.input(|i|i.events.iter().any(|e|matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty()&&t!="\n"&&t!="\r")))
}
fn number(
    ui: &mut egui::Ui,
    p: &mut Presentation,
    m: &mut Metrics,
    n: usize,
    entry_focus: Option<egui::Id>,
) -> bool {
    let id = egui::Id::new(("physical-number", &p.owner, p.input_generation, n));
    let pending=entry_focus==Some(id)&&ui.input(|i|i.events.iter().any(|e|matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty()&&t!="\n"&&t!="\r")));
    let editing = entry_focus == Some(id)
        && ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::Cut))
        });
    let saved = (pending || editing)
        .then(|| {
            egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
                let undo = state.undoer();
                (state, undo)
            })
        })
        .flatten();
    let mut buffer = SmallBuffer {
        text: &mut p.numbers[n],
        before: None,
        received: false,
        rejected: false,
    };
    let width = controls::button_width(ui, "1024");
    let r = ui.add(
        controls::singleline(ui, &mut buffer)
            .id(id)
            .desired_width(width)
            .hint_text("12"),
    );
    name_control(
        &r,
        if n == 0 {
            "Frames left to right"
        } else {
            "Frames right to left"
        },
    );
    m.numbers.push((n, r.rect, r.id));
    p.fields[n] = Some(id);
    let after_return=pending&&ui.input(|input|{let mut returned=false;input.events.iter().any(|e|{if matches!(e,egui::Event::Key{key:egui::Key::Enter,pressed:true,modifiers,..}if modifiers.matches_logically(egui::Modifiers::NONE)){returned=true;}returned&&matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty()&&t!="\n"&&t!="\r")})});
    let rejected = buffer.rejected || (pending && (!buffer.received || after_return));
    if rejected {
        buffer.rollback();
        if let Some((mut state, undo)) = saved {
            state.set_undoer(undo);
            state.store(ui.ctx(), id);
        }
    }
    rejected
}
fn scope_state(a: &Attempt, practice: bool) -> &core::State {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .current
    } else {
        let State::PhysicalLink { simulation } = &a.state else {
            unreachable!()
        };
        simulation
    }
}
fn scope_draft(a: &Attempt, practice: bool) -> &assignment::Draft {
    if practice {
        &a.workspace
            .as_ref()
            .unwrap()
            .physical_link_learning
            .as_ref()
            .unwrap()
            .draft
    } else {
        a.workspace
            .as_ref()
            .unwrap()
            .physical_link_draft
            .as_ref()
            .unwrap()
    }
}
fn scope_view(a: &Attempt) -> View {
    if a.challenge {
        View::Assignment
    } else {
        a.workspace
            .as_ref()
            .and_then(|w| w.physical_link_learning.as_ref())
            .map_or(View::Assignment, |l| l.view)
    }
}
fn checked(a: &Attempt, practice: bool) -> bool {
    a.workspace.as_ref().is_some_and(|w| {
        if practice {
            w.physical_link_learning.as_ref().is_some_and(|l| l.checked)
        } else {
            w.checked
        }
    })
}
fn definition(s: &Scenario, practice: bool) -> &assignment::Definition {
    if practice {
        &s.physical_link_teaching.as_ref().unwrap().practice
    } else {
        let Goal::PhysicalLink { definition } = &s.goal else {
            unreachable!()
        };
        definition
    }
}
fn results(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    d: &assignment::Definition,
    rows: &[(bool, String)],
) {
    let mut row = |ui: &mut egui::Ui, n: usize, label: &str, ok: bool| {
        let r = ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{label} · {}",
                    if ok { "Pass" } else { "Needs work" }
                ))
                .color(if ok {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            )
            .wrap(),
        );
        m.rows.push((n, ok, r.rect));
    };
    if let Some((ok, _)) = rows.first() {
        row(ui, 0, "Applied configuration", *ok);
    }
    let prefix = 2 + d.protected_ports.len() + d.protected_links.len() + d.retired_ports.len();
    let mut at = prefix;
    for requirement in &d.requirements {
        let end = at + 2 + requirement.probes.len();
        if let Some(group) = rows.get(at..end) {
            let ok = group.iter().all(|r| r.0);
            row(ui, at, &requirement.label, ok);
        }
        at = end;
    }
    if !d.retired_ports.is_empty() {
        let start = 2 + d.protected_ports.len() + d.protected_links.len();
        if let Some(group) = rows.get(start..prefix) {
            row(
                ui,
                start,
                "Old interfaces retired",
                group.iter().all(|r| r.0),
            );
        }
    }
    let preserved = 2 + d.protected_ports.len() + d.protected_links.len();
    if preserved > 2
        && let Some(group) = rows.get(2..preserved)
    {
        row(
            ui,
            2,
            "Other working links preserved",
            group.iter().all(|r| r.0),
        );
    }
    if let Some((_, why)) = rows.iter().find(|r| !r.0) {
        m.first_failure = Some(ui.add(egui::Label::new(why).wrap()).rect);
    }
    let response = controls::collapsing_header(ui, "All objective details").show(ui, |ui| {
        for (n, (ok, text)) in rows.iter().enumerate() {
            let r = ui.add(
                egui::Label::new(format!(
                    "{} · {text}",
                    if *ok { "Pass" } else { "Needs work" }
                ))
                .wrap(),
            );
            m.detail_rows.push((n, *ok, r.rect));
        }
    });
    m.details_header = Some(response.header_response.rect);
}
#[derive(Default)]
struct Intent {
    view: Option<View>,
    fill: bool,
    apply: bool,
    check: bool,
    probe: Option<(u32, u32)>,
    clear: Vec<String>,
    reset: bool,
}
fn editor(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    practice: bool,
) -> bool {
    let state = scope_state(a, practice);
    let d = definition(s, practice);
    let shadow = p.draft.as_mut().unwrap();
    let mut changed = false;
    let targets = d
        .requirements
        .iter()
        .map(|r| (r.link.as_str(), r.label.as_str()))
        .collect::<Vec<_>>();
    if targets.len() > 1 {
        controls::form_row(ui, "Required link", |ui, width| {
            let mut selected = shadow.selected_link.clone();
            let selected_text = targets
                .iter()
                .find(|(id, _)| *id == selected)
                .map_or(selected.as_str(), |(_, label)| *label);
            let response = controls::dropdown(ui, "physical-link", width)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    for (id, label) in &targets {
                        let r = ui.selectable_value(&mut selected, (*id).into(), *label);
                        m.choices
                            .push(("required-link".into(), (*id).into(), r.rect, true));
                        if r.changed() {
                            changed = true;
                        }
                    }
                });
            m.controls.push((
                "required-link".into(),
                response.response.rect,
                response.response.id,
            ));
            shadow.selected_link = selected;
        });
    }
    let selected = if state.topology().links.contains_key(&shadow.selected_link) {
        shadow.selected_link.clone()
    } else {
        d.requirements[0].link.clone()
    };
    if shadow.selected_link != selected {
        shadow.selected_link = selected.clone();
        changed = true;
    }
    let selected = selected.as_str();
    let link = &state.topology().links[selected];
    let report = p
        .memo
        .link(a, s, practice, selected)
        .expect("Admitted memo caches every supplied link");
    let text = status(report);
    let cable = &state.topology().cables[&state.configuration().cables[selected]];
    let caption = format!(
        "Applied: {} · {} m · route needs {} m. {text}",
        cable.label, cable.length_m, link.required_span_m
    );
    let left = &state.topology().ports[&link.a];
    let right = &state.topology().ports[&link.b];
    let start = ui.next_widget_position();
    let left_mode = operating(report.as_ref().ok().and_then(|r| r.a));
    let right_mode = operating(report.as_ref().ok().and_then(|r| r.b));
    controls::topology_pair(
        ui,
        [&left.label, &left.device, &left_mode],
        [&right.label, &right.device, &right_mode],
        if report
            .as_ref()
            .is_ok_and(|r| r.state == core::LinkState::Up)
        {
            palette(ui).success
        } else {
            palette(ui).error
        },
        &caption,
    );
    m.topology = Some(egui::Rect::from_min_max(start, ui.min_rect().max));
    controls::form_row(ui, "Draft cable", |ui, width| {
        let mut current = shadow.configuration.cables[selected].clone();
        let label = state
            .topology()
            .cables
            .get(&current)
            .map_or("Unknown saved cable", |c| c.label.as_str());
        let r = controls::dropdown(ui, "physical-cable", width)
            .selected_text(label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (id, cable) in &state.topology().cables {
                    let available = true;
                    let label = format!("{} · {} m", cable.label, cable.length_m);
                    let r =
                        ui.add_enabled(available, egui::Button::selectable(&current == id, &label));
                    m.choices
                        .push(("cable".into(), id.clone(), r.rect, available));
                    m.choice_clips
                        .push(("cable".into(), id.clone(), ui.clip_rect()));
                    if r.clicked() {
                        current = id.clone();
                        changed = true;
                    }
                }
            });
        m.controls
            .push(("cable".into(), r.response.rect, r.response.id));
        shadow.configuration.cables.insert(selected.into(), current);
    });
    let expander=controls::collapsing_header(ui,"Other supplied cable assignments · Optional").show(ui,|ui|{ui.add(egui::Label::new("A cable can serve only one applied link. You may stage a complete two-link swap before applying; duplicate draft choices must be resolved first.").wrap());for other in state.topology().links.keys().filter(|id|id.as_str()!=selected){controls::form_row(ui,other,|ui,width|{let mut current=shadow.configuration.cables[other].clone();let selected_text=state.topology().cables.get(&current).map_or("Unknown saved cable",|c|c.label.as_str());let r=controls::dropdown(ui,("other-cable",other),width).selected_text(selected_text).show_ui(ui,|ui|{controls::fit_popup_contents(ui);for(id,cable)in &state.topology().cables{let r=ui.selectable_value(&mut current,id.clone(),format!("{} · {} m",cable.label,cable.length_m));m.choices.push((format!("cable:{other}"),id.clone(),r.rect,true));m.choice_clips.push((format!("cable:{other}"),id.clone(),ui.clip_rect()));changed|=r.changed();}});m.controls.push((format!("cable:{other}"),r.response.rect,r.response.id));shadow.configuration.cables.insert(other.clone(),current);});}});
    m.expanders.push((
        "other-cables".into(),
        expander.header_response.rect,
        expander.header_response.enabled(),
        ui.clip_rect(),
    ));
    if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 44. {
        ui.columns(2, |columns| {
            changed |= port_editor(
                &mut columns[0],
                m,
                &link.a,
                left,
                shadow.configuration.ports.get_mut(&link.a).unwrap(),
            );
            changed |= port_editor(
                &mut columns[1],
                m,
                &link.b,
                right,
                shadow.configuration.ports.get_mut(&link.b).unwrap(),
            );
        });
    } else {
        changed |= port_editor(
            ui,
            m,
            &link.a,
            left,
            shadow.configuration.ports.get_mut(&link.a).unwrap(),
        );
        ui.add_space(controls::GAP);
        changed |= port_editor(
            ui,
            m,
            &link.b,
            right,
            shadow.configuration.ports.get_mut(&link.b).unwrap(),
        );
    }
    if !d.retired_ports.is_empty() {
        ui.strong("Retire the old interfaces");
        for id in &d.retired_ports {
            let value = shadow.configuration.ports.get_mut(id).unwrap();
            let r = ui.checkbox(
                &mut value.admin_up,
                format!("{} · Up", state.topology().ports[id].label),
            );
            m.controls.push((format!("up:{id}"), r.rect, r.id));
            changed |= r.changed();
        }
    }
    let expander = controls::collapsing_header(ui, "Other supplied interfaces").show(ui, |ui| {
        ui.add(
            egui::Label::new("Keep the working neighbors stated in the assignment unchanged.")
                .wrap(),
        );
        for (id, spec) in &state.topology().ports {
            if id != &link.a && id != &link.b && !d.retired_ports.contains(id) {
                changed |= port_editor(
                    ui,
                    m,
                    id,
                    spec,
                    shadow.configuration.ports.get_mut(id).unwrap(),
                );
            }
        }
    });
    m.expanders.push((
        "other-interfaces".into(),
        expander.header_response.rect,
        expander.header_response.enabled(),
        ui.clip_rect(),
    ));
    changed
}

// Read-only counter tables use the shared body-font row height. Size the two-end
// layout from its actual labels, values and existing Clear actions, not pixels.
const COUNTER_LABELS: [&str; 5] = [
    "Frames offered",
    "Frames received",
    "Receive CRC errors",
    "Collisions",
    "Late collisions",
];
const CLEAR_COUNTERS: &str = "Clear this interface's counters";
fn counter_values(c: &core::Counters) -> [u64; 5] {
    [
        c.offered,
        c.received,
        c.crc_errors,
        c.collisions,
        c.late_collisions,
    ]
}
fn counter_columns(ui: &egui::Ui, c: &core::Counters) -> (f32, f32) {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let label = COUNTER_LABELS
        .iter()
        .map(|label| {
            ui.fonts_mut(|f| {
                f.layout_no_wrap((*label).into(), body.clone(), color)
                    .size()
                    .x
                    .ceil()
            })
        })
        .fold(0., f32::max);
    let value = counter_values(c)
        .iter()
        .map(|value| {
            ui.fonts_mut(|f| {
                f.layout_no_wrap(value.to_string(), mono.clone(), color)
                    .size()
                    .x
                    .ceil()
            })
        })
        .fold(0., f32::max);
    (label, value)
}
fn counter_table(
    ui: &mut egui::Ui,
    port: (&str, &str),
    counters: &core::Counters,
    heading_height: f32,
    m: &mut Metrics,
    intent: &mut Intent,
) {
    ui.add_sized(
        [ui.available_width(), heading_height],
        egui::Label::new(RichText::new(port.1).strong())
            .wrap()
            .halign(egui::Align::Min),
    );
    let (labels, values) = counter_columns(ui, counters);
    for (label, value) in COUNTER_LABELS.iter().zip(counter_values(counters)) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), controls::table_height(ui)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                controls::table_text(ui, *label, labels);
                controls::table_text(ui, RichText::new(value.to_string()).monospace(), values);
            },
        );
    }
    if action(ui, m, &format!("clear:{}", port.0), CLEAR_COUNTERS, false) {
        intent.clear.push(port.0.into());
    }
}
fn link_counters(
    ui: &mut egui::Ui,
    state: &core::State,
    link: &core::LinkSpec,
    m: &mut Metrics,
    intent: &mut Intent,
) {
    let ends = [&link.a, &link.b];
    let gap = ui.spacing().item_spacing.x;
    let minimum = ends
        .iter()
        .map(|id| {
            let (labels, values) = counter_columns(ui, &state.counters()[*id]);
            (labels + gap + values).max(controls::button_width(ui, CLEAR_COUNTERS))
        })
        .fold(0., f32::max);
    if ui.available_width() >= minimum * 2. + gap {
        let width = (ui.available_width() - gap) / 2.;
        let font = egui::TextStyle::Body.resolve(ui.style());
        let color = palette(ui).text;
        let heading_height = ends
            .iter()
            .map(|id| {
                ui.painter()
                    .layout(
                        state.topology().ports[*id].label.clone(),
                        font.clone(),
                        color,
                        width,
                    )
                    .size()
                    .y
            })
            .fold(ui.text_style_height(&egui::TextStyle::Body), f32::max);
        ui.columns(2, |columns| {
            for (column, id) in columns.iter_mut().zip(ends) {
                counter_table(
                    column,
                    (id, &state.topology().ports[id].label),
                    &state.counters()[id],
                    heading_height,
                    m,
                    intent,
                );
            }
        });
    } else {
        for id in ends {
            let font = egui::TextStyle::Body.resolve(ui.style());
            let height = ui
                .painter()
                .layout(
                    state.topology().ports[id].label.clone(),
                    font,
                    palette(ui).text,
                    ui.available_width(),
                )
                .size()
                .y;
            counter_table(
                ui,
                (id, &state.topology().ports[id].label),
                &state.counters()[id],
                height,
                m,
                intent,
            );
        }
    }
}
fn sample_log(ui: &mut egui::Ui, logs: &[String], m: &mut Metrics) {
    if logs.is_empty() {
        m.output = Some(ui.add(egui::Label::new(RichText::new(
            "Sample results appear here; saved counters remain available after reopening.",
        ).small()).wrap()).rect);
        return;
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 6.;
    let (rect, _) = controls::fixed_viewport(ui, egui::vec2(ui.available_width(), height), |ui| {
        let line = ui.text_style_height(&egui::TextStyle::Monospace);
        egui::ScrollArea::both()
            .id_salt("physical-sample-log")
            .auto_shrink([false, false])
            .show_rows(ui, line, logs.len(), |ui, range| {
                for n in range {
                    ui.add(egui::Label::new(RichText::new(&logs[n]).monospace()).extend());
                }
            });
    });
    m.output = Some(rect);
}

/// Supplied observations and current simulated counters are deliberately separate.
/// No label displays immutable impairment parameters or treats a reported sample
/// as a diagnosis. Current receiving errors remain visible for investigation.
fn investigation(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    interaction: (bool, Option<egui::Id>),
    intent: &mut Intent,
) -> bool {
    let (practice, entry_focus) = interaction;
    let state = scope_state(a, practice);
    let d = definition(s, practice);
    let selected = &p.draft.as_ref().unwrap().selected_link;
    let link = &state.topology().links[selected];
    ui.strong("Inspect and sample this link");
    ui.add(egui::Label::new("These are simulated cumulative counters. Record a baseline, send a small sample, then compare the changes. Clearing a counter does not repair a link.").wrap());
    link_counters(ui, state, link, m, intent);
    let mut rejected = false;
    controls::form_row(ui, "Frames left to right", |ui, _| {
        rejected |= number(ui, p, m, 0, entry_focus);
    });
    controls::form_row(ui, "Frames right to left", |ui, _| {
        rejected |= number(ui, p, m, 1, entry_focus);
    });
    if action(ui, m, "probe", "Send simulated sample", false) {
        match(p.numbers[0].parse::<u32>(),p.numbers[1].parse::<u32>()){(Ok(left),Ok(right))if left<=1024&&right<=1024&&(left>0||right>0)=>intent.probe=Some((left,right)),_=>p.notice=Some("Enter 0–1024 frames for each direction, with at least one frame in total. Nothing was sent.".into())}
    }
    if !d.observations.is_empty() {
        let expander=controls::collapsing_header(ui,"Supplied observations · Earlier, not a live diagnosis").show(ui,|ui|{for o in &d.observations{ui.strong(&state.topology().ports[&o.port].label);ui.add(egui::Label::new(&o.reading.note).wrap());let raw=&o.reading.counters;ui.add(egui::Label::new(format!("Reported input {} · output {} · CRC {} · collisions {} · late {}",raw.input_packets,raw.output_packets,raw.crc_errors,raw.collisions,raw.late_collisions)).wrap());}ui.label("Reported counters have platform-specific meanings. They are not the simulator's offered/received totals and never award completion.");});
        m.expanders.push((
            "raw-observations".into(),
            expander.header_response.rect,
            expander.header_response.enabled(),
            ui.clip_rect(),
        ));
    }
    sample_log(ui, &p.logs, m);
    rejected
}
fn help(ui: &mut egui::Ui, s: &Scenario, p: &mut Presentation, challenge: bool) {
    controls::collapsing_header(
        ui,
        if challenge {
            "Hints and references · Optional"
        } else {
            "Help and technical boundaries · Optional"
        },
    )
    .show(ui, |ui| {
        if !s.hints.is_empty() {
            if ui.add(button("Show the next hint")).clicked() {
                p.hint = (p.hint + 1).min(s.hints.len());
            }
            for hint in s.hints.iter().take(p.hint) {
                ui.add(egui::Label::new(hint).wrap());
            }
        }
        controls::collapsing_header(ui, "Why these settings work · Optional").show(ui, |ui| {
            ui.add(egui::Label::new(&s.explanation).wrap());
        });
        controls::collapsing_header(ui, "Model limits and sources").show(ui, |ui| {
            ui.add(egui::Label::new(&s.simplifications).wrap());
            for (n, url) in s.sources.iter().enumerate() {
                ui.hyperlink_to(format!("Technical source {}", n + 1), url);
            }
        });
    });
}
fn page(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    active: bool,
    entry_focus: Option<egui::Id>,
) -> bool {
    let current = match scope_view(a) {
        View::Introduction => 0,
        View::Example => 1,
        View::Assignment => 2,
    };
    let salt = (
        "physical-workspace",
        s.id.as_str(),
        s.version,
        a.challenge,
        current,
        p.scope_generation,
    );
    ui.push_id(salt, |ui| body(ui, s, a, p, m, active, entry_focus))
        .inner
}
/// The complete assignment stays visible; the three tabs change assistance and
/// isolated example state, never replace meaningful configuration with a quiz.
fn body(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &mut Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    active: bool,
    entry_focus: Option<egui::Id>,
) -> bool {
    if let Err(error) = p.memo.prepare(a, s, false) {
        p.notice = Some(error);
        ui.add(egui::Label::new(p.notice.as_ref().unwrap()).wrap());
        return false;
    }
    let root_frame = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
    let blocked = p.blocked_gesture || p.veto_frame == Some(root_frame);
    let current = scope_view(a);
    let practice = current == View::Example;
    let mut intent = Intent::default();
    let pending_on_entry = pending(ui.ctx(), &p.fields, entry_focus);
    let old_numbers = p.numbers.clone();
    let native_edit = ui.input(|i| {
        i.events.iter().any(|e| {
            matches!(
                e,
                egui::Event::Key { pressed: true, .. }
                    | egui::Event::Cut
                    | egui::Event::Paste(_)
                    | egui::Event::Text(_)
            )
        })
    });
    let native_checkpoint = entry_focus
        .filter(|id| native_edit && p.fields.contains(&Some(*id)))
        .and_then(|id| {
            egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
                let undo = state.undoer();
                (id, state, undo)
            })
        });
    m.title = Some(ui.heading(&s.title).rect);
    m.goal = Some(
        ui.add(egui::Label::new(RichText::new(&s.objective).strong()).wrap())
            .rect,
    );
    ui.label(
        RichText::new("In-process simulated links · No host interfaces or IOS commands").small(),
    );
    ui.add_enabled_ui(active && !pending_on_entry && !blocked, |ui| {
        if !a.challenge {
            ui.horizontal_wrapped(|ui| {
                for (view, label) in [
                    (View::Introduction, "Introduction"),
                    (View::Example, "Try example · Ungraded"),
                    (View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(current == view, label);
                    m.tabs.push((view, r.rect));
                    if r.clicked() {
                        intent.view = Some(view);
                    }
                }
            });
        }
    });
    if current == View::Introduction {
        let lesson = s.physical_link_teaching.as_ref().unwrap();
        ui.add(egui::Label::new(&lesson.introduction).wrap());
        for concept in &lesson.concepts {
            ui.strong(&concept.title);
            ui.add(egui::Label::new(&concept.text).wrap());
        }
        ui.add_enabled_ui(active && !pending_on_entry && !blocked, |ui| {
            ui.horizontal_wrapped(|ui| {
                if action(ui, m, "try-example", "Try the separate example", true) {
                    intent.view = Some(View::Example);
                }
                if action(ui, m, "skip-intro", "Open full assignment", false) {
                    intent.view = Some(View::Assignment);
                }
            });
        });
        if let Some(view) = intent.view {
            match work::set_view(a, s, view) {
                Ok(()) => {
                    p.clear();
                    let _ = p.memo.prepare(a, s, false);
                    return true;
                }
                Err(error) => p.notice = Some(error),
            }
        }
        return false;
    }
    let task = definition(s, practice);
    ui.strong(if practice {
        "Example task · Separate equipment"
    } else {
        "Configuration requirements"
    });
    for requirement in &task.requirements {
        ui.add(
            egui::Label::new(format!(
                "{}: at least {}, {}and loss-free fresh samples in both directions.",
                requirement.label,
                rate(requirement.minimum_rate),
                if requirement.full_duplex {
                    "full duplex, "
                } else {
                    ""
                }
            ))
            .wrap(),
        );
    }
    if !task.protected_ports.is_empty() || !task.protected_links.is_empty() {
        ui.add(egui::Label::new(format!("Preserve {} working interfaces and {} cable assignments; inspect their settings under Other supplied interfaces/cables.",task.protected_ports.len(),task.protected_links.len())).wrap());
    }
    p.bind(ui.ctx(), s, practice, scope_draft(a, practice));
    let mut changed = false;
    let mut rejected = false;
    ui.add_enabled_ui(active,|ui|{
  if practice{let lesson=s.physical_link_teaching.as_ref().unwrap();ui.strong("Separate example · No assignment credit");ui.add(egui::Label::new(lesson.practice.requirements.iter().map(|r|r.label.as_str()).collect::<Vec<_>>().join("; ")).wrap());ui.label("Fill worked example changes only this example's draft. Inspect it, Apply configuration, then Check configuration.");ui.horizontal_wrapped(|ui|{intent.fill=action(ui,m,"fill-example","Fill worked example",false);intent.reset=action(ui,m,"reset-example","Reset example",false);});controls::collapsing_header(ui,"Try a change · Optional").show(ui,|ui|{ui.add(egui::Label::new(&lesson.experiment_description).wrap());});}
  let wide=ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*78.;
  if wide{ui.columns(2,|columns|{changed|=editor(&mut columns[0],s,a,p,m,practice);rejected|=investigation(&mut columns[1],s,a,p,m,(practice,entry_focus),&mut intent);});}else{changed|=editor(ui,s,a,p,m,practice);ui.add_space(controls::GAP);rejected|=investigation(ui,s,a,p,m,(practice,entry_focus),&mut intent);}
  let unapplied=&p.draft.as_ref().unwrap().configuration!=scope_state(a,practice).configuration();
  ui.horizontal_wrapped(|ui|{intent.apply=action(ui,m,"apply","Apply configuration",unapplied);intent.check=action(ui,m,"check","Check configuration",!unapplied);});
  if unapplied{ui.label("Draft changes are saved but not applied. Apply the complete configuration, then check the outcome.");}
 });
    let delivered = pending_on_entry && m.numbers.iter().any(|(_, _, id)| Some(*id) == entry_focus);
    rejected |= pending_on_entry && !delivered;
    let navigation = intent.view.is_some() || intent.fill || intent.reset;
    rejected |= navigation && (changed || pending_on_entry);
    if rejected {
        p.veto_frame = Some(root_frame);
        p.blocked_gesture |= ui.input(|i| i.pointer.primary_down());
    }
    let gesture_blocked = blocked || p.veto_frame == Some(root_frame);
    if p.blocked_gesture && !ui.input(|i| i.pointer.primary_down()) {
        p.blocked_gesture = false;
        p.veto_frame = Some(root_frame);
    }
    if !rejected && !gesture_blocked && p.numbers != old_numbers {
        p.notice = None;
    }
    rejected |= gesture_blocked;
    if !active || rejected {
        p.draft = Some(scope_draft(a, practice).clone());
        p.numbers = old_numbers;
        if let Some((id, mut state, undo)) = native_checkpoint {
            state.set_undoer(undo);
            state.store(ui.ctx(), id);
        }
        if rejected {
            p.notice=Some("That edit could not be accepted. Retry in the visible field; nothing was applied, sent or checked.".into());
        }
        m.rejected = rejected;
    } else {
        let scope = if practice {
            work::Scope::Practice
        } else {
            work::Scope::Assignment
        };
        let mut dirty = false;
        let accepted = if changed {
            match work::set_draft(a, s, scope, p.draft.as_ref().unwrap()) {
                Ok(()) => {
                    dirty = true;
                    p.notice = None;
                    true
                }
                Err(error) => {
                    p.notice = Some(error);
                    p.draft = Some(scope_draft(a, practice).clone());
                    false
                }
            }
        } else {
            true
        };
        if accepted {
            let request = if let Some(view) = intent.view {
                work::set_view(a, s, view)
                    .map(|_| "Section changed. Assignment and example drafts remain separate.")
            } else if intent.fill {
                work::fill_example(a,s).map(|_|"Worked example draft filled only. Apply it to change the separate example.")
            } else if intent.reset {
                work::reset(a, s, scope)
                    .map(|_| "Only the separate example was reset. Assignment work is retained.")
            } else if intent.apply {
                work::apply(a,s,scope).map(|_|"Applied to the simulated link. Check configuration to verify the full task.")
            } else if intent.check {
                work::set_checked(a, s, scope, true).map(|_| "")
            } else {
                Ok("")
            };
            let requested = navigation || intent.apply || intent.check;
            match request {
                Ok(notice) => {
                    dirty |= requested;
                    if requested {
                        p.notice = (!notice.is_empty()).then(|| notice.into());
                        if navigation {
                            p.owner = None;
                        }
                    }
                }
                Err(error) => p.notice = Some(error),
            }
            // Configuration, navigation and sample/reset actions are mutually exclusive in
            // this frame. A failed requested action cannot send a sample against old data.
            if !requested {
                let link = p.draft.as_ref().unwrap().selected_link.clone();
                let action = if let Some((a_frames, b_frames)) = intent.probe {
                    Some(core::Action::Probe {
                        link,
                        a_frames,
                        b_frames,
                    })
                } else {
                    intent
                        .clear
                        .first()
                        .map(|port| core::Action::ClearCounters { port: port.clone() })
                };
                if let Some(action) = action {
                    match work::perform(a, s, scope, &action) {
                        Ok(report) => {
                            dirty = true;
                            p.notice = None;
                            if let Some(report) = report {
                                for (port, before) in &report.before {
                                    let after = &report.after[port];
                                    p.append(format!("{}: offered +{}; received +{}; CRC +{}; collisions +{}; late +{}",port,after.offered-before.offered,after.received-before.received,after.crc_errors-before.crc_errors,after.collisions-before.collisions,after.late_collisions-before.late_collisions));
                                }
                            } else {
                                p.append("Selected simulated counter baseline cleared; configuration unchanged.".into());
                            }
                        }
                        Err(error) => p.notice = Some(error),
                    }
                }
            }
        }
        if dirty {
            if let Err(error) = p.memo.prepare(a, s, false) {
                p.notice = Some(error);
            }
            if navigation {
                ui.ctx().request_repaint();
                return true;
            }
            return finish(ui, s, a, p, m, practice, dirty);
        }
    }
    finish(ui, s, a, p, m, practice, false)
}
fn finish(
    ui: &mut egui::Ui,
    s: &Scenario,
    a: &Attempt,
    p: &mut Presentation,
    m: &mut Metrics,
    practice: bool,
    dirty: bool,
) -> bool {
    if let Some(notice) = &p.notice {
        m.notice = Some(ui.add(egui::Label::new(notice).wrap()).rect);
    }
    if checked(a, practice)
        && let Some(rows) = p.memo.rows(a, s, practice)
    {
        results(ui, m, definition(s, practice), rows);
    }
    m.complete = !practice && checked(a, false) && p.memo.passed(a, s) == Some(true);
    if m.complete {
        ui.strong("Assignment complete · Current configuration passes");
        controls::collapsing_header(ui, "Review the reasoning · Optional").show(ui, |ui| {
            ui.add(egui::Label::new(&s.explanation).wrap());
        });
    }
    help(ui, s, p, a.challenge);
    ui.label(RichText::new("Leaving keeps both drafts, applied configurations and counters. Sample entry amounts and the on-screen sample log are temporary; use the saved counters after reopening.").small());
    dirty
}
impl LabApp {
    pub(super) fn physical_library_prepass(&mut self, ctx: &egui::Context) {
        if self.selected.is_some()
            || self.backup_presentation.checking()
            || self.settings_open
            || self.theme_preview
            || self.data_tools.modal_open()
        {
            return;
        }
        if let Some(i) = self
            .completion_index
            .pending(&self.progress, &self.scenarios)
            && matches!(self.scenarios[i].initial, State::PhysicalLink { .. })
            && self.completion_index.take_work()
        {
            let s = &self.scenarios[i];
            if let Some(a) = self.progress.attempts.get(&s.id) {
                if self
                    .physical_presentation
                    .memo
                    .prepare(a, s, false)
                    .is_err()
                {
                    self.completion_index.refuse(&s.id);
                }
                ctx.request_repaint();
            }
        }
    }

    pub(super) fn physical_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let focus = ui.memory(|memory| memory.focused());
        let frame = ui.ctx().cumulative_frame_nr_for(egui::ViewportId::ROOT);
        let pending = pending(ui.ctx(), &self.physical_presentation.fields, focus)
            || self.physical_presentation.blocked_gesture
            || self.physical_presentation.veto_frame == Some(frame);
        ui.add_enabled_ui(!pending, |ui| self.workspace_header(ui, i));
        let active = self.selected == Some(i)
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let source = &self.scenarios[i];
        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
        self.dirty |= card(ui)
            .show(ui, |ui| {
                page(
                    ui,
                    source,
                    attempt,
                    &mut self.physical_presentation,
                    &mut self.metrics.physical,
                    active,
                    focus,
                )
            })
            .inner;
        if self.metrics.physical.complete {
            ui.add_enabled_ui(active && !pending, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui.add(button("Back to library")).clicked() {
                        self.selected = None;
                        self.physical_presentation.clear();
                    }
                    if self.selected == Some(i) {
                        self.next_unit_lab(ui, i);
                    }
                });
            });
        }
    }
}

#[cfg(test)]
mod tests;
