//! Native terminal/traffic-trial workspaces, no host network or shell.
use super::*;
use dlms_learning_lab::{
    linux_connections as ct, linux_flow_source as binding, linux_flow_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub focus: Vec<(String, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
        self.focus.push((key.into(), r.id));
    }
}
fn apply(
    scope: &mut work::Scope,
    task: &work::Task,
    action: &work::Action,
    notice: &mut Option<String>,
) {
    if let Err(e) = scope.apply(task, action) {
        *notice = Some(e);
    }
}
fn topology(ui: &mut egui::Ui, task: &work::Task) -> egui::Response {
    let lan = &task.facts.links[0];
    let wan = &task.facts.links[1];
    let a = format!("{}/{}", lan.address, lan.prefix);
    let b = format!("{}/{}", wan.address, wan.prefix);
    let left = format!("{} · {}", lan.name, if lan.up { "Up" } else { "Down" });
    let right = format!("{} · {}", wan.name, if wan.up { "Up" } else { "Down" });
    controls::topology_chain(
        ui,
        &[
            ["LAN interface", &a, &left],
            [
                "Linux router",
                "FORWARD and NAT",
                if task.forwarding {
                    "Forwarding on"
                } else {
                    "Forwarding off"
                },
            ],
            ["Public interface", &b, &right],
        ],
        "Supplied simulated router · routes and local settings stay fixed in this activity.",
    )
}
fn tuple(t: ct::Tuple) -> String {
    format!(
        "{}:{} -> {}:{}",
        t.source.address, t.source.port, t.destination.address, t.destination.port
    )
}
fn traffic(
    ui: &mut egui::Ui,
    task: &work::Task,
    scope: &mut work::Scope,
    source_id: &str,
    practice: bool,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Traffic tests · simulated sample flows");
    let mut index = scope.selected_case;
    controls::form_row(ui, "Traffic test", |ui, width| {
        let r = controls::dropdown(ui, "flow-case", width)
            .selected_text(&task.cases[index].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (n, c) in task.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut index, n, &c.label);
                    metrics.record(ui, &format!("case:{n}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        metrics.record(ui, "case", &r.response);
    });
    if index != scope.selected_case {
        apply(scope, task, &work::Action::SelectCase { index }, notice);
    }
    let case = &task.cases[scope.selected_case];
    let key = egui::Id::new(("flow-selected-event", source_id, practice, &case.id));
    let r = ui.add(button("Run traffic test"));
    metrics.record(ui, "run-traffic", &r);
    if r.clicked() {
        apply(scope, task, &work::Action::RunTraffic, notice);
        ui.ctx().data_mut(|d| d.remove::<usize>(key));
    }
    if let Some(trial) = &scope.last_trial {
        let passed = trial.steps.iter().filter(|s| s.check.passed).count();
        let r = ui.label(format!(
            "Actual traffic trial · {passed}/{} events meet the requirements",
            trial.steps.len()
        ));
        metrics.record(ui, "trial-result", &r);
        let fallback = trial
            .steps
            .iter()
            .position(|s| !s.check.passed)
            .unwrap_or(0);
        let mut event = ui
            .ctx()
            .data(|d| d.get_temp::<usize>(key))
            .unwrap_or(fallback)
            .min(trial.steps.len() - 1);
        controls::form_row(ui, "Inspect event", |ui, width| {
            let r = controls::dropdown(ui, "flow-event", width)
                .selected_text(format!(
                    "{} · {}",
                    case.events[event].label,
                    if trial.steps[event].check.passed {
                        "Passed"
                    } else {
                        "Needs work"
                    }
                ))
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    for (n, step) in trial.steps.iter().enumerate() {
                        let r = ui.selectable_value(
                            &mut event,
                            n,
                            format!(
                                "{} · {}",
                                case.events[n].label,
                                if step.check.passed {
                                    "Passed"
                                } else {
                                    "Needs work"
                                }
                            ),
                        );
                        metrics.record(ui, &format!("event:{n}"), &r);
                    }
                });
            controls::retain_dropdown_focus(ui, &r.response);
            metrics.record(ui, "event", &r.response);
        });
        ui.ctx().data_mut(|d| d.insert_temp(key, event));
        let step = &trial.steps[event];
        if let Some(trace) = &step.trace {
            for (label, value) in [
                ("Incoming", tuple(trace.incoming)),
                ("At filter", tuple(trace.filter)),
                (
                    "Outgoing",
                    trace.outgoing.map_or("Not forwarded".into(), tuple),
                ),
            ] {
                controls::form_row(ui, label, |ui, _| {
                    lesson_presentation::monospace(ui, &value, true);
                });
            }
            let state = trace.class.map_or("Not observed", |s| {
                if s == ct::Class::New {
                    "NEW"
                } else {
                    "ESTABLISHED"
                }
            });
            lesson_presentation::prose(
                ui,
                &format!(
                    "Connection: {state} · {}",
                    if trace.dnat == Some(true) {
                        "destination translated (DNAT)"
                    } else if trace.dnat == Some(false) {
                        "destination unchanged"
                    } else {
                        "translation not observed"
                    }
                ),
            );
        }
        let explanation = format!(
            "{} · {}",
            if step.check.passed {
                "Passed"
            } else {
                "Needs work"
            },
            step.check.detail
        );
        let r = lesson_presentation::prose(ui, &explanation);
        metrics.record(ui, "event-feedback", &r);
        controls::collapsing_header(ui, "All events and mappings").show(ui, |ui| {
            for step in &trial.steps {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {}",
                        if step.check.passed {
                            "Passed"
                        } else {
                            "Needs work"
                        },
                        step.check.label
                    ),
                );
            }
            for entry in &trial.mappings.entries {
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "Mapping {}\nOriginal: {}\nOutgoing: {}\nReply observed: {}",
                        entry.id,
                        tuple(entry.original),
                        tuple(entry.wire),
                        if entry.seen_reply { "Yes" } else { "No" }
                    ),
                    true,
                );
            }
        });
    }
}

