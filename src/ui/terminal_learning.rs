use super::*;
use dlms_learning_lab::terminal_teaching;
impl LabApp {
    pub(super) fn terminal_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.terminal_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_terminal_view(&mut self, i: usize, view: teaching::View) {
        self.progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap()
            .view = view;
        self.focus_input = true;
        self.scroll_command = view != teaching::View::Introduction;
        self.help_open = false;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn terminal_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.terminal_teaching.as_ref().unwrap();
        let view = self.terminal_view(i);
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
                        self.set_terminal_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A small example to try").strong());
                for command in &lesson.example {
                    ui.add(egui::Label::new(RichText::new(command).monospace()).wrap());
                }
                ui.label(&lesson.inputs);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_terminal_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_terminal_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Not graded; your assignment stays separate.");
                ui.label(&lesson.inputs);
                let learning = self
                    .progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .terminal_learning
                    .as_mut()
                    .unwrap();
                learning.example_command = learning.example_command.min(lesson.example.len() - 1);
                let before = learning.example_command;
                ui.horizontal_wrapped(|ui| {
                    let width = (ui.available_width() - 125.)
                        .clamp(100., 500.)
                        .min(ui.available_width());
                    ui.allocate_ui_with_layout(
                        egui::vec2(width, CONTROL),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(width);
                            egui::ComboBox::from_id_salt(("practice-example-command", i))
                                .width(width)
                                .truncate()
                                .selected_text(format!(
                                    "{}. {}",
                                    learning.example_command + 1,
                                    lesson.example[learning.example_command]
                                ))
                                .show_ui(ui, |ui| {
                                    for (index, command) in lesson.example.iter().enumerate() {
                                        ui.selectable_value(
                                            &mut learning.example_command,
                                            index,
                                            format!("{}. {command}", index + 1),
                                        );
                                    }
                                });
                        },
                    );
                    let r = ui.add(button("Insert command"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        learning.command = lesson.example[learning.example_command].clone();
                        self.focus_input = true;
                        self.scroll_command = true;
                        self.dirty = true;
                    }
                });
                if before != learning.example_command {
                    self.dirty = true;
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                self.practice_terminal(ui, i);
                ui.horizontal_wrapped(|ui| {
                    if ui.add(button("Reset example only")).clicked() {
                        let mut learning = terminal_teaching::Learning::new(lesson);
                        learning.view = teaching::View::Example;
                        self.progress
                            .attempts
                            .get_mut(&s.id)
                            .unwrap()
                            .workspace
                            .as_mut()
                            .unwrap()
                            .terminal_learning = Some(learning);
                        self.dirty = true;
                    }
                    let r = ui.add(button("Open full assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_terminal_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                self.terminal(ui, i);
                self.check_terminal_work(ui, i);
            }
        }
    }
    fn practice_terminal(&mut self, ui: &mut egui::Ui, i: usize) {
        let l = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .terminal_learning
            .as_mut()
            .unwrap();
        let height = (ui.ctx().content_rect().height() * 0.28).clamp(140., 260.);
        let area = egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(10.)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "Practice terminal · {} · {}",
                        l.tree.user, l.tree.cwd
                    ))
                    .small(),
                );
                egui::ScrollArea::both()
                    .id_salt(("practice-terminal", i))
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if l.transcript.is_empty() {
                            ui.label(
                                "Insert an example command or type your own, then choose Run.",
                            );
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
        self.metrics.output = Some(area.response.rect);
        let mut run = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new("alex $").monospace());
            let r = ui.add_sized(
                [(ui.available_width() - 90.).max(40.), CONTROL],
                egui::TextEdit::singleline(&mut l.command)
                    .id_salt(("practice-command", i))
                    .hint_text("Command · Enter to run")
                    .char_limit(4096),
            );
            self.metrics.terminal = Some(r.rect);
            self.metrics.inputs.push(r.rect);
            self.metrics.focus = Some(r.id);
            if self.focus_input
                && !ui.input(|input| input.pointer.any_pressed() || input.pointer.any_released())
            {
                r.request_focus();
                if self.scroll_command {
                    r.scroll_to_me(Some(egui::Align::BOTTOM));
                    self.scroll_command = false;
                }
                self.focus_input = false;
            }
            if r.changed() {
                workspace::bound(&mut l.command, 4096);
                self.dirty = true;
            }
            run = r.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            run |= ui.add_sized([78., CONTROL], button("Run")).clicked();
        });
        if run {
            l.execute();
            self.dirty = true;
            self.focus_input = true;
        }
    }
}
