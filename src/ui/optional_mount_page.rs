//! Native optional-media configuration with separate requirement and mount outcomes.
use super::*;
use dlms_learning_lab::removable_workspace as work;
use std::sync::Arc;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub running: bool,
    pub editor_scroll: Option<(f32, f32)>,
    pub actual_required: Option<bool>,
}
pub(super) struct ActiveRun {
    pub scenario: String,
    pub practice: bool,
    draft: String,
    run: work::Run,
}
#[derive(Clone)]
struct Report {
    task: work::Task,
    draft: String,
    rows: Vec<work::CaseResult>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn dependency_name(d: dlms_learning_lab::removable_mount::Dependency) -> &'static str {
    use dlms_learning_lab::removable_mount::Dependency::*;
    match d {
        Required => "Required automatic request",
        Wanted => "Optional automatic request",
        NotEnrolled => "Not requested automatically",
    }
}
fn facts(ui: &mut egui::Ui, task: &work::Task, selected: &mut usize, m: &mut Metrics) {
    lesson_presentation::rich(ui, RichText::new("Supplied media sample").strong());
    let r = controls::dropdown(ui, "optional-input-case", ui.available_width())
        .selected_text(&task.cases[*selected].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let start = m.controls.len();
            for (n, c) in task.cases.iter().enumerate() {
                let r = ui.selectable_value(selected, n, &c.label);
                record(m, &format!("case:{n}"), &r, ui);
            }
            let clip = controls::popup_content_clip(ui);
            for c in &mut m.controls[start..] {
                c.2 = clip;
            }
        });
    record(m, "case", &r.response, ui);
    let env = &task.cases[*selected].environment;
    for (device, id) in env.inventory.devices() {
        let v = &env.inventory.volumes()[id];
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{id} · {}",
                if env.present.contains(id) {
                    "Available"
                } else {
                    "Absent in this sample"
                }
            ))
            .strong(),
        );
        lesson_presentation::monospace(
            ui,
            &format!(
                "UUID={}\nTYPE={} · alias {device}",
                v.uuid,
                v.filesystem.name()
            ),
            true,
        );
    }
}
fn report(
    ui: &mut egui::Ui,
    rows: &[work::CaseResult],
    _task: &work::Task,
    selected: usize,
    m: &mut Metrics,
) {
    let passed = rows.iter().filter(|r| r.passed()).count();
    let color = if passed == rows.len() {
        palette(ui).success
    } else {
        palette(ui).error
    };
    let r = lesson_presentation::rich(
        ui,
        RichText::new(format!(
            "{} · {passed}/{} samples meet task requirements",
            if passed == rows.len() {
                "Task requirements met"
            } else {
                "Policy needs work"
            },
            rows.len()
        ))
        .color(color)
        .strong(),
    );
    m.result = Some((passed, rows.len(), r.rect));
    for row in rows {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if row.passed() {
                    "Requirement checks met"
                } else {
                    "Requirements need work"
                },
                row.label
            ))
            .strong(),
        );
        if let Some((_, text)) = row.checks.iter().find(|r| !r.0) {
            lesson_presentation::prose(ui, text);
        }
        controls::collapsing_header(ui, "Objective results")
            .id_salt(("optional-objectives", &row.id))
            .show(ui, |ui| {
                for (ok, text) in &row.checks {
                    lesson_presentation::prose(
                        ui,
                        &format!("{} · {text}", if *ok { "Met" } else { "Needs work" }),
                    );
                }
            });
    }
    let row = &rows[selected];
    if let Some(p) = &row.projection {
        m.actual_required = Some(p.required_attempts_satisfied());
        ui.separator();
        lesson_presentation::rich(
            ui,
            RichText::new(format!("Actual mount attempts · {}", row.label)).strong(),
        );
        for attempt in &p.rows {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "{} · {}",
                    attempt.target,
                    dependency_name(attempt.dependency)
                ),
                true,
            );
            use dlms_learning_lab::removable_mount::Outcome;
            match &attempt.outcome {
                Outcome::Mounted => {
                    let mount = &p.state.live()[&attempt.target];
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "Mounted volume {} · {}",
                            mount.volume,
                            if mount.read_only {
                                "Read only"
                            } else {
                                "Read/write"
                            }
                        ),
                    );
                }
                Outcome::Manual => {
                    lesson_presentation::prose(
                        ui,
                        "Not mounted · no automatic request from this table.",
                    );
                }
                Outcome::Failed(e) => {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!("Failed · {e}")).color(palette(ui).error),
                    );
                }
            }
        }
        lesson_presentation::rich(
            ui,
            RichText::new(
                if !p.rows.iter().any(|r| {
                    r.dependency == dlms_learning_lab::removable_mount::Dependency::Required
                }) {
                    "Required-storage result · no required requests remain"
                } else if p.required_attempts_satisfied() {
                    "Required-storage result · all required attempts succeed"
                } else {
                    "Required-storage result · a required mount failed"
                },
            )
            .strong(),
        );
    }
}
struct EditContext<'a> {
    source: &'a Scenario,
    definition: &'a work::Definition,
    visited: u64,
    practice: bool,
    active: bool,
    running: bool,
}
impl EditContext<'_> {
    fn editor(
        &self,
        ui: &mut egui::Ui,
        scope: &mut work::Scope,
        m: &mut Metrics,
    ) -> Result<(bool, bool), String> {
        let Self {
            source,
            definition,
            visited,
            practice,
            active,
            running,
        } = *self;
        let mut start = false;
        let mut stop = false;
        lesson_presentation::rich(
            ui,
            RichText::new("Mount table editor · virtual /etc/fstab").strong(),
        );
        if practice {
            let r = ui.add_enabled(active && !running, button("Insert worked policy"));
            record(m, "insert", &r, ui);
            if r.clicked() {
                scope
                    .replace(&definition.example_worked)
                    .expect("validated example");
            }
        }
        let mut draft = scope.draft.clone();
        let editor_height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.
            + 2. * f32::from(controls::PAD_Y);
        let mut visible_editor = egui::Rect::NOTHING;
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt(("optional-editor-scroll", &source.id, visited, practice))
                .max_height(editor_height)
                .min_scrolled_height(editor_height)
                .auto_shrink([false, false]),
            |ui| {
                visible_editor = ui.clip_rect();
                ui.add_enabled(
                    active && !running,
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .id(egui::Id::new((
                            "optional-config-editor",
                            &source.id,
                            visited,
                            practice,
                        )))
                        .desired_width(f32::INFINITY)
                        .desired_rows(9)
                        .char_limit(16384),
                )
            },
        );
        m.editor_scroll = Some((area.inner_rect.height(), area.state.offset.y));
        let r = area.inner;
        name_control(&r, "Virtual mount table configuration");
        m.editor = Some((r.rect.intersect(visible_editor), r.id));
        let shortcut = r.has_focus()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            r.surrender_focus();
        }
        if r.changed() {
            workspace::bound(&mut draft, 16384);
            scope.replace(&draft)?;
        }
        lesson_presentation::rich(
            ui,
            RichText::new("Tab indents · Ctrl+Enter runs · Esc releases focus").small(),
        );
        if practice {
            lesson_presentation::practice_step(ui, source, 1);
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(active && !running, primary(ui, "Run mount checks"));
            record(m, "run", &r, ui);
            start = r.clicked() || shortcut;

            let r = ui.add_enabled(active && running, button("Stop"));
            record(m, "stop", &r, ui);
            stop = r.clicked();
        });
        if running {
            lesson_presentation::prose(ui, "Checking supplied media samples…");
        } else if scope.tested.as_deref() != Some(scope.draft.as_str()) {
            lesson_presentation::prose(
                ui,
                "Not checked · run the current policy for actual results.",
            );
        }
        Ok((start, stop))
    }
}

