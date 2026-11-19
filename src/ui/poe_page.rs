//! Native power allocation, not a quiz or equipment command shell.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    poe::{self, Class, Source},
    poe_source as binding, poe_task as task, poe_workspace as work,
};
fn name(id: &str) -> String {
    id.split('-')
        .enumerate()
        .map(|(i, w)| {
            if w.eq_ignore_ascii_case("ap") {
                "AP".into()
            } else if i == 0 {
                let mut s = w.to_owned();
                if let Some(c) = s.get_mut(..1) {
                    c.make_ascii_uppercase()
                }
                s
            } else {
                w.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn class(c: Class) -> &'static str {
    match c {
        Class::Unknown0 => "Class 0",
        Class::One => "Class 1",
        Class::Two => "Class 2",
        Class::Three => "Class 3",
        Class::Four => "Class 4",
    }
}
fn source(s: Source) -> &'static str {
    match s {
        Source::DataOnly => "Data only",
        Source::Ieee8023af => "PoE",
        Source::Ieee8023at => "PoE+",
    }
}
fn state(s: poe::State) -> &'static str {
    match s {
        poe::State::Disconnected => "Disconnected",
        poe::State::Disabled => "Power disabled",
        poe::State::DataOnly => "Data-only port",
        poe::State::Incompatible => "Source too small",
        poe::State::BudgetDenied => "Budget denied",
        poe::State::Powered => "Powered",
    }
}
fn cell<R>(
    ui: &mut egui::Ui,
    width: f32,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    ui.allocate_ui_with_layout(
        egui::vec2(width, controls::height(ui)),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            body(ui)
        },
    )
}
fn port_choice(
    ui: &mut egui::Ui,
    t: &task::Task,
    p: &mut poe::Plan,
    device: usize,
    width: f32,
    m: &mut Metrics,
) {
    let text = p.attachments[device]
        .map(|i| {
            format!(
                "{} · {}",
                t.facts.ports[i].id,
                source(t.facts.ports[i].source)
            )
        })
        .unwrap_or_else(|| "Disconnected".into());
    let r = controls::dropdown(ui, ("poe-port", &t.id, device), width)
        .selected_text(text)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            let r = ui.selectable_value(&mut p.attachments[device], None, "Disconnected");
            m.record(ui, &format!("port-option:{device}:none"), &r);
            for (i, port) in t.facts.ports.iter().enumerate() {
                let occupied = p
                    .attachments
                    .iter()
                    .enumerate()
                    .any(|(other, at)| other != device && *at == Some(i));
                ui.add_enabled_ui(!occupied, |ui| {
                    let r = ui.selectable_value(
                        &mut p.attachments[device],
                        Some(i),
                        format!(
                            "{} · {}{}",
                            port.id,
                            source(port.source),
                            if occupied { " · in use" } else { "" }
                        ),
                    );
                    m.record(ui, &format!("port-option:{device}:{i}"), &r);
                });
            }
        });
    controls::retain_dropdown_focus(ui, &r.response);
    m.record(ui, &format!("port-choice:{device}"), &r.response);
}
fn connections(ui: &mut egui::Ui, t: &task::Task, p: &mut poe::Plan, m: &mut Metrics) {
    ui.strong("Device connections");
    lesson_presentation::prose(
        ui,
        "Each port accepts one device. Choose Disconnected before reusing an occupied port.",
    );
    let report = t.facts.observe(p).expect("source-bound power plan");
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 55.;
    if wide {
        let available = (ui.available_width() - controls::GAP * 3.).max(1.);
        let widths = [
            available * 0.22,
            available * 0.19,
            available * 0.36,
            available * 0.23,
        ];
        egui::Grid::new(("poe-devices", &t.id))
            .num_columns(4)
            .spacing(egui::vec2(controls::GAP, controls::GAP))
            .show(ui, |ui| {
                for (i, text) in [
                    "Device",
                    "Source reservation",
                    "Connect to",
                    "Current power",
                ]
                .iter()
                .enumerate()
                {
                    cell(ui, widths[i], |ui| {
                        ui.strong(*text);
                    });
                }
                ui.end_row();
                for (i, d) in t.facts.devices.iter().enumerate() {
                    cell(ui, widths[0], |ui| {
                        ui.add(egui::Label::new(name(&d.id)).wrap());
                    });
                    cell(ui, widths[1], |ui| {
                        ui.add(
                            egui::Label::new(format!(
                                "{} · {}",
                                class(d.class),
                                task::watts(d.class.reservation_mw())
                            ))
                            .wrap(),
                        );
                    });
                    cell(ui, widths[2], |ui| port_choice(ui, t, p, i, widths[2], m));
                    cell(ui, widths[3], |ui| {
                        let r = ui.add(egui::Label::new(state(report.devices[i].state)).wrap());
                        m.record(ui, &format!("device-status:{i}"), &r);
                    });
                    ui.end_row();
                }
            });
    } else {
        for (i, d) in t.facts.devices.iter().enumerate() {
            ui.add_space(controls::GAP);
            ui.strong(name(&d.id));
            lesson_presentation::prose(
                ui,
                &format!(
                    "{} · {} source reservation",
                    class(d.class),
                    task::watts(d.class.reservation_mw())
                ),
            );
            controls::form_row(ui, "Connect to", |ui, width| {
                port_choice(ui, t, p, i, width, m)
            });
            let r = lesson_presentation::prose(
                ui,
                &format!("Current power: {}", state(report.devices[i].state)),
            );
            m.record(ui, &format!("device-status:{i}"), &r);
        }
    }
}
fn power_controls(ui: &mut egui::Ui, t: &task::Task, p: &mut poe::Plan, m: &mut Metrics) {
    ui.strong("Switch port power");
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 44.;
    let mut control = |ui: &mut egui::Ui, i: usize| {
        if t.facts.ports[i].source == Source::DataOnly {
            ui.label("No PoE hardware");
        } else {
            let r = ui.checkbox(&mut p.enabled[i], "PoE enabled");
            m.record(ui, &format!("enabled:{i}"), &r);
        }
    };
    if wide {
        let available = (ui.available_width() - controls::GAP * 2.).max(1.);
        let widths = [available * 0.25, available * 0.4, available * 0.35];
        egui::Grid::new(("poe-source-ports", &t.id))
            .num_columns(3)
            .spacing(egui::vec2(controls::GAP, controls::GAP))
            .show(ui, |ui| {
                for (i, label) in ["Port", "Source maximum", "Power control"]
                    .iter()
                    .enumerate()
                {
                    cell(ui, widths[i], |ui| {
                        ui.strong(*label);
                    });
                }
                ui.end_row();
                for (i, port) in t.facts.ports.iter().enumerate() {
                    cell(ui, widths[0], |ui| {
                        ui.monospace(&port.id);
                    });
                    cell(ui, widths[1], |ui| {
                        ui.label(format!(
                            "{} · {}",
                            source(port.source),
                            task::watts(port.source.capacity_mw())
                        ));
                    });
                    cell(ui, widths[2], |ui| control(ui, i));
                    ui.end_row();
                }
            });
    } else {
        for (i, port) in t.facts.ports.iter().enumerate() {
            ui.add_space(controls::GAP);
            ui.monospace(format!(
                "{} · {} · {} maximum",
                port.id,
                source(port.source),
                task::watts(port.source.capacity_mw())
            ));
            control(ui, i);
        }
    }
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &task::Task, practice: bool, m: &mut Metrics) {
    if !s.checked {
        let r = ui.label("Power plan unchecked. Check after your final change.");
        m.record(ui, "unchecked", &r);
        return;
    }
    let checks = s.checks(t);
    let passed = checks.iter().filter(|r| r.passed).count();
    let r = ui.strong(if passed == checks.len() {
        if practice {
            "Example checks passed".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs work · {passed}/{} device outcomes pass",
            checks.len()
        )
    });
    m.record(ui, "result", &r);
    m.result = Some((passed, checks.len(), r.rect));
    for c in checks.iter().filter(|c| !c.passed) {
        ui.strong(format!("Needs work · {}", c.label));
        let r = lesson_presentation::prose(ui, &c.detail);
        m.record(ui, "failed-case", &r);
    }
    if passed > 0 {
        controls::collapsing_header(ui, &format!("Inspect passing checks ({passed})")).show(
            ui,
            |ui| {
                for c in checks.iter().filter(|c| c.passed) {
                    lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
                }
            },
        );
    }
}
impl LabApp {
    pub(super) fn poe_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[i].clone();
        let original = &self.progress.attempts[&scenario.id];
        let d = match binding::source(&scenario) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(e) = binding::context(original, &scenario) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::Poe { simulation } = &original.state else {
            return;
        };
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
        lesson_presentation::rich(ui, RichText::new(&scenario.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, text) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(text).selected(view == v));
                    self.metrics.poe.record(ui, key, &r);
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
            let r =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            self.metrics
                .poe
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(ui, "Expected source reservations in the separate example");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            self.metrics.poe.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.poe.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&d.example, &mut next.example)
            } else {
                (&d.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &scenario);
                self.metrics.poe.record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
    let report=t.facts.observe(&s.plan).expect("bound power facts");let total=format!("{} shared source budget",task::watts(t.facts.budget_mw));let reserved=format!("{} reserved",task::watts(report.reserved_mw));let remaining=format!("{} available",task::watts(report.remaining_mw));let powered=format!("{} of {} devices powered",report.devices.iter().filter(|r|r.state==poe::State::Powered).count(),report.devices.len());let r=controls::topology_chain(ui,&[["Switch source",&total,&reserved],["Current allocation",&powered,&remaining]],"Power allocation only · this is not a test of data connectivity.");self.metrics.poe.record(ui,"topology",&r);
    if practice{lesson_presentation::practice_step(ui,&scenario,0);}
    let mut plan=s.plan.clone();connections(ui,t,&mut plan,&mut self.metrics.poe);ui.add_space(controls::GAP);if practice{lesson_presentation::practice_step(ui,&scenario,1);}power_controls(ui,t,&mut plan,&mut self.metrics.poe);
    if plan!=s.plan && let Err(e)=s.edit(t,plan){self.notice=Some(e);}
    ui.add_space(controls::GAP);if practice{lesson_presentation::practice_step(ui,&scenario,2);}ui.separator();let r=ui.add(primary(ui,if practice{"Check example"}else{"Check power plan"}));self.metrics.poe.record(ui,"check",&r);if r.clicked(){s.checked=true;}
    results(ui,s,t,practice,&mut self.metrics.poe);if s.solved(t){lesson_presentation::prose(ui,if practice{"Apply the same source-and-budget reasoning to the independent assignment."}else{&scenario.explanation});}
    if practice{let disclosure=controls::collapsing_header(ui,"Show worked answer · example only").id_salt(("poe-example-worked",&scenario.id)).show(ui,|ui|{let r=ui.add(button("Fill worked example"));self.metrics.poe.record(ui,"fill-example",&r);if r.clicked() && let Err(e)=s.edit(t,t.reference.clone()){self.notice=Some(e);}lesson_presentation::prose(ui,"Fills the separate worked plan. Inspect its reservations, then select Check example.");});self.metrics.poe.record(ui,"worked-answer",&disclosure.header_response);}
   });
            controls::collapsing_header(ui, "Help, supported model and references").show(
                ui,
                |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                            a.hints = (a.hints + 1).min(scenario.hints.len());
                            self.dirty = true;
                        }
                        for hint in scenario
                            .hints
                            .iter()
                            .take(self.progress.attempts[&scenario.id].hints)
                        {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Why does this work?").show(ui, |ui| {
                            lesson_presentation::prose(ui, &scenario.explanation);
                        });
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match d.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state =
                        engine::State::Poe {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&scenario.id], &scenario);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
        Self::workspace_footer(ui);
    }
}
