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
                lesson_presentation::concepts(
                    ui,
                    &s,
                    &lesson.prerequisite_note,
                    lesson
                        .concepts
                        .iter()
                        .map(|c| (c.title.clone(), c.text.clone())),
                );
                ui.label(RichText::new("Two commands to begin investigating").strong());
                for c in lesson.commands.iter().take(2) {
                    ui.add(egui::Label::new(RichText::new(&c.command).monospace()).wrap());
                    ui.label(&c.explanation);
                }
                lesson_presentation::worked(ui, &lesson.example_description);
                lesson_presentation::worked_rows(ui, &s);
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
                lesson_presentation::assignment(ui, &s);
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
        if ui.available_width() >= 850. {
            ui.columns(2, |columns| {
                example_terminal(
                    &mut columns[0],
                    &s,
                    lesson,
                    l,
                    &mut self.metrics,
                    &mut self.notice,
                    &mut self.focus_input,
                );
                example_configuration(
                    &mut columns[1],
                    &s,
                    lesson,
                    l,
                    &mut self.metrics,
                    &mut self.notice,
                );
            });
        } else {
            example_terminal(
                ui,
                &s,
                lesson,
                l,
                &mut self.metrics,
                &mut self.notice,
                &mut self.focus_input,
            );
            example_configuration(ui, &s, lesson, l, &mut self.metrics, &mut self.notice);
        }
        lesson_presentation::practice_step(ui, &s, 2);
        let r = ui.add_sized(
            [220_f32.min(ui.available_width()), controls::height(ui)],
            primary(ui, "Check example"),
        );
        self.metrics.check = Some(r.rect);
        if r.clicked() {
            l.check();
        }
        if l.solved(lesson) {
            ui.colored_label(
                palette(ui).success,
                "Example verified · Saved port, current run and boot setting all match.",
            );
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

// Existing separate simulation, arranged around investigation then editing.
fn example_configuration(
    ui: &mut egui::Ui,
    s: &Scenario,
    lesson: &service_teaching::Lesson,
    l: &mut service_teaching::Learning,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    lesson_presentation::practice_step(ui, s, 1);
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Use example port"));
        metrics.insert = Some(r.rect);
        if r.clicked() {
            l.port_draft = lesson.required_port.to_string();
            *notice = None;
        }
        let r = ui.add(button("Reset example only"));
        metrics.example_reset = Some(r.rect);
        if r.clicked() {
            *l = service_teaching::Learning::new(lesson);
            l.view = teaching::View::Example;
            *notice = None;
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
    controls::form_row(ui, "Port", |ui, width| {
        let r = ui.add_sized(
            [width.min(220.), controls::height(ui)],
            controls::singleline(ui, &mut l.port_draft)
                .id(egui::Id::new(("service-example-port", &s.id)))
                .char_limit(64),
        );
        name_control(&r, "Practice example service configuration port — ungraded");
        metrics.inputs.push(r.rect);
        if r.changed() {
            workspace::bound(&mut l.port_draft, 64);
            *notice = None;
        }
    });
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save configuration"));
        metrics.editor_save = Some(r.rect);
        if r.clicked() {
            *notice = Some(match l.save_port() {
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
    if let Some(notice) = notice.as_ref() {
        ui.label(RichText::new(notice).small().color(palette(ui).accent));
    }
}
fn example_terminal(
    ui: &mut egui::Ui,
    s: &Scenario,
    lesson: &service_teaching::Lesson,
    l: &mut service_teaching::Learning,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
    focus: &mut bool,
) {
    lesson_presentation::practice_step(ui, s, 0);
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
        metrics.command_insert = Some(r.rect);
        if r.clicked() {
            l.command = lesson.commands[l.command_index].command.clone();
            *focus = true;
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
    metrics.output = Some(output.response.rect);
    let mut run = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("practice #").monospace());
        let width = (ui.available_width() - 90.).max(40.);
        let r = ui.add_sized(
            [width, controls::height(ui)],
            controls::singleline(ui, &mut l.command)
                .id(egui::Id::new(("service-example-input", &s.id)))
                .hint_text("Command · Enter to run")
                .char_limit(4096),
        );
        name_control(&r, "Service practice terminal command — ungraded");
        metrics.terminal = Some(r.rect);
        metrics.focus = Some(r.id);
        if *focus && !ui.input(|v| v.pointer.any_pressed() || v.pointer.any_released()) {
            r.request_focus();
            *focus = false;
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
        *focus = true;
        *notice = None;
    }
}
