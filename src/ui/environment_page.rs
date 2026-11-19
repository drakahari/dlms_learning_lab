//! Native setup editor and actual inherited-state evidence. No host child exists.
use super::*;
use dlms_learning_lab::environment_workspace as env;

pub(super) struct ActiveRun {
    pub scenario: String,
    pub example: bool,
    pub visited: u64,
    pub automatic: bool,
    runner: env::Job,
}
impl LabApp {
    fn start_environment(&mut self, i: usize, example: bool, automatic: bool) {
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        if env::context(a, s).is_err() {
            return;
        }
        let engine::State::Environment { simulation } = &a.state else {
            return;
        };
        let task = if example {
            &simulation.definition.example
        } else {
            &simulation.definition.assignment
        };
        let draft = if example {
            &simulation.example
        } else {
            &simulation.assignment
        };
        match env::Job::new(task, &draft.source) {
            Ok(runner) => {
                let id = s.id.clone();
                let visited = a.visited;
                if !automatic {
                    let engine::State::Environment { simulation } =
                        &mut self.progress.attempts.get_mut(&id).unwrap().state
                    else {
                        unreachable!()
                    };
                    if example {
                        simulation.example.result = None;
                    } else {
                        simulation.assignment.result = None;
                    }
                    self.environment_suppressed.remove(&(id.clone(), example));
                    self.notice = None;
                    self.dirty = true;
                }
                self.script_job = None;
                self.environment_run = Some(ActiveRun {
                    scenario: id,
                    example,
                    visited,
                    automatic,
                    runner,
                });
            }
            Err(e) => self.notice = Some(e),
        }
    }
    pub(super) fn advance_environment(&mut self, ctx: &egui::Context) {
        let allowed = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if let Some(job) = &self.environment_run {
            let valid = allowed
                && self.script_job.is_none()
                && self
                    .progress
                    .attempts
                    .get(&job.scenario)
                    .is_some_and(|a| a.visited == job.visited)
                && (job.automatic
                    || self.selected.is_some_and(|i| {
                        self.scenarios[i].id == job.scenario
                            && self.progress.attempts.get(&job.scenario).is_some_and(|a| {
                                let engine::State::Environment { simulation } = &a.state else {
                                    return false;
                                };
                                (if a.challenge {
                                    teaching::View::Assignment
                                } else {
                                    simulation.view
                                }) == if job.example {
                                    teaching::View::Example
                                } else {
                                    teaching::View::Assignment
                                }
                            })
                    }));
            if !valid {
                self.environment_run = None;
            }
        }
        if self.environment_run.is_none() && allowed && self.script_job.is_none() {
            let next = self.scenarios.iter().enumerate().find_map(|(i, s)| {
                let a = self.progress.attempts.get(&s.id)?;
                let engine::State::Environment { simulation } = &a.state else {
                    return None;
                };
                if env::context(a, s).is_err() {
                    return None;
                }
                [false, true]
                    .into_iter()
                    .find(|example| {
                        !self
                            .environment_suppressed
                            .contains(&(s.id.clone(), *example))
                            && if *example {
                                simulation
                                    .example
                                    .needs_reverification(&simulation.definition.example)
                            } else {
                                simulation
                                    .assignment
                                    .needs_reverification(&simulation.definition.assignment)
                            }
                    })
                    .map(|example| (i, example))
            });
            if let Some((i, example)) = next {
                self.start_environment(i, example, true);
            }
        }
        let Some(job) = &mut self.environment_run else {
            return;
        };
        job.runner.advance(16);
        if !job.runner.done() {
            ctx.request_repaint();
            return;
        }
        let job = self.environment_run.take().unwrap();
        if let Some(record) = job.runner.record() {
            if let Some(a) = self.progress.attempts.get_mut(&job.scenario)
                && a.visited == job.visited
                && let engine::State::Environment { simulation } = &mut a.state
            {
                let (task, draft) = if job.example {
                    (&simulation.definition.example, &mut simulation.example)
                } else {
                    (
                        &simulation.definition.assignment,
                        &mut simulation.assignment,
                    )
                };
                if draft.store(record, task) {
                    if self
                        .selected
                        .is_some_and(|i| self.scenarios[i].id == job.scenario)
                        && (if a.challenge {
                            teaching::View::Assignment
                        } else {
                            simulation.view
                        }) == if job.example {
                            teaching::View::Example
                        } else {
                            teaching::View::Assignment
                        }
                    {
                        self.script_case = env::checks(task, draft)
                            .iter()
                            .position(|(ok, _)| !*ok)
                            .unwrap_or(0);
                        self.checked = !job.example;
                    }
                    self.completion_index.forget(&job.scenario);
                    self.dirty = true;
                }
            }
        } else {
            self.environment_suppressed
                .insert((job.scenario.clone(), job.example));
            self.notice=Some("Run stopped or reached its result limit. Your script stays saved; no completion was awarded.".into());
        }
        ctx.request_repaint();
    }
    pub(super) fn environment_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            self.environment_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = env::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::Environment { simulation } = &a.state else {
            return;
        };
        let mut next = (**simulation).clone();
        let original = next.clone();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_value(&mut view, v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                }
            });
            if view != next.view {
                next.view = view;
                self.environment_run = None;
                self.focus_input = true;
                self.lesson_scroll_reset = true;
                self.notice = None;
            }
        }
        let lesson = source
            .teaching
            .as_ref()
            .expect("validated environment teaching");
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                &lesson.prerequisite_note,
                lesson
                    .concepts
                    .iter()
                    .map(|c| (c.title.clone(), c.text.clone())),
            );
            lesson_presentation::worked(ui, &lesson.inputs);
            let task = &next.definition.example;
            if ui.available_width() >= 780. {
                ui.columns(2, |col| {
                    col[0].strong("Worked setup script");
                    lesson_presentation::code_value(
                        &mut col[0],
                        ("env-worked", &source.id),
                        &task.starter,
                        f32::INFINITY,
                    );
                    expected_child(&mut col[1], task);
                });
            } else {
                ui.strong("Worked setup script");
                lesson_presentation::code_value(
                    ui,
                    ("env-worked", &source.id),
                    &task.starter,
                    ui.available_width(),
                );
                expected_child(ui, task);
            }
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(primary(ui, "Try this example"));
                self.metrics.example_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Example;
                    self.focus_input = true;
                    self.lesson_scroll_reset = true;
                }
                let r = ui.add(button("Go straight to assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.focus_input = true;
                    self.lesson_scroll_reset = true;
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if practice {
                lesson_presentation::prose(ui, &lesson.inputs);
            } else {
                self.metrics.script_goal = Some(lesson_presentation::assignment(ui, &source).rect);
            }
            let task = if practice {
                next.definition.example.clone()
            } else {
                next.definition.assignment.clone()
            };
            self.script_case = self.script_case.min(task.cases.len() - 1);
            ui.horizontal_wrapped(|ui| {
                ui.label("Inspect sample inputs");
                controls::dropdown(ui, ("env-inputs", &source.id, practice), 280.)
                    .selected_text(&task.cases[self.script_case].input.name)
                    .width(ui.available_width().min(280.))
                    .show_ui(ui, |ui| {
                        for (n, c) in task.cases.iter().enumerate() {
                            ui.selectable_value(&mut self.script_case, n, &c.input.name);
                        }
                    });
            });
            let selected = &task.cases[self.script_case];
            inputs(ui, selected, task.path_lookup.is_some());
            environment_lookup::supplied(ui, &task);
            lesson_presentation::prose(
                ui,
                "Each test uses its own sample inputs. Simulated files reset for each test; your script stays saved.",
            );
            let running = self
                .environment_run
                .as_ref()
                .is_some_and(|j| j.scenario == source.id && j.example == practice);
            let automatic = self
                .environment_run
                .as_ref()
                .is_some_and(|j| j.scenario == source.id && j.example == practice && j.automatic);
            let draft = if practice {
                &mut next.example
            } else {
                &mut next.assignment
            };
            let mut run = false;
            let mut stop = false;
            if ui.available_width() >= 850. {
                let width = ui.available_width();
                ui.columns(2, |cols| {
                    cols[0].set_max_width(width * 0.5 - 8.);
                    if practice {
                        lesson_presentation::practice_step(&mut cols[0], &source, 0);
                    }
                    self.environment_editor(&mut cols[0], &source, practice, draft, active);
                    if practice {
                        lesson_presentation::practice_step(&mut cols[0], &source, 1);
                    }
                    (run, stop) = self.environment_actions(
                        &mut cols[0],
                        practice,
                        task.path_lookup.is_some(),
                        running,
                        active,
                    );
                    if practice {
                        expected_sample(&mut cols[1], &task, self.script_case);
                    }
                    environment_result(
                        &mut cols[1],
                        &task,
                        draft,
                        self.script_case,
                        running,
                        automatic,
                        &mut self.metrics,
                    );
                });
            } else {
                if practice {
                    lesson_presentation::practice_step(ui, &source, 0);
                }
                self.environment_editor(ui, &source, practice, draft, active);
                if practice {
                    lesson_presentation::practice_step(ui, &source, 1);
                }
                (run, stop) = self.environment_actions(
                    ui,
                    practice,
                    task.path_lookup.is_some(),
                    running,
                    active,
                );
                if practice {
                    expected_sample(ui, &task, self.script_case);
                }
                environment_result(
                    ui,
                    &task,
                    draft,
                    self.script_case,
                    running,
                    automatic,
                    &mut self.metrics,
                );
            }
            if practice {
                if let Some(entry) = lesson_presentation::entry(&source) {
                    for n in 2..entry.practice.len() {
                        lesson_presentation::practice_step(ui, &source, n);
                    }
                }
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.focus_input = true;
                    self.lesson_scroll_reset = true;
                }
            }
            if !practice && draft.result.as_ref().is_some_and(|r| r.verified()) {
                let results = env::checks(&task, draft);
                let pass = results.iter().filter(|(ok, _)| *ok).count();
                self.metrics.script_case_rows.clear();
                if pass != results.len() {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "Overall result · {pass}/{} sample inputs pass",
                            results.len()
                        ))
                        .color(palette(ui).error),
                    );
                    for (n, (ok, message)) in results.iter().enumerate() {
                        let r = ui.selectable_label(
                            n == self.script_case,
                            format!(
                                "{} · {}",
                                if *ok { "Pass" } else { "Needs an edit" },
                                message
                            ),
                        );
                        self.metrics.script_case_rows.push((n, r.rect));
                        if r.clicked() {
                            self.script_case = n;
                        }
                    }
                }
            }
            controls::collapsing_header(ui, "Help and simulation scope")
                .id_salt(("env-help", &source.id, practice, challenge))
                .show(ui, |ui| {
                    lesson_presentation::prose(ui, &source.simplifications);
                    ui.strong("Supplied child · read-only");
                    lesson_presentation::code_value(
                        ui,
                        ("env-child", &source.id, practice),
                        &task.child_source,
                        ui.available_width(),
                    );
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        if ui
                            .add_enabled(
                                active && a.hints < source.hints.len(),
                                button("Show a hint"),
                            )
                            .clicked()
                        {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals a solution")
                            .id_salt(("env-solution-help", &source.id, challenge))
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                                for code in &source.solution {
                                    lesson_presentation::code_value(
                                        ui,
                                        ("env-solution", &source.id),
                                        code,
                                        ui.available_width(),
                                    );
                                }
                            });
                    }
                    ui.strong("References");
                    for url in &source.sources {
                        ui.hyperlink(url);
                    }
                });
            // Publish edits before starting a job; late results are source/context checked.
            if next != original {
                if next.view != original.view {
                    self.environment_run = None;
                }
                if next.assignment.source != original.assignment.source
                    || next.example.source != original.example.source
                {
                    self.environment_run = None;
                    self.notice = None;
                }
                self.progress.attempts.get_mut(&source.id).unwrap().state =
                    engine::State::Environment {
                        simulation: Box::new(next.clone()),
                    };
                self.completion_index.forget(&source.id);
                self.dirty = true;
            }
            if stop {
                self.environment_suppressed
                    .insert((source.id.clone(), practice));
                self.environment_run = None;
                self.notice = Some(
                    "Stopped. Your script stays saved; no incomplete run earns credit.".into(),
                );
            }
            if run {
                self.start_environment(i, practice, false);
            }
        }
        if next != original && view == teaching::View::Introduction {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::Environment {
                    simulation: Box::new(next),
                };
            self.dirty = true;
        }
        if let Some(notice) = &self.notice {
            lesson_presentation::prose(ui, notice);
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            egui::Frame::new().fill(palette(ui).input).inner_margin(12.).show(ui,|ui|{
                ui.strong("Assignment complete");lesson_presentation::prose(ui,"All sample inputs match the required parent state and actual child behavior.");lesson_presentation::prose(ui,&source.explanation);
                ui.horizontal_wrapped(|ui|{
                    if ui.add(button("Back to library")).clicked(){self.environment_run=None;self.selected=None;}
                    self.next_unit_lab(ui,i);
                });
            });
        }
    }
    fn environment_editor(
        &mut self,
        ui: &mut egui::Ui,
        s: &Scenario,
        practice: bool,
        draft: &mut env::Draft,
        active: bool,
    ) {
        ui.strong(if practice {
            "Example setup editor"
        } else {
            "Setup script"
        });
        let mut text = draft.source.clone();
        let height = (ui.ctx().content_rect().height() * 0.32).clamp(180., 320.);
        let area = controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("env-editor-scroll", &s.id, practice))
                .max_height(height)
                .min_scrolled_height(height),
            |ui| {
                ui.add_enabled_ui(active, |ui| {
                    ui.add_sized(
                        [ui.available_width(), height],
                        egui::TextEdit::multiline(&mut text)
                            .id_salt(("env-editor", &s.id, practice))
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .char_limit(env::MAX_SOURCE),
                    )
                })
                .inner
            },
        );
        let r = area.inner;
        name_control(&r, "Setup script — simulated shell");
        self.metrics.editor_viewport = Some(area.inner_rect);
        self.metrics.inputs.push(r.rect);
        self.metrics.focus = Some(r.id);
        if self.focus_input && active {
            r.request_focus();
            self.focus_input = false;
        }
        if r.changed() {
            workspace::bound(&mut text, env::MAX_SOURCE);
            if let Err(e) = draft.edit(text) {
                self.notice = Some(e);
            }
            self.environment_run = None;
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
    fn environment_actions(
        &mut self,
        ui: &mut egui::Ui,
        practice: bool,
        lookup: bool,
        running: bool,
        active: bool,
    ) -> (bool, bool) {
        let mut run = false;
        let mut stop = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(
                active && !running,
                primary(ui, environment_lookup::run_label(practice, lookup)),
            );
            self.metrics.check = Some(r.rect);
            run = r.clicked()
                || (active
                    && !running
                    && self
                        .metrics
                        .focus
                        .is_some_and(|id| ui.memory(|m| m.has_focus(id)))
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter)));
            let r = ui.add_enabled(active && running, button("Stop"));
            self.metrics.stop = Some(r.rect);
            stop = r.clicked();
        });
        (run, stop)
    }
}
fn inputs(ui: &mut egui::Ui, c: &env::Case, lookup: bool) {
    if !lookup || !c.input.args.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.strong("Arguments supplied by the lab:");
            for (n, value) in c.input.args.iter().enumerate() {
                ui.monospace(format!("${} = {value:?}", n + 1));
            }
        });
    }
    ui.horizontal_wrapped(|ui| {
        ui.strong("Already inherited by the setup:");
        for (n, v) in &c.input.env {
            ui.monospace(format!("{n}={v:?}"));
        }
    });
}
fn expected_child(ui: &mut egui::Ui, t: &env::Task) {
    environment_lookup::expected(ui, t, 0);
    ui.strong("Expected child report · original example");
    lesson_presentation::code_value(
        ui,
        ("env-expected", &t.id),
        &t.cases[0].child.stdout,
        ui.available_width(),
    );
    lesson_presentation::prose(
        ui,
        "The supplied child reads a copy of exported settings; the setup script does not print this report.",
    );
}
fn expected_sample(ui: &mut egui::Ui, task: &env::Task, case: usize) {
    environment_lookup::expected(ui, task, case);
    ui.strong("Expected child report · original example");
    lesson_presentation::code_value(
        ui,
        ("env-practice-expected", &task.id),
        &task.cases[case].child.stdout,
        ui.available_width(),
    );
    ui.separator();
}
fn environment_result(
    ui: &mut egui::Ui,
    t: &env::Task,
    d: &env::Draft,
    case: usize,
    running: bool,
    automatic: bool,
    m: &mut super::Metrics,
) {
    let result = ui.scope(|ui| {
        ui.strong(if t.path_lookup.is_some() {
            "Actual setup, lookup and report"
        } else {
            "Actual parent and child"
        });
        if running {
            lesson_presentation::prose(
                ui,
                if automatic {
                    "Verifying the saved run · Stop remains available."
                } else {
                    "Running setup and supplied child…"
                },
            );
        }
        let Some(record) = &d.result else {
            lesson_presentation::prose(ui, "Not run. Choose the Run action beside the editor.");
            return;
        };
        let trusted = record.verified();
        if !trusted {
            lesson_presentation::prose(
                ui,
                if d.needs_reverification(t) {
                    "Saved observations · awaiting verification. These are not a current pass."
                } else {
                    "Earlier saved observations · run this script again to check the current task."
                },
            );
        }
        let Some(row) = record.results.get(case) else {
            return;
        };
        if trusted {
            let rows = env::checks(t, d);
            let pass = rows.iter().filter(|(ok, _)| *ok).count();
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "Case {}/{} · {} · Run: {pass}/{} pass",
                    case + 1,
                    t.cases.len(),
                    if rows.get(case).is_some_and(|r| r.0) {
                        "Pass"
                    } else {
                        "Needs an edit"
                    },
                    t.cases.len()
                ))
                .strong(),
            );
            if let Some((ok, message)) = rows.get(case)
                && !*ok
            {
                lesson_presentation::prose(ui, message);
            }
        }
        if let Some(snapshot) = &row.bindings {
            // Permit wrapped values without collapsing to the column's remaining
            // height. Shrink to actual content when fewer lines are needed.
            let height = (snapshot.parent.len() + 1).min(6) as f32
                * (controls::table_height(ui)
                    + ui.text_style_height(&egui::TextStyle::Body)
                    + ui.spacing().item_spacing.y);
            controls::nested_scroll(
                ui,
                egui::ScrollArea::vertical()
                    .id_salt(("env-bindings", &t.id))
                    .min_scrolled_height(height)
                    .max_height(height)
                    .auto_shrink([false, true]),
                |ui| {
                    let width = (ui.available_width() - 2. * ui.spacing().item_spacing.x).max(180.);
                    egui::Grid::new(("env-state", &t.id))
                        .striped(true)
                        .show(ui, |ui| {
                            for (label, fraction) in [
                                ("Name", 0.22),
                                ("Parent value", 0.48),
                                ("Child receives", 0.30),
                            ] {
                                controls::table_text(
                                    ui,
                                    RichText::new(label).strong(),
                                    width * fraction,
                                );
                            }
                            ui.end_row();
                            for (n, v) in &snapshot.parent {
                                controls::table_text(
                                    ui,
                                    RichText::new(n).monospace(),
                                    width * 0.22,
                                );
                                let value = v
                                    .value
                                    .as_ref()
                                    .map(|s| {
                                        if s.is_empty() {
                                            "Empty string".into()
                                        } else {
                                            format!("{s:?}")
                                        }
                                    })
                                    .unwrap_or_else(|| "Not set".into());
                                controls::table_text(
                                    ui,
                                    RichText::new(value).monospace(),
                                    width * 0.48,
                                );
                                controls::table_text(
                                    ui,
                                    if !v.exported {
                                        "Not passed"
                                    } else if v.value.is_none() {
                                        "Marked, no value"
                                    } else {
                                        "Value passed"
                                    },
                                    width * 0.30,
                                );
                                ui.end_row();
                            }
                        });
                },
            );
        }
        if row.parent.status != 0 {
            lesson_presentation::prose(
                ui,
                &format!("Setup exit status: {} (expected 0)", row.parent.status),
            );
        }
        if let Some(error) = &row.parent.error {
            ui.strong(match teaching::diagnostic(error) {
                teaching::Diagnostic::Unsupported => "Unsupported syntax or feature",
                teaching::Diagnostic::Syntax => "Syntax error",
                teaching::Diagnostic::Limit => "Execution limit",
                _ => "Setup execution error",
            });
            lesson_presentation::prose(ui, error);
        }
        if !row.parent.stdout.is_empty() || !row.parent.stderr.is_empty() {
            ui.strong("Setup output · actual");
            lesson_presentation::prose(
                ui,
                "The setup should produce no output; the supplied child prints the report.",
            );
            controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(("env-parent-output", &t.id))
                    .max_height(100.),
                |ui| {
                    if !row.parent.stdout.is_empty() {
                        ui.monospace(&row.parent.stdout);
                    }
                    if !row.parent.stderr.is_empty() {
                        ui.label("Standard error");
                        ui.monospace(&row.parent.stderr);
                    }
                },
            );
        }
        environment_lookup::actual(ui, t, row, case);
        if let Some(child) = &row.child {
            ui.strong("Child report · actual");
            let rows = (child.stdout.lines().count() + child.stderr.lines().count()).clamp(1, 8);
            let scroll = ui.spacing().scroll;
            let height = rows as f32 * ui.text_style_height(&egui::TextStyle::Monospace)
                + scroll.bar_width
                + scroll.bar_inner_margin
                + scroll.bar_outer_margin
                + 2. * ui.spacing().item_spacing.y;
            controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(("env-output", &t.id))
                    .min_scrolled_height(height)
                    .max_height(height)
                    .auto_shrink([false, false]),
                |ui| {
                    ui.monospace(if child.stdout.is_empty() {
                        "(No standard output)"
                    } else {
                        &child.stdout
                    });
                    if !child.stderr.is_empty() {
                        ui.label("Standard error");
                        ui.monospace(&child.stderr);
                    }
                },
            );
            ui.label(format!("Child exit status: {}", child.status));
        } else {
            lesson_presentation::prose(ui, "No child report was produced.");
        }
    });
    m.output = Some(result.response.rect);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn environment_interrupted_automatic_verification_is_not_restarted_or_credited() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        let i = a
            .scenarios
            .iter()
            .position(|s| s.id == "lpic1-env-child-settings")
            .unwrap();
        a.launch(i);
        let s = a.scenarios[i].clone();
        assert!(env::context(&a.progress.attempts[&s.id], &s).is_ok());
        let engine::State::Environment { simulation } =
            &mut a.progress.attempts.get_mut(&s.id).unwrap().state
        else {
            panic!()
        };
        simulation.assignment.edit(s.solution[0].clone()).unwrap();
        let record = env::run(
            &simulation.definition.assignment,
            &simulation.assignment.source,
        )
        .unwrap();
        assert!(
            simulation
                .assignment
                .store(record, &simulation.definition.assignment)
        );
        **simulation = serde_json::from_str(&serde_json::to_string(simulation).unwrap()).unwrap();
        assert!(
            simulation
                .assignment
                .needs_reverification(&simulation.definition.assignment)
        );
        a.advance_environment(&ctx);
        let job = a
            .environment_run
            .as_mut()
            .expect("saved source must actually be verified");
        assert!(job.automatic);
        job.runner.stop();
        a.advance_environment(&ctx);
        assert!(a.environment_run.is_none());
        assert!(a.environment_suppressed.contains(&(s.id.clone(), false)));
        for _ in 0..4 {
            a.advance_environment(&ctx);
            assert!(a.environment_run.is_none());
        }
        assert!(!progress::current_solved(&a.progress.attempts[&s.id], &s));
        a.reset_attempt(i);
        assert!(env::context(&a.progress.attempts[&s.id], &s).is_ok());
        assert_eq!(a.progress.attempts[&s.id].state, s.initial);
    }
}
