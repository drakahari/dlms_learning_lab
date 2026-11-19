//! Native virtual-volume investigation; no host command route.
use super::*;
use dlms_learning_lab::engine::State;
use dlms_learning_lab::{fs_integrity_terminal as terminal, fs_integrity_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub command: Option<(egui::Rect, egui::Id)>,
    pub output: Option<egui::Rect>,
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
    pub(super) fn start_fs_integrity(&mut self, i: usize, example: bool, automatic: bool) {
        let source = &self.scenarios[i];
        let id = source.id.clone();
        let a = self.progress.attempts.get_mut(&id).unwrap();
        if let Err(error) = work::context(a, source) {
            self.fs_integrity_suppressed.insert((id.clone(), example));
            self.notice = Some(error);
            return;
        }
        let State::FsIntegrity { simulation } = &mut a.state else {
            return;
        };
        let (task, scope) = if example {
            (&simulation.definition.example, &mut simulation.example)
        } else {
            (
                &simulation.definition.assignment,
                &mut simulation.assignment,
            )
        };
        match scope.begin(task) {
            Ok(runner) => {
                self.fs_integrity_suppressed.remove(&(id.clone(), example));
                self.fs_integrity_run = Some(ActiveRun {
                    scenario: id.clone(),
                    example,
                    visited: a.visited,
                    automatic,
                    runner,
                });
                self.completion_index.forget(&id);
                self.dirty = true;
            }
            Err(error) => {
                self.fs_integrity_suppressed.insert((id, example));
                self.notice = Some(error);
            }
        }
    }
    pub(super) fn advance_fs_integrity(&mut self, ctx: &egui::Context) {
        let frame = ctx.cumulative_frame_nr();
        let key = egui::Id::new("fs-integrity-root-tick");
        if ctx.data(|d| d.get_temp::<u64>(key)) == Some(frame) {
            return;
        }
        ctx.data_mut(|d| d.insert_temp(key, frame));
        let allowed = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if let Some(job) = &self.fs_integrity_run {
            let current = self.progress.attempts.get(&job.scenario);
            let valid = allowed
                && current.is_some_and(|a| a.visited == job.visited)
                && (job.automatic
                    || self.selected.is_some_and(|i| {
                        self.scenarios[i].id == job.scenario
                            && current.is_some_and(|a| {
                                let State::FsIntegrity { simulation } = &a.state else {
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
                self.fs_integrity_run = None;
            }
        }
        if self.fs_integrity_run.is_none() && allowed && self.script_job.is_none() {
            let pending = self.scenarios.iter().enumerate().find_map(|(i, s)| {
                let a = self.progress.attempts.get(&s.id)?;
                let State::FsIntegrity { simulation } = &a.state else {
                    return None;
                };
                [false, true]
                    .into_iter()
                    .find(|example| {
                        !self
                            .fs_integrity_suppressed
                            .contains(&(s.id.clone(), *example))
                            && if *example {
                                simulation.example.pending_recheck()
                            } else {
                                simulation.assignment.pending_recheck()
                            }
                    })
                    .map(|example| (i, example))
            });
            if let Some((i, example)) = pending {
                self.start_fs_integrity(i, example, true);
            }
        }
        let Some(job) = &mut self.fs_integrity_run else {
            return;
        };
        job.runner.tick();
        if !job.runner.done() {
            ctx.request_repaint();
            return;
        }
        let job = self.fs_integrity_run.take().unwrap();
        if let Some(a) = self.progress.attempts.get_mut(&job.scenario)
            && a.visited == job.visited
            && let State::FsIntegrity { simulation } = &mut a.state
        {
            let (task, scope) = if job.example {
                (&simulation.definition.example, &mut simulation.example)
            } else {
                (
                    &simulation.definition.assignment,
                    &mut simulation.assignment,
                )
            };
            if let Err(error) = scope.finish(task, &job.runner) {
                self.notice = Some(error);
                self.fs_integrity_suppressed
                    .insert((job.scenario.clone(), job.example));
            } else {
                self.completion_index.forget(&job.scenario);
                self.dirty = true;
                self.checked = !job.example;
            }
        }
        ctx.request_repaint();
    }
    pub(super) fn fs_integrity_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            self.fs_integrity_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(error) = work::context(a, &source) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::FsIntegrity { simulation } = &a.state else {
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
                    record(&mut self.metrics.fs_integrity, key, &r, ui);
                    if r.clicked() {
                        view = value;
                        self.lesson_scroll_reset = true;
                        self.focus_input = value != teaching::View::Introduction;
                        self.fs_integrity_run = None;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        let mut check = false;
        let mut stop = false;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on file inodes, hard links and mounting non-root volumes. Everything here is a supplied in-memory exercise.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &next.definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active, button("Try this example"));
                record(&mut self.metrics.fs_integrity, "try", &r, ui);
                if r.clicked() {
                    next.view = teaching::View::Example;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
                let r = ui.add_enabled(active, button("Open full assignment"));
                record(&mut self.metrics.fs_integrity, "start", &r, ui);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            });
        } else {
            let (task, scope) = if practice {
                (&next.definition.example, &mut next.example)
            } else {
                (&next.definition.assignment, &mut next.assignment)
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let running = self
                .fs_integrity_run
                .as_ref()
                .is_some_and(|job| job.scenario == source.id && job.example == practice);
            if practice {
                let picker =
                    ui.id()
                        .with(("fs-example-command", &source.id, self.fs_integrity_epoch));
                let mut selected = ui
                    .ctx()
                    .data(|d| d.get_temp::<usize>(picker))
                    .unwrap_or(0)
                    .min(next.definition.example_commands.len() - 1);
                ui.horizontal_wrapped(|ui| {
                    let r = controls::dropdown(ui, picker, ui.available_width())
                        .selected_text(format!(
                            "Worked command {} of {}",
                            selected + 1,
                            next.definition.example_commands.len()
                        ))
                        .show_ui(ui, |ui| {
                            for (n, command) in next.definition.example_commands.iter().enumerate()
                            {
                                let r = ui.selectable_value(
                                    &mut selected,
                                    n,
                                    RichText::new(command).monospace(),
                                );
                                record(
                                    &mut self.metrics.fs_integrity,
                                    &format!("command:{n}"),
                                    &r,
                                    ui,
                                );
                            }
                        });
                    record(&mut self.metrics.fs_integrity, "picker", &r.response, ui);
                    let r = ui.add_enabled(active && !running, button("Insert command"));
                    record(&mut self.metrics.fs_integrity, "insert", &r, ui);
                    if r.clicked() {
                        scope.command = next.definition.example_commands[selected].clone();
                        self.focus_input = true;
                    }
                });
                ui.ctx().data_mut(|d| d.insert_temp(picker, selected));
                lesson_presentation::practice_step(ui, &source, 1);
            }
            let id = ui
                .id()
                .with(("fs-console", &source.id, practice, self.fs_integrity_epoch));
            console(
                ui,
                scope,
                task,
                &mut self.metrics.fs_integrity,
                &mut self.focus_input,
                id,
                active && !running,
            );
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active && !running, primary(ui, "Check work"));
                record(&mut self.metrics.fs_integrity, "check", &r, ui);
                self.metrics.check = Some(r.rect);
                check = r.clicked();
                if running {
                    let r = ui.add_enabled(active, button("Stop"));
                    record(&mut self.metrics.fs_integrity, "stop", &r, ui);
                    stop = r.clicked();
                    ui.spinner();
                }
            });
            if running {
                lesson_presentation::prose(
                    ui,
                    "Checking the current actions against the supplied samples…",
                );
            } else if let Some(rows) = scope.results(task) {
                let passed = rows.iter().filter(|row| row.passed()).count();
                let all = passed == rows.len();
                ui.separator();
                let text = if all && !practice {
                    format!("Assignment complete · {passed}/{} samples pass", rows.len())
                } else if all {
                    format!(
                        "Example meets every requirement · {passed}/{} samples pass",
                        rows.len()
                    )
                } else {
                    format!("Needs work · {passed}/{} samples pass", rows.len())
                };
                let r = ui.colored_label(
                    if all {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    },
                    RichText::new(text).strong(),
                );
                self.metrics.fs_integrity.result = Some((passed, rows.len(), r.rect));
                for row in rows {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {}",
                            if row.passed() { "Passed" } else { "Needs work" },
                            row.label
                        ),
                    );
                    if let Some((_, reason)) = row.checks.iter().find(|r| !r.0) {
                        lesson_presentation::prose(ui, reason);
                    }
                    controls::collapsing_header(ui, "Objective results")
                        .id_salt((&source.id, practice, &row.id))
                        .show(ui, |ui| {
                            for (ok, reason) in &row.checks {
                                lesson_presentation::prose(
                                    ui,
                                    &format!(
                                        "{}: {reason}",
                                        if *ok { "Passed" } else { "Needs work" }
                                    ),
                                );
                            }
                        });
                }
                if all && !practice {
                    lesson_presentation::prose(ui, &source.explanation);
                    ui.horizontal_wrapped(|ui| {
                        if ui.add(button("Back to library")).clicked() {
                            self.selected = None;
                            self.fs_integrity_run = None;
                        }
                        self.next_unit_lab(ui, i);
                    });
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    "Not checked · Check work verifies the current simulated state across the supplied samples.",
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active && !running, button("Reset example only"));
                record(&mut self.metrics.fs_integrity, "reset-example", &r, ui);
                if r.clicked() {
                    match work::Scope::new(&next.definition.example) {
                        Ok(scope) => next.example = scope,
                        Err(error) => self.notice = Some(error),
                    }
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Help and simulation scope")
                .id_salt((&source.id, "fs-help", practice, challenge))
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
                    controls::collapsing_header(ui, "Supported commands and limits").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, terminal::HELP);
                            lesson_presentation::prose(ui, &source.simplifications);
                        },
                    );
                    controls::collapsing_header(ui, "References").show(ui, |ui| {
                        for url in &source.sources {
                            ui.hyperlink(url);
                        }
                    });
                });
        }
        if next != original {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::FsIntegrity {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        if stop {
            if let Some(mut job) = self.fs_integrity_run.take() {
                job.runner.cancel();
            }
            self.fs_integrity_suppressed
                .insert((source.id.clone(), practice));
        }
        if check {
            self.start_fs_integrity(i, practice, false);
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
    }
}
fn console(
    ui: &mut egui::Ui,
    scope: &mut work::Scope,
    task: &work::Task,
    metrics: &mut Metrics,
    focus: &mut bool,
    id: egui::Id,
    active: bool,
) {
    ui.strong("Virtual filesystem terminal");
    let h = ui.text_style_height(&egui::TextStyle::Monospace);
    let height = (ui.ctx().content_rect().height() * 0.3).clamp(h * 6., h * 14.);
    let r=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
        egui::ScrollArea::vertical().id_salt((id,"output")).max_height(height).min_scrolled_height(height).auto_shrink([false,false]).stick_to_bottom(true).show(ui,|ui|{
            if scope.transcript.is_empty(){lesson_presentation::prose(ui,"Type a command below. Start with findmnt to inspect the supplied mounts; help lists the supported subset.");}
            for message in &scope.transcript {
                lesson_presentation::monospace(ui,&format!("$ {}",message.input),true);
                if let Some(kind)=message.diagnostic{
                    let label=match kind{terminal::DiagnosticKind::Syntax=>"Command syntax",terminal::DiagnosticKind::Unsupported=>"Simulator limitation",terminal::DiagnosticKind::Unknown=>"Unknown virtual path or device",terminal::DiagnosticKind::Policy=>"Simulator safety policy",terminal::DiagnosticKind::Invalid=>"Invalid simulated input",terminal::DiagnosticKind::Limit=>"Simulator limit"};
                    ui.colored_label(palette(ui).error,format!("{label}: {}",message.text));
                }else{
                    lesson_presentation::monospace(ui,&message.text,true);
                    if !matches!(terminal::parse(&message.input),Ok(terminal::Command::Check{..})){ui.label(RichText::new(format!("Exit status {}",message.status)).small().color(palette(ui).muted));}
                }
                ui.add_space(4.);
            }
        });
    });
    metrics.output = Some(r.response.rect);
    let previous = scope.command.clone();
    let available = ui.available_width();
    let button_width = controls::button_width(ui, "Run command");
    let width = if available > h * 8. + button_width + controls::GAP {
        available - button_width - controls::GAP
    } else {
        available
    };
    let enter = active
        && ui.memory(|m| m.focused() == Some(id))
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
    ui.horizontal_wrapped(|ui| {
        let field = ui.add_enabled(
            active,
            controls::singleline(ui, &mut scope.command)
                .id(id)
                .desired_width((width - 2. * f32::from(controls::PAD_X)).max(0.))
                .hint_text("Enter a supported command · Enter to run")
                .char_limit(terminal::MAX_COMMAND),
        );
        name_control(&field, "Simulated filesystem command");
        metrics.command = Some((field.rect, id));
        record(metrics, "command", &field, ui);
        if field.changed() && scope.command.len() > terminal::MAX_COMMAND {
            scope.command = previous;
        }
        if *focus && active && !ui.input(|i| i.pointer.any_down()) {
            field.request_focus();
            *focus = false;
        }

        let r = ui.add_enabled(
            active && !scope.command.trim().is_empty(),
            button("Run command"),
        );
        record(metrics, "run", &r, ui);
        if enter || r.clicked() {
            let _ = scope.submit(task);
            field.request_focus();
        }
    });
}
