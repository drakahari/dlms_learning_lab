//! Learn introductions and ungraded examples share the existing bounded VM, never assignment state.
use super::*;
impl LabApp {
    pub(super) fn script_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.script_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_script_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .and_then(|a| a.workspace.as_mut())
            .and_then(|w| w.script_learning.as_mut())
        {
            l.view = view;
            self.script_job = None;
            self.notice = None;
            self.focus_input = true;
            self.help_open = false;
            self.lesson_scroll_reset = true;
            self.dirty = true;
        }
    }
    pub(super) fn script_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.script_view(i);
        let challenge = self.progress.attempts[&s.id].challenge;
        if !challenge && s.teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, name) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, name);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_script_view(i, v);
                    }
                }
            });
            ui.add_space(4.);
        }
        let Some(lesson) = &s.teaching else {
            ui.label(&s.objective);
            self.scripting(ui, i);
            return;
        };
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.add_space(4.);
                ui.label(RichText::new("A small example to try").strong());
                ui.add(egui::Label::new(RichText::new(&lesson.example).monospace()).wrap());
                ui.label(&lesson.inputs);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_script_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_script_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => self.script_example(ui, i, lesson),
            teaching::View::Assignment => {
                ui.label(&s.objective);
                self.scripting(ui, i);
            }
        }
    }
    fn script_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &teaching::Lesson) {
        ui.label("Practice example · Not graded; your assignment draft stays separate.");
        ui.label(&lesson.inputs);
        ui.label(format!("Try a change: {}", lesson.try_change));
        let id = self.scenarios[i].id.clone();
        let mut run_clicked = false;
        let mut stop_clicked = false;
        if ui.available_width() >= 850. {
            let width = ui.available_width();
            let left = width * 0.58;
            let height = (ui.ctx().content_rect().height() * 0.30).clamp(180., 300.) + 110.;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left, height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        self.example_editor(ui, i);
                        (run_clicked, stop_clicked) = self.example_actions(ui);
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2((width - left - 12.).max(200.), height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.example_output(ui, i);
                    },
                );
            });
        } else {
            self.example_editor(ui, i);
            (run_clicked, stop_clicked) = self.example_actions(ui);
            self.example_output(ui, i);
        }
        if run_clicked {
            let l = self
                .progress
                .attempts
                .get_mut(&id)
                .unwrap()
                .workspace
                .as_mut()
                .unwrap()
                .script_learning
                .as_mut()
                .unwrap();
            l.example_result = None;
            self.script_job = Some(scripting::Job::example(
                id,
                l.example_source.clone(),
                lesson.fixture.clone(),
            ));
            self.notice = None;
            self.dirty = true;
            ui.ctx().request_repaint();
        }
        if stop_clicked {
            self.script_job = None;
            self.notice =
                Some("Example stopped. Your example and assignment drafts stay saved.".into());
        }
        if let Some(notice) = &self.notice {
            ui.label(notice);
        }
        if self.script_job.is_some() {
            ui.label("Running example · bounded simulation");
        }
        let r = ui.add(button("Open full assignment"));
        self.metrics.assignment_action = Some(r.rect);
        if r.clicked() {
            self.set_script_view(i, teaching::View::Assignment);
        }
    }
    fn example_editor(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.label("Type in this example editor, then choose Run example.");
        let l = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .script_learning
            .as_mut()
            .unwrap();
        let height = (ui.ctx().content_rect().height() * 0.30).clamp(180., 300.);
        let area = egui::ScrollArea::both()
            .id_salt(("example-scroll", i))
            .max_height(height)
            .min_scrolled_height(height)
            .show(ui, |ui| {
                ui.add_sized(
                    [ui.available_width(), height],
                    egui::TextEdit::multiline(&mut l.example_source)
                        .id_salt(("example-editor", i))
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .char_limit(shell::MAX_SOURCE),
                )
            });
        name_control(&area.inner, "Practice example script source — ungraded");
        self.metrics.editor_viewport = Some(area.inner_rect);
        let r = area.inner;
        self.metrics.inputs.push(r.rect);
        self.metrics.focus = Some(r.id);
        if self.focus_input && ui.is_enabled() {
            r.request_focus();
            self.focus_input = false;
        }
        if r.changed() {
            workspace::bound(&mut l.example_source, shell::MAX_SOURCE);
            l.example_result = None;
            self.script_job = None;
            self.notice = None;
            self.dirty = true;
        }
        ui.label(
            RichText::new("Tab indents · Ctrl+Enter runs · Esc releases editor focus")
                .small()
                .color(palette(ui).muted),
        );
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            r.surrender_focus();
        }
    }
    fn example_actions(&mut self, ui: &mut egui::Ui) -> (bool, bool) {
        let running = self.script_job.is_some();
        let mut run = false;
        let mut stop = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(!running, primary(ui, "Run example"));
            self.metrics.check = Some(r.rect);
            run = r.clicked()
                || (!running
                    && ui.is_enabled()
                    && self
                        .metrics
                        .focus
                        .is_some_and(|id| ui.memory(|m| m.has_focus(id)))
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter)));
            let r = ui.add_enabled(running, button("Stop"));
            self.metrics.stop = Some(r.rect);
            stop = r.clicked();
        });
        (run, stop)
    }
    fn example_output(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.label("Example output · No completion credit");
        let l = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref()
            .unwrap()
            .script_learning
            .as_ref()
            .unwrap();
        if l.example_result.is_some() && l.example_revision != shell::EVALUATOR_REVISION {
            ui.label("Earlier saved example output · Run the example again to check it with this simulator.");
        }
        let output = egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui, |ui| {
            egui::ScrollArea::both().id_salt(("example-output", i)).max_height(300.).min_scrolled_height(if l.example_result.is_some() { 180. } else { 60. }).auto_shrink([false, l.example_result.is_none()]).show(ui, |ui| {
                if let Some(r) = &l.example_result {
                    scripting::outcome(ui, r);
                    for (path, content) in &r.files { ui.label(RichText::new(format!("Virtual file {path}\n{content}")).monospace()); }
                    if l.example_revision==shell::EVALUATOR_REVISION && r.error.is_none() { ui.label("Example finished. Change a value and run again to see what changes."); }
                } else { ui.label("Run the example to see its printed text and any virtual file changes here."); }
            });
        });
        self.metrics.output = Some(output.response.rect);
    }
}
