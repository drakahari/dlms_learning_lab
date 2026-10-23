//! Native virtual-administrator practice; every command stays in the typed simulator.
use super::*;
use dlms_learning_lab::accounts_teaching::{self, Learning};

impl LabApp {
    pub(super) fn accounts_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.accounts_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_accounts_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.accounts_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.focus_input = true;
        self.scroll_command = view != teaching::View::Introduction;
        self.help_open = false;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn accounts_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        ui.heading(&s.title);
        ui.label(
            RichText::new(
                "Virtual administrator · Simulation only; no host accounts or credentials.",
            )
            .small()
            .color(palette(ui).muted),
        );
        let Some(lesson) = s.accounts_teaching.as_ref() else {
            self.checked = self.progress.attempts[&s.id]
                .workspace
                .as_ref()
                .unwrap()
                .checked;
            ui.label(&s.objective);
            self.terminal(ui, i);
            self.accounts_assignment_check(ui, i);
            return;
        };
        if !self.progress.attempts[&s.id].challenge {
            let view = self.accounts_view(i);
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_accounts_view(i, v);
                    }
                }
            });
        }
        match self.accounts_view(i) {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.inputs);
                for c in &lesson.example_commands {
                    ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                    ui.label(&c.explanation);
                    if c.simulator_only {
                        ui.label(
                            RichText::new("Simulator-only helper")
                                .small()
                                .color(palette(ui).muted),
                        );
                    }
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_accounts_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_accounts_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Ungraded; assignment state, input and command history stay separate.");
                ui.label(&lesson.inputs);
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .accounts_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&l).unwrap();
                ui.horizontal_wrapped(|ui| {
                    let width = (ui.available_width() - 145.)
                        .clamp(80., 500.)
                        .min(ui.available_width());
                    let r = egui::ComboBox::from_id_salt(("accounts-example-selection", &s.id))
                        .width(width)
                        .truncate()
                        .selected_text(format!(
                            "Command {} of {}",
                            l.example_command + 1,
                            lesson.example.len()
                        ))
                        .show_ui(ui, |ui| {
                            for (n, c) in lesson.example.iter().enumerate() {
                                ui.selectable_value(
                                    &mut l.example_command,
                                    n,
                                    format!("{}. {c}", n + 1),
                                );
                            }
                        });
                    name_control(&r.response, "Practice account command to insert");
                    let r = ui.add(button("Insert command"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        l.fill_example(lesson).expect("validated account example");
                        self.focus_input = true;
                        self.scroll_command = true;
                    }
                });
                let c = &lesson.example_commands[l.example_command];
                ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                ui.label(&c.explanation);
                self.accounts_practice_terminal(ui, &s.id, &mut l, lesson);
                let r = ui.add(primary(ui, "Check example"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    l.checked = true;
                }
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if l.checked {
                        let checks = l.checks(lesson);
                        if checks.iter().all(|c| c.0) {
                            ui.colored_label(
                                palette(ui).success,
                                "Example checks pass · No assignment credit",
                            );
                        }
                        account_checks(ui, &checks);
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        l = Learning::new(lesson);
                        l.view = teaching::View::Example;
                        self.focus_input = true;
                    }
                    let r = ui.add(button("Open full assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        l.view = teaching::View::Assignment;
                        self.set_accounts_view(i, teaching::View::Assignment);
                    }
                });
                if serde_json::to_vec(&l).unwrap() != before {
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .accounts_learning = Some(l);
            }
            teaching::View::Assignment => {
                self.checked = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .checked;
                ui.label(&s.objective);
                self.terminal(ui, i);
                self.accounts_assignment_check(ui, i);
            }
        }
    }
    fn accounts_assignment_check(&mut self, ui: &mut egui::Ui, i: usize) {
        let r = ui.add(primary(ui, "Check work"));
        self.metrics.check = Some(r.rect);
        if r.clicked() {
            self.checked = true;
            self.progress
                .attempts
                .get_mut(&self.scenarios[i].id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .checked = true;
            self.dirty = true;
        }
        let result = controls::feedback(ui, r.clicked(), |ui| {
            if !self.checked {
                return;
            }
            let s = self.scenarios[i].clone();
            let a = &self.progress.attempts[&s.id];
            let complete = progress::current_solved(a, &s);
            self.metrics.completion = complete;
            if complete {
                ui.colored_label(
                    palette(ui).success,
                    "Objective complete · All current account checks pass.",
                );
            }
            account_checks(ui, &engine::checks(&a.state, &s.goal));
            if complete {
                if !a.challenge {
                    ui.label(&s.explanation);
                }
                self.next_unit_lab(ui, i);
            }
        });
        self.metrics.feedback = Some(result.response.rect);
    }
    fn accounts_practice_terminal(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        l: &mut Learning,
        lesson: &accounts_teaching::Lesson,
    ) {
        let height = (ui.ctx().content_rect().height() * 0.24).clamp(100., 220.);
        let area = egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(10.)
            .corner_radius(6.)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Practice terminal · Virtual administrator")
                        .small()
                        .color(palette(ui).muted),
                );
                let scroll = egui::ScrollArea::both()
                    .id_salt(("accounts-practice-output", id))
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if l.transcript.is_empty() {
                            ui.label("Insert or type one command, then choose Run.");
                        }
                        for line in &l.transcript {
                            let row = ui.add(
                                egui::Label::new(RichText::new(line).monospace())
                                    .wrap()
                                    .selectable(true),
                            );
                            self.metrics.accounts_latest_output = Some(row.rect);
                        }
                    });
                self.metrics.accounts_output_scroll =
                    Some((scroll.state.offset.y, scroll.inner_rect));
            });
        self.metrics.output = Some(area.response.rect);
        ui.label(
            RichText::new("Virtual administrator #")
                .monospace()
                .color(palette(ui).accent),
        );
        let mut run = false;
        ui.horizontal(|ui| {
            let r = ui.add_sized(
                [(ui.available_width() - 90.).max(40.), controls::height(ui)],
                controls::singleline(ui, &mut l.command)
                    .id_salt(("accounts-practice-command", id))
                    .hint_text("Command · Enter to run")
                    .char_limit(accounts_teaching::MAX_COMMAND_BYTES),
            );
            name_control(&r, "Practice account command — ungraded");
            self.metrics.terminal = Some(r.rect);
            self.metrics.focus = Some(r.id);
            if self.focus_input
                && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released())
            {
                r.request_focus();
                if self.scroll_command {
                    r.scroll_to_me(Some(egui::Align::Center));
                    self.scroll_command = false;
                }
                self.focus_input = false;
            } else if self.focus_input {
                ui.ctx().request_repaint();
            }
            if r.changed() {
                workspace::bound(&mut l.command, accounts_teaching::MAX_COMMAND_BYTES);
                self.dirty = true;
            }
            run = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let r = ui.add_sized([78., controls::height(ui)], button("Run"));
            self.metrics.accounts_run = Some(r.rect);
            run |= r.clicked();
        });
        if run && !l.command.trim().is_empty() {
            let _ = l.execute(lesson);
            self.dirty = true;
            self.focus_input = true;
        }
    }
}
fn account_checks(ui: &mut egui::Ui, checks: &[(bool, String)]) {
    let mut shown = std::collections::BTreeSet::new();
    for (passed, message) in checks {
        // Preservation checks may report the same failing field as its direct
        // objective. Keep that explanation once; scoring still checks both.
        let detail = message
            .strip_prefix("First remaining difference: ")
            .unwrap_or(message);
        if !*passed && !shown.insert(detail) {
            continue;
        }
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{} {message}",
                    if *passed {
                        "Pass ·"
                    } else {
                        "Needs attention ·"
                    }
                ))
                .color(if *passed {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            )
            .wrap(),
        );
    }
}
