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
                );
                if learning.draft != old {
                    self.dirty = true;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), CONTROL],
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
                for (pass, message) in learning.checks(lesson) {
                    ui.colored_label(
                        if pass {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        format!("{} {message}", if pass { "Pass ·" } else { "Retry ·" }),
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
                    [240_f32.min(ui.available_width()), CONTROL],
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
                if let Some(notice) = &self.notice {
                    ui.colored_label(palette(ui).error, notice);
                }
                self.report_switching(ui, i);
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
        if pass {
            ui.colored_label(
                palette(ui).success,
                "Objective complete · All current checks pass.",
            );
            self.next_unit_lab(ui, i);
        } else {
            for (okay, message) in engine::checks(&a.state, &s.goal) {
                if !okay {
                    ui.colored_label(palette(ui).error, format!("Retry · {message}"));
                }
            }
        }
    }
}
fn configuration_editor(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    fabric: &Fabric,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    ui.label(
        RichText::new(if draft.matches(fabric) {
            "Applied settings · simulated switches"
        } else if practice {
            "Practice draft · Apply and test example uses these edits"
        } else {
            "Draft changes · Check configuration applies and tests them"
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
        ui.add_space(6.);
        if ui.available_width() >= 1000. {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [240., CONTROL],
                    egui::Label::new(RichText::new(format!("{port} to {connection}")).strong())
                        .wrap(),
                );
                ui.checkbox(&mut p.up, "Up");
                egui::ComboBox::from_id_salt(("port-mode", index, practice, &name, port))
                    .width(100.)
                    .selected_text(if p.trunk { "Trunk" } else { "Access" })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut p.trunk, false, "Access");
                        ui.selectable_value(&mut p.trunk, true, "Trunk");
                    });
                if p.trunk {
                    ui.label("Native");
                    compact_field(
                        ui,
                        &mut p.native,
                        4,
                        70.,
                        (index, practice, &name, port, "native"),
                        metrics,
                        focus,
                    );
                    ui.label("Allowed");
                    compact_field(
                        ui,
                        &mut p.allowed,
                        384,
                        220.,
                        (index, practice, &name, port, "allowed"),
                        metrics,
                        focus,
                    );
                } else {
                    ui.label("VLAN");
                    compact_field(
                        ui,
                        &mut p.access,
                        4,
                        70.,
                        (index, practice, &name, port, "access"),
                        metrics,
                        focus,
                    );
                }
            });
            continue;
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(format!("{port} to {connection}")).strong());
            ui.checkbox(&mut p.up, "Port up");
            ui.selectable_value(&mut p.trunk, false, "Access");
            ui.selectable_value(&mut p.trunk, true, "Trunk");
        });
        if p.trunk {
            super::ipv6_workspace::field(
                ui,
                "Native VLAN",
                &mut p.native,
                4,
                (index, practice, &name, port, "native"),
                metrics,
                focus,
            );
            super::ipv6_workspace::field(
                ui,
                "Allowed VLANs",
                &mut p.allowed,
                384,
                (index, practice, &name, port, "allowed"),
                metrics,
                focus,
            );
        } else {
            super::ipv6_workspace::field(
                ui,
                "Access VLAN",
                &mut p.access,
                4,
                (index, practice, &name, port, "access"),
                metrics,
                focus,
            );
        }
    }
}
fn compact_field(
    ui: &mut egui::Ui,
    value: &mut String,
    limit: usize,
    width: f32,
    id: impl std::hash::Hash,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let r = ui.add_sized(
        [width, CONTROL],
        egui::TextEdit::singleline(value)
            .id_salt(id)
            .char_limit(limit),
    );
    metrics.inputs.push(r.rect);
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
