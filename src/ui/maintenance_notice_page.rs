//! Focused native notices: editor/stdin, literal terminal and derived observations.
use super::*;
use dlms_learning_lab::{maintenance_notice as core, maintenance_notice_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn title(editor: work::Editor) -> &'static str {
    match editor {
        work::Editor::Issue => "/etc/issue · before login",
        work::Editor::Motd => "/etc/motd · after login",
        work::Editor::Stdin => "Live message · wall input",
    }
}
fn advice(ui: &mut egui::Ui, source: &pack::Scenario, step: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, source, step);
    }
}
#[derive(Clone, Copy)]
struct Lesson<'a> {
    source: &'a pack::Scenario,
    practice: bool,
    active: bool,
}
fn tools(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    lesson: Lesson<'_>,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Lesson {
        source,
        practice,
        active,
    } = lesson;
    advice(ui, source, 0, practice);
    let menu = controls::dropdown(ui, "notice-document", ui.available_width())
        .selected_text(title(d.selected))
        .show_ui(ui, |ui| {
            for (choice, key) in [
                (work::Editor::Motd, "document-motd"),
                (work::Editor::Issue, "document-issue"),
                (work::Editor::Stdin, "document-stdin"),
            ] {
                let r = ui.selectable_value(&mut d.selected, choice, title(choice));
                metrics.record(ui, key, &r);
            }
            let clip = controls::popup_content_clip(ui);
            for row in &mut metrics.controls {
                if row.0.starts_with("document-") {
                    row.2 = clip;
                }
            }
        });
    metrics.record(ui, "document", &menu.response);
    let label = ui.strong(if d.selected == work::Editor::Stdin {
        "Message to send with wall"
    } else {
        match d.selected {
            work::Editor::Issue => "/etc/issue · simulated file",
            work::Editor::Motd => "/etc/motd · simulated file",
            work::Editor::Stdin => unreachable!(),
        }
    });
    if practice {
        let r = ui.add_enabled(
            active && d.selected != work::Editor::Issue,
            button("Fill worked notice"),
        );
        metrics.record(ui, "fill-worked", &r);
        if r.clicked() {
            *notice = d.edit(t.required.render()).err();
        }
        ui.label("Fills this example editor only; Save or Run remains your action.");
    }
    let mut text = d.text().to_owned();
    let editor_height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 7.
        + 2. * f32::from(controls::PAD_Y);
    let editor_size = egui::vec2(ui.available_width(), editor_height);
    let (viewport, r) = controls::fixed_viewport(ui, editor_size, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("notice-editor-scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                        )
                        .desired_rows(6)
                        .char_limit(core::MAX_TEXT),
                )
                .labelled_by(label.id)
            })
            .inner
    });
    metrics
        .controls
        .push(("editor-viewport".into(), viewport, ui.clip_rect()));
    metrics.record(ui, "editor", &r);
    if r.changed() {
        *notice = d.edit(text).err();
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(
            active && d.selected != work::Editor::Stdin,
            button("Save file"),
        );
        metrics.record(ui, "save", &r);
        if r.clicked() {
            *notice = d.save(t).err();
        }
        if d.selected == work::Editor::Stdin {
            ui.label("wall reads this composer as its simulated standard input.");
        }
        if d.stdin_pending {
            let r = ui.add_enabled(active, button("Discard unsent message"));
            metrics.record(ui, "discard-stdin", &r);
            if r.clicked() {
                d.discard_stdin();
                *notice = None;
            }
        }
    });
    if let Ok(world) = d.world(t) {
        let pending = d.documents != world.files;
        ui.label(if pending {
            "Unsaved file edits. Save before checking."
        } else {
            "The file editors match the saved simulated files."
        });
    }
    ui.add_space(controls::GAP);
    advice(ui, source, 1, practice);
    let label = ui.strong("Simulated terminal");
    let available = ui.available_width();
    let mut command = d.command.clone();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut command, available)
                .hint_text("Command from Supported commands")
                .char_limit(1024),
        )
        .labelled_by(label.id);
    metrics.record(ui, "command", &r);
    if r.changed() {
        if command.len() <= 1024 {
            d.command = command;
        } else {
            *notice = Some("Keep one command draft within 1024 bytes.".into());
        }
    }
    let enter = r.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(active, button("Run command"));
        metrics.record(ui, "run", &r);
        if r.clicked() || active && enter {
            let result = d.run_and_record(t);
            *notice = if d.last_run.is_some() {
                None
            } else {
                result.err().map(|e| e.to_string())
            };
        }
    });
    let actual_run = ui.scope(|ui| {
        let label = ui.strong("Actual last run");
        metrics.record(ui, "last-result", &label);
        if let Some(record) = &d.last_run {
            ui.add(
                egui::Label::new(RichText::new(format!("$ {}", record.command)).monospace())
                    .wrap()
                    .selectable(true),
            );
            match d.last_output(t) {
                Some(Ok(output)) => {
                    ui.label("Command finished · status 0");
                    if output.is_empty() {
                        ui.label("No standard output. Inspect delivery/schedule observations.");
                    } else {
                        egui::ScrollArea::vertical()
                            .id_salt("notice-stdout")
                            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 8.)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(RichText::new(output).monospace())
                                        .wrap()
                                        .selectable(true),
                                );
                            });
                    }
                }
                Some(Err(error)) => {
                    ui.label(
                        RichText::new(format!(
                            "Command failed · {}: {}",
                            match error.kind {
                                core::command::Kind::Syntax => "Syntax error",
                                core::command::Kind::Unsupported => "Unsupported command or syntax",
                                core::command::Kind::Limit => "Simulation limit",
                                core::command::Kind::State => "Cannot apply command",
                            },
                            error.message
                        ))
                        .color(palette(ui).error),
                    );
                }
                None => {
                    ui.label("No compatible run observation is available.");
                }
            }
        } else {
            ui.label("No command has run here yet.");
        }
    });
    metrics.record(ui, "actual-run", &actual_run.response);
}

