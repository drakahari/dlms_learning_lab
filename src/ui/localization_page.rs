//! Native locale editor, sample application and byte-conversion terminal.
use super::*;
use dlms_learning_lab::localization_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub summary: Option<String>,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn code(ui: &mut egui::Ui, text: &str) {
    lesson_presentation::monospace(ui, text, true);
}
#[derive(Clone, Copy)]
struct Lesson {
    practice: bool,
    active: bool,
}
fn guidance(ui: &mut egui::Ui, s: &pack::Scenario, step: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, s, step);
    }
}
fn handle(
    result: Result<
        Result<
            dlms_learning_lab::localization::command::Output,
            dlms_learning_lab::boot_entry::Error,
        >,
        String,
    >,
    notice: &mut Option<String>,
) {
    *notice = result.err();
}
fn terminal(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::Draft,
    lesson: Lesson,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Lesson { practice, active } = lesson;
    ui.add_space(ui.spacing().item_spacing.y);
    let label = ui.strong("Simulated terminal");
    if practice {
        let commands = task
            .reference
            .iter()
            .filter_map(|a| {
                if let work::Action::Command { text } = a {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let id = ui.make_persistent_id("locale-worked-command");
        let mut selected = ui
            .ctx()
            .data(|d| d.get_temp::<usize>(id))
            .unwrap_or(0)
            .min(commands.len().saturating_sub(1));
        let menu = controls::dropdown(ui, "locale-command-menu", ui.available_width())
            .selected_text(
                commands
                    .get(selected)
                    .copied()
                    .unwrap_or("No worked command"),
            )
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (n, command) in commands.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, n, *command);
                    m.record(ui, &format!("worked-{n}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for row in &mut m.controls {
                    if row.0.starts_with("worked-") {
                        row.2 = clip;
                    }
                }
            });
        m.record(ui, "worked-menu", &menu.response);
        ui.ctx().data_mut(|d| d.insert_temp(id, selected));
        let r = ui.add_enabled(active && !commands.is_empty(), button("Insert command"));
        m.record(ui, "insert", &r);
        if r.clicked() {
            draft.input = commands[selected].into();
        }
        ui.label("Fills the input only. Choose Run command to execute it in this simulator.");
    }
    let mut input = draft.input.clone();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut input, ui.available_width())
                .hint_text("Command from Supported commands")
                .char_limit(1024),
        )
        .labelled_by(label.id);
    m.record(ui, "command", &r);
    if r.changed() {
        if input.len() <= 1024 {
            draft.input = input;
        } else {
            *notice = Some("Keep one command within 1,024 bytes.".into());
        }
    }
    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let r = ui.add_enabled(active, button("Run command"));
    m.record(ui, "run", &r);
    if r.clicked() || active && enter {
        handle(draft.run(task), notice);
    }
    let actual = ui.scope(|ui| {
        ui.strong("Actual last run");
        if let Some(run) = &draft.last_run {
            match &run.action {
                work::Action::Command { text } => code(ui, &format!("$ {text}")),
                work::Action::SaveLocale { .. } => {
                    ui.label("Native editor · Save locale settings");
                }
                work::Action::NewSession {} => {
                    ui.label("Native activity · Open fresh login");
                }
                work::Action::Report { input } => {
                    ui.label(format!("Sample application · {input}"));
                }
            }
        }
        match draft.last_output(task) {
            Ok(None) => {
                ui.label("Nothing has run in this draft yet.");
            }
            Ok(Some(Ok(output))) => {
                ui.label(format!("Finished · exit status {}", output.status));
                if !output.stderr.is_empty() {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(&output.stderr).color(palette(ui).error),
                    );
                }
                let height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.;
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt("locale-actual-output")
                        .max_height(height)
                        .auto_shrink([false, true]),
                    |ui| {
                        if output.stdout.is_empty() {
                            ui.label("No standard output.");
                        } else {
                            lesson_presentation::monospace(
                                ui,
                                &dlms_learning_lab::localization::readable(&output.stdout),
                                false,
                            );
                        }
                    },
                );
            }
            Ok(Some(Err(error))) => {
                let label = match error.kind {
                    dlms_learning_lab::boot_entry::ErrorKind::Syntax => "Command syntax",
                    dlms_learning_lab::boot_entry::ErrorKind::Unsupported => {
                        "Outside the supported subset"
                    }
                    dlms_learning_lab::boot_entry::ErrorKind::Limit => "Simulator limit",
                    dlms_learning_lab::boot_entry::ErrorKind::Invalid => {
                        "Cannot perform this action"
                    }
                };
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!("{label}: {error}")).color(palette(ui).error),
                );
            }
            Err(error) => {
                lesson_presentation::rich(ui, RichText::new(error).color(palette(ui).error));
            }
        }
    });
    m.record(ui, "last-result", &actual.response);
}
fn provided(ui: &mut egui::Ui, t: &work::Task) {
    ui.strong("Supplied read-only inputs · /work");
    for document in &t.facts.inputs {
        controls::form_row(ui, &document.label, |ui, _| {
            lesson_presentation::monospace(ui, &document.path, true)
        });
    }
    ui.small("Byte displays use UTF-8 text where valid and escaped bytes otherwise; they do not convert saved files.");
}
fn documents(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let Ok(world) = d.world(t) else {
        return;
    };
    let mut paths = world
        .files()
        .inputs()
        .iter()
        .map(|f| f.path.clone())
        .collect::<Vec<_>>();
    paths.extend(world.files().outputs().keys().cloned());
    let id = ui.make_persistent_id(("locale-document-view", &t.id));
    let mut selected = ui
        .ctx()
        .data(|data| data.get_temp::<String>(id))
        .filter(|p| paths.contains(p))
        .unwrap_or_else(|| paths[0].clone());
    ui.strong("Inspect document · actual saved bytes");
    let menu = controls::dropdown(ui, "locale-document-menu", ui.available_width())
        .selected_text(&selected)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (n, path) in paths.iter().enumerate() {
                let label = format!(
                    "{path} · {}",
                    if world.files().outputs().contains_key(path) {
                        "saved output"
                    } else {
                        "supplied input"
                    }
                );
                let r = ui.selectable_value(&mut selected, path.clone(), label);
                m.record(ui, &format!("document-{n}"), &r);
            }
            let clip = controls::popup_content_clip(ui);
            for row in &mut m.controls {
                if row.0.starts_with("document-") {
                    row.2 = clip;
                }
            }
        });
    m.record(ui, "document-menu", &menu.response);
    ui.ctx()
        .data_mut(|data| data.insert_temp(id, selected.clone()));
    let body = ui.scope(|ui| {
        let bytes = world.files().read(&selected).unwrap();
        ui.label(format!(
            "{} bytes · {}",
            bytes.len(),
            if std::str::from_utf8(bytes).is_ok() {
                "valid UTF-8"
            } else {
                "not valid UTF-8"
            }
        ));
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8.;
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("locale-document-bytes")
                .max_height(height)
                .auto_shrink([false, true]),
            |ui| {
                lesson_presentation::monospace(
                    ui,
                    &dlms_learning_lab::localization::readable(bytes),
                    false,
                )
            },
        );
    });
    m.record(ui, "document-body", &body.response);
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let label = ui.strong("Saved locale defaults · /etc/locale.conf");
    let mut text = d.configuration.clone();
    let r = ui
        .add(
            egui::TextEdit::multiline(&mut text)
                .font(egui::TextStyle::Monospace)
                .desired_width(ui.available_width())
                .desired_rows(5)
                .char_limit(2048),
        )
        .labelled_by(label.id);
    m.record(ui, "configuration", &r);
    if r.changed()
        && let Err(e) = d.edit_configuration(text)
    {
        *notice = Some(e);
    }
    if practice {
        let r = ui.add(button("Fill worked settings"));
        m.record(ui, "fill-worked", &r);
        if r.clicked()
            && let Some(work::Action::SaveLocale { text }) = t
                .reference
                .iter()
                .find(|a| matches!(a, work::Action::SaveLocale { .. }))
            && let Err(e) = d.edit_configuration(text.clone())
        {
            *notice = Some(e);
        }
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save locale settings"));
        m.record(ui, "save-config", &r);
        if r.clicked() {
            handle(d.save_configuration(t), notice);
        }
        let r = ui.add(button("Open fresh login"));
        m.record(ui, "fresh-login", &r);
        if r.clicked() {
            handle(d.perform(t, work::Action::NewSession {}), notice);
        }
    });
    if let Ok(world) = d.world(t) {
        ui.label(if world.saved_locale() == d.configuration {
            "Editor matches saved defaults."
        } else {
            "Unsaved edits · current checks are invalidated."
        });
    }
    ui.label("A separate simulated login reads saved defaults. It does not inherit this terminal's overrides.");
}
fn report_control(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let mut inputs = t
        .cases
        .iter()
        .filter_map(|c| {
            if let work::Case::Report { input, .. } = c {
                Some(input.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    inputs.sort();
    inputs.dedup();
    let label = ui.strong("Sample reporting application");
    let menu = controls::dropdown(ui, "locale-report-input", ui.available_width())
        .selected_text(&d.report_input)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (n, input) in inputs.iter().enumerate() {
                let r = ui.selectable_value(&mut d.report_input, input.clone(), input);
                m.record(ui, &format!("report-input-{n}"), &r);
            }
        });
    m.record(ui, "report-input", &menu.response);
    let r = ui.add(button("Run sample report")).labelled_by(label.id);
    m.record(ui, "run-report", &r);
    if r.clicked() {
        handle(d.run_report(t), notice);
    }
}

fn feedback(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &work::session::Draft,
    practice: bool,
    m: &mut Metrics,
) {
    if !d.attempted {
        let r = ui.label("Not checked");
        m.record(ui, "result", &r);
        return;
    }
    let rows = match d.checks(t) {
        Ok(rows) => rows,
        Err(error) => {
            ui.label(error);
            return;
        }
    };
    let failed = rows.iter().filter(|r| !r.correct).count();
    let summary = if failed == 0 {
        if practice {
            "Example requirements met".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs attention · {failed} {}",
            if failed == 1 {
                "requirement fails"
            } else {
                "requirements fail"
            }
        )
    };
    m.summary = Some(summary.clone());
    let r = lesson_presentation::rich(
        ui,
        RichText::new(summary).color(if failed == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    let draw = |ui: &mut egui::Ui, index: usize, row: work::Check, m: &mut Metrics| {
        let r = ui.push_id(index, |ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}: {}",
                    if row.correct {
                        "Meets requirement"
                    } else {
                        "Needs attention"
                    },
                    row.label,
                    row.feedback
                ))
                .color(if row.correct {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            if !row.correct
                && let (Some(expected), Some(actual)) = (row.expected, row.actual)
            {
                controls::collapsing_header(ui, "Expected versus actual sample").show(ui, |ui| {
                    ui.strong("Expected");
                    code(ui, &dlms_learning_lab::localization::readable(&expected));
                    ui.strong("Actual");
                    code(ui, &dlms_learning_lab::localization::readable(&actual));
                });
            }
        });
        m.record(ui, &format!("feedback-{index}"), &r.response);
    };
    for (index, row) in rows.iter().enumerate().filter(|(_, r)| !r.correct) {
        draw(ui, index, row.clone(), m);
    }
    let passed = rows.len() - failed;
    if passed > 0 {
        let r = controls::collapsing_header(ui, &format!("Show {passed} passing checks")).show(
            ui,
            |ui| {
                for (index, row) in rows.into_iter().enumerate().filter(|(_, r)| r.correct) {
                    draw(ui, index, row, m);
                }
            },
        );
        m.record(ui, "passing-checks", &r.header_response);
    }
}
impl LabApp {
    pub(super) fn localization_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let def =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(error) => {
                    self.notice = Some(error);
                    return;
                }
            };
        let engine::State::Localization { simulation } = &attempt.state else {
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
        self.metrics.localization = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.localization.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8),|ui|{
            if view==teaching::View::Introduction {
                lesson_presentation::concepts(ui,&source,"Builds on file and terminal foundations. The supplied process, settings and input documents are entirely simulated.",Vec::<(String,String)>::new());
                lesson_presentation::worked(ui,"Different supplied inputs and settings demonstrate the method before independent work.");for (number,response) in lesson_presentation::worked_command_rows(ui,&source).iter().enumerate(){self.metrics.localization.record(ui,&format!("worked-value-{number}"),response);}
                ui.horizontal_wrapped(|ui|{for (choice,key,label) in [(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.localization.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let (task,draft)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    provided(ui,task);ui.separator();
                    let wide=ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*48.;
                    if task.has_reports() {
                        if wide {
                            ui.columns(2,|columns|{
                                guidance(&mut columns[0],&source,0,practice);
                                report_control(&mut columns[0],task,draft,&mut self.metrics.localization,&mut self.notice);
                                terminal(&mut columns[0],task,draft,Lesson{practice,active},&mut self.metrics.localization,&mut self.notice);
                                guidance(&mut columns[1],&source,1,practice);
                                editor(&mut columns[1],task,draft,practice,&mut self.metrics.localization,&mut self.notice);
                                controls::collapsing_header(&columns[1],"Inspect input bytes · optional").show(&mut columns[1],|ui|documents(ui,task,draft,&mut self.metrics.localization));
                            });
                        }else{
                            guidance(ui,&source,0,practice);
                            report_control(ui,task,draft,&mut self.metrics.localization,&mut self.notice);
                            terminal(ui,task,draft,Lesson{practice,active},&mut self.metrics.localization,&mut self.notice);
                            guidance(ui,&source,1,practice);
                            editor(ui,task,draft,practice,&mut self.metrics.localization,&mut self.notice);
                            controls::collapsing_header(ui,"Inspect input bytes · optional").show(ui,|ui|documents(ui,task,draft,&mut self.metrics.localization));
                        }
                    }else if wide {
                        ui.columns(2,|columns|{
                            guidance(&mut columns[0],&source,0,practice);guidance(&mut columns[0],&source,1,practice);
                            documents(&mut columns[0],task,draft,&mut self.metrics.localization);
                            terminal(&mut columns[1],task,draft,Lesson{practice,active},&mut self.metrics.localization,&mut self.notice);
                        });
                    }else{
                        guidance(ui,&source,0,practice);
                        terminal(ui,task,draft,Lesson{practice,active},&mut self.metrics.localization,&mut self.notice);
                        guidance(ui,&source,1,practice);documents(ui,task,draft,&mut self.metrics.localization);
                    }
                    ui.separator();guidance(ui,&source,2,practice);
                    let r=ui.add_enabled(active,button("Check work").fill(palette(ui).accent));self.metrics.localization.record(ui,"check",&r);
                    if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,practice,&mut self.metrics.localization);
                    if practice{let r=ui.add_enabled(active,button("Reset example only"));self.metrics.localization.record(ui,"reset-example",&r);if r.clicked(){*draft=work::session::Draft::new(task);self.notice=None;}}
                    controls::collapsing_header(ui,"Supported commands and help · optional").show(ui,|ui|{
                        lesson_presentation::prose(ui,work::HELP);let attempt=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&attempt.hints<source.hints.len(),button("Show a hint"));if r.clicked(){attempt.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}lesson_presentation::prose(ui,&source.simplifications);controls::collapsing_header(ui,"Worked approach · reveals one method").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                    });
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::Localization {
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
