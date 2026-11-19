//! Native policy tables and actual packet replay; no host networking.
use super::*;
use dlms_learning_lab::{qos as core, qos_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn field(m: &mut Metrics, key: impl Into<String>, r: &egui::Response) {
    m.fields.push((key.into(), r.rect, r.id));
}
fn marking(ui: &mut egui::Ui, task: &work::Task, p: &mut core::Policy, m: &mut Metrics) {
    ui.strong("Marking at this boundary");
    for stream in &task.streams {
        ui.push_id(&stream.id, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.strong(&stream.label);
                ui.monospace(&stream.id);
            });
            lesson_presentation::prose(ui, &stream.description);
            let mut rewrite = matches!(p.marks[&stream.id], core::Mark::Set(_));
            ui.horizontal_wrapped(|ui| {
                let width =
                    (ui.text_style_height(&egui::TextStyle::Body) * 9.).min(ui.available_width());
                let r = controls::dropdown(ui, "mark-mode", width)
                    .selected_text(if rewrite {
                        "Rewrite DSCP"
                    } else {
                        "Keep incoming"
                    })
                    .show_ui(ui, |ui| {
                        for (v, label) in [(false, "Keep incoming"), (true, "Rewrite DSCP")] {
                            let r = ui.selectable_value(&mut rewrite, v, label);
                            record(
                                m,
                                format!(
                                    "mark:{}:{}",
                                    stream.id,
                                    if v { "rewrite" } else { "keep" }
                                ),
                                &r,
                                ui,
                            );
                        }
                    });
                record(m, format!("mark:{}:mode", stream.id), &r.response, ui);
                let previous = p.marks[&stream.id];
                let mut code = if let core::Mark::Set(n) = previous {
                    n
                } else {
                    0
                };
                if rewrite {
                    ui.label("DSCP");
                    let r = ui.add(egui::DragValue::new(&mut code).range(0..=63).speed(1.));
                    field(m, format!("mark:{}:code", stream.id), &r);
                } else {
                    ui.label("Preserve each packet’s label");
                }
                p.marks.insert(
                    stream.id.clone(),
                    if rewrite {
                        core::Mark::Set(code)
                    } else {
                        core::Mark::Keep
                    },
                );
            });
            ui.add_space(6.);
        });
    }
}
fn queues(ui: &mut egui::Ui, task: &work::Task, p: &mut core::Policy, m: &mut Metrics) {
    ui.separator();
    ui.strong("Waiting queues");
    controls::form_row(ui, "Scheduling", |ui, width| {
        let previous = p.scheduler;
        let r = controls::dropdown(ui, "qos-scheduler", width)
            .selected_text(match p.scheduler {
                core::Scheduler::Fifo => "FIFO · one shared queue",
                core::Scheduler::StrictPriority => "Strict priority · two queues",
            })
            .show_ui(ui, |ui| {
                for (v, label, key) in [
                    (core::Scheduler::Fifo, "FIFO · one shared queue", "fifo"),
                    (
                        core::Scheduler::StrictPriority,
                        "Strict priority · two queues",
                        "priority",
                    ),
                ] {
                    let r = ui.selectable_value(&mut p.scheduler, v, label);
                    record(m, key, &r, ui);
                }
            });
        record(m, "scheduler", &r.response, ui);
        if p.scheduler == core::Scheduler::Fifo && previous != p.scheduler {
            p.priority_dscp.clear();
            p.high_buffer = 0;
        }
    });
    if p.scheduler == core::Scheduler::StrictPriority {
        ui.label("High queue · explicit DSCP mapping");
        let mut edited = p.priority_dscp.iter().copied().collect::<Vec<_>>();
        let mut remove = None;
        let mut add = false;
        let count = edited.len();
        for (index, code) in edited.iter_mut().enumerate() {
            ui.push_id(("high-code", index), |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label("DSCP");
                    let r = ui.add(egui::DragValue::new(code).range(0..=63).speed(1.));
                    field(m, format!("high-code:{index}"), &r);
                    let r = ui.add(button("Remove label"));
                    record(m, format!("remove-high:{index}"), &r, ui);
                    if r.clicked() {
                        remove = Some(index);
                    }
                    if index + 1 == count {
                        let r = ui.add_enabled(count < 4, button("Add label"));
                        record(m, "add-high", &r, ui);
                        add |= r.clicked();
                    }
                });
            });
        }
        if count == 0 {
            ui.horizontal(|ui| {
                let r = ui.add(button("Add label"));
                record(m, "add-high", &r, ui);
                add |= r.clicked();
            });
        }
        if let Some(index) = remove {
            edited.remove(index);
        }
        p.priority_dscp = edited.into_iter().collect();
        if add && let Some(code) = (0..=63).find(|n| !p.priority_dscp.contains(n)) {
            p.priority_dscp.insert(code);
        }
        controls::form_row(ui, "High queue places", |ui, _| {
            let r = ui.add(
                egui::DragValue::new(&mut p.high_buffer)
                    .range(0..=core::MAX_BUFFER)
                    .speed(1.),
            );
            field(m, "high-places", &r);
        });
    }
    controls::form_row(
        ui,
        if p.scheduler == core::Scheduler::Fifo {
            "Shared queue places"
        } else {
            "Standard queue places"
        },
        |ui, _| {
            let r = ui.add(
                egui::DragValue::new(&mut p.standard_buffer)
                    .range(0..=core::MAX_BUFFER)
                    .speed(1.),
            );
            field(m, "standard-places", &r);
        },
    );
    let budget = task.workloads[0].fixture.buffer_budget;
    lesson_presentation::prose(
        ui,
        &format!(
            "Available: {budget} waiting places in total. Arrivals are admitted before that slot’s transmission selection."
        ),
    );
}
fn facts_and_results(
    ui: &mut egui::Ui,
    task: &work::Task,
    scope: &work::Scope,
    m: &mut Metrics,
    selected: &mut usize,
    ran: bool,
    practice: bool,
) -> Option<Vec<work::CaseResult>> {
    ui.strong("Inspect a packet pattern");
    let r = controls::dropdown(ui, "qos-workload", ui.available_width())
        .selected_text(&task.workloads[*selected].label)
        .show_ui(ui, |ui| {
            for (index, w) in task.workloads.iter().enumerate() {
                let r = ui.selectable_value(selected, index, &w.label);
                record(m, format!("workload:{index}"), &r, ui);
            }
        });
    record(m, "workload", &r.response, ui);
    lesson_presentation::prose(
        ui,
        "Supplied inputs · equal-size packets; one transmission per virtual slot.",
    );
    controls::nested_scroll(
        ui,
        egui::ScrollArea::horizontal()
            .id_salt("qos-input-x")
            .auto_shrink([false, true]),
        |ui| {
            egui::Grid::new("qos-input").striped(true).show(ui, |ui| {
                for t in ["Stream", "Arrival slot", "Incoming DSCP"] {
                    ui.strong(t);
                }
                ui.end_row();
                for p in &task.workloads[*selected].fixture.packets {
                    ui.monospace(&p.flow);
                    ui.monospace(p.arrival.to_string());
                    ui.monospace(p.dscp.to_string());
                    ui.end_row();
                }
            });
        },
    );
    if let Some(e) = &scope.diagnostic {
        let r = ui.colored_label(palette(ui).error, format!("Configuration problem · {e}"));
        m.result = Some((0, task.workloads.len(), r.rect));
        if ran {
            r.scroll_to_me(Some(egui::Align::Center));
        }
        return None;
    }
    if scope.tested.as_ref() != Some(&scope.draft) {
        let r = ui.label("Not checked · run the current policy to see actual results.");
        m.result = Some((0, task.workloads.len(), r.rect));
        return None;
    }
    let results = match task.run(&scope.draft) {
        Ok(r) => r,
        Err(e) => {
            ui.colored_label(palette(ui).error, e);
            return None;
        }
    };
    let passed = results.iter().filter(|r| r.passed()).count();
    ui.separator();
    let r = ui.colored_label(
        if passed == results.len() {
            palette(ui).success
        } else {
            palette(ui).error
        },
        format!(
            "Overall: {} · {passed}/{} packet patterns meet every outcome",
            if passed == results.len() {
                if practice {
                    "Example passed"
                } else {
                    "Objective complete"
                }
            } else {
                "Needs work"
            },
            results.len()
        ),
    );
    m.result = Some((passed, results.len(), r.rect));
    if ran {
        r.scroll_to_me(Some(egui::Align::Center));
    }
    for (index, result) in results.iter().enumerate() {
        ui.strong(format!(
            "{} · {}",
            if result.passed() {
                "Passed"
            } else {
                "Needs work"
            },
            result.label
        ));
        if index == *selected {
            for (_, text) in result.checks.iter().filter(|r| !r.0).take(2) {
                lesson_presentation::prose(ui, text);
            }
        }
        controls::collapsing_header(ui, &format!("Objective details · {}", result.label))
            .id_salt(("qos-case-goals", index))
            .show(ui, |ui| {
                for (ok, text) in &result.checks {
                    lesson_presentation::prose(
                        ui,
                        &format!("{} · {text}", if *ok { "Pass" } else { "Needs work" }),
                    );
                }
            });
    }
    if results[*selected].passed()
        && let Some(failed) = results.iter().find(|r| !r.passed())
    {
        ui.strong(format!("First failing packet pattern · {}", failed.label));
        for (_, text) in failed.checks.iter().filter(|r| !r.0).take(2) {
            lesson_presentation::prose(ui, text);
        }
    }
    Some(results)
}

