use super::*;
use dlms_learning_lab::{
    switching::{Fabric, Jack},
    switching_workspace::Draft,
};
use engine::State;
impl LabApp {
    pub(super) fn switching_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.switching_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_switching_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .switching_learning
            .as_mut()
        {
            l.view = view;
        }
        self.focus_input = true;
        self.help_open = false;
        self.notice = None;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn switching_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.switching_view(i);
        if !self.progress.attempts[&s.id].challenge && s.switching_teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_switching_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let lesson = s.switching_teaching.as_ref().unwrap();
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A configuration example to try").strong());
                ui.label(&lesson.example_description);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_switching_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_switching_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = s.switching_teaching.as_ref().unwrap();
                ui.label("Practice example · Not graded; your assignment stays separate.");
                ui.label(&lesson.example_description);
                let mut learning = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .switching_learning
                    .clone()
                    .unwrap();
                ui.horizontal_wrapped(|ui| {
                    if ui.add(button("Load example settings")).clicked() {
                        learning.fill_example(lesson).expect("validated example");
                        self.dirty = true;
                    }
                    if ui.add(button("Reset example only")).clicked() {
                        learning = dlms_learning_lab::switching_teaching::Learning::new(lesson);
                        learning.view = teaching::View::Example;
                        self.dirty = true;
                    }
                });
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    learning.view = teaching::View::Assignment;
                    self.set_switching_view(i, teaching::View::Assignment);
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                let old = learning.draft.clone();
                configuration_editor(
                    ui,
                    i,
                    true,
                    &learning.fabric,
                    &mut learning.draft,
                    &mut self.metrics,
                    &mut self.focus_input,
                    "Practice draft · Apply and test example uses these edits",
                );
                if learning.draft != old {
                    self.dirty = true;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Apply and test example").wrap(),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let _ = learning.apply();
                    self.dirty = true;
                }
                if let Some(notice) = &learning.notice {
                    ui.colored_label(palette(ui).error, notice);
                }
                let checks = learning.checks(lesson);
                if !checks.is_empty() {
                    outcome_feedback(
                        ui,
                        checks,
                        "Example checks pass · Your assignment remains separate.",
                        (i, true),
                    );
                }
                inspect_paths(ui, i, true, &learning.fabric, &mut self.metrics);
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .switching_learning = Some(learning);
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let State::Switching { fabric } = self.progress.attempts[&s.id].state.clone()
                else {
                    return;
                };
                let mut draft = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .switching_draft
                    .clone()
                    .unwrap();
                let before = draft.clone();
                configuration_editor(
                    ui,
                    i,
                    false,
                    &fabric,
                    &mut draft,
                    &mut self.metrics,
                    &mut self.focus_input,
                    "Draft changes · Check configuration applies and tests them",
                );
                if before != draft {
                    self.dirty = true;
                }
                if before.switches != draft.switches {
                    self.checked = false;
                    self.notice = None;
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = false;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Check configuration").wrap(),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let mut candidate = fabric.clone();
                    match draft.apply(&mut candidate) {
                        Ok(()) => {
                            self.progress.attempts.get_mut(&s.id).unwrap().state =
                                State::Switching { fabric: candidate };
                            self.checked = true;
                            self.notice = None;
                        }
                        Err(e) => {
                            self.checked = false;
                            self.notice = Some(e);
                        }
                    }
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = self.checked;
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .switching_draft = Some(draft);
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if let Some(notice) = &self.notice {
                        ui.colored_label(palette(ui).error, notice);
                    }
                    self.report_switching(ui, i);
                });
                self.metrics.feedback = Some(result.response.rect);
                if let (State::Switching { fabric }, engine::Goal::Switching { .. }) =
                    (&self.progress.attempts[&s.id].state, &s.goal)
                {
                    inspect_paths(ui, i, false, fabric, &mut self.metrics);
                }
            }
        }
    }
    fn report_switching(&mut self, ui: &mut egui::Ui, i: usize) {
        if !self.checked {
            return;
        }
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        let pass = progress::current_solved(a, s);
        self.metrics.completion = pass;
        let checks = engine::checks(&a.state, &s.goal);
        let pending = if let State::Switching { fabric } = &a.state {
            a.workspace
                .as_ref()
                .and_then(|w| w.switching_draft.as_ref())
                .is_some_and(|d| !d.matches(fabric))
        } else {
            false
        };
        if !pass && (pending || checks.iter().all(|c| c.0)) {
            ui.label("Draft not verified · Check configuration to test the current draft.");
            return;
        }
        outcome_feedback(
            ui,
            checks,
            "Objective complete · All current checks pass.",
            (i, false),
        );
        if pass {
            self.next_unit_lab(ui, i);
        }
    }
}
fn outcome_feedback(
    ui: &mut egui::Ui,
    checks: Vec<(bool, String)>,
    complete: &str,
    id: (usize, bool),
) {
    let passed = checks.iter().filter(|c| c.0).count();
    if passed == checks.len() {
        ui.colored_label(palette(ui).success, complete);
    } else {
        for (_, message) in checks.iter().filter(|c| !c.0).take(2) {
            ui.colored_label(palette(ui).error, format!("Retry · {message}"));
        }
    }
    egui::CollapsingHeader::new(format!(
        "Objective results · {passed}/{} passing",
        checks.len()
    ))
    .id_salt(("vlan-objectives", id))
    .show(ui, |ui| {
        for (okay, message) in &checks {
            ui.colored_label(
                if *okay {
                    palette(ui).success
                } else {
                    palette(ui).error
                },
                format!("{} · {message}", if *okay { "Pass" } else { "Retry" }),
            );
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn configuration_editor(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    fabric: &Fabric,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
    pending_help: &str,
) {
    ui.label(
        RichText::new(if draft.matches(fabric) {
            "Applied settings · simulated switches"
        } else {
            pending_help
        })
        .small()
        .color(palette(ui).muted),
    );
    // Keep the physical connections visible; field values show the learner's current draft.
    ui.horizontal_wrapped(|ui| {
        ui.label("Switch");
        egui::ComboBox::from_id_salt(("switch", index, practice))
            .width(180.)
            .selected_text(&draft.selected_switch)
            .show_ui(ui, |ui| {
                for (name, s) in &fabric.switches {
                    ui.selectable_value(
                        &mut draft.selected_switch,
                        name.clone(),
                        format!("{} ({name})", s.label),
                    );
                }
            });
    });
    let name = draft.selected_switch.clone();
    let switch = draft.switches.get_mut(&name).unwrap();
    super::ipv6_workspace::field(
        ui,
        "Active VLANs",
        &mut switch.vlans,
        384,
        (index, practice, &name, "vlans"),
        metrics,
        focus,
    );
    ui.label(RichText::new("Lists use commas, for example 30,40. Access connects a workstation to one VLAN; a trunk carries several VLANs between switches.").small().color(palette(ui).muted));
    let wide = ui.available_width() >= 850.;
    let columns = [
        220.,
        56.,
        112.,
        128.,
        (ui.available_width() - 220. - 56. - 112. - 128. - 4. * controls::GAP).max(170.),
    ];
    egui::Grid::new(("vlan-port-settings", index, practice, &name))
        .num_columns(if wide { 5 } else { 2 })
        .spacing([controls::GAP, 8.])
        .min_row_height(controls::height(ui))
        .show(ui, |ui| {
            if wide {
                for (label, width) in [
                    "Port / connected device",
                    "State",
                    "Mode",
                    "VLAN / native",
                    "Allowed VLANs",
                ]
                .into_iter()
                .zip(columns)
                {
                    controls::label(ui, RichText::new(label).strong(), width);
                }
                ui.end_row();
            }
            for (port, p) in &mut switch.ports {
                let jack = Jack::Switch {
                    switch: name.clone(),
                    port: port.clone(),
                };
                let peer = fabric.cables.iter().find_map(|c| {
                    if c.a == jack {
                        Some(&c.b)
                    } else if c.b == jack {
                        Some(&c.a)
                    } else {
                        None
                    }
                });
                let connection = match peer {
                    Some(Jack::Host { host }) => format!("{} ({host})", fabric.hosts[host].label),
                    Some(Jack::Switch { switch, port }) => format!("{switch}/{port}"),
                    None => "No cable".into(),
                };
                let scope = format!(
                    "{} switch {name}, port {port}",
                    if practice {
                        "Ungraded practice"
                    } else {
                        "Assignment"
                    }
                );
                let port_width = if wide {
                    columns[0]
                } else {
                    (ui.available_width() * 0.38).clamp(100., 180.)
                };
                controls::label(
                    ui,
                    RichText::new(if wide {
                        format!("{port}\n{connection}")
                    } else {
                        port.to_string()
                    })
                    .strong(),
                    port_width,
                );
                if !wide {
                    let width = (ui.available_width() - 8.).min(380.);
                    controls::label(ui, &connection, width);
                    ui.end_row();
                }
                if !wide {
                    controls::label(ui, "State", port_width);
                }
                let up = controls::cell(ui, columns[1], |ui| ui.checkbox(&mut p.up, "Up"));
                name_control(&up.inner, format!("{scope} — port up"));
                if !wide {
                    ui.end_row();
                    controls::label(ui, "Mode", port_width);
                }
                let mode = controls::cell(ui, columns[2], |ui| {
                    egui::ComboBox::from_id_salt(("port-mode", index, practice, &name, port))
                        .width(columns[2])
                        .truncate()
                        .selected_text(if p.trunk { "Trunk" } else { "Access" })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut p.trunk, false, "Access");
                            ui.selectable_value(&mut p.trunk, true, "Trunk");
                        })
                        .response
                });
                name_control(&mode.inner, format!("{scope} — port mode"));
                if !wide {
                    ui.end_row();
                    controls::label(
                        ui,
                        if p.trunk {
                            "Native VLAN"
                        } else {
                            "Access VLAN"
                        },
                        port_width,
                    );
                }
                let vlan_label = format!(
                    "{scope} — {} VLAN",
                    if p.trunk { "native" } else { "access" }
                );
                compact_field(
                    ui,
                    if p.trunk {
                        &mut p.native
                    } else {
                        &mut p.access
                    },
                    (&vlan_label, 4, columns[3]),
                    (
                        index,
                        practice,
                        &name,
                        port,
                        if p.trunk { "native" } else { "access" },
                    ),
                    metrics,
                    focus,
                );
                if !wide {
                    ui.end_row();
                    if p.trunk {
                        controls::label(ui, "Allowed VLANs", port_width);
                    }
                }
                if p.trunk {
                    compact_field(
                        ui,
                        &mut p.allowed,
                        (
                            &format!("{scope} — allowed VLANs"),
                            384,
                            if wide {
                                columns[4]
                            } else {
                                (ui.available_width() - 8.).min(380.)
                            },
                        ),
                        (index, practice, &name, port, "allowed"),
                        metrics,
                        focus,
                    );
                } else if wide {
                    controls::label(ui, "— Access port", if wide { columns[4] } else { 160. });
                }
                if wide || p.trunk {
                    ui.end_row();
                }
            }
        });
}
fn compact_field(
    ui: &mut egui::Ui,
    value: &mut String,
    spec: (&str, usize, f32),
    id: impl std::hash::Hash,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let (label, limit, width) = spec;
    let r = ui.add_sized(
        [width, controls::height(ui)],
        controls::singleline(ui, value)
            .id_salt(id)
            .char_limit(limit),
    );
    name_control(&r, label);
    metrics.inputs.push(r.rect);
    metrics.vlan_fields.push((r.id, r.rect));
    metrics.vlan_field_enabled.push((r.id, r.enabled()));
    metrics.focus.get_or_insert(r.id);
    if *focus && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
        r.request_focus();
        *focus = false;
    }
    if r.changed() {
        workspace::bound(value, limit);
    }
}

