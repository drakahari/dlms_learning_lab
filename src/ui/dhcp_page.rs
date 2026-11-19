//! Native DHCP configuration editing over bounded virtual clients.
use super::*;
use dlms_learning_lab::dhcp_server_workspace as work;
use std::sync::Arc;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub running: bool,
    pub editor_scroll: Option<(f32, f32)>,
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
fn readable(text: &str, task: &work::Task) -> String {
    let mut out = text.to_owned();
    for (role, label) in [
        ("management", "Management"),
        ("new-a", "New client A"),
        ("new-b", "New client B"),
        ("remote", "New beta client"),
        ("untouched", "Untouched beta client"),
    ] {
        out = out.replace(&format!("{}-{role}", task.id), label);
    }
    out
}
fn facts(ui: &mut egui::Ui, task: &work::Task, selected: &mut usize, m: &mut Metrics) {
    ui.strong("Supplied inputs · before each test");
    let r = controls::dropdown(ui, "dhcp-input-case", ui.available_width())
        .selected_text(&task.cases[*selected].label)
        .show_ui(ui, |ui| {
            for (index, case) in task.cases.iter().enumerate() {
                let r = ui.selectable_value(selected, index, &case.label);
                record(m, &format!("case:{index}"), &r, ui);
            }
        });
    record(m, "case", &r.response, ui);
    let case = &task.cases[*selected];
    for (name, segment) in &case.initial.topology().segments {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{name} · {}/{}",
                segment.subnet.network, segment.subnet.prefix
            ))
            .strong(),
        );
        let clients = case
            .actions
            .iter()
            .filter_map(|op| match op {
                work::Operation::Acquire { client }
                    if case.initial.topology().clients[client].segment == *name =>
                {
                    Some(readable(client, task))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        lesson_presentation::prose(ui, &format!("New requests: {}", clients.join(", ")));
        let occupied = segment
            .occupied
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        lesson_presentation::prose(
            ui,
            &format!(
                "Already occupied: {}",
                if occupied.is_empty() {
                    "none supplied"
                } else {
                    &occupied
                }
            ),
        );
    }
    for (client, lease) in case.initial.leases() {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{}: held {} · DNS {:?} · granted {}s",
                readable(client, task),
                lease.address,
                lease.parameters.dns,
                lease.granted_seconds
            ),
            true,
        );
    }
    controls::collapsing_header(ui, "Client actions in this test")
        .id_salt("dhcp-actions")
        .show(ui, |ui| {
            for action in &case.actions {
                let text = match action {
                    work::Operation::Acquire { client } => {
                        format!("Request configuration: {client}")
                    }
                    work::Operation::Renew { client } => format!("Renew held lease: {client}"),
                    work::Operation::Advance { seconds } => {
                        format!("Advance virtual clock by {seconds}s")
                    }
                };
                lesson_presentation::prose(ui, &readable(&text, task));
            }
        });
}
fn report(
    ui: &mut egui::Ui,
    rows: &[work::CaseResult],
    task: &work::Task,
    selected: usize,
    m: &mut Metrics,
) {
    let passed = rows.iter().filter(|r| r.passed()).count();
    let r = ui.colored_label(
        if passed == rows.len() {
            palette(ui).success
        } else {
            palette(ui).error
        },
        format!(
            "{} · {passed}/{} client cases pass",
            if passed == rows.len() {
                "Configuration meets every requirement"
            } else {
                "Configuration needs work"
            },
            rows.len()
        ),
    );
    m.result = Some((passed, rows.len(), r.rect));
    for row in rows {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if row.passed() { "Passed" } else { "Needs work" },
                row.label
            ))
            .strong(),
        );
        for (_, text) in row.checks.iter().filter(|r| !r.0).take(1) {
            lesson_presentation::prose(ui, &readable(text, task));
        }
        controls::collapsing_header(ui, "Objective results")
            .id_salt(("dhcp-objectives", &row.id))
            .show(ui, |ui| {
                for (ok, text) in &row.checks {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {}",
                            if *ok { "Pass" } else { "Needs work" },
                            readable(text, task)
                        ),
                    );
                }
            });
    }
    let case = &rows[selected];
    if let Some(state) = &case.after {
        ui.separator();
        ui.strong(format!("Actual client results · {}", case.label));
        for (client, b) in state.leases() {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "{}: {}\nRouter {} · DNS {:?}\nGranted {}s · {}s remaining at virtual {}s",
                    readable(client, task),
                    b.address,
                    b.parameters
                        .router
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| "not supplied".into()),
                    b.parameters.dns,
                    b.granted_seconds,
                    b.remaining(state.now()),
                    state.now()
                ),
                true,
            );
        }
    }
    controls::collapsing_header(ui, "Actual exchange trace")
        .id_salt("dhcp-trace")
        .show(ui, |ui| {
            for event in &case.events {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {}: {}",
                        if event.ok {
                            "Completed"
                        } else {
                            "Did not complete"
                        },
                        readable(&event.action, task),
                        readable(&event.message, task)
                    ),
                );
            }
        });
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
        ui.strong("Configuration editor · simulated /etc/dhcp/dhcpd.conf");
        if practice {
            let r = ui.add_enabled(active && !running, button("Insert worked configuration"));
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
                .id_salt(("dhcp-editor-scroll", &source.id, visited, practice))
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
                            "dhcp-config-editor",
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
        name_control(&r, "Simulated DHCP server configuration");
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
        ui.label(RichText::new("Tab indents · Ctrl+Enter runs · Esc releases focus").small());
        if practice {
            lesson_presentation::practice_step(ui, source, 1);
        }
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(active && !running, primary(ui, "Run client tests"));
            record(m, "run", &r, ui);
            start = r.clicked() || shortcut;

            let r = ui.add_enabled(active && running, button("Stop"));
            record(m, "stop", &r, ui);
            stop = r.clicked();
        });
        if running {
            ui.label("Checking supplied client cases…");
        } else if scope.tested.as_deref() != Some(scope.draft.as_str()) {
            ui.label("Not checked · run the current configuration for actual results.");
        }
        Ok((start, stop))
    }
}

