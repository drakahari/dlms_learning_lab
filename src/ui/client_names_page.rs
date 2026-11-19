//! Native local-name editing and explicit source/status/action traces.
use super::*;
use dlms_learning_lab::client_names_workspace as work;
use std::sync::Arc;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub running: bool,
    pub case_passes: Vec<bool>,
    pub editor_scroll: Option<(f32, f32)>,
    pub editor_rows: Option<usize>,
    pub diagnostic_input: Option<(egui::Rect, egui::Id)>,
    pub diagnostic_output: Option<egui::Rect>,
    pub diagnostic_status: Option<Result<u8, String>>,
}
pub(super) struct ActiveRun {
    pub scenario: String,
    pub practice: bool,
    draft: work::Files,
    run: work::Run,
}
#[derive(Clone)]
struct Report {
    task: work::Task,
    draft: work::Files,
    rows: Vec<work::CaseResult>,
}
#[derive(Clone)]
struct DiagnosticReport {
    files: work::Files,
    command: String,
    sample: usize,
    result: Result<dlms_learning_lab::client_diagnostic::Output, String>,
}
fn diagnostic(
    ui: &mut egui::Ui,
    task: &work::Task,
    scope: &mut work::Scope,
    selected: usize,
    active: bool,
    m: &mut Metrics,
) {
    let Some(draft) = scope.diagnostic.as_mut() else {
        return;
    };
    draft.sample = selected;
    lesson_presentation::rich(
        ui,
        RichText::new("Investigate this sample · simulated terminal").strong(),
    );
    lesson_presentation::prose(
        ui,
        "getent ahostsv4 NAME asks the configured sources for one IPv4 name. Run command inspects this sample; Run lookups checks all samples.",
    );
    let input = ui.add_enabled(
        active,
        controls::singleline_fitting(ui, &mut draft.input, ui.available_width())
            .hint_text("getent ahostsv4 NAME")
            .char_limit(512)
            .id_salt(("names-diagnostic-input", &task.id)),
    );
    record(m, "diagnostic-input", &input, ui);
    m.diagnostic_input = Some((input.rect, input.id));
    name_control(&input, "Simulated getent command");
    // Single-line TextEdit surrenders focus on Enter in this egui version.
    let enter =
        (input.has_focus() || input.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let r = ui.add_enabled(active, button("Run command"));
    record(m, "diagnostic-run", &r, ui);
    workspace::bound(&mut draft.input, 512);
    let cache = ui.id().with(("names-diagnostic-result", &task.id));
    if active && (r.clicked() || enter) {
        let value = DiagnosticReport {
            files: scope.draft.clone(),
            command: draft.input.clone(),
            sample: selected,
            result: task.diagnose(&scope.draft, selected, &draft.input),
        };
        ui.ctx().data_mut(|d| d.insert_temp(cache, Arc::new(value)));
    }
    let report = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Arc<DiagnosticReport>>(cache));
    let Some(report) = report else {
        lesson_presentation::prose(ui, "No command run for this workspace.");
        return;
    };
    if report.files != scope.draft || report.command != draft.input || report.sample != selected {
        lesson_presentation::prose(
            ui,
            "Out of date · run the command again with the current files, command and sample.",
        );
        return;
    }
    match &report.result {
        Err(e) => {
            m.diagnostic_status = Some(Err(e.clone()));
            let r = lesson_presentation::prose(ui, e);
            m.diagnostic_output = Some(r.rect);
        }
        Ok(output) => {
            m.diagnostic_status = Some(Ok(output.status));
            let r = lesson_presentation::prose(
                ui,
                &format!(
                    "Actual command result · exit status {}{}",
                    output.status,
                    if output.status == 2 {
                        " · name not found"
                    } else {
                        ""
                    }
                ),
            );
            m.diagnostic_output = Some(r.rect);
            if output.stdout.is_empty() {
                lesson_presentation::prose(ui, "No standard output.");
            } else {
                lesson_presentation::code_value(
                    ui,
                    "names-getent-output",
                    &output.stdout,
                    ui.available_width(),
                );
            }
            if let Some(trace) = &output.resolution {
                let panel = controls::collapsing_header(
                    ui,
                    "Explain this lookup · simulator trace",
                )
                .show(ui, |ui| {
                    for step in &trace.steps {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "{} · {}; {}",
                                step.service.label(),
                                step.status.label(),
                                if step.action == dlms_learning_lab::client_names::Action::Return {
                                    "return: stop here"
                                } else {
                                    "continue: try next source"
                                }
                            ),
                        );
                    }
                    for attempt in &trace.dns_attempts {
                        lesson_presentation::monospace(
                            ui,
                            &format!(
                                "{} → {}",
                                attempt.name,
                                attempt
                                    .address
                                    .map(|a| a.to_string())
                                    .unwrap_or_else(|| "No matching A record".into())
                            ),
                            true,
                        );
                    }
                });
                record(m, "diagnostic-trace", &panel.header_response, ui);
            }
        }
    }
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn sample_choice(ui: &mut egui::Ui, task: &work::Task, selected: &mut usize, m: &mut Metrics) {
    lesson_presentation::rich(
        ui,
        RichText::new(if task.mode == work::Mode::Search {
            "Supplied DNS sample"
        } else {
            "Supplied name-source sample"
        })
        .strong(),
    );
    let r = controls::dropdown(ui, "names-input-case", ui.available_width())
        .selected_text(&task.cases[*selected].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let first = m.controls.len();
            for (n, c) in task.cases.iter().enumerate() {
                let r = ui.selectable_value(selected, n, &c.label);
                record(m, &format!("case:{n}"), &r, ui);
            }
            let clip = controls::popup_content_clip(ui);
            for c in &mut m.controls[first..] {
                c.2 = clip;
            }
        });
    record(m, "case", &r.response, ui);
}
fn facts(ui: &mut egui::Ui, task: &work::Task, selected: &mut usize, m: &mut Metrics) {
    sample_choice(ui, task, selected, m);
    facts_body(ui, task, *selected);
}
fn facts_body(ui: &mut egui::Ui, task: &work::Task, selected: usize) {
    let c = &task.cases[selected].context;
    if let Some(resolver) = &task.resolver {
        lesson_presentation::rich(
            ui,
            RichText::new("Supplied resolver search settings · read only").strong(),
        );
        lesson_presentation::code_value(ui, "names-fixed-resolver", resolver, ui.available_width());
    }
    if task.mode == work::Mode::Search {
        lesson_presentation::prose(
            ui,
            &format!(
                "Supplied hostname · {}",
                task.cases[selected].hostname.as_deref().unwrap()
            ),
        );
    } else {
        lesson_presentation::prose(
            ui,
            if c.files_available {
                "Local hosts source · Available"
            } else {
                "Local hosts source · Unavailable in this sample. The saved draft stays visible; this sample cannot consult it."
            },
        );
    }
    lesson_presentation::rich(
        ui,
        RichText::new(match c.dns.status {
            dlms_learning_lab::client_names::Status::Success => "Supplied DNS · Responding",
            dlms_learning_lab::client_names::Status::NotFound => "Supplied DNS · No matching names",
            dlms_learning_lab::client_names::Status::Unavailable => "Supplied DNS · Unavailable",
            dlms_learning_lab::client_names::Status::TryAgain => {
                "Supplied DNS · Temporarily unavailable"
            }
        })
        .strong(),
    );
    for (name, ip) in &c.dns.records {
        lesson_presentation::code_value(
            ui,
            ("names-dns-record", name),
            &format!("{name} → {ip}"),
            ui.available_width(),
        );
    }
    if c.dns.records.is_empty() {
        lesson_presentation::prose(ui, "No successful DNS answers supplied in this sample.");
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
    m.case_passes = rows.iter().map(work::CaseResult::passed).collect();
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
                "Name resolution meets the task"
            } else {
                "Name resolution needs work"
            },
            rows.len()
        ))
        .strong()
        .color(color),
    );
    m.result = Some((passed, rows.len(), r.rect));
    for row in rows {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if row.passed() {
                    "Requirements met"
                } else {
                    "Needs work"
                },
                row.label
            ))
            .strong(),
        );
        if let Some((_, why)) = row.checks.iter().find(|r| !r.0) {
            lesson_presentation::prose(ui, why);
        }
        controls::collapsing_header(ui, "Individual lookup requirements")
            .id_salt(("names-objectives", &row.id))
            .show(ui, |ui| {
                for (ok, why) in &row.checks {
                    lesson_presentation::prose(
                        ui,
                        &format!("{} · {why}", if *ok { "Met" } else { "Needs work" }),
                    );
                }
            });
    }
    let row = &rows[selected];
    if _task.mode == work::Mode::SourcesWithSearch {
        controls::collapsing_header(ui, "Actual batch traces · selected sample").show(ui, |ui| {
            for lookup in &row.lookups {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} → {}",
                        lookup.name,
                        lookup
                            .address
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "Unresolved".into())
                    ),
                );
                for step in &lookup.steps {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {} · {:?}",
                            step.service.label(),
                            step.status.label(),
                            step.action
                        ),
                    );
                }
            }
            for search in &row.searches {
                for attempt in &search.observed.attempts {
                    lesson_presentation::monospace(
                        ui,
                        &format!(
                            "{} → {}",
                            attempt.name,
                            attempt
                                .address
                                .map(|a| a.to_string())
                                .unwrap_or_else(|| "No matching A record".into())
                        ),
                        true,
                    );
                }
            }
        });
        return;
    }
    ui.separator();
    lesson_presentation::rich(
        ui,
        RichText::new(format!("Actual lookup traces · {}", row.label)).strong(),
    );
    if let Some(why) = &row.diagnostic {
        lesson_presentation::prose(ui, why);
        return;
    }
    if _task.mode == work::Mode::Search {
        for trace in &row.searches {
            lesson_presentation::rich(
                ui,
                RichText::new(format!("Query: {}", trace.query)).strong(),
            );
            for (index, attempt) in trace.observed.attempts.iter().enumerate() {
                let answer = attempt
                    .address
                    .map(|a| format!("Answer {a}; stop"))
                    .unwrap_or_else(|| "No matching A record".into());
                lesson_presentation::code_value(
                    ui,
                    ("search-attempt", &trace.query, index),
                    &format!("{}. {} · {answer}", index + 1, attempt.name),
                    ui.available_width(),
                );
            }
            lesson_presentation::prose(
                ui,
                &format!(
                    "Actual result: {}",
                    trace
                        .observed
                        .address
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "Unresolved after the supported candidates".into())
                ),
            );
            ui.add_space(6.);
        }
        return;
    }
    for lookup in &row.lookups {
        lesson_presentation::code_value(
            ui,
            ("names-actual-query", &lookup.name),
            &format!(
                "{} → {}",
                lookup.name,
                lookup
                    .address
                    .map(|ip| ip.to_string())
                    .unwrap_or_else(|| "Unresolved".into())
            ),
            ui.available_width(),
        );
        for (step_index, step) in lookup.steps.iter().enumerate() {
            use dlms_learning_lab::client_names::{Action, Status};
            let status = match step.status {
                Status::Success => "Answer found (SUCCESS)",
                Status::NotFound => "No matching name (NOTFOUND)",
                Status::Unavailable => "Source unavailable (UNAVAIL)",
                Status::TryAgain => "Try again later (TRYAGAIN)",
            };
            let action = match step.action {
                Action::Return => "return · stop here",
                Action::Continue if step_index + 1 < lookup.steps.len() => {
                    "continue · try next source"
                }
                Action::Continue => "continue · no further sources",
            };
            lesson_presentation::prose(
                ui,
                &format!("{} · {status}; {action}", step.service.label()),
            );
        }
        ui.add_space(6.);
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
        let mode = if practice {
            definition.example.mode
        } else {
            definition.initial.mode
        };
        lesson_presentation::rich(
            ui,
            RichText::new(if mode == work::Mode::Search {
                "Resolver search editor · virtual /etc/resolv.conf excerpt"
            } else {
                "File · virtual /etc editor"
            })
            .strong(),
        );
        if scope.draft.len() > 1 {
            let r = controls::dropdown(ui, "names-file", ui.available_width())
                .selected_text(&scope.selected)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let start = m.controls.len();
                    for name in scope.draft.keys() {
                        let r = ui.selectable_value(&mut scope.selected, name.clone(), name);
                        record(m, &format!("file:{name}"), &r, ui);
                    }
                    let clip = controls::popup_content_clip(ui);
                    for c in &mut m.controls[start..] {
                        c.2 = clip;
                    }
                });
            record(m, "file", &r.response, ui);
            name_control(&r.response, "Virtual name-source file");
            if scope.selected == "nsswitch.conf" {
                lesson_presentation::prose(
                    ui,
                    "Hosts database excerpt · other database settings are outside this activity.",
                );
            }
        } else {
            lesson_presentation::prose(
                ui,
                "Search/settings excerpt · supplied nameserver configuration is outside this editor.",
            );
        }
        if practice {
            let r = ui.add_enabled(active && !running, button("Insert worked configuration"));
            record(m, "insert", &r, ui);
            if r.clicked() {
                scope
                    .replace(&definition.example_worked)
                    .expect("validated example");
            }
        }
        let selected_file = scope.selected.clone();
        let mut draft = scope.draft[&selected_file].clone();
        let editor_height = ui.text_style_height(&egui::TextStyle::Monospace)
            * if mode == work::Mode::SourcesWithSearch {
                6.
            } else {
                10.
            }
            + 2. * f32::from(controls::PAD_Y);
        let mut visible_editor = egui::Rect::NOTHING;
        let combined = mode == work::Mode::SourcesWithSearch;
        let editor_width = (ui.available_width()
            - 2. * f32::from(controls::PAD_X)
            - ui.spacing().scroll.allocated_width())
        .max(24.);
        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let color = palette(ui).text;
        let mut rows = 0;
        let mut nowrap = |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
            let galley =
                ui.fonts_mut(|f| f.layout_no_wrap(text.as_str().into(), font.clone(), color));
            rows = galley.rows.len();
            galley
        };
        let area = controls::nested_scroll(
            ui,
            (if combined {
                egui::ScrollArea::both()
            } else {
                egui::ScrollArea::vertical()
            })
            .id_salt((
                "names-editor-scroll",
                &source.id,
                visited,
                practice,
                &selected_file,
            ))
            .max_height(editor_height)
            .min_scrolled_height(editor_height)
            .auto_shrink([false, false]),
            |ui| {
                visible_editor = ui.clip_rect();
                let mut edit = egui::TextEdit::multiline(&mut draft)
                    .code_editor()
                    .id(egui::Id::new((
                        "names-config-editor",
                        &source.id,
                        visited,
                        practice,
                        &selected_file,
                    )))
                    .desired_width(f32::INFINITY)
                    .desired_rows(if mode == work::Mode::SourcesWithSearch {
                        5
                    } else {
                        9
                    })
                    .char_limit(16384);
                if combined {
                    edit = edit
                        .layouter(&mut nowrap)
                        .desired_width(editor_width)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y));
                }
                ui.add_enabled(active && !running, edit)
            },
        );
        m.editor_scroll = Some((area.inner_rect.height(), area.state.offset.y));
        m.editor_rows = combined.then_some(rows);
        let r = area.inner;
        name_control(
            &r,
            if mode == work::Mode::Search {
                "Simulated resolver search configuration"
            } else {
                "Simulated name-source configuration"
            },
        );
        m.editor = Some((r.rect.intersect(visible_editor), r.id));
        let shortcut = r.has_focus()
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
        if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            r.surrender_focus();
        }
        if r.changed() {
            workspace::bound(&mut draft, 16384);
            scope.edit(&selected_file, &draft)?;
        }
        lesson_presentation::rich(
            ui,
            RichText::new("Tab indents · Ctrl+Enter runs · Esc releases focus").small(),
        );
        if practice {
            lesson_presentation::practice_step(ui, source, 1);
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(active && !running, primary(ui, "Run lookups"));
            record(m, "run", &r, ui);
            start = r.clicked() || shortcut;

            let r = ui.add_enabled(active && running, button("Stop"));
            record(m, "stop", &r, ui);
            stop = r.clicked();
        });
        if running {
            lesson_presentation::prose(ui, "Checking supplied name lookups…");
        } else if scope.tested.as_ref() != Some(&scope.draft) {
            lesson_presentation::prose(
                ui,
                "Not checked · run the current configuration to see actual lookup results.",
            );
        }
        Ok((start, stop))
    }
}

