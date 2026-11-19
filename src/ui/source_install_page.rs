//! Native supplied-project terminal with current build state and optional file view.
use super::*;
use dlms_learning_lab::{source_install as core, source_install_workspace as work};
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
struct Lesson<'a> {
    source: &'a pack::Scenario,
    practice: bool,
    active: bool,
}

fn guidance(ui: &mut egui::Ui, source: &pack::Scenario, step: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, source, step);
    }
}
fn terminal(
    ui: &mut egui::Ui,
    task: &work::Task,
    draft: &mut work::Draft,
    lesson: Lesson<'_>,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Lesson {
        source,
        practice,
        active,
    } = lesson;
    guidance(ui, source, 0, practice);
    if practice {
        let id = ui.make_persistent_id("source-worked-command");
        let mut selected = ui
            .ctx()
            .data(|d| d.get_temp::<usize>(id))
            .unwrap_or(0)
            .min(task.reference.len().saturating_sub(1));
        let selected_text = task
            .reference
            .get(selected)
            .map(|c| c.0.as_str())
            .unwrap_or("No worked command");
        let menu = controls::dropdown(ui, "source-command-menu", ui.available_width())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, command) in task.reference.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, index, &command.0);
                    metrics.record(ui, &format!("worked-{index}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for entry in &mut metrics.controls {
                    if entry.0.starts_with("worked-") {
                        entry.2 = clip;
                    }
                }
            });
        metrics.record(ui, "worked-menu", &menu.response);
        ui.ctx().data_mut(|d| d.insert_temp(id, selected));
        let r = ui.add_enabled(active, button("Insert command"));
        metrics.record(ui, "insert", &r);
        if r.clicked()
            && let Some(command) = task.reference.get(selected)
        {
            draft.input = command.0.clone();
        }
        ui.label("Fills the input only. Run command executes it in this simulated workspace.");
    }
    let label = ui.strong("Simulated terminal");
    let mut input = draft.input.clone();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut input, ui.available_width())
                .hint_text("Command from Supported commands")
                .char_limit(1024),
        )
        .labelled_by(label.id);
    metrics.record(ui, "command", &r);
    if r.changed() {
        if input.len() <= 1024 {
            draft.input = input;
        } else {
            *notice = Some("Keep one command within1024bytes.".into());
        }
    }
    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let r = ui.add_enabled(active, button("Run command"));
    metrics.record(ui, "run", &r);
    if r.clicked() || active && enter {
        match draft.run(task) {
            Ok(_) => *notice = None,
            Err(error) => *notice = Some(error),
        }
    }
    let output = ui.scope(|ui| {
        ui.strong("Actual last run");
        if let Some(run) = &draft.last_run {
            code(ui, &format!("$ {}", run.command.0));
        }
        match draft.last_output(task) {
            Ok(None) => {
                ui.label("No command has run in this draft yet.");
            }
            Ok(Some(Ok(output))) => {
                ui.label(format!("Command finished · exit status {}", output.status));
                let height = ui.text_style_height(&egui::TextStyle::Monospace) * 12.;
                egui::ScrollArea::vertical()
                    .id_salt("source-actual-output")
                    .max_height(height)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if output.stdout.is_empty() {
                            ui.label("No standard output.");
                        } else {
                            code(ui, &output.stdout);
                        }
                        if !output.stderr.is_empty() {
                            ui.strong("Standard error");
                            code(ui, &output.stderr);
                        }
                    });
            }
            Ok(Some(Err(error))) => {
                let heading = match error.kind {
                    dlms_learning_lab::boot_entry::ErrorKind::Syntax => "Command syntax",
                    dlms_learning_lab::boot_entry::ErrorKind::Unsupported => {
                        "Outside the supported subset"
                    }
                    dlms_learning_lab::boot_entry::ErrorKind::Limit => "Simulator limit",
                    dlms_learning_lab::boot_entry::ErrorKind::Invalid => {
                        "Cannot perform this command"
                    }
                };
                ui.colored_label(palette(ui).error, format!("{heading}: {error}"));
            }
            Err(error) => {
                ui.colored_label(palette(ui).error, error);
            }
        }
    });
    metrics.record(ui, "last-result", &output.response);
}
fn state(
    ui: &mut egui::Ui,
    source: &pack::Scenario,
    task: &work::Task,
    draft: &work::Draft,
    practice: bool,
    metrics: &mut Metrics,
) {
    guidance(ui, source, 1, practice);
    ui.strong("Current simulated project");
    let frame = match task.replay(&draft.history) {
        Ok(frame) => frame,
        Err(error) => {
            ui.colored_label(palette(ui).error, error);
            return;
        }
    };
    let world = &frame.world;
    for (label, value) in [
        ("Source archive", task.facts.archive.clone()),
        ("Current directory", frame.cwd.clone()),
        (
            "Source tree",
            world
                .source_root
                .clone()
                .unwrap_or_else(|| "Not unpacked".into()),
        ),
        (
            "Configured prefix",
            world
                .configuration
                .as_ref()
                .map(|c| c.prefix.clone())
                .unwrap_or_else(|| "Not configured".into()),
        ),
        (
            "Build",
            if world.is_build_current(&task.facts) {
                "Current"
            } else if world.built.is_some() {
                "Needs rebuilding"
            } else {
                "Not built"
            }
            .into(),
        ),
        (
            "Package self-check",
            if world.is_check_current(&task.facts) {
                "Current"
            } else {
                "Not checked for this build"
            }
            .into(),
        ),
    ] {
        controls::form_row(ui, label, |ui, width| {
            let response = controls::cell(ui, width, |ui| {
                lesson_presentation::monospace(ui, &value, true)
            });
            metrics.record(ui, &format!("state-{label}"), &response.inner);
        });
    }
    controls::collapsing_header(ui,"Inspect virtual files · optional").show(ui,|ui|{
        let id=ui.make_persistent_id("source-file-selection");
        let prior=ui.ctx().data(|d|d.get_temp::<String>(id));
        let mut selected=prior.filter(|p|world.files.contains_key(p)).or_else(||world.source_root.as_ref().and_then(|p|core::join(p,"config.status").ok()).filter(|p|world.files.contains_key(p))).unwrap_or_else(||task.facts.archive.clone());
        let menu=controls::dropdown(ui,"source-file-menu",ui.available_width()).selected_text(&selected).show_ui(ui,|ui|{controls::fit_popup_contents(ui);for path in world.files.keys(){let r=ui.selectable_value(&mut selected,path.clone(),path);metrics.record(ui,&format!("file-{path}"),&r);}let clip=controls::popup_content_clip(ui);for entry in &mut metrics.controls{if entry.0.starts_with("file-"){entry.2=clip;}}});metrics.record(ui,"file-menu",&menu.response);
        ui.ctx().data_mut(|d|d.insert_temp(id,selected.clone()));
        if let Some(file)=world.files.get(&selected){ui.label(format!("Virtual file · mode{:04o}",file.mode()));
            match file {
                core::File::Archive=>{ui.label("Supplied structured uncompressed archive. Use tar -tf to inspect its members; byte-format/compression emulation is outside scope.");},
                core::File::Text{text,..}=>{egui::ScrollArea::vertical().id_salt("source-file-text").max_height(ui.text_style_height(&egui::TextStyle::Monospace)*10.).show(ui,|ui|code(ui,text));},
                core::File::Program{artifact,..}=>{ui.label(format!("Supplied program: {} {}",artifact.program.name,artifact.program.version));code(ui,&format!("Compiled prefix: {}\nGreeting: {}\nUppercase: {}",artifact.configuration.prefix,artifact.configuration.greeting,if artifact.configuration.uppercase{"Enabled"}else{"Disabled"}));ui.label("This copy keeps these compiled settings until it is replaced.");},
            }
        }
    });
}
fn feedback(ui: &mut egui::Ui, task: &work::Task, draft: &work::Draft, metrics: &mut Metrics) {
    if !draft.attempted {
        let r = ui.label("Not checked");
        metrics.record(ui, "result", &r);
        return;
    }
    let frame = match task.replay(&draft.history) {
        Ok(f) => f,
        Err(error) => {
            ui.colored_label(palette(ui).error, error);
            return;
        }
    };
    let rows = task.grade(&frame);
    let failed = rows.iter().filter(|r| !r.correct).count();
    let color = if failed == 0 {
        palette(ui).success
    } else {
        palette(ui).error
    };
    let text = if failed == 0 {
        "All project requirements met".into()
    } else {
        format!("Needs attention · {failed} requirements fail")
    };
    metrics.summary = Some(text.clone());
    let summary = lesson_presentation::rich(ui, RichText::new(text).color(color));
    metrics.record(ui, "result", &summary);
    for (number, row) in rows.into_iter().enumerate() {
        let key = row.label.clone();
        let area = ui.push_id(&key, |ui| {
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
                && let Some(expected) = row.expected
            {
                controls::collapsing_header(ui, "Expected versus actual output").show(ui, |ui| {
                    ui.strong("Expected");
                    code(
                        ui,
                        &format!(
                            "stdout: {:?}\nstderr: {:?}\nexit status: {}",
                            expected.stdout, expected.stderr, expected.status
                        ),
                    );
                    ui.strong("Actual");
                    if let Some(actual) = row.actual {
                        code(
                            ui,
                            &format!(
                                "stdout: {:?}\nstderr: {:?}\nexit status: {}",
                                actual.stdout, actual.stderr, actual.status
                            ),
                        );
                    } else {
                        ui.label("No runnable program at the requested virtual path.");
                    }
                });
            }
        });
        metrics.record(ui, &format!("feedback-{number}"), &area.response);
    }
}
impl LabApp {
    pub(super) fn source_install_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::State::SourceInstall { simulation } = &attempt.state else {
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
        self.metrics.source_install = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.source_install.record(ui, key, &r);
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
                lesson_presentation::concepts(ui,&source,"Builds on virtual archive navigation, file paths and package installation concepts.",Vec::<(String,String)>::new());
                lesson_presentation::worked(ui,"A different supplied project demonstrates configuration, build checks and staged files.");for (number,response) in lesson_presentation::worked_command_rows(ui,&source).iter().enumerate(){self.metrics.source_install.record(ui,&format!("worked-value-{number}"),response);}
                ui.horizontal_wrapped(|ui|{for (choice,key,label) in [(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.source_install.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let (task,draft)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    if ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*48. {ui.columns(2,|columns|{terminal(&mut columns[0],task,draft,Lesson{source:&source,practice,active},&mut self.metrics.source_install,&mut self.notice);state(&mut columns[1],&source,task,draft,practice,&mut self.metrics.source_install);});}else{terminal(ui,task,draft,Lesson{source:&source,practice,active},&mut self.metrics.source_install,&mut self.notice);ui.separator();state(ui,&source,task,draft,practice,&mut self.metrics.source_install);}
                    ui.separator();guidance(ui,&source,2,practice);
                    let r=ui.add_enabled(active,button(task.check_label(practice)).fill(palette(ui).accent));self.metrics.source_install.record(ui,"check",&r);
                    if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,&mut self.metrics.source_install);
                    if practice{let r=ui.add_enabled(active,button("Reset example only"));self.metrics.source_install.record(ui,"reset-example",&r);if r.clicked(){*draft=work::Draft::new(task);self.notice=None;}}
                    controls::collapsing_header(ui,"Supported commands and help · optional").show(ui,|ui|{
                        lesson_presentation::prose(ui,work::HELP);let attempt=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&attempt.hints<source.hints.len(),button("Show a hint"));if r.clicked(){attempt.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}lesson_presentation::prose(ui,&source.simplifications);controls::collapsing_header(ui,"Worked approach · reveals one method").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                    });
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state =
                engine::State::SourceInstall {
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
