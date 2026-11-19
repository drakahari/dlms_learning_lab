//! Native virtual PAM editor, supplied inputs and actual control-flow traces.
use super::*;
use dlms_learning_lab::pam_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub running: bool,
}
pub(super) struct ActiveRun {
    scenario: String,
    practice: bool,
    draft: String,
    run: work::Run,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn facts(ui: &mut egui::Ui, task: &work::Task) {
    lesson_presentation::rich(
        ui,
        RichText::new("Supplied samples · expected decisions").strong(),
    );
    lesson_presentation::prose(
        ui,
        "These are synthetic password-check results, not credentials to enter.",
    );
    for case in &task.cases {
        ui.add_space(5.);
        lesson_presentation::rich(ui, RichText::new(&case.label).strong());
        lesson_presentation::monospace(ui, case.facts.unix_result.label(), true);
        lesson_presentation::prose(
            ui,
            if case.accepted {
                "Expected: Accepted"
            } else {
                "Expected: Rejected"
            },
        );
    }
}
fn results(ui: &mut egui::Ui, rows: &[work::CaseResult], m: &mut Metrics) {
    ui.separator();
    lesson_presentation::rich(ui, RichText::new("Actual results").strong());
    let passed = rows.iter().filter(|r| r.passed()).count();
    let all = passed == rows.len();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(format!(
            "{} · {passed}/{} samples match",
            if all {
                "Stack meets every requirement"
            } else {
                "Stack needs work"
            },
            rows.len()
        ))
        .strong()
        .color(if all {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, rows.len(), r.rect));
    if let Some(diagnostic) = rows.first().and_then(|r| r.diagnostic.as_deref())
        && rows
            .iter()
            .all(|r| r.actual.is_none() && r.diagnostic.as_deref() == Some(diagnostic))
    {
        lesson_presentation::prose(ui, diagnostic);
        lesson_presentation::prose(
            ui,
            "No authentication decisions were produced. Fix the configuration, then choose Test stack; a parser error is not a successful rejection.",
        );
        return;
    }
    for (index, row) in rows.iter().enumerate() {
        ui.add_space(6.);
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if row.passed() { "Passed" } else { "Needs work" },
                row.case.label
            ))
            .strong(),
        );
        lesson_presentation::prose(ui, &row.feedback());
        if let Some(actual) = &row.actual {
            ui.push_id(&row.case.id, |ui| {
                let response = controls::collapsing_header(ui, "Visited rules").show(ui, |ui| {
                    for s in &actual.trace {
                        lesson_presentation::monospace(
                            ui,
                            &format!(
                                "Line {}: auth {} {} → {}",
                                s.line,
                                s.control.label(),
                                s.module.label(),
                                s.returned.label()
                            ),
                            true,
                        );
                        lesson_presentation::prose(ui, s.action.label());
                        if let Some(code) = s.remembered_failure {
                            lesson_presentation::prose(
                                ui,
                                &format!("Failure still remembered: {}", code.label()),
                            );
                        }
                    }
                    lesson_presentation::monospace(
                        ui,
                        &format!("Final result: {}", actual.result.label()),
                        true,
                    );
                });
                record(m, &format!("trace:{index}"), &response.header_response, ui);
            });
        }
    }
}
struct Editor<'a> {
    source: &'a Scenario,
    definition: &'a work::Definition,
    visited: u64,
    practice: bool,
    active: bool,
    running: bool,
}
impl Editor<'_> {
    fn show(
        &self,
        ui: &mut egui::Ui,
        scope: &mut work::Scope,
        m: &mut Metrics,
    ) -> Result<(bool, bool), String> {
        lesson_presentation::rich(
            ui,
            RichText::new(if self.practice {
                "example-login · virtual auth stack"
            } else {
                "lab-login · virtual auth stack"
            })
            .strong(),
        );
        if self.practice {
            let r = ui.add_enabled(self.active && !self.running, button("Fill worked example"));
            record(m, "insert", &r, ui);
            if r.clicked() {
                scope.edit(&self.definition.example_worked)?;
            }
        }
        let mut draft = scope.draft.clone();
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.
            + 2. * f32::from(controls::PAD_Y);
        let mut clip = egui::Rect::NOTHING;
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt((
                    "pam-editor-scroll",
                    &self.source.id,
                    self.visited,
                    self.practice,
                ))
                .max_height(height)
                .min_scrolled_height(height)
                .auto_shrink([false, false]),
            |ui| {
                clip = ui.clip_rect();
                ui.add_enabled(
                    self.active && !self.running,
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .id(egui::Id::new((
                            "pam-editor",
                            &self.source.id,
                            self.visited,
                            self.practice,
                        )))
                        .desired_width(f32::INFINITY)
                        .desired_rows(9)
                        .char_limit(8192),
                )
            },
        );
        let r = area.inner;
        name_control(&r, "Virtual PAM authentication stack");
        m.editor = Some((r.rect.intersect(clip), r.id));
        let shortcut = r.has_focus()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            r.surrender_focus();
        }
        if r.changed() {
            workspace::bound(&mut draft, 8192);
            scope.edit(&draft)?;
        }
        lesson_presentation::rich(
            ui,
            RichText::new("Tab indents · Ctrl+Enter tests · Esc releases focus").small(),
        );
        if self.practice {
            lesson_presentation::practice_step(ui, self.source, 1);
        }
        let mut start = false;
        let mut stop = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(self.active && !self.running, primary(ui, "Test stack"));
            record(m, "run", &r, ui);
            start = r.clicked() || shortcut;
            let r = ui.add_enabled(self.active && self.running, button("Stop"));
            record(m, "stop", &r, ui);
            stop = r.clicked();
        });
        if self.running {
            lesson_presentation::prose(ui, "Checking the supplied samples…");
        } else if scope.tested.as_ref() != Some(&scope.draft) {
            lesson_presentation::prose(ui, "Not checked · test this draft for actual results.");
        }
        Ok((start, stop))
    }
}
impl LabApp {
    pub(super) fn pam_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.pam_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            self.pam_run = None;
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::PamStack { definition }, engine::State::PamStack { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
        let visited = a.visited;
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            candidate.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    self.metrics.lesson_tabs.push(r.rect);
                    record(&mut self.metrics.pam, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            candidate.view = view;
        }
        let practice = view == teaching::View::Example;
        let task = if practice {
            &definition.example
        } else {
            &definition.initial
        };
        if self.pam_run.as_ref().is_some_and(|j| {
            j.scenario != source.id
                || j.practice != practice
                || view == teaching::View::Introduction
                || j.draft
                    != if practice {
                        &candidate.practice.draft
                    } else {
                        &candidate.assignment.draft
                    }
                    .as_str()
        }) {
            self.pam_run = None;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on ordinary local account administration.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.pam, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let running = self.pam_run.is_some();
            let edit = Editor {
                source: &source,
                definition,
                visited,
                practice,
                active,
                running,
            };
            let mut actions = Ok((false, false));
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    actions = edit.show(&mut columns[0], scope, &mut self.metrics.pam);
                    facts(&mut columns[1], task);
                });
            } else {
                actions = edit.show(ui, scope, &mut self.metrics.pam);
                ui.separator();
                facts(ui, task);
            }
            match actions {
                Err(e) => self.notice = Some(e),
                Ok((start, stop)) => {
                    if stop {
                        if let Some(mut j) = self.pam_run.take() {
                            j.run.cancel();
                        }
                        self.notice = Some(
                            "Stopped · draft retained; unfinished checks award no credit.".into(),
                        );
                    }
                    if start && !running && active {
                        match scope.begin(task) {
                            Ok(run) => {
                                self.pam_run = Some(ActiveRun {
                                    scenario: source.id.clone(),
                                    practice,
                                    draft: scope.draft.clone(),
                                    run,
                                })
                            }
                            Err(e) => self.notice = Some(e),
                        }
                    }
                }
            }
            if active && let Some(mut job) = self.pam_run.take() {
                if job.run.tick() {
                    if let Err(e) = scope.finish(task, &job.run) {
                        self.notice = Some(e);
                    }
                } else {
                    self.pam_run = Some(job);
                }
                ui.ctx().request_repaint();
            }
            self.metrics.pam.running = self.pam_run.is_some();
            self.metrics.check = self
                .metrics
                .pam
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .map(|r| r.1);
            if !self.metrics.pam.running && scope.tested.as_ref() == Some(&scope.draft) {
                match task.run(&scope.draft) {
                    Ok(rows) => results(ui, &rows, &mut self.metrics.pam),
                    Err(e) => {
                        lesson_presentation::prose(ui, &e);
                    }
                }
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r =
                    ui.add_enabled(active && !self.metrics.pam.running, button("Reset example"));
                record(&mut self.metrics.pam, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("pam-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.pam, "hint", &r, ui);
                        if r.clicked() {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals a solution")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                                for answer in &source.solution {
                                    if let Some(raw) = answer.strip_prefix(work::PREFIX)
                                        && let Ok(text) = serde_json::from_str::<String>(raw)
                                    {
                                        lesson_presentation::monospace(ui, &text, false);
                                    }
                                }
                            });
                    }
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::PamStack {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
