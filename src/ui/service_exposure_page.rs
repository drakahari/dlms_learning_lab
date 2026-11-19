//! Two native command contexts over supplied in-process server/observer state.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    service_exposure_source as binding,
    service_exposure_task::Task,
    service_exposure_workspace::{self as work, Action, Event},
    telnet_client::Mode,
};
fn apply(s: &mut work::Scope, t: &Task, a: Action, notice: &mut Option<String>) {
    if let Err(e) = s.apply(t, &a) {
        *notice = Some(e);
    }
}
fn terminal(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    admin: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let key = if admin { "admin" } else { "scan" };
    let phase = if admin {
        Mode::Idle
    } else {
        s.telnet.unwrap_or_default()
    };
    let connected = matches!(phase, Mode::Connected { .. });
    let local = matches!(phase, Mode::Command { .. });
    ui.strong(if admin {
        "Server terminal"
    } else {
        "Observer terminal"
    });
    if !admin {
        let old = s.viewpoint;
        let mut selected = old;
        controls::form_row(ui, "Observe from", |ui, width| {
            let r = ui
                .add_enabled_ui(phase == Mode::Idle, |ui| {
                    controls::dropdown(ui, ("exposure-observer", t.initial.server), width)
                        .selected_text(format!(
                            "{} · {}",
                            s.world.viewpoints[old].id, s.world.viewpoints[old].address
                        ))
                        .show_ui(ui, |ui| {
                            controls::fit_popup_contents(ui);
                            for (i, v) in s.world.viewpoints.iter().enumerate() {
                                let r = ui.selectable_value(
                                    &mut selected,
                                    i,
                                    format!("{} · {}", v.id, v.address),
                                );
                                m.record(ui, &format!("observer-choice:{i}"), &r);
                            }
                        })
                })
                .inner;
            controls::retain_dropdown_focus(ui, &r.response);
            m.record(ui, "observer-choice", &r.response);
        });
        if selected != old {
            apply(
                s,
                t,
                Action::Event {
                    event: Event::Select {
                        viewpoint: selected,
                    },
                },
                notice,
            );
        }
    }
    let text = if admin {
        &s.admin_output
    } else {
        &s.scan_output
    };
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 8.;
    let r = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_X)
        .show(ui, |ui| {
            ui.strong(if admin {
                "Actual server results"
            } else {
                "Actual port-test results"
            });
            controls::nested_scroll(
                ui,
                egui::ScrollArea::both()
                    .id_salt(("exposure-output", t.initial.server, key))
                    .auto_shrink([false, false])
                    .min_scrolled_height(height)
                    .max_height(height)
                    .stick_to_bottom(true),
                |ui| {
                    if text.is_empty() {
                        lesson_presentation::prose(ui, "No command has run in this context.");
                    } else {
                        ui.add(egui::Label::new(RichText::new(text).monospace()).extend());
                    }
                },
            );
        });
    m.record(ui, &format!("{key}-output"), &r.response);
    if !admin && phase != Mode::Idle {
        let r = lesson_presentation::prose(
            ui,
            if connected {
                "TCP transport open · client still running. Choose Escape to local prompt, then type quit to close it. Application data is not supported here."
            } else {
                "Local Telnet prompt · client still running. Type quit, then Run command to close the transport."
            },
        );
        m.record(ui, "scan-status", &r);
    } else if !admin && let Some(status) = s.scan_status {
        let detail = if status == 0 {
            "command completed"
        } else if let Some(scan) = &s.last_scan {
            match scan.rows.as_slice() {
                [
                    dlms_learning_lab::service_exposure::Observation {
                        state: dlms_learning_lab::service_exposure::PortState::Closed,
                        ..
                    },
                ] => "connection refused · no listener accepts this port",
                [
                    dlms_learning_lab::service_exposure::Observation {
                        state: dlms_learning_lab::service_exposure::PortState::Filtered,
                        ..
                    },
                ] => "modeled timeout · a request or reply is dropped",
                _ => "no TCP connection from this viewpoint",
            }
        } else {
            "command needs attention; read the diagnostic"
        };
        let r = ui.label(format!("Exit status {status} · {detail}"));
        m.record(ui, "scan-status", &r);
    }
    let value = if admin {
        s.admin_input.clone()
    } else {
        s.scan_input.clone()
    };
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &value,
            egui::Id::new(("exposure-input", t.initial.server, key)),
            if admin { 4096 } else { 1024 },
            ui.is_enabled() && !connected,
            if local {
                "Local command · Enter runs"
            } else if connected {
                "Connection open · choose Escape"
            } else {
                "Command · Enter runs"
            },
        );
        m.record(ui, &format!("{key}-input"), &r.response);
        m.fields
            .push((format!("{key}-input"), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(
                s,
                t,
                if admin {
                    Action::AdminInput { text }
                } else {
                    Action::ScanInput { text }
                },
                notice,
            );
        }
        run |= ui.is_enabled()
            && !connected
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "This command exceeds the displayed simulator's input limit; previous draft retained.",
            );
        }
    };
    let caption = if connected {
        "Escape to local prompt"
    } else {
        "Run command"
    };
    let bw = controls::button_width(ui, caption);
    let clicked = if ui.available_width()
        > bw + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - bw - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| field(ui, m),
            );
            let r = ui.add(button(caption));
            m.record(ui, &format!("{key}-run"), &r);
            if connected {
                m.record(ui, "scan-escape", &r);
            }
            r.clicked()
        })
        .inner
    } else {
        field(ui, m);
        let r = ui.add(button(caption));
        m.record(ui, &format!("{key}-run"), &r);
        if connected {
            m.record(ui, "scan-escape", &r);
        }
        r.clicked()
    };
    if clicked || run {
        let input = if admin {
            s.admin_input.clone()
        } else {
            s.scan_input.clone()
        };
        apply(
            s,
            t,
            Action::Event {
                event: if admin {
                    Event::Admin { input }
                } else if connected {
                    Event::TelnetEscapeV3
                } else {
                    Event::ObserverV3 { input }
                },
            },
            notice,
        );
    }
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &Task, practice: bool, m: &mut Metrics) {
    if !s.checked {
        return;
    }
    let checks = t.checks(&s.world);
    let pass = checks.iter().filter(|c| c.passed).count();
    let r = ui.strong(if pass == checks.len() {
        if practice {
            "Example checks passed".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!("Not complete · {pass}/{} outcomes pass", checks.len())
    });
    m.result = Some((pass, checks.len(), r.rect));
    m.record(ui, "result", &r);
    let mut groups: Vec<(&str, Vec<&str>)> = vec![];
    for c in checks.iter().filter(|c| !c.passed) {
        if let Some((_, labels)) = groups.iter_mut().find(|(detail, _)| *detail == c.detail) {
            labels.push(&c.label);
        } else {
            groups.push((&c.detail, vec![&c.label]));
        }
    }
    for (i, (detail, labels)) in groups.iter().enumerate() {
        ui.strong(if labels.len() == 1 {
            format!("Needs work · {}", labels[0])
        } else {
            format!("Needs work · {} outcomes share this cause", labels.len())
        });
        let r = lesson_presentation::prose(ui, detail);
        m.record(ui, "failed-case", &r);
        if labels.len() > 1 {
            controls::collapsing_header(ui, &format!("Affected outcomes ({})", labels.len()))
                .id_salt(("exposure-failure", i))
                .show(ui, |ui| {
                    for l in labels {
                        lesson_presentation::prose(ui, &format!("Not passed · {l}"));
                    }
                });
        }
    }
    if pass > 0 {
        controls::collapsing_header(ui, &format!("Inspect passing checks ({pass})")).show(
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
    pub(super) fn service_exposure_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let d = match binding::source(&source) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(e) = binding::context(original, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::ServiceExposure { simulation } = &original.state else {
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.service_exposure.record(ui, key, &r);
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
            let r = lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .service_exposure
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(ui, "Expected observations and actions");
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics
                .service_exposure
                .record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.service_exposure.record(ui, "try-example", &r);
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
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics
                    .service_exposure
                    .record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active, |ui| {
                let observer = &s.world.viewpoints[s.viewpoint];
                let client = observer.address.to_string();
                let server = s.world.server.to_string();
                let r = controls::topology_chain(
                    ui,
                    &[
                        ["Observer (selected)", &client, &observer.id],
                        ["Virtual server", &server, "Services and host filter"],
                    ],
                    "A TCP connection needs a request and a reply.",
                );
                self.metrics.service_exposure.record(ui, "topology", &r);
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54. {
                    ui.columns(2, |cols| {
                        if practice {
                            lesson_presentation::practice_step(&mut cols[0], &source, 0);
                            lesson_presentation::practice_step(&mut cols[1], &source, 1);
                        }
                        terminal(
                            &mut cols[0],
                            t,
                            s,
                            true,
                            &mut self.metrics.service_exposure,
                            &mut self.notice,
                        );
                        terminal(
                            &mut cols[1],
                            t,
                            s,
                            false,
                            &mut self.metrics.service_exposure,
                            &mut self.notice,
                        );
                    });
                } else {
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 0);
                    }
                    terminal(
                        ui,
                        t,
                        s,
                        true,
                        &mut self.metrics.service_exposure,
                        &mut self.notice,
                    );
                    ui.add_space(12.);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 1);
                    }
                    terminal(
                        ui,
                        t,
                        s,
                        false,
                        &mut self.metrics.service_exposure,
                        &mut self.notice,
                    );
                }
                if practice {
                    lesson_presentation::practice_step(ui, &source, 2);
                }
                ui.separator();
                let r = ui.add(primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check configuration"
                    },
                ));
                self.metrics.service_exposure.record(ui, "check", &r);
                if r.clicked() {
                    apply(
                        s,
                        t,
                        Action::Event {
                            event: Event::Check,
                        },
                        &mut self.notice,
                    );
                }
                results(ui, s, t, practice, &mut self.metrics.service_exposure);
                if s.solved(t) {
                    lesson_presentation::prose(
                        ui,
                        if practice {
                            "Apply the same reasoning independently to the assignment."
                        } else {
                            &source.explanation
                        },
                    );
                }
                if practice {
                    controls::collapsing_header(ui, "Show worked answer · example only").show(
                        ui,
                        |ui| {
                            for a in &d.worked {
                                if let Action::Event { event } = a {
                                    match event {
                                        Event::Admin { input } => {
                                            ui.monospace(format!("Server: {input}"));
                                        }
                                        Event::Scan { input }
                                        | Event::ProbeV2 { input }
                                        | Event::ObserverV3 { input } => {
                                            ui.monospace(format!("Observer: {input}"));
                                        }
                                        Event::Select { viewpoint } => {
                                            ui.label(format!(
                                                "Observe from {}",
                                                t.initial.viewpoints[*viewpoint].id
                                            ));
                                        }
                                        Event::Check => {
                                            ui.label("Check example after the repair.");
                                        }
                                        Event::TelnetEscapeV3 => {
                                            ui.label(
                                                "Native action: Escape to the local Telnet prompt.",
                                            );
                                        }
                                    }
                                }
                            }
                        },
                    );
                }
            });
            controls::collapsing_header(ui, "Help, supported subset and references").show(
                ui,
                |ui| {
                    if !challenge {
                        if ui.add(button("Show a hint")).clicked() {
                            let a = self.progress.attempts.get_mut(&source.id).unwrap();
                            a.hints = (a.hints + 1).min(source.hints.len());
                            self.dirty = true;
                        }
                        for hint in source
                            .hints
                            .iter()
                            .take(self.progress.attempts[&source.id].hints)
                        {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
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
            match d.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::ServiceExposure {
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
        Self::workspace_footer(ui);
    }
}
