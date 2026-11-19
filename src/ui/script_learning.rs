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
            self.metrics.script_goal = Some(ui.label(&s.objective).rect);
            self.scripting(ui, i);
            return;
        };
        match view {
            teaching::View::Introduction => {
                lesson_presentation::concepts(
                    ui,
                    &s,
                    &lesson.prerequisite_note,
                    lesson
                        .concepts
                        .iter()
                        .map(|c| (c.title.clone(), c.text.clone())),
                );
                lesson_presentation::worked(ui, &lesson.inputs);
                if ui.available_width() >= 760. {
                    ui.columns(2, |columns| {
                        worked_script(&mut columns[0], &s.id, &lesson.example);
                        expected_preview(&mut columns[1], lesson);
                    });
                } else {
                    worked_script(ui, &s.id, &lesson.example);
                    expected_preview(ui, lesson);
                }
                ui.add_space(8.);
                lesson_presentation::worked_rows(ui, &s);
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
                self.metrics.script_goal = Some(lesson_presentation::assignment(ui, &s).rect);
                self.scripting(ui, i);
            }
        }
    }
    fn script_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &teaching::Lesson) {
        let scenario = self.scenarios[i].clone();
        lesson_presentation::prose(ui, &lesson.inputs);
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
                        lesson_presentation::practice_step(ui, &scenario, 0);
                        self.example_editor(ui, i);
                        lesson_presentation::practice_step(ui, &scenario, 1);
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
            lesson_presentation::practice_step(ui, &scenario, 0);
            self.example_editor(ui, i);
            lesson_presentation::practice_step(ui, &scenario, 1);
            (run_clicked, stop_clicked) = self.example_actions(ui);
            self.example_output(ui, i);
        }
        // Keep authored follow-up investigations visible after the editor and result.
        // Most lessons have two actions; repair projects also teach a retry step.
        if let Some(entry) = lesson_presentation::entry(&scenario) {
            for n in 2..entry.practice.len() {
                lesson_presentation::practice_step(ui, &scenario, n);
            }
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
        ui.strong("Example editor");
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
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("example-scroll", i))
                .max_height(height)
                .min_scrolled_height(height),
            |ui| {
                ui.add_sized(
                    [ui.available_width(), height],
                    egui::TextEdit::multiline(&mut l.example_source)
                        .id_salt(("example-editor", i))
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .char_limit(shell::MAX_SOURCE),
                )
            },
        );
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
        ui.strong("Actual result");
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
        let output = egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(10.)
            .show(ui, |ui| {
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt(("example-output", i))
                        .max_height(300.)
                        .min_scrolled_height(if l.example_result.is_some() {
                            180.
                        } else {
                            60.
                        })
                        .auto_shrink([false, l.example_result.is_none()]),
                    |ui| {
                        if let Some(r) = &l.example_result {
                            let _ = scripting::outcome(ui, r);
                            for (path, content) in &r.files {
                                ui.label(
                                    RichText::new(format!("Virtual file {path}\n{content}"))
                                        .monospace(),
                                );
                            }
                            if l.example_revision == shell::EVALUATOR_REVISION && r.error.is_none()
                            {
                                ui.label("Run finished.");
                            }
                        } else {
                            ui.label("Not run. Choose Run example.");
                        }
                    },
                );
            });
        self.metrics.output = Some(output.response.rect);
        let lesson = self.scenarios[i].teaching.as_ref().unwrap();
        controls::collapsing_header(ui, "Expected preview · original example")
            .id_salt(("example-expected-preview", i))
            .show(ui, |ui| expected_preview(ui, lesson));
    }
}

pub(super) fn worked_script(ui: &mut egui::Ui, id: &str, source: &str) -> egui::Rect {
    ui.strong("Example script");
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let height = ui
        .fonts_mut(|f| f.layout_no_wrap(source.into(), font, color).size().y)
        .min(300.);
    egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(10.)
        .show(ui, |ui| {
            controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(("intro-script", id))
                    .min_scrolled_height(height)
                    .max_height(height),
                |ui| {
                    lesson_presentation::monospace(ui, source, false);
                },
            );
        })
        .response
        .rect
}
fn expected_preview(ui: &mut egui::Ui, lesson: &teaching::Lesson) {
    ui.strong("Expected preview · original example");
    ui.label("These values describe the supplied script, before your edits.");
    for (label, text) in [
        ("Standard output", &lesson.fixture.stdout),
        ("Standard error", &lesson.fixture.stderr),
    ] {
        ui.strong(label);
        if text.is_empty() {
            ui.label("(empty)");
        } else {
            lesson_presentation::monospace(ui, text, true);
        }
    }
    ui.label(format!("Expected exit status: {}", lesson.fixture.status));
    let changed: Vec<_> = lesson
        .fixture
        .after
        .iter()
        .filter(|(path, text)| lesson.fixture.files.get(*path) != Some(*text))
        .collect();
    if !changed.is_empty() {
        controls::collapsing_header(ui, "Expected simulated file changes").show(ui, |ui| {
            for (path, text) in changed {
                ui.monospace(path);
                lesson_presentation::monospace(ui, text, true);
            }
        });
    }
}