impl LabApp {
    pub(super) fn client_names_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.client_names_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            self.client_names_run = None;
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::ClientNames { definition }, engine::State::ClientNames { simulation }) =
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
        let title = lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        record(&mut self.metrics.client_names, "top", &title, ui);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    self.metrics.lesson_tabs.push(r.rect);
                    record(&mut self.metrics.client_names, key, &r, ui);
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
        if self.client_names_run.as_ref().is_some_and(|job| {
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
            self.client_names_run = None;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                if definition.initial.mode == work::Mode::SourcesWithSearch {
                    "Builds on local aliases, source order and DNS search. Investigate what the application actually receives before repairing the two virtual files."
                } else if definition.initial.mode == work::Mode::Search {
                    "Builds on local-name/source-order practice. This DNS search excerpt uses only the supplied hostname and DNS records."
                } else {
                    "Builds on the Linux host resolver lesson. These two virtual file excerpts and supplied DNS samples never touch real configuration."
                },
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                if definition.example.mode == work::Mode::SourcesWithSearch {
                    "Workshop: preview should follow DNS, while studio stays local. These worked files remove only the stale aliases and allow a local miss to reach DNS."
                } else if definition.example.mode == work::Mode::Search {
                    "Workshop example: compare the candidate names generated by this resolver excerpt with the supplied DNS records."
                } else {
                    "A separate workshop example: inspect each virtual file, then compare the local and supplied DNS answers."
                },
            );
            for (name, text) in &definition.example_worked {
                lesson_presentation::rich(ui, RichText::new(format!("/etc/{name}")).strong());
                lesson_presentation::code_value(
                    ui,
                    ("names-worked-file", &source.id, name),
                    text,
                    ui.available_width(),
                );
            }
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.client_names, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                if task.mode == work::Mode::SourcesWithSearch {
                    lesson_presentation::lead(
                        ui,
                        "PRACTICE TASK",
                        "Repair the workshop client's local aliases and DNS fallback.",
                    );
                    for clause in task.instructions.lines() {
                        lesson_presentation::prose(ui, &format!("• {clause}"));
                    }
                } else {
                    lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
                }
                if task.mode != work::Mode::SourcesWithSearch {
                    lesson_presentation::practice_step(ui, &source, 0);
                }
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let running = self.client_names_run.is_some();
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
            let selected_id = ui.id().with(("names-selected-case", practice));
            let mut selected = scope
                .diagnostic
                .as_ref()
                .map(|d| d.sample)
                .unwrap_or_else(|| {
                    ui.ctx()
                        .data_mut(|d| d.get_temp::<usize>(selected_id).unwrap_or(0))
                })
                .min(task.cases.len() - 1);
            let mut edit_error = None;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    match edit_context.editor(
                        &mut columns[0],
                        scope,
                        &mut self.metrics.client_names,
                    ) {
                        Ok((a, b)) => {
                            start = a;
                            stop = b;
                        }
                        Err(e) => edit_error = Some(e),
                    }
                    if task.mode == work::Mode::SourcesWithSearch {
                        sample_choice(
                            &mut columns[1],
                            task,
                            &mut selected,
                            &mut self.metrics.client_names,
                        );
                        if practice {
                            lesson_presentation::practice_step(&mut columns[1], &source, 0);
                        }
                        diagnostic(
                            &mut columns[1],
                            task,
                            scope,
                            selected,
                            active && !running,
                            &mut self.metrics.client_names,
                        );
                        facts_body(&mut columns[1], task, selected);
                        if practice {
                            lesson_presentation::practice_step(&mut columns[1], &source, 2);
                        }
                    } else {
                        facts(
                            &mut columns[1],
                            task,
                            &mut selected,
                            &mut self.metrics.client_names,
                        );
                    }
                });
            } else {
                if task.mode == work::Mode::SourcesWithSearch {
                    sample_choice(ui, task, &mut selected, &mut self.metrics.client_names);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 0);
                    }
                    diagnostic(
                        ui,
                        task,
                        scope,
                        selected,
                        active && !running,
                        &mut self.metrics.client_names,
                    );
                    facts_body(ui, task, selected);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    ui.separator();
                }
                match edit_context.editor(ui, scope, &mut self.metrics.client_names) {
                    Ok((a, b)) => {
                        start = a;
                        stop = b;
                    }
                    Err(e) => edit_error = Some(e),
                }
                ui.separator();
                if task.mode != work::Mode::SourcesWithSearch {
                    facts(ui, task, &mut selected, &mut self.metrics.client_names);
                }
            }
            ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
            if let Some(e) = edit_error {
                self.notice = Some(e);
            }
            self.metrics.check = self
                .metrics
                .client_names
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .map(|r| r.1);
            if stop {
                if let Some(mut job) = self.client_names_run.take() {
                    job.run.cancel();
                }
                self.notice =
                    Some("Stopped · draft retained; unfinished tests award no credit.".into());
            }
            if start && !running && active {
                match scope.begin(task) {
                    Ok(run) => {
                        self.client_names_run = Some(ActiveRun {
                            scenario: source.id.clone(),
                            practice,
                            draft: scope.draft.clone(),
                            run,
                        })
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            if active && let Some(mut job) = self.client_names_run.take() {
                if job.run.tick() {
                    if let Err(e) = scope.finish(task, &job.run) {
                        self.notice = Some(e);
                    }
                } else {
                    self.client_names_run = Some(job);
                }
                ui.ctx().request_repaint();
            }
            self.metrics.client_names.running = self.client_names_run.is_some();
            if scope.tested.as_ref() == Some(&scope.draft) {
                let cache_id = ui.id().with(("names-report", &source.id, practice));
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
                    &mut self.metrics.client_names,
                );
            }
            if practice {
                if task.mode != work::Mode::SourcesWithSearch {
                    lesson_presentation::practice_step(ui, &source, 2);
                }
                let r = ui.add_enabled(
                    active && !self.metrics.client_names.running,
                    button("Reset example only"),
                );
                record(&mut self.metrics.client_names, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("names-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, work::help(task.mode));
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.client_names, "hint", &r, ui);
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
                                        &answer
                                            .strip_prefix(work::PREFIX)
                                            .and_then(|s| {
                                                serde_json::from_str::<work::Files>(s).ok()
                                            })
                                            .map(|files| {
                                                files
                                                    .into_iter()
                                                    .map(|(name, text)| format!("{name}\n{text}"))
                                                    .collect::<Vec<_>>()
                                                    .join("\n")
                                            })
                                            .unwrap_or_else(|| answer.clone()),
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
                        engine::State::ClientNames {
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
