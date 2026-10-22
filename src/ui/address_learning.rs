//! Teaching for the two existing addressing assignments, with isolated examples.
use super::*;
use dlms_learning_lab::address_teaching;
impl LabApp {
    pub(super) fn address_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.address_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_address_view(&mut self, i: usize, view: teaching::View) {
        self.progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .address_learning
            .as_mut()
            .unwrap()
            .view = view;
        self.focus_input = true;
        if let Some(e) = &mut self.address_example {
            e.focus = true;
        }
        if let Some(e) = &mut self.ipv4_view {
            e.focus = true;
        }
        self.help_open = false;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn load_address_example(&mut self, i: usize) {
        self.address_example = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .and_then(|w| w.address_learning.as_ref())
            .and_then(|l| {
                let e = ipv4::Exercise::from_state(&l.state)?;
                let checked = l
                    .checked
                    .as_ref()
                    .and_then(ipv4::Exercise::from_state)
                    .map(|e| model::snapshot(&e));
                Some(ipv4_workspace::Assignment::from_saved(e, checked, l.hints))
            });
    }
    pub(super) fn address_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.address_teaching.as_ref().unwrap();
        let view = self.address_view(i);
        let challenge = self.progress.attempts[&s.id].challenge;
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(v == view, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_address_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                ui.heading(&s.title);
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                if let engine::State::Subnet { ip, prefix, .. } = lesson.fixture {
                    self.metrics.address_diagram = Some(block_visual(ui, ip, prefix));
                } else {
                    egui::Grid::new("capacity-example")
                        .spacing([20., 6.])
                        .show(ui, |ui| {
                            for text in ["Prefix", "All addresses", "Usable hosts"] {
                                ui.label(RichText::new(text).strong());
                            }
                            ui.end_row();
                            for (p, total, usable) in [(29, 8, 6), (28, 16, 14), (27, 32, 30)] {
                                ui.label(format!("/{p}"));
                                ui.label(total.to_string());
                                ui.label(usable.to_string());
                                ui.end_row();
                            }
                        });
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_address_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_address_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.push_id(("address-example", &s.id), |ui| {
                    self.address_example_workspace(ui, i)
                });
            }
            teaching::View::Assignment => {
                if matches!(s.initial, engine::State::Subnet { .. }) {
                    self.ipv4_assignment(ui, i);
                } else {
                    ui.heading(&s.title);
                    ui.label(&s.objective);
                    self.allocations(ui, i);
                    self.check_terminal_work(ui, i);
                }
            }
        }
    }
    fn address_example_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.address_teaching.as_ref().unwrap();

        ui.label(
            RichText::new(
                "Practice example · No assignment credit; your original work stays saved.",
            )
            .color(palette(ui).accent),
        );
        if matches!(lesson.fixture, engine::State::Vlsm { .. }) {
            ui.label(&lesson.example_description);
        }
        let mut reload = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(button("Use example values"));
            self.metrics.insert = Some(r.rect);
            if r.clicked() {
                let l = self
                    .progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .address_learning
                    .as_mut()
                    .unwrap();
                if let Err(e) = l.fill_example(lesson) {
                    self.notice = Some(e);
                } else {
                    reload = true;
                    self.dirty = true;
                }
            }
            let r = ui.add(button("Reset example only"));
            self.metrics.example_reset = Some(r.rect);
            if r.clicked() {
                let mut l = address_teaching::Learning::new(lesson);
                l.view = teaching::View::Example;
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .address_learning = Some(l);
                reload = true;
                self.dirty = true;
            }
        });
        if reload {
            self.load_address_example(i);
        }
        let l = self
            .progress
            .attempts
            .get_mut(&s.id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .address_learning
            .as_mut()
            .unwrap();
        let before = l.clone();
        if let Some(e) = &mut self.address_example {
            e.render(ui, false, &mut self.metrics);
            ipv4::write_exercise(&mut l.state, &e.exercise);
            l.checked = e.checked_answers.as_ref().map(|v| engine::State::Subnet {
                ip: e.exercise.ip,
                prefix: e.exercise.prefix,
                answers: std::array::from_fn(|i| v[i].clone()),
                second: Some(v[4].clone()),
            });
            l.hints = e.hints;
        } else {
            let checked = l.checked.is_some();
            workspace::allocation_fields(
                ui,
                &mut l.state,
                checked,
                &mut self.metrics,
                &mut self.focus_input,
            );
            let r = ui.add_sized(
                [220_f32.min(ui.available_width()), CONTROL],
                primary(ui, "Check example"),
            );
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                l.check();
            }
            if l.solved() {
                ui.colored_label(
                    palette(ui).success,
                    "Example verified · All allocations fit; no assignment credit.",
                );
            }
            if checked
                && ui.available_width() < 640.
                && let engine::State::Vlsm { allocations, .. } = &l.state
            {
                for (n, a) in allocations.iter().enumerate() {
                    if let Some(issue) = model::vlsm_issue(&l.state, n) {
                        ui.colored_label(palette(ui).error, format!("{}: {issue}", a.name));
                    }
                }
            }
        }
        self.dirty |= before != *l;
        // An example never sets completion or assignment-check metadata.
        ui.label(format!("Try a change: {}", lesson.try_change));
        let r = ui.add(button("Open full assignment"));
        self.metrics.assignment_action = Some(r.rect);
        if r.clicked() {
            self.set_address_view(i, teaching::View::Assignment);
        }
    }
}
fn block_visual(ui: &mut egui::Ui, ip: std::net::Ipv4Addr, prefix: u8) -> egui::Rect {
    let d = address_teaching::block_diagram(ip, prefix).expect("validated conventional prefix");
    ui.label(
        RichText::new(format!(
            "Example address {ip}/{prefix} · number {} is {}",
            d.octet + 1,
            d.value
        ))
        .strong(),
    );
    let columns = ((ui.available_width() / 220.).floor() as usize).clamp(1, 4);
    let width = ((ui.available_width() - 12. * (columns - 1) as f32) / columns as f32).min(210.);
    let rect = egui::Grid::new("address-blocks")
        .num_columns(columns)
        .spacing([12., 8.])
        .show(ui, |ui| {
            for (index, (start, end)) in d.visible.iter().enumerate() {
                let selected = *start == d.start;
                egui::Frame::new()
                    .fill(palette(ui).input)
                    .stroke(egui::Stroke::new(
                        if selected { 2_f32 } else { 1_f32 },
                        if selected {
                            palette(ui).accent
                        } else {
                            palette(ui).border
                        },
                    ))
                    .inner_margin(8.)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.set_width((width - 18.).max(60.));
                            ui.label(RichText::new(format!("{start}–{end}")).monospace());
                            ui.label(if selected {
                                format!("contains {}", d.value)
                            } else {
                                "neighboring block".into()
                            });
                        });
                    });
                if (index + 1) % columns == 0 {
                    ui.end_row();
                }
            }
        })
        .response
        .rect;
    if d.later_octets == 0 {
        ui.label(format!(
            "Each block has {} addresses; the earlier numbers stay fixed.",
            d.block_size
        ));
    } else {
        ui.label(format!("These blocks group {} values of number {}. Each of the {} later numbers can run from 0 to 255; total addresses in each subnet: {}.",d.block_size,d.octet+1,d.later_octets,1u64<<(32-prefix)));
    }
    rect
}
