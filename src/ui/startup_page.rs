//! Native startup-file editor and actual session/new-program feedback.
use super::*;
use dlms_learning_lab::{
    engine::{Goal, State},
    startup_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
pub(super) struct ActiveRun {
    pub scenario: String,
    pub example: bool,
    pub visited: u64,
    pub automatic: bool,
    runner: work::Run,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
impl LabApp {
    pub(super) fn start_startup_files(&mut self, i: usize, example: bool, automatic: bool) {
        let source = self.scenarios[i].clone();
        let id = source.id.clone();
        let a = self.progress.attempts.get_mut(&id).unwrap();
        if let Err(error) = work::context(a, &source) {
            self.startup_files_suppressed.insert((id, example));
            self.notice = Some(error);
            return;
        }
        let (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) =
            (&mut a.state, &source.goal)
        else {
            return;
        };
        let (task, scope) = if example {
            (&definition.example, &mut simulation.practice)
        } else {
            (&definition.initial, &mut simulation.assignment)
        };
        match scope.begin(task) {
            Ok(runner) => {
                self.startup_files_suppressed.remove(&(id.clone(), example));
                self.startup_files_run = Some(ActiveRun {
                    scenario: id.clone(),
                    example,
                    visited: a.visited,
                    automatic,
                    runner,
                });
                self.completion_index.forget(&id);
                self.dirty = true;
                self.notice = None;
            }
            Err(error) => {
                self.startup_files_suppressed.insert((id, example));
                self.notice = Some(error);
            }
        }
    }
    pub(super) fn advance_startup_files(&mut self, ctx: &egui::Context) {
        let frame = ctx.cumulative_frame_nr();
        let key = egui::Id::new("startup-files-root-tick");
        if ctx.data(|d| d.get_temp::<u64>(key)) == Some(frame) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(key, frame));
        let allowed = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if let Some(job) = &self.startup_files_run {
            let current = self.progress.attempts.get(&job.scenario);
            let valid = allowed
                && current.is_some_and(|a| a.visited == job.visited)
                && (job.automatic
                    || self.selected.is_some_and(|i| {
                        self.scenarios[i].id == job.scenario
                            && current.is_some_and(|a| {
                                let State::StartupFiles { simulation } = &a.state else {
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
                self.startup_files_run = None;
            }
        }
        if self.startup_files_run.is_none() && allowed && self.script_job.is_none() {
            let pending = self.scenarios.iter().enumerate().find_map(|(i, s)| {
                let a = self.progress.attempts.get(&s.id)?;
                let State::StartupFiles { simulation } = &a.state else {
                    return None;
                };
                [false, true]
                    .into_iter()
                    .find(|example| {
                        !self
                            .startup_files_suppressed
                            .contains(&(s.id.clone(), *example))
                            && if *example {
                                simulation.practice.pending()
                            } else {
                                simulation.assignment.pending()
                            }
                    })
                    .map(|example| (i, example))
            });
            if let Some((i, example)) = pending {
                self.start_startup_files(i, example, true);
            }
        }
        let Some(job) = &mut self.startup_files_run else {
            return;
        };
        job.runner.tick();
        if !job.runner.done() {
            ctx.request_repaint();
            return;
        }
        let job = self.startup_files_run.take().unwrap();
        let source = self
            .scenarios
            .iter()
            .find(|s| s.id == job.scenario)
            .cloned();
        if let Some(source) = source
            && let Some(a) = self.progress.attempts.get_mut(&job.scenario)
            && a.visited == job.visited
            && let (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) =
                (&mut a.state, &source.goal)
        {
            let (task, scope) = if job.example {
                (&definition.example, &mut simulation.practice)
            } else {
                (&definition.initial, &mut simulation.assignment)
            };
            if let Err(e) = scope.finish(task, &job.runner) {
                self.notice = Some(e);
                self.startup_files_suppressed
                    .insert((job.scenario.clone(), job.example));
            } else {
                self.completion_index.forget(&job.scenario);
                self.dirty = true;
                self.checked = !job.example;
            }
        }
        ctx.request_repaint();
    }
    pub(super) fn startup_files_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            self.startup_files_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (State::StartupFiles { simulation }, Goal::StartupFiles { definition }) =
            (&a.state, &source.goal)
        else {
            return;
        };
        let mut next = (**simulation).clone();
        let original = next.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (value, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == value));
                    record(&mut self.metrics.startup_files, key, &r, ui);
                    if r.clicked() {
                        view = value;
                        self.lesson_scroll_reset = true;
                        self.focus_input = value != teaching::View::Introduction;
                        self.startup_files_run = None;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        let mut run = false;
        let mut stop = false;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on ordinary variables, quoting and exporting settings to a new program.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (key, label, value) in [
                    ("try", "Try this example", teaching::View::Example),
                    ("start", "Open full assignment", teaching::View::Assignment),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.startup_files, key, &r, ui);
                    if r.clicked() {
                        next.view = value;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
        } else {
            let (task, scope) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let running = self
                .startup_files_run
                .as_ref()
                .is_some_and(|r| r.scenario == source.id && r.example == practice);
            ui.horizontal_wrapped(|ui| {
                ui.label("Sample session");
                let response = controls::dropdown(
                    ui,
                    ("startup-case", &source.id, practice),
                    ui.available_width(),
                )
                .selected_text(&task.cases[scope.case].label)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let start = self.metrics.startup_files.controls.len();
                    for (n, c) in task.cases.iter().enumerate() {
                        let r = ui.selectable_value(&mut scope.case, n, &c.label);
                        record(
                            &mut self.metrics.startup_files,
                            &format!("case:{n}"),
                            &r,
                            ui,
                        );
                    }
                    let clip = controls::popup_content_clip(ui);
                    for c in &mut self.metrics.startup_files.controls[start..] {
                        c.2 = clip;
                    }
                });
                record(
                    &mut self.metrics.startup_files,
                    "case",
                    &response.response,
                    ui,
                );
            });
            let sample = &task.cases[scope.case];
            lesson_presentation::prose(ui, invocation(&sample.invocation));
            ui.horizontal_wrapped(|ui| {
                for (name, value) in &sample.input.env {
                    lesson_presentation::monospace(ui, &format!("{name}={value}"), true);
                }
            });
            lesson_presentation::prose(
                ui,
                "These values are supplied by the lab. The same file edits are tested in every sample session.",
            );
            ui.separator();
            lesson_presentation::rich(ui, RichText::new("Startup file to edit").strong());
            let response = controls::dropdown(
                ui,
                ("startup-file", &source.id, practice),
                ui.available_width(),
            )
            .selected_text(&scope.selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = self.metrics.startup_files.controls.len();
                for name in scope.files.0.keys() {
                    let r = ui.selectable_value(
                        &mut scope.selected,
                        name.clone(),
                        RichText::new(name).monospace(),
                    );
                    record(
                        &mut self.metrics.startup_files,
                        &format!("file:{name}"),
                        &r,
                        ui,
                    );
                }
                let clip = controls::popup_content_clip(ui);
                for c in &mut self.metrics.startup_files.controls[start..] {
                    c.2 = clip;
                }
            });
            record(
                &mut self.metrics.startup_files,
                "file",
                &response.response,
                ui,
            );
            if practice {
                let r = ui.add_enabled(active && !running, button("Insert worked files"));
                record(&mut self.metrics.startup_files, "insert", &r, ui);
                if r.clicked() {
                    if let Err(e) = scope.replace(task, &definition.example_worked) {
                        self.notice = Some(e);
                    }
                    self.focus_input = true;
                }
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let file = scope.selected.clone();
            let mut text = scope.files.0[&file].text.clone();
            let height = ui.text_style_height(&egui::TextStyle::Monospace) * 9.
                + 2. * f32::from(controls::PAD_Y);
            let id = ui.id().with((
                "startup-editor",
                &source.id,
                practice,
                &file,
                self.startup_files_epoch,
            ));
            let mut clip = egui::Rect::NOTHING;
            let response = controls::nested_scroll(
                ui,
                egui::ScrollArea::vertical()
                    .id_salt((id, "scroll"))
                    .max_height(height)
                    .min_scrolled_height(height)
                    .auto_shrink([false, false]),
                |ui| {
                    clip = ui.clip_rect();
                    ui.add_enabled(
                        active && !running,
                        egui::TextEdit::multiline(&mut text)
                            .code_editor()
                            .id(id)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .char_limit(8192),
                    )
                },
            );
            let editor = response.inner;
            name_control(&editor, "Simulated Bash startup file contents");
            self.metrics.startup_files.editor = Some((editor.rect.intersect(clip), editor.id));
            record(&mut self.metrics.startup_files, "editor", &editor, ui);
            if self.focus_input && active && !running && !ui.input(|i| i.pointer.any_down()) {
                editor.request_focus();
                self.focus_input = false;
            }
            if editor.changed()
                && let Err(e) = scope.edit(task, &file, &text)
            {
                self.notice = Some(e);
            }
            let shortcut = active
                && !running
                && editor.has_focus()
                && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
            if editor.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                editor.surrender_focus();
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active && !running, primary(ui, "Run startup checks"));
                record(&mut self.metrics.startup_files, "run", &r, ui);
                self.metrics.check = Some(r.rect);
                run = r.clicked() || shortcut;
                if running {
                    let r = ui.add_enabled(active, button("Stop"));
                    record(&mut self.metrics.startup_files, "stop", &r, ui);
                    stop = r.clicked();
                    ui.spinner();
                }
            });
            if running {
                lesson_presentation::prose(
                    ui,
                    "Checking the edited files and the settings received by a new simulated program…",
                );
            } else if let Some(rows) = scope.results(task) {
                let count = rows.iter().filter(|r| r.passed()).count();
                let all = count == rows.len();
                ui.separator();
                let title = if all && !practice {
                    "Assignment complete"
                } else if all {
                    "Example meets every requirement"
                } else {
                    "Needs work"
                };
                let r = ui.colored_label(
                    if all {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    RichText::new(format!("{title} · {count}/{} sessions pass", rows.len()))
                        .strong(),
                );
                self.metrics.startup_files.result = Some((count, rows.len(), r.rect));
                for row in rows {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {}",
                            if row.passed() { "Passed" } else { "Needs work" },
                            row.label
                        ),
                    );
                    if let Some(why) = row
                        .error
                        .as_deref()
                        .or_else(|| row.checks.iter().find(|r| !r.0).map(|r| r.1.as_str()))
                    {
                        lesson_presentation::prose(ui, why);
                    }
                }
                let row = &rows[scope.case];
                controls::collapsing_header(ui, "Actual output and settings")
                    .id_salt((&source.id, practice, "startup-actual"))
                    .default_open(true)
                    .show(ui, |ui| {
                        controls::nested_scroll(
                            ui,
                            egui::ScrollArea::vertical()
                                .id_salt((&source.id, practice, "startup-output"))
                                .max_height(ui.text_style_height(&egui::TextStyle::Body) * 18.)
                                .auto_shrink([false, true]),
                            |ui| observed(ui, row, sample),
                        );
                    });
                controls::collapsing_header(ui, "Objective results for this session")
                    .id_salt((&source.id, practice, scope.case, "startup-objectives"))
                    .show(ui, |ui| {
                        for (ok, why) in &row.checks {
                            lesson_presentation::prose(
                                ui,
                                &format!("{}: {why}", if *ok { "Passed" } else { "Needs work" }),
                            );
                        }
                    });
                if all && !practice {
                    lesson_presentation::prose(ui, &source.explanation);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(button("Back to library")).clicked() {
                            self.selected = None;
                            self.startup_files_run = None;
                        }
                        self.next_unit_lab(ui, i);
                    });
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    "Not yet verified · Run startup checks after editing. Ctrl+Enter runs checks; Escape leaves the editor.",
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active && !running, button("Reset example only"));
                record(&mut self.metrics.startup_files, "reset-example", &r, ui);
                if r.clicked() {
                    next.practice = work::Scope::new(&definition.example);
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Help and simulation scope")
                .id_salt((&source.id, practice, challenge, "startup-help"))
                .show(ui, |ui| {
                    let mut shown = self.progress.attempts[&source.id]
                        .hints
                        .min(source.hints.len());
                    if shown < source.hints.len() && ui.add(button("Show a hint")).clicked() {
                        shown += 1;
                        self.progress.attempts.get_mut(&source.id).unwrap().hints = shown;
                        self.dirty = true;
                    }
                    for hint in source.hints.iter().take(shown) {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(ui, "Supported settings and boundaries").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, work::HELP);
                            lesson_presentation::prose(ui, &source.simplifications);
                        },
                    );
                    if let Some(system) = &sample.system_profile {
                        controls::collapsing_header(ui, "Supplied system profile · read only")
                            .show(ui, |ui| {
                                lesson_presentation::monospace(ui, system, true);
                            });
                    }
                    controls::collapsing_header(ui, "References").show(ui, |ui| {
                        for url in &source.sources {
                            ui.hyperlink(url);
                        }
                    });
                });
        }
        if next != original {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::StartupFiles {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        if stop {
            if let Some(mut job) = self.startup_files_run.take() {
                job.runner.stop();
            }
            self.startup_files_suppressed
                .insert((source.id.clone(), practice));
        }
        if run {
            self.start_startup_files(i, practice, false);
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
    }
}
fn invocation(mode: &dlms_learning_lab::shell::startup::Invocation) -> &'static str {
    use dlms_learning_lab::shell::startup::Invocation::*;
    match mode {
        InteractiveLogin { noprofile: false } => {
            "Interactive login · supplied system profile, then the first available personal profile."
        }
        InteractiveLogin { noprofile: true } => {
            "Interactive login · startup profiles suppressed by the supplied --noprofile setting."
        }
        InteractiveNonLogin { norc: false } => {
            "Interactive non-login · reads the supplied .bashrc."
        }
        InteractiveNonLogin { norc: true } => {
            "Interactive non-login · .bashrc suppressed by the supplied --norc setting."
        }
        NonInteractive {
            bash_env: Some(path),
        } if !path.is_empty() => {
            "Noninteractive script · supplied BASH_ENV selects /work/startup.env."
        }
        NonInteractive { .. } => {
            "Noninteractive script · no startup environment file was supplied."
        }
    }
}
fn output(ui: &mut egui::Ui, label: &str, actual: &str, expected: &str) {
    lesson_presentation::rich(ui, RichText::new(label).strong());
    lesson_presentation::prose(ui, "Actual");
    lesson_presentation::monospace(
        ui,
        if actual.is_empty() {
            "(no output)"
        } else {
            actual
        },
        true,
    );
    if actual != expected {
        lesson_presentation::prose(ui, "Expected");
        lesson_presentation::monospace(
            ui,
            if expected.is_empty() {
                "(no output)"
            } else {
                expected
            },
            true,
        );
    }
}
fn observed(ui: &mut egui::Ui, row: &work::CaseResult, sample: &work::Case) {
    if let Some(error) = &row.error {
        lesson_presentation::prose(ui, error);
    }
    if let Some(parent) = &row.parent {
        lesson_presentation::rich(ui, RichText::new("Selected startup files").strong());
        for path in &parent.selection {
            lesson_presentation::monospace(ui, path, true);
        }
        output(
            ui,
            "Startup output",
            &parent.output.stdout,
            &sample.input.stdout,
        );
        for name in sample.parent.keys() {
            let actual = parent.variables.parent.get(name);
            let value = actual
                .and_then(|v| v.value.as_deref())
                .unwrap_or("(not set)");
            let sharing = if actual.is_some_and(|v| v.exported) {
                "shared with new programs"
            } else {
                "only this shell"
            };
            lesson_presentation::prose(ui, &format!("{name} = {value} · {sharing}"));
        }
    }
    if let Some(child) = &row.child {
        output(
            ui,
            "New-program output",
            &child.stdout,
            &sample.child.stdout,
        );
        if !child.stderr.is_empty() {
            lesson_presentation::rich(ui, RichText::new("New-program error output").strong());
            lesson_presentation::monospace(ui, &child.stderr, true);
        }
    }
}