impl LabApp {
    pub(super) fn dhcp_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            self.dhcp_config_run = None;
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            self.dhcp_config_run = None;
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::DhcpConfig { definition }, engine::State::DhcpConfig { simulation }) =
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
                    record(&mut self.metrics.dhcp_config, key, &r, ui);
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
        if self.dhcp_config_run.as_ref().is_some_and(|job| {
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
            self.dhcp_config_run = None;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on IPv4 host interfaces, routes and resolver inputs. This legacy configuration-format exercise uses virtual clients only.",
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
                    record(&mut self.metrics.dhcp_config, key, &r, ui);
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
            let running = self.dhcp_config_run.is_some();
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
            let selected_id = ui.id().with(("dhcp-selected-case", practice));
            let mut selected = ui
                .ctx()
                .data_mut(|d| d.get_temp::<usize>(selected_id).unwrap_or(0))
                .min(task.cases.len() - 1);
            let mut edit_error = None;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 50. {
                ui.columns(2, |columns| {
                    match edit_context.editor(&mut columns[0], scope, &mut self.metrics.dhcp_config)
                    {
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
                        &mut self.metrics.dhcp_config,
                    );
                });
            } else {
                match edit_context.editor(ui, scope, &mut self.metrics.dhcp_config) {
                    Ok((a, b)) => {
                        start = a;
                        stop = b;
                    }
                    Err(e) => edit_error = Some(e),
                }
                ui.separator();
                facts(ui, task, &mut selected, &mut self.metrics.dhcp_config);
            }
            ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
            if let Some(e) = edit_error {
                self.notice = Some(e);
            }
            self.metrics.check = self
                .metrics
                .dhcp_config
                .controls
                .iter()
                .find(|r| r.0 == "run")
                .map(|r| r.1);
            if stop {
                if let Some(mut job) = self.dhcp_config_run.take() {
                    job.run.cancel();
                }
                self.notice =
                    Some("Stopped · draft retained; unfinished tests award no credit.".into());
            }
            if start && !running && active {
                match scope.begin(task) {
                    Ok(run) => {
                        self.dhcp_config_run = Some(ActiveRun {
                            scenario: source.id.clone(),
                            practice,
                            draft: scope.draft.clone(),
                            run,
                        })
                    }
                    Err(e) => self.notice = Some(e),
                }
            }
            if active && let Some(mut job) = self.dhcp_config_run.take() {
                if job.run.tick() {
                    if let Err(e) = scope.finish(task, &job.run) {
                        self.notice = Some(e);
                    }
                } else {
                    self.dhcp_config_run = Some(job);
                }
                ui.ctx().request_repaint();
            }
            self.metrics.dhcp_config.running = self.dhcp_config_run.is_some();
            if scope.tested.as_deref() == Some(scope.draft.as_str()) {
                let cache_id = ui.id().with(("dhcp-report", &source.id, practice));
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
                    &mut self.metrics.dhcp_config,
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(
                    active && !self.metrics.dhcp_config.running,
                    button("Reset example only"),
                );
                record(&mut self.metrics.dhcp_config, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("dhcp-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.dhcp_config, "hint", &r, ui);
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
                        engine::State::DhcpConfig {
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