fn inspect_paths(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    fabric: &Fabric,
    metrics: &mut Metrics,
) {
    let response=egui::CollapsingHeader::new("Inspect broadcast delivery · applied settings").id_salt(("vlan-probe",index,practice)).show(ui,|ui| {
        ui.label("Send one simulated broadcast, which asks every workstation in its local group to listen. This is not ping: it checks the VLAN data plane only.");
        ui.label(if practice {"Edits take effect after Apply and test example."}else{"Edits take effect after Check configuration."});
        let id=ui.id().with(("vlan-source",index,practice));
        let mut source=ui.ctx().data_mut(|d|d.get_temp::<String>(id)).filter(|s|fabric.hosts.contains_key(s)).unwrap_or_else(||fabric.hosts.keys().next().unwrap().clone());
        egui::ComboBox::from_id_salt(("vlan-probe-source",index,practice)).selected_text(&source).show_ui(ui,|ui|{for (name,h) in &fabric.hosts{ui.selectable_value(&mut source,name.clone(),format!("{} ({name})",h.label));}});
        ui.ctx().data_mut(|d|d.insert_temp(id,source.clone()));
        match fabric.broadcast(&source) {
            Err(e)=>{ui.colored_label(palette(ui).error,format!("Unsupported / cannot test: {e}"));},
            Ok(result)=>{
                if result.recipients.is_empty(){ui.label("No other workstation received this broadcast.");}
                for (host,path) in &result.recipients {
                    egui::CollapsingHeader::new(format!("Delivered to {} ({host})",fabric.hosts[host].label)).id_salt((index,practice,&source,host)).show(ui,|ui| {
                        for hop in path {ui.label(format!("{}/{} to {} · local VLAN {} · {}",hop.switch,hop.ingress,hop.egress,hop.vlan,hop.tag.map_or("untagged on cable".into(),|v|format!("tag {v} on cable"))));}
                    });
                }
                egui::CollapsingHeader::new("Why other paths stopped").id_salt(("vlan-drops",index,practice)).show(ui,|ui|{for reason in result.drops {ui.label(reason);}if result.omitted_drops>0{ui.label(format!("{} additional drop events omitted",result.omitted_drops));}});
            }
        }
        ui.label(RichText::new("No STP/PVST, MAC learning, routing or hardware timing is modeled. Native-VLAN mismatch handling varies by platform.").small().color(palette(ui).muted));
    });
    metrics.probe = Some(response.header_response.rect);
}