fn replay_table(ui: &mut egui::Ui, task: &work::Task, result: &work::CaseResult) {
    ui.strong(format!("Actual packet replay · {}", result.label));
    let width = (ui.available_width() - 3. * ui.spacing().item_spacing.x).max(1.);
    let widths = [width * 0.30, width * 0.18, width * 0.16, width * 0.36];
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt("qos-actual-table")
            .max_height(260.)
            .auto_shrink([false, true]),
        |ui| {
            egui::Grid::new("qos-actual-grid")
                .striped(true)
                .min_col_width(0.)
                .show(ui, |ui| {
                    for (i, t) in [
                        "Packet / stream",
                        "DSCP in / out",
                        "Queue",
                        "Result (slots)",
                    ]
                    .iter()
                    .enumerate()
                    {
                        controls::table_text(ui, RichText::new(*t).strong(), widths[i]);
                    }
                    ui.end_row();
                    for p in &result.trace.packets {
                        let label = &task
                            .streams
                            .iter()
                            .find(|s| s.id == p.flow)
                            .expect("validated stream identity")
                            .label;
                        controls::table_text(
                            ui,
                            RichText::new(format!("{}\n{label}", p.id)).monospace(),
                            widths[0],
                        );
                        controls::table_text(
                            ui,
                            RichText::new(format!("{} / {}", p.incoming_dscp, p.outgoing_dscp))
                                .monospace(),
                            widths[1],
                        );
                        controls::table_text(
                            ui,
                            match p.queue {
                                core::Queue::Shared => "Shared",
                                core::Queue::High => "High",
                                core::Queue::Standard => "Standard",
                            },
                            widths[2],
                        );
                        controls::table_text(
                            ui,
                            match p.delivery {
                                core::Delivery::Delivered {
                                    departure,
                                    waiting_slots,
                                } => format!("Delivered\nOut {departure} / wait {waiting_slots}"),
                                core::Delivery::TailDrop { at } => {
                                    format!("Dropped at {at}\nQueue full")
                                }
                            },
                            widths[3],
                        );
                        ui.end_row();
                    }
                });
        },
    );
}
fn panel(
    ui: &mut egui::Ui,
    task: &work::Task,
    scope: &mut work::Scope,
    m: &mut Metrics,
    practice: bool,
) {
    let selected_id = ui.id().with("qos-selected-workload");
    let mut selected = ui
        .ctx()
        .data_mut(|d| d.get_temp::<usize>(selected_id).unwrap_or(0))
        .min(task.workloads.len() - 1);
    let mut edited = scope.draft.clone();
    let before = edited.clone();
    let queue_task = task.requirements.iter().any(|r| {
        matches!(
            r,
            work::Requirement::MaxWait { .. } | work::Requirement::AvoidPriority { .. }
        )
    });
    let mut config = |ui: &mut egui::Ui, scope: &mut work::Scope, m: &mut Metrics| {
        marking(ui, task, &mut edited, m);
        if queue_task {
            queues(ui, task, &mut edited, m);
        }
        if edited != before {
            scope
                .replace(task, edited.clone())
                .expect("bounded native edits preserve fixed stream identity");
        }
        let r = ui.add(primary(
            ui,
            if practice {
                "Run example"
            } else {
                "Check policy"
            },
        ));
        record(m, "run", &r, ui);
        if r.clicked() {
            let _ = scope.run(task);
        }
        r.clicked()
    };
    let mut results = None;
    if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 48. {
        ui.columns(2, |cols| {
            let ran = config(&mut cols[0], scope, m);
            results = facts_and_results(&mut cols[1], task, scope, m, &mut selected, ran, practice);
        });
    } else {
        let ran = config(ui, scope, m);
        ui.separator();
        results = facts_and_results(ui, task, scope, m, &mut selected, ran, practice);
    }
    if let Some(results) = results {
        replay_table(ui, task, &results[selected]);
    }
    ui.ctx().data_mut(|d| d.insert_temp(selected_id, selected));
}
impl LabApp {
    pub(super) fn qos_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            ui.label(e);
            return;
        }
        let (engine::Goal::Qos { definition }, engine::State::Qos { simulation }) =
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
                    record(&mut self.metrics.qos, key, &r, ui);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            candidate.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on forwarding IPv4 traffic. These concepts introduce labels, waiting and the specific policy before the assignment.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.task);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(
                        active,
                        if v == teaching::View::Example {
                            primary(ui, label)
                        } else {
                            button(label)
                        },
                    );
                    record(&mut self.metrics.qos, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            let task = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.task);
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            let budget = format!("{} places", task.workloads[0].fixture.buffer_budget);
            controls::topology_chain(
                ui,
                &[
                    [
                        "Supplied streams",
                        "Equal-size packets",
                        "Named-flow classification",
                    ],
                    ["Mark and queue", "Finite waiting space", budget.as_str()],
                    ["One shared link", "One packet per slot", "No real traffic"],
                ],
                "Classify; mark; queue; then transmit or drop",
            );
            if practice {
                let r = ui.add_enabled(active, button("Fill worked policy"));
                record(&mut self.metrics.qos, "insert", &r, ui);
                if r.clicked() {
                    scope
                        .replace(task, definition.example_worked.clone())
                        .expect("validated separate example");
                }
                lesson_presentation::practice_step(ui, &source, 1);
            }
            ui.add_enabled_ui(active, |ui| {
                ui.push_id((&source.id, practice), |ui| {
                    panel(ui, task, scope, &mut self.metrics.qos, practice)
                })
            });
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.qos, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Session::new(definition).unwrap().practice;
                }
            }
            ui.push_id(("qos-help", &source.id, visited, challenge), |ui| {
                controls::collapsing_header(ui, "Help and model scope").show(ui, |ui| {
                    lesson_presentation::prose(ui, core::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.qos, "hint", &r, ui);
                        if r.clicked() {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for h in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, h);
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
                        engine::State::Qos {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.check = self
            .metrics
            .qos
            .controls
            .iter()
            .find(|r| r.0 == "run")
            .map(|r| r.1);
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