impl LabApp {
    pub(super) fn optional_mount_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.optional_mount_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            self.optional_mount_run = None;
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (
            engine::Goal::RemovableMounts { definition },
            engine::State::RemovableMounts { simulation },
        ) = (&source.goal, &a.state)
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
                    record(&mut self.metrics.optional_mount, key, &r, ui);
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
        if self.optional_mount_run.as_ref().is_some_and(|job| {
            job.scenario != source.id
                || job.practice != practice
                || view == teaching::View::Introduction
                || &job.draft
                    != if practice {
                        &candidate.practice.draft
                    } else {
                        &candidate.assignment.draft
                    }
        }) {
            self.optional_mount_run = None;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on filesystem identity and the supplied inventory. This task edits saved virtual mount requests.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::monospace(ui, &definition.example_worked, true);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.optional_mount, key, &r, ui);
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
            let running = self.optional_mount_run.is_some();
            let mut start = false;
            let mut stop = false;
            let edit_context = EditContext {
                source: &source,
                definition,
                visited,
                practice,
                active,
                running,
            };
            let selected_id = ui.id().with(("optional-selected-case", practice));
            let mut selected = ui
                .ctx()
                .data_mut(|d| d.get_temp::<usize>(selected_id).unwrap_or(0))
                .min(task.cases.len() - 1);
            let mut edit_error = None;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    match edit_context.editor(
                        &mut columns[0],
                        scope,
                        &mut self.metrics.optional_mount,
                    ) {
                        Ok((a, b)) => {
                            start = a;
                            stop = b;
                        }
                        Err(e) => edit_error = Some(e),
                    }
                    facts(
                        &mut columns[1],
                        task,
                        &mut selected,
                        &mut self.metrics.optional_mount,
                    );
                });
            } else {
                match edit_context.editor(ui, scope, &mut self.metrics.optional_mount) {
                    Ok((a, b)) => {
                        start = a;
                        stop = b;
                    }
                    Err(e) => edit_error = Some(e),
                }
                ui.separator();
                facts(ui, task, &mut selected, &mut self.metrics.optional_mount);
            }
            ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
            if let Some(e) = edit_error {
                self.notice = Some(e);
            }
            self.metrics.check = self
                .metrics
                .optional_mount
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .map(|r| r.1);
            if stop {
                if let Some(mut job) = self.optional_mount_run.take() {
                    job.run.cancel();
                }
                self.notice =
                    Some("Stopped · draft retained; unfinished tests award no credit.".into());
            }
            if start && !running && active {
                match scope.begin(task) {
                    Ok(run) => {
                        self.optional_mount_run = Some(ActiveRun {
                            scenario: source.id.clone(),
                            practice,
                            draft: scope.draft.clone(),
                            run,
                        })
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            if active && let Some(mut job) = self.optional_mount_run.take() {
                if job.run.tick() {
                    if let Err(e) = scope.finish(task, &job.run) {
                        self.notice = Some(e);
                    }
                } else {
                    self.optional_mount_run = Some(job);
                }
                ui.ctx().request_repaint();
            }
            self.metrics.optional_mount.running = self.optional_mount_run.is_some();
            if scope.tested.as_deref() == Some(scope.draft.as_str()) {
                let cache_id = ui.id().with(("optional-report", &source.id, practice));
                let cached = ui.ctx().data_mut(|d| d.get_temp::<Arc<Report>>(cache_id));
                let cached =
                    if let Some(r) = cached.filter(|r| r.task == *task && r.draft == scope.draft) {
                        r
                    } else {
                        let r = Arc::new(Report {
                            task: task.clone(),
                            draft: scope.draft.clone(),
                            rows: task.run(&scope.draft).expect("validated bounded test"),
                        });
                        ui.ctx().data_mut(|d| d.insert_temp(cache_id, r.clone()));
                        r
                    };
                ui.separator();
                report(
                    ui,
                    &cached.rows,
                    task,
                    selected,
                    &mut self.metrics.optional_mount,
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(
                    active && !self.metrics.optional_mount.running,
                    button("Reset example only"),
                );
                record(&mut self.metrics.optional_mount, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("optional-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.optional_mount, "hint", &r, ui);
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
                                    lesson_presentation::monospace(
                                        ui,
                                        answer.strip_prefix(work::PREFIX).unwrap_or(answer),
                                        false,
                                    );
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
                        engine::State::RemovableMounts {
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
