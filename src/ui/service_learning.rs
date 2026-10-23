//! Existing service assignment plus a separate editor/terminal example.
use super::*;
use dlms_learning_lab::service_teaching;
impl LabApp {
    pub(super) fn service_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.service_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_service_view(&mut self, i: usize, view: teaching::View) {
        self.progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .service_learning
            .as_mut()
            .unwrap()
            .view = view;
        self.focus_input = true;
        self.scroll_command = false;
        self.lesson_scroll_reset = true;
        self.help_open = false;
        self.notice = None;
        self.dirty = true;
    }
    pub(super) fn service_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.service_teaching.as_ref().unwrap();
        let view = self.service_view(i);
        if !self.progress.attempts[&s.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(v == view, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_service_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                ui.heading(&s.title);
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("Two commands to begin investigating").strong());
                for c in lesson.commands.iter().take(2) {
                    ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                    ui.label(&c.explanation);
                }
                ui.label(&lesson.example_description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_service_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_service_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.push_id(("service-example", &s.id), |ui| {
                    self.service_example_workspace(ui, i)
                });
            }
            teaching::View::Assignment => {
                ui.heading(&s.title);
                ui.label(&s.objective);
                ui.push_id(("service-assignment", &s.id), |ui| {
                    self.configuration(ui, i);
                    self.terminal(ui, i);
                    self.check_terminal_work(ui, i);
                });
            }
        }
    }
    fn service_example_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.service_teaching.as_ref().unwrap();
        ui.label(
            RichText::new("Practice example · No assignment credit; separate simulated service.")
                .color(palette(ui).accent),
        );
        ui.label(&lesson.example_description);
        let l = self
            .progress
            .attempts
            .get_mut(&s.id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .service_learning
            .as_mut()
            .unwrap();
        let before = l.clone();
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(button("Use example port"));
            self.metrics.insert = Some(r.rect);
            if r.clicked() {
                l.port_draft = lesson.required_port.to_string();
                self.notice = None;
            }
            let r = ui.add(button("Reset example only"));
            self.metrics.example_reset = Some(r.rect);
            if r.clicked() {
                *l = service_teaching::Learning::new(lesson);
                l.view = teaching::View::Example;
                self.notice = None;
            }
        });
        ui.label(
            RichText::new(engine::describe(&l.state))
                .small()
                .color(palette(ui).muted),
        );
        ui.label(
            RichText::new("/etc/atlas.conf · example’s simulated file")
                .small()
                .color(palette(ui).accent),
        );
        if workspace::editor_row(
            ui,
            "Port",
            "Practice example service configuration port — ungraded",
            &mut l.port_draft,
            "example-service-port",
            &mut self.metrics,
            false,
        ) {
            self.notice = None;
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add(button("Save configuration"));
            self.metrics.editor_save = Some(r.rect);
            if r.clicked() {
                self.notice = Some(match l.save_port() {
                    Ok(()) => {
                        "Saved example configuration; restart to apply it to the example service."
                            .into()
                    }
                    Err(e) => e,
                });
            }
            if l.has_unsaved_port() {
                ui.label(
                    RichText::new("Unsaved example draft")
                        .small()
                        .color(palette(ui).accent),
                );
            }
        });
        if let Some(notice) = &self.notice {
            ui.label(RichText::new(notice).small().color(palette(ui).accent));
        }
        l.command_index = l.command_index.min(lesson.commands.len() - 1);
        ui.horizontal_wrapped(|ui| {
            let width = (ui.available_width() - 145.)
                .clamp(120., 500.)
                .min(ui.available_width());
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.set_width(width);
                    let picker = egui::ComboBox::from_id_salt("service-example-command")
                        .width(width)
                        .truncate()
                        .selected_text(&lesson.commands[l.command_index].command)
                        .show_ui(ui, |ui| {
                            for (n, c) in lesson.commands.iter().enumerate() {
                                ui.selectable_value(&mut l.command_index, n, &c.command);
                            }
                        });
                    name_control(&picker.response, "Service practice command to insert");
                },
            );
            let r = ui.add(button("Insert command"));
            self.metrics.command_insert = Some(r.rect);
            if r.clicked() {
                l.command = lesson.commands[l.command_index].command.clone();
                self.focus_input = true;
            }
        });
        ui.label(&lesson.commands[l.command_index].explanation);
        let height = (ui.ctx().content_rect().height() * 0.22).clamp(120., 220.);
        let output = egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(10.)
            .corner_radius(6.)
            .show(ui, |ui| {
                ui.label(
                    RichText::new("Simulated terminal · example only")
                        .small()
                        .color(palette(ui).muted),
                );
                egui::ScrollArea::both()
                    .id_salt("service-example-output")
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if l.transcript.is_empty() {
                            ui.label("Insert a command or type your own, then press Enter or Run.");
                        }
                        for line in &l.transcript {
                            ui.add(
                                egui::Label::new(RichText::new(line).monospace())
                                    .extend()
                                    .selectable(true),
                            );
                        }
                    });
            });
        self.metrics.output = Some(output.response.rect);
        let mut run = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new("practice #").monospace());
            let width = (ui.available_width() - 90.).max(40.);
            let r = ui.add_sized(
                [width, controls::height(ui)],
                controls::singleline(ui, &mut l.command)
                    .id_salt("service-example-input")
                    .hint_text("Command · Enter to run")
                    .char_limit(4096),
            );
            name_control(&r, "Service practice terminal command — ungraded");
            self.metrics.terminal = Some(r.rect);
            self.metrics.focus = Some(r.id);
            if self.focus_input
                && !ui.input(|v| v.pointer.any_pressed() || v.pointer.any_released())
            {
                r.request_focus();
                self.focus_input = false;
            }
            if r.changed() {
                workspace::bound(&mut l.command, 4096);
            }
            run = r.lost_focus() && ui.input(|v| v.key_pressed(egui::Key::Enter));
            run |= ui
                .add_sized([78., controls::height(ui)], button("Run"))
                .clicked();
        });
        if run {
            let _ = l.submit();
            self.focus_input = true;
            self.notice = None;
        }
        let r = ui.add_sized(
            [220_f32.min(ui.available_width()), controls::height(ui)],
            primary(ui, "Check example"),
        );
        self.metrics.check = Some(r.rect);
        if r.clicked() {
            l.check();
        }
        if l.solved(lesson) {
            ui.colored_label(palette(ui).success,"Example verified · Saved port, current run and boot setting all match. No assignment credit.");
        } else if l.checked.is_some() {
            if l.has_unsaved_port() {
                ui.label("Save the example draft before checking its configuration.");
            } else {
                for issue in model::issues(&l.state, &lesson.goal()) {
                    ui.add(egui::Label::new(RichText::new(issue).color(palette(ui).error)).wrap());
                }
            }
        }
        self.dirty |= before != *l;
        egui::CollapsingHeader::new("Try a change").show(ui, |ui| {
            ui.label(&lesson.try_change);
        });
        let r = ui.add(button("Open full assignment"));
        self.metrics.assignment_action = Some(r.rect);
        if r.clicked() {
            self.set_service_view(i, teaching::View::Assignment);
        }
    }
}