fn show_worked(ui: &mut egui::Ui, source: &Scenario, definition: &work::Definition) {
    for action in &definition.worked {
        if let work::Action::Command { line } = action {
            lesson_presentation::monospace(ui, line, true);
            if let Some(command) = source
                .guide
                .iter()
                .flat_map(|g| &g.commands)
                .find(|c| c.command == *line)
            {
                lesson_presentation::prose(ui, &command.explanation);
            }
            ui.add_space(8.);
        }
    }
    lesson_presentation::prose(
        ui,
        "Apply these commands only in the separate example, run its traffic tests, then choose Check example. Assignment values and draft stay separate.",
    );
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut Metrics,
    notice: &mut Option<String>,
    phase: (bool, &mut bool),
) {
    let (practice, focus) = phase;
    ui.strong("Router terminal · simulated commands");
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("flow-command-choice", &source.id));
            let mut n = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "flow-command-choice", width)
                    .selected_text(&commands[n].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (at, c) in commands.iter().enumerate() {
                            ui.selectable_value(&mut n, at, &c.command);
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[n].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                apply(
                    s,
                    t,
                    &work::Action::Input {
                        text: commands[n].command.clone(),
                    },
                    notice,
                );
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, n));
        }
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace)
        * s.output.lines().count().clamp(4, 10) as f32;
    let output=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
  ui.strong("Actual command results");
  controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("flow-command-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{if s.output.is_empty(){lesson_presentation::prose(ui,"No command has run. Inspect the running rules, forwarding setting or routes to begin.");}else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}});
 });
    m.record(ui, "output-viewport", &output.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("flow-command-input", &source.id, practice)),
            2048,
            ui.is_enabled(),
            "Command · Enter to run",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(s, t, &work::Action::Input { text }, notice);
        }
        if *focus && ui.is_enabled() {
            r.response.request_focus();
            *focus = false;
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Keep one command within 2 KiB; previous input retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| field(ui, m),
            );
            let r = ui.add(button("Run command"));
            m.record(ui, "run", &r);
            r.clicked()
        })
        .inner
    } else {
        field(ui, m);
        let r = ui.add(button("Run command"));
        m.record(ui, "run", &r);
        r.clicked()
    };
    if run || clicked {
        let line = s.input.clone();
        apply(s, t, &work::Action::Command { line }, notice);
        *focus = true;
    }
}
impl LabApp {
    pub(super) fn flow_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (engine::Goal::LinuxFlow { definition }, engine::State::LinuxFlow { simulation }) =
            (&source.goal, &original.state)
        else {
            return;
        };
        let first = egui::Id::new(("flow-first", &source.id, original.visited));
        if !ui
            .ctx()
            .data(|d| d.get_temp::<bool>(first))
            .unwrap_or(false)
        {
            if let Err(e) = binding::context(original, &source) {
                lesson_presentation::prose(ui, &e);
                return;
            }
            self.focus_input = false;
            ui.ctx().data_mut(|d| d.insert_temp(first, true));
        }
        if let Err(e) = definition.bound(simulation) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let mut next = (**simulation).clone();
        let before = next.clone();
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
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.flow.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .flow
                .record(ui, "introduction", &intro.objective);
            lesson_presentation::worked(
                ui,
                "Separate worked flow · use Try example to investigate and change it.",
            );
            let worked = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.flow.record(ui, "worked", &worked.response);
            controls::collapsing_header(ui, "Complete worked command sequence · example only")
                .show(ui, |ui| {
                    show_worked(ui, &source, definition);
                });
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.flow.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics.flow.record(ui, "assignment", &r);
            }
            ui.add_enabled_ui(active, |ui| {
                let r = topology(ui, t);
                self.metrics.flow.record(ui, "topology", &r);
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54. {
                    ui.columns(2,|columns|{
                        console(&mut columns[0],t,s,&source,&mut self.metrics.flow,&mut self.notice,(practice,&mut self.focus_input));
                        if practice{lesson_presentation::practice_step(&mut columns[1],&source,1);}
                        traffic(&mut columns[1],t,s,&source.id,practice,&mut self.metrics.flow,&mut self.notice);
                    });
                }else{
                    console(ui,t,s,&source,&mut self.metrics.flow,&mut self.notice,(practice,&mut self.focus_input));
                    if practice{lesson_presentation::practice_step(ui,&source,1);}
                    traffic(ui,t,s,&source.id,practice,&mut self.metrics.flow,&mut self.notice);
                }
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                let r = ui.add(primary(ui, if practice { "Check example" } else { "Check work" }));
                self.metrics.flow.record(ui, "check", &r);
                if r.clicked() {
                    apply(s, t, &work::Action::Check, &mut self.notice);
                }
                if s.checked {
                    let checks = t.checks(s);
                    let passed = checks.iter().filter(|c| c.passed).count();
                    let r = ui.strong(if passed == checks.len() {
                        if practice { "Example configuration checks passed" } else { "Assignment complete" }.into()
                    } else {
                        format!("Not complete · {passed}/{} outcomes pass", checks.len())
                    });
                    self.metrics.flow.result = Some((passed, checks.len(), r.rect));
                    self.metrics.flow.record(ui, "result", &r);
                    for c in checks.iter().filter(|c| !c.passed) {
                        let r = lesson_presentation::prose(ui, &format!("Needs work · {}\n{}", c.label, c.detail));
                        self.metrics.flow.record(ui, "failed-case", &r);
                    }
                    controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
                        for c in checks.iter().filter(|c| c.passed) {
                            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
                        }
                    });
                    if passed == checks.len() {
                        lesson_presentation::prose(ui, if practice {
                            "The example now meets both the policy comparison and the supplied traffic outcomes. "
                        } else { &source.explanation });
                    }
                }
            });
            controls::collapsing_header(ui, "Optional help and supported commands").show(
                ui,
                |ui| {
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
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
                    } else {
                        controls::collapsing_header(ui, "Show worked answer · example only").show(
                            ui,
                            |ui| {
                                show_worked(ui, &source, definition);
                            },
                        );
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LinuxFlow {
                            simulation: Box::new(next),
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