fn observations(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    lesson: Lesson<'_>,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Lesson {
        source,
        practice,
        active,
    } = lesson;
    let world = match d.world(t) {
        Ok(w) => w,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let stamp = t
        .facts
        .date
        .after(world.minute)
        .map(|m| m.text())
        .unwrap_or_else(|_| "Outside recording".into());
    let label = ui.strong("Current simulated state");
    metrics.record(ui, "observations", &label);
    lesson_presentation::prose(ui, &format!("{} · {} UTC", t.facts.host, stamp));
    lesson_presentation::prose(
        ui,
        &match &world.restart {
            None => "No restart pending.".into(),
            Some(r) => format!(
                "Pending restart: {} UTC.",
                t.facts
                    .date
                    .after(r.at)
                    .map(|m| m.text())
                    .unwrap_or_else(|_| "Outside recording".into())
            ),
        },
    );
    ui.label(if world.ordinary_logins_blocked() {
        "Ordinary logins: blocked by the simulated restart window."
    } else {
        "Ordinary logins: allowed."
    });
    ui.separator();
    ui.strong("Live delivery observations · simulator receipts");
    if let Some(last) = world.deliveries.last() {
        ui.label(format!(
            "Latest announcement reached {} terminals.",
            last.terminals.len()
        ));
        egui::Grid::new("notice-audience")
            .num_columns(2)
            .spacing([controls::GAP, controls::GAP])
            .show(ui, |ui| {
                ui.strong("User / terminal");
                ui.strong("Latest receipt");
                ui.end_row();
                for login in &t.facts.logins {
                    ui.label(format!("{} / {}", login.user, login.tty));
                    ui.label(if last.terminals.contains(&login.tty) {
                        "Received"
                    } else {
                        "Not received"
                    });
                    ui.end_row();
                }
            });
        controls::collapsing_header(ui, "Read latest delivered message").show(ui, |ui| {
            ui.add(
                egui::Label::new(RichText::new(&last.body).monospace())
                    .wrap()
                    .selectable(true),
            );
        });
    } else {
        ui.label("No live announcement has been sent.");
    }
    controls::collapsing_header(ui,"Preview future login messages").show(ui,|ui|{
        ui.strong("Before login · /etc/issue");ui.add(egui::Label::new(RichText::new(&world.files.issue).monospace()).wrap());
        if world.ordinary_logins_blocked(){ui.label("An ordinary account cannot complete login at this simulated time; its post-login message is not displayed.");}else{ui.strong("After successful login · /etc/motd");ui.add(egui::Label::new(RichText::new(&world.files.motd).monospace()).wrap());}
    });
    let clock_help=controls::collapsing_header(ui,"Explore the recording clock · optional").show(ui,|ui|{
        lesson_presentation::prose(ui,"Advance changes only this recorded host. Near a pending restart it can block ordinary logins or end the activity; it does not change your computer's clock.");
        for n in [1,5]{let r=ui.add_enabled(active,button(&format!("Advance {n} simulated minute{}",if n==1{""}else{"s"})));metrics.record(ui,&format!("advance-{n}"),&r);if r.clicked(){*notice=d.advance(t,n).err();}}
    });
    metrics.record(ui, "clock-help", &clock_help.header_response);
    ui.add_space(controls::GAP);
    advice(ui, source, 2, practice);
    let r = ui.add_enabled(
        active,
        primary(
            ui,
            if practice {
                "Check example notices"
            } else {
                "Check notices"
            },
        ),
    );
    metrics.record(ui, "check", &r);
    if r.clicked() {
        *notice = d.check(t).err();
    }
}
fn feedback(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, metrics: &mut Metrics) {
    let checks = match d.checks(t) {
        Ok(c) => c,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let failures = checks.iter().filter(|c| !c.correct).count();
    let label = ui.strong(if !d.checked {
        "Not checked".into()
    } else if failures == 0 {
        "Notices verified · every recipient and policy check passes".into()
    } else {
        format!("Needs attention · {failures} checks fail")
    });
    metrics.record(ui, "result", &label);
    if d.checked {
        for (number, check) in checks.into_iter().enumerate() {
            let palette = palette(ui);
            let response = lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}: {}",
                    if check.correct {
                        "Meets requirement"
                    } else {
                        "Needs attention"
                    },
                    check.label,
                    check.feedback
                ))
                .color(if check.correct {
                    palette.success
                } else {
                    palette.error
                }),
            );
            metrics.record(ui, &format!("feedback-{number}"), &response);
        }
    }
}
impl LabApp {
    pub(super) fn maintenance_notice_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        let def = match work::source(&source).and_then(|d| work::context(a, &source).map(|()| d)) {
            Ok(d) => d,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let engine::State::MaintenanceNotice { simulation } = &a.state else {
            return;
        };
        let before = (**simulation).clone();
        let mut next = before.clone();
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
        self.metrics.maintenance_notice = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.maintenance_notice.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i, view as u8), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Builds on virtual file editing and local user/session concepts.",
                    Vec::<(String, String)>::new(),
                );
                lesson_presentation::worked(
                    ui,
                    "A different recorded host demonstrates the notice and timing decisions.",
                );
                lesson_presentation::worked_rows(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (v, key, label) in [
                        (teaching::View::Example, "try", "Try this example"),
                        (teaching::View::Assignment, "start", "Open full assignment"),
                    ] {
                        let r = ui.add_enabled(active, button(label));
                        self.metrics.maintenance_notice.record(ui, key, &r);
                        if r.clicked() {
                            next.view = v;
                            self.lesson_scroll_reset = true;
                        }
                    }
                });
            } else {
                let practice = view == teaching::View::Example;
                if !practice {
                    lesson_presentation::assignment(ui, &source);
                }
                let (t, d) = if practice {
                    (&def.example, &mut next.practice)
                } else {
                    (&def.initial, &mut next.assignment)
                };
                let lesson = Lesson {
                    source: &source,
                    practice,
                    active,
                };
                ui.add_enabled_ui(active, |ui| {
                    if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 48. {
                        ui.columns(2, |columns| {
                            tools(
                                &mut columns[0],
                                t,
                                d,
                                lesson,
                                &mut self.metrics.maintenance_notice,
                                &mut self.notice,
                            );
                            observations(
                                &mut columns[1],
                                t,
                                d,
                                lesson,
                                &mut self.metrics.maintenance_notice,
                                &mut self.notice,
                            );
                        });
                    } else {
                        tools(
                            ui,
                            t,
                            d,
                            lesson,
                            &mut self.metrics.maintenance_notice,
                            &mut self.notice,
                        );
                        ui.separator();
                        observations(
                            ui,
                            t,
                            d,
                            lesson,
                            &mut self.metrics.maintenance_notice,
                            &mut self.notice,
                        );
                    }
                    ui.separator();
                    feedback(ui, t, d, &mut self.metrics.maintenance_notice);
                    if practice {
                        let r = ui.add_enabled(active, button("Reset example only"));
                        self.metrics
                            .maintenance_notice
                            .record(ui, "reset-example", &r);
                        if r.clicked() {
                            match work::Draft::new(&def.example) {
                                Ok(reset) => *d = reset,
                                Err(e) => self.notice = Some(e),
                            }
                        }
                    }
                    controls::collapsing_header(ui, "Supported commands and help · optional").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, work::HELP);
                            let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                            let hint = ui.add_enabled(
                                active && attempt.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            if hint.clicked() {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(attempt.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            lesson_presentation::prose(ui, &source.simplifications);
                            controls::collapsing_header(ui, "Worked approach · reveals one method")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &source.explanation);
                                });
                        },
                    );
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::MaintenanceNotice {
                    simulation: Box::new(next),
                };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, i);
        }
    }
}
