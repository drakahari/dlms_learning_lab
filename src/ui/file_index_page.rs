//! Native private-index editing and terminal investigation. All actions stay in memory.
use super::*;
use dlms_learning_lab::{engine::State, file_index_workspace as work, script_teaching::View};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub last_output: Option<(egui::Rect, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
#[derive(Clone)]
enum Edit {
    Config(String),
    Save,
    Command(String),
    Run,
    Check,
    Reset,
}
fn action_text(step: &work::Step) -> String {
    match step {
        work::Step::SaveConfig { text } => format!("Save this practice configuration:\n{text}"),
        work::Step::Command { input } => input.clone(),
    }
}
#[derive(Clone, Copy)]
struct Page<'a> {
    s: &'a work::Session,
    d: &'a work::Definition,
    scope: work::Scope,
    key: egui::Id,
    enabled: bool,
}
struct Editing<'a> {
    m: &'a mut Metrics,
    edits: &'a mut Vec<Edit>,
    reject: &'a mut bool,
}
fn worked_example(ui: &mut egui::Ui, source: &Scenario) {
    if let Some(entry) = lesson_presentation::entry(source) {
        let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 38.;
        for row in &entry.worked {
            if wide {
                ui.columns(2, |columns| {
                    lesson_presentation::rich(&mut columns[0], RichText::new(&row.label).strong());
                    lesson_presentation::prose(&mut columns[0], &row.explanation);
                    lesson_presentation::monospace(&mut columns[1], &row.value, true);
                });
            } else {
                lesson_presentation::rich(ui, RichText::new(&row.label).strong());
                lesson_presentation::monospace(ui, &row.value, true);
                lesson_presentation::prose(ui, &row.explanation);
            }
            ui.add_space(8.);
        }
    }
}
fn configuration(ui: &mut egui::Ui, page: Page<'_>, editing: Editing<'_>) {
    let Page {
        s,
        d,
        scope,
        key,
        enabled,
    } = page;
    let Editing { m, edits, reject } = editing;
    let c = s.current(scope);
    let t = match scope {
        work::Scope::Assignment => &d.assignment,
        work::Scope::Example => &d.example,
    };
    lesson_presentation::rich(ui, RichText::new("Saved-name exclusions").strong());
    lesson_presentation::monospace(ui, &t.initial.context.config, true);
    lesson_presentation::prose(
        ui,
        "Edit PRUNEPATHS, then save. Saving does not rebuild the private index.",
    );
    let e = shared_libraries_page::text_input(
        ui,
        &c.config_draft,
        key.with("config"),
        4096,
        true,
        enabled,
        "PRUNEPATHS=\"/supplied/excluded/folder\"",
    );
    m.fields
        .push(("config".into(), e.response.rect, e.response.id, e.clip));
    *reject |= e.rejected;
    if let Some(v) = e.value {
        edits.push(Edit::Config(v));
    }
    let r = ui.add_enabled(enabled && !*reject, button("Save configuration"));
    record(m, "save", &r, ui);
    if r.clicked() {
        edits.push(Edit::Save);
    }
    if c.config_draft
        != c.tree
            .files
            .get(&t.initial.context.config)
            .cloned()
            .unwrap_or_default()
    {
        lesson_presentation::prose(
            ui,
            "Unsaved changes · searches still use the last built index.",
        );
    }
    ui.add_space(controls::GAP);
    lesson_presentation::rich(ui, RichText::new("Private index").strong());
    lesson_presentation::monospace(ui, &t.initial.context.database, true);
    let count = c.index.as_ref().map(|x| x.entries().len());
    lesson_presentation::prose(
        ui,
        &match count {
            Some(n) => format!(
                "{n} saved names · rebuild explicitly with updatedb. This is a simulated index, separate from ordinary files."
            ),
            None => "No private index has been built.".into(),
        },
    );
}
fn terminal(ui: &mut egui::Ui, page: Page<'_>, editing: Editing<'_>) {
    let Page {
        s,
        scope,
        key,
        enabled,
        ..
    } = page;
    let Editing { m, edits, reject } = editing;
    let c = s.current(scope);
    lesson_presentation::rich(ui, RichText::new("Investigate and verify").strong());
    lesson_presentation::prose(
        ui,
        "Type a supported command below. Enter runs it in the simulated filesystem.",
    );
    lesson_presentation::monospace(ui, &format!("{}: {}", c.tree.user, c.tree.cwd), true);
    let e = shared_libraries_page::text_input(
        ui,
        &c.command,
        key.with("command"),
        4096,
        false,
        enabled,
        "find, locate, updatedb, cat, cp …",
    );
    m.fields
        .push(("command".into(), e.response.rect, e.response.id, e.clip));
    *reject |= e.rejected;
    let enter = (e.response.has_focus() || e.response.lost_focus())
        && ui.input(|i| i.key_pressed(egui::Key::Enter));
    if let Some(v) = e.value {
        edits.push(Edit::Command(v));
    }
    let r = ui.add_enabled(enabled && !*reject, button("Run command"));
    record(m, "run", &r, ui);
    if r.clicked() || enter {
        edits.push(Edit::Run);
    }
    lesson_presentation::rich(ui, RichText::new("Actual terminal output").strong());
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.;
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt(key.with("output"))
            .stick_to_bottom(true)
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false]),
        |ui| {
            if c.transcript.is_empty() {
                lesson_presentation::prose(ui, "No command has run yet.");
            }
            for row in &c.transcript {
                lesson_presentation::monospace(ui, &format!("$ {}", row.input), true);
                if row.output.is_empty() {
                    lesson_presentation::prose(ui, "(No output)");
                } else {
                    let response = lesson_presentation::monospace(ui, &row.output, true);
                    m.last_output = Some((response.rect, ui.clip_rect()));
                }
                match row.outcome {
                    work::Outcome::IndexRebuilt => {
                        lesson_presentation::prose(
                            ui,
                            "Simulator result: private index rebuilt from saved configuration.",
                        );
                    }
                    work::Outcome::NoMatch => {
                        lesson_presentation::prose(
                            ui,
                            "Simulator result: no matching saved names.",
                        );
                    }
                    _ => {}
                }
            }
        },
    );
}
impl LabApp {
    pub(super) fn file_index_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = dlms_learning_lab::file_index_source::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (State::FileIndex { simulation }, engine::Goal::FileIndex { definition }) =
            (&a.state, &source.goal)
        else {
            return;
        };
        let mut next = (**simulation).clone();
        let original = next.clone();
        let visit = a.visited;
        let enabled = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            View::Assignment
        } else {
            next.view
        };
        self.metrics.file_index = Metrics::default();
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label, key) in [
                    (View::Introduction, "Introduction", "intro"),
                    (View::Example, "Try example", "example"),
                    (View::Assignment, "Assignment", "assignment"),
                ] {
                    let r = ui.selectable_value(&mut view, v, label);
                    record(&mut self.metrics.file_index, key, &r, ui);
                }
            });
            if view != next.view && enabled {
                next.view = view;
            }
        }
        let scope = if view == View::Example {
            work::Scope::Example
        } else {
            work::Scope::Assignment
        };
        let key = egui::Id::new(("file-index", &source.id, source.version, visit, view as u8));
        let mut edits = vec![];
        let mut reject = false;
        if view == View::Introduction {
            lesson_presentation::lead(ui, "WHAT YOU’LL LEARN", &source.summary);
            lesson_presentation::prose(ui, &definition.teaching.prerequisite_note);
            for (n, c) in definition.teaching.concepts.iter().enumerate() {
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!("{}. {}", n + 1, c.title)).strong(),
                );
                lesson_presentation::prose(ui, &c.text);
                ui.add_space(6.);
            }
            lesson_presentation::worked(ui, &definition.teaching.description);
            worked_example(ui, &source);
            let r = ui.add_enabled(enabled, button("Try example"));
            record(&mut self.metrics.file_index, "start-example", &r, ui);
            if r.clicked() {
                next.view = View::Example;
            }
            let r = ui.add_enabled(enabled, button("Go to assignment"));
            record(&mut self.metrics.file_index, "start-assignment", &r, ui);
            if r.clicked() {
                next.view = View::Assignment;
            }
        } else {
            if view == View::Example {
                lesson_presentation::lead(
                    ui,
                    "GUIDED PRACTICE",
                    "Diagnose the separate practice index",
                );
                lesson_presentation::prose(
                    ui,
                    "Practice is ungraded; assignment work is kept separately.",
                );
                let steps = &definition.teaching.try_steps;
                let selected = next.selected_example.min(steps.len().saturating_sub(1));
                ui.horizontal_wrapped(|ui| {
                    for n in 0..steps.len() {
                        let r = ui.selectable_label(n == selected, format!("{}", n + 1));
                        record(&mut self.metrics.file_index, &format!("step:{n}"), &r, ui);
                        if r.clicked() && enabled {
                            next.selected_example = n;
                        }
                    }
                });
                if let Some(step) = steps.get(selected) {
                    lesson_presentation::rich(ui, RichText::new(&step.title).strong());
                    lesson_presentation::prose(ui, &step.explanation);
                    lesson_presentation::monospace(ui, &action_text(&step.step), true);
                    let label = match step.step {
                        work::Step::Command { .. } => "Insert practice command",
                        _ => "Fill practice configuration",
                    };
                    let r = ui.add_enabled(enabled, button(label));
                    record(&mut self.metrics.file_index, "insert", &r, ui);
                    if r.clicked() {
                        match &step.step {
                            work::Step::Command { input } => {
                                edits.push(Edit::Command(input.clone()))
                            }
                            work::Step::SaveConfig { text } => {
                                edits.push(Edit::Config(text.clone()))
                            }
                        }
                    }
                }
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_space(controls::GAP);
            let page = Page {
                s: &next,
                d: definition,
                scope,
                key,
                enabled,
            };
            if ui.available_width()
                > ui.text_style_height(&egui::TextStyle::Body) * 52. + controls::GAP
            {
                ui.columns(2, |col| {
                    configuration(
                        &mut col[0],
                        page,
                        Editing {
                            m: &mut self.metrics.file_index,
                            edits: &mut edits,
                            reject: &mut reject,
                        },
                    );
                    terminal(
                        &mut col[1],
                        page,
                        Editing {
                            m: &mut self.metrics.file_index,
                            edits: &mut edits,
                            reject: &mut reject,
                        },
                    );
                });
            } else {
                configuration(
                    ui,
                    page,
                    Editing {
                        m: &mut self.metrics.file_index,
                        edits: &mut edits,
                        reject: &mut reject,
                    },
                );
                ui.add_space(controls::GAP);
                terminal(
                    ui,
                    page,
                    Editing {
                        m: &mut self.metrics.file_index,
                        edits: &mut edits,
                        reject: &mut reject,
                    },
                );
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(
                    enabled && !reject,
                    button(if scope == work::Scope::Example {
                        "Check practice"
                    } else {
                        "Check work"
                    }),
                );
                record(&mut self.metrics.file_index, "check", &r, ui);
                if r.clicked() {
                    edits.push(Edit::Check);
                }
                if scope == work::Scope::Example {
                    let r = ui.add_enabled(enabled, button("Reset example"));
                    record(&mut self.metrics.file_index, "reset-example", &r, ui);
                    if r.clicked() {
                        edits.push(Edit::Reset);
                    }
                }
            });
            if next.current(scope).checked {
                if let Ok(r) = next.checks(definition, scope) {
                    let passed = r.checks.iter().filter(|c| c.passed).count();
                    let heading = if r.passed() {
                        if scope == work::Scope::Example {
                            "Practice requirements satisfied"
                        } else {
                            "Assignment complete"
                        }
                    } else {
                        "Needs another look"
                    };
                    let response = lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "{heading} · {passed}/{} requirements",
                            r.checks.len()
                        ))
                        .strong(),
                    );
                    self.metrics.file_index.result = Some((passed, r.checks.len(), response.rect));
                    for c in r.checks.iter().filter(|c| !c.passed) {
                        lesson_presentation::prose(ui, &format!("Needs attention · {}", c.message));
                    }
                    controls::collapsing_header(ui, "Objective results").show(ui, |ui| {
                        for c in &r.checks {
                            lesson_presentation::prose(
                                ui,
                                &format!(
                                    "{} · {}",
                                    if c.passed { "Met" } else { "Needs attention" },
                                    c.message
                                ),
                            );
                        }
                    });
                    if r.passed() && scope == work::Scope::Assignment {
                        self.metrics.completion = true;
                        lesson_presentation::prose(ui, &source.explanation);
                        let back = ui.add_enabled(enabled, button("Back to library"));
                        if back.clicked() {
                            self.selected = None;
                        }
                        ui.add_enabled_ui(enabled, |ui| self.next_unit_lab(ui, i));
                    }
                }
            } else {
                let r = lesson_presentation::prose(
                    ui,
                    "Not checked · inspect, repair and verify, then check your work.",
                );
                record(&mut self.metrics.file_index, "unchecked", &r, ui);
            }
            ui.push_id((key, challenge, "optional-help"), |ui| {
                let help = controls::collapsing_header(ui, "Optional help and simulation scope")
                    .show(ui, |ui| {
                        if scope == work::Scope::Assignment {
                            let a = self.progress.attempts.get_mut(&source.id).unwrap();
                            let r = ui.add_enabled(
                                enabled && a.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            if r.clicked() {
                                a.hints += 1;
                                self.dirty = true;
                            }
                            for h in source.hints.iter().take(a.hints) {
                                lesson_presentation::prose(ui, h);
                            }
                            let worked = controls::collapsing_header(
                                ui,
                                "Worked approach · reveals a solution",
                            )
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                            });
                            record(
                                &mut self.metrics.file_index,
                                "worked-help",
                                &worked.header_response,
                                ui,
                            );
                        }
                        lesson_presentation::prose(ui, work::HELP);
                        controls::collapsing_header(ui, "References and limitations").show(
                            ui,
                            |ui| {
                                lesson_presentation::prose(ui, &source.simplifications);
                                for url in &source.sources {
                                    ui.hyperlink_to(
                                        url,
                                        url.split_whitespace().next().unwrap_or(url),
                                    );
                                }
                            },
                        );
                    });
                record(
                    &mut self.metrics.file_index,
                    "help",
                    &help.header_response,
                    ui,
                );
            });
        }
        if reject {
            self.notice=Some("Input exceeds the supported size or contains an invalid character; previous text retained.".into());
            edits.clear();
        }
        if enabled && !reject {
            for edit in edits {
                let result = match edit {
                    Edit::Config(v) => next.set_config_draft(definition, scope, v),
                    Edit::Save => next.save_config(definition, scope).map(|_| ()),
                    Edit::Command(v) => next.set_command(definition, scope, v),
                    Edit::Run => next.run(definition, scope).map(|_| ()),
                    Edit::Check => next.check(definition, scope).map(|_| ()),
                    Edit::Reset => next.reset_example(definition),
                };
                if let Err(e) = result {
                    self.notice = Some(e.to_string());
                    break;
                }
            }
        }
        if next != original {
            if let Err(e) = next.validate_for(definition) {
                self.notice = Some(e.to_string());
                return;
            }
            if let State::FileIndex { simulation } =
                &mut self.progress.attempts.get_mut(&source.id).unwrap().state
            {
                **simulation = next;
                self.completion_index.forget(&source.id);
                self.dirty = true;
                ui.ctx().request_repaint();
            }
        }
    }
}
