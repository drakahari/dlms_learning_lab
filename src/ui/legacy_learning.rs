//! Two concrete legacy terminal teaching views; old assignment models are retained.
use super::seed_learning::{seed_checks, seed_message, seed_picker, seed_terminal};
use super::*;
use dlms_learning_lab::{legacy_access_teaching as access, legacy_logs_teaching as logs};
impl LabApp {
    pub(super) fn legacy_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        let w = a.workspace.as_ref().unwrap();
        w.legacy_access_learning
            .as_ref()
            .map(|p| p.view)
            .or_else(|| w.legacy_logs_learning.as_ref().map(|p| p.view))
            .unwrap_or(teaching::View::Assignment)
    }
    pub(super) fn set_legacy_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(p) = &mut w.legacy_access_learning {
            p.view = view;
        }
        if let Some(p) = &mut w.legacy_logs_learning {
            p.view = view;
        }
        self.focus_input = true;
        self.scroll_command = false;
        self.lesson_scroll_reset = true;
        self.help_open = false;
        self.notice = None;
        self.dirty = true;
    }
    pub(super) fn legacy_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.legacy_view(i);
        ui.heading(&s.title);
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
                        self.set_legacy_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let (note, concepts, description) = if let Some(l) = &s.legacy_access_teaching {
                    (&l.prerequisite_note, &l.concepts, &l.example_description)
                } else {
                    let l = s.legacy_logs_teaching.as_ref().unwrap();
                    (&l.prerequisite_note, &l.concepts, &l.example_description)
                };
                ui.label(RichText::new(note).color(palette(ui).muted));
                for c in concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(description);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_legacy_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_legacy_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                if let Some(l) = &s.legacy_access_teaching {
                    ui.push_id(("access-example", &s.id), |ui| {
                        self.legacy_access_example(ui, i, l)
                    });
                } else {
                    ui.push_id(("log-example", &s.id), |ui| {
                        self.legacy_logs_example(ui, i, s.legacy_logs_teaching.as_ref().unwrap())
                    });
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let active = ui.is_enabled()
                    && !(self.reset_pending
                        || self.settings_open
                        || self.theme_preview
                        || self.data_tools.modal_open());
                ui.push_id(("legacy-assignment", &s.id), |ui| {
                    ui.add_enabled_ui(active, |ui| {
                        self.terminal(ui, i);
                        self.check_terminal_work(ui, i);
                    });
                });
            }
        }
    }
    fn legacy_access_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &access::Lesson) {
        let id = self.scenarios[i].id.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        ui.label("Practice example · Independent; no assignment credit.");
        ui.label(&lesson.example_description);
        ui.label("Virtual administrator · Changes affect only these supplied practice records.");
        let l = self
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .legacy_access_learning
            .as_mut()
            .unwrap();
        let mut changed = false;
        let mut assignment = false;
        ui.add_enabled_ui(active, |ui| {
            let labels: Vec<_> = lesson
                .commands
                .iter()
                .map(|c| (c.command.as_str(), c.explanation.as_str()))
                .collect();
            let before = l.selected_action;
            if seed_picker(ui, &mut l.selected_action, &labels, &mut self.metrics) {
                if let Err(e) = l.insert(lesson) {
                    self.notice = Some(seed_message(&e));
                }
                changed = true;
                self.focus_input = true;
            }
            changed |= before != l.selected_action;
            if lesson.commands[l.selected_action].simulator_only {
                ui.label("Simulator-only access probe; this is not a Linux command.");
            }
            ui.label(format!(
                "Target {} · requested user {}",
                lesson.task.path, lesson.task.user
            ));
            ui.label(format!("Try a change: {}", lesson.try_change));
            let (run, edited) = seed_terminal(
                ui,
                &mut l.command,
                &l.transcript,
                &id,
                "legacy-access",
                &mut self.metrics,
                &mut self.focus_input,
                &mut self.scroll_command,
                active,
            );
            changed |= edited;
            if run {
                let _ = l.submit(lesson);
                changed = true;
                self.focus_input = true;
            }
            let r = ui.add(primary(ui, "Check example"));
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                l.checked = true;
                changed = true;
            }
            if l.checked {
                seed_checks(ui, l.checks(lesson), &mut self.metrics);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Reset example only"));
                self.metrics.example_reset = Some(r.rect);
                if r.clicked() {
                    *l = access::Learning::new(lesson).expect("validated lesson");
                    l.view = teaching::View::Example;
                    self.notice = None;
                    changed = true;
                }
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                assignment = r.clicked();
            });
        });
        self.dirty |= changed;
        if assignment {
            self.set_legacy_view(i, teaching::View::Assignment);
        }
    }
    fn legacy_logs_example(&mut self, ui: &mut egui::Ui, i: usize, lesson: &logs::Lesson) {
        let id = self.scenarios[i].id.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        ui.label("Practice example · Independent; no assignment credit.");
        ui.label(&lesson.example_description);
        ui.label("Supplied virtual logs · Queries select text; they never edit a file.");
        let l = self
            .progress
            .attempts
            .get_mut(&id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .legacy_logs_learning
            .as_mut()
            .unwrap();
        let mut changed = false;
        let mut assignment = false;
        ui.add_enabled_ui(active, |ui| {
            let labels: Vec<_> = lesson
                .commands
                .iter()
                .map(|c| (c.command.as_str(), c.explanation.as_str()))
                .collect();
            let before = l.selected_action;
            if seed_picker(ui, &mut l.selected_action, &labels, &mut self.metrics) {
                if let Err(e) = l.insert(lesson) {
                    self.notice = Some(seed_message(&e));
                }
                changed = true;
                self.focus_input = true;
            }
            changed |= before != l.selected_action;
            ui.label(format!("Try a change: {}", lesson.try_change));
            let (run, edited) = seed_terminal(
                ui,
                &mut l.command,
                &l.transcript,
                &id,
                "legacy-logs",
                &mut self.metrics,
                &mut self.focus_input,
                &mut self.scroll_command,
                active,
            );
            changed |= edited;
            if run {
                let _ = l.submit(lesson);
                changed = true;
                self.focus_input = true;
            }
            let r = ui.add(primary(ui, "Check example"));
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                l.checked = true;
                changed = true;
            }
            if l.checked {
                seed_checks(ui, l.checks(lesson), &mut self.metrics);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Reset example only"));
                self.metrics.example_reset = Some(r.rect);
                if r.clicked() {
                    *l = logs::Learning::new(lesson).expect("validated lesson");
                    l.view = teaching::View::Example;
                    self.notice = None;
                    changed = true;
                }
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                assignment = r.clicked();
            });
        });
        self.dirty |= changed;
        if assignment {
            self.set_legacy_view(i, teaching::View::Assignment);
        }
    }
}
