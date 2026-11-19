//! Native endpoint settings and ACL editing around actual bounded packet replay.
use super::*;
use dlms_learning_lab::{transport_core as core, transport_workspace as work};
mod echo;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub case_feedback: Option<egui::Rect>,
    pub editor_clip: Option<egui::Rect>,
    /// Unclipped inner viewport geometry, so navigation can reveal an offscreen editor.
    pub editor_region: Option<egui::Rect>,
}
fn mono(ui: &mut egui::Ui, text: &str) {
    lesson_presentation::monospace(ui, text, true);
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn protocol(ui: &mut egui::Ui, key: &str, value: &mut core::Protocol, m: &mut Metrics) {
    let r = controls::dropdown(ui, key, 140.)
        .selected_text(match value {
            core::Protocol::Tcp => "TCP",
            core::Protocol::Udp => "UDP",
        })
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (v, label) in [(core::Protocol::Tcp, "TCP"), (core::Protocol::Udp, "UDP")] {
                let r = ui.selectable_value(value, v, label);
                record(m, format!("{key}-{label}"), &r, ui);
            }
        });
    record(m, key, &r.response, ui);
}
fn port(ui: &mut egui::Ui, key: &str, value: &mut String, m: &mut Metrics) {
    let width = (ui.text_style_height(&egui::TextStyle::Body) * 7.).min(ui.available_width());
    let r = ui.add(
        controls::singleline_fitting(ui, value, width)
            .char_limit(16)
            .hint_text("1–65535")
            .id_salt(key),
    );
    m.fields.push((key.into(), r.rect, r.id));
}
fn settings(ui: &mut egui::Ui, task: &work::Task, draft: &mut work::Draft, m: &mut Metrics) {
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 39.;
    if wide {
        let control_width = ui.text_style_height(&egui::TextStyle::Body) * 8.;
        ui.strong("Application listeners");
        lesson_presentation::prose(
            ui,
            "Clients must target a matching protocol and service port. Supplied settings stay fixed.",
        );
        egui::Grid::new("transport-listeners")
            .spacing(egui::vec2(12., 8.))
            .show(ui, |ui| {
                ui.strong("Host");
                ui.strong("Protocol");
                ui.strong("Service port");
                ui.end_row();
                for (i, (source, row)) in task
                    .transport
                    .listeners
                    .iter()
                    .zip(&mut draft.listeners)
                    .enumerate()
                {
                    let editable = task.editable_listeners.contains(&i);
                    controls::cell(ui, 190., |ui| {
                        ui.label(format!(
                            "{}{}",
                            source.endpoint.host,
                            if editable { "" } else { " · fixed" }
                        ));
                    });
                    controls::cell(ui, control_width, |ui| {
                        ui.add_enabled_ui(editable, |ui| {
                            protocol(ui, &format!("listener-{i}-protocol"), &mut row.protocol, m);
                        });
                    });
                    controls::cell(ui, control_width, |ui| {
                        ui.add_enabled_ui(editable, |ui| {
                            port(ui, &format!("listener-{i}-port"), &mut row.port, m);
                        });
                    });
                    ui.end_row();
                }
            });
        ui.add_space(8.);
        ui.separator();
        ui.strong("Supplied conversations");
        lesson_presentation::prose(
            ui,
            "The client/source port identifies a conversation. The server/service port selects its destination application.",
        );
        egui::Grid::new("transport-exchanges")
            .spacing(egui::vec2(12., 8.))
            .show(ui, |ui| {
                ui.strong("Conversation");
                ui.strong("Protocol");
                ui.strong("Client/source port");
                ui.strong("Server/service port");
                ui.end_row();
                for (source, row) in task.transport.exchanges.iter().zip(&mut draft.exchanges) {
                    let editable = task.editable_exchanges.contains(&row.id);
                    let id = row.id.clone();
                    controls::cell(ui, 190., |ui| {
                        ui.vertical(|ui| {
                            ui.strong(&id);
                            ui.label(format!(
                                "{} to {}{}",
                                source.client.host,
                                source.server.host,
                                if editable { "" } else { " · fixed" }
                            ));
                        });
                    });
                    controls::cell(ui, control_width, |ui| {
                        ui.add_enabled_ui(editable, |ui| {
                            protocol(ui, &format!("{id}-protocol"), &mut row.protocol, m);
                        });
                    });
                    controls::cell(ui, control_width, |ui| {
                        ui.add_enabled_ui(editable, |ui| {
                            port(ui, &format!("{id}-client"), &mut row.client_port, m);
                        });
                    });
                    controls::cell(ui, control_width, |ui| {
                        ui.add_enabled_ui(editable, |ui| {
                            port(ui, &format!("{id}-server"), &mut row.server_port, m);
                        });
                    });
                    ui.end_row();
                }
            });
    } else {
        ui.strong("Application listeners");
        for (i, (source, row)) in task
            .transport
            .listeners
            .iter()
            .zip(&mut draft.listeners)
            .enumerate()
        {
            ui.push_id(("listener", i), |ui| {
                let editable = task.editable_listeners.contains(&i);
                if editable {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · choose the protocol and service port clients must target.",
                            source.endpoint.host
                        ),
                    );
                    controls::form_row(ui, "Protocol", |ui, _| {
                        protocol(ui, &format!("listener-{i}-protocol"), &mut row.protocol, m)
                    });
                    controls::form_row(ui, "Service port", |ui, _| {
                        port(ui, &format!("listener-{i}-port"), &mut row.port, m)
                    });
                } else {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {} service port {} · supplied setting",
                            source.endpoint.host,
                            work::protocol_name(row.protocol),
                            row.port
                        ),
                    );
                }
            });
        }
        ui.add_space(8.);
        ui.separator();
        ui.strong("Supplied conversations");
        for (source, row) in task.transport.exchanges.iter().zip(&mut draft.exchanges) {
            ui.push_id(&row.id,|ui|{
            ui.strong(&row.id);
            if task.editable_exchanges.contains(&row.id){
                lesson_presentation::prose(ui,&format!("{} to {}. Match the destination service; keep a different source port for each conversation.",source.client.host,source.server.host));
                controls::form_row(ui,"Protocol",|ui,_|protocol(ui,&format!("{}-protocol",row.id),&mut row.protocol,m));
                let id=row.id.clone();
                controls::form_row(ui,"Client/source port",|ui,_|port(ui,&format!("{id}-client"),&mut row.client_port,m));
                controls::form_row(ui,"Server/service port",|ui,_|port(ui,&format!("{id}-server"),&mut row.server_port,m));
            }else{
                lesson_presentation::prose(ui,&format!("{} {}:{} to {}:{} · supplied settings",work::protocol_name(row.protocol),source.client.host,row.client_port,source.server.host,row.server_port));
            }
            ui.add_space(6.);
        });
        }
    }
}
fn state(s: Option<core::TcpState>) -> &'static str {
    match s {
        None => "No TCP state",
        Some(core::TcpState::Closed) => "Closed",
        Some(core::TcpState::Listen) => "Listening",
        Some(core::TcpState::SynSent) => "SYN sent",
        Some(core::TcpState::SynReceived) => "SYN received",
        Some(core::TcpState::Established) => "Established",
    }
}
fn packet(k: core::PacketKind) -> &'static str {
    match k {
        core::PacketKind::Syn => "SYN",
        core::PacketKind::SynAck => "SYN-ACK",
        core::PacketKind::Ack => "ACK",
        core::PacketKind::TcpData => "TCP data fragment",
        core::PacketKind::Datagram => "UDP datagram",
    }
}
fn feedback(ui: &mut egui::Ui, task: &work::Task, scope: &work::Scope, m: &mut Metrics) {
    if scope.tested.is_none() {
        ui.label("Not checked · use Check configuration after editing.");
        return;
    }
    let checks = scope.checks(task);
    let pass = checks.iter().filter(|r| r.0).count();
    let r = ui.strong(format!(
        "{} · {pass}/{} requirements",
        if pass == checks.len() {
            "Configuration passes"
        } else {
            "Needs work"
        },
        checks.len()
    ));
    m.result = Some((pass, checks.len(), r.rect));
    let mut case_feedback = r.rect;
    let observations = work::reports(task, &scope.draft);
    for (ok, text) in &checks {
        let row = lesson_presentation::prose(
            ui,
            &format!("{} · {text}", if *ok { "Pass" } else { "Needs work" }),
        );
        case_feedback = case_feedback.union(row.rect);
    }
    m.case_feedback = Some(case_feedback);
    echo::feedback(ui, task, scope, m);
    if let Ok(reports) = observations {
        for report in &reports {
            controls::collapsing_header(ui, &format!("Actual packet trace · {}", report.exchange))
                .id_salt(("transport-trace", &report.exchange))
                .show(ui, |ui| {
                    if report.protocol == core::Protocol::Tcp {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "Current client: {}. Current server: {}.",
                                state(report.client_state),
                                state(report.server_state)
                            ),
                        );
                    }
                    for (i, event) in report.trace.iter().enumerate() {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "{}. {} · {}",
                                i + 1,
                                packet(event.packet.kind),
                                if event.decision.disposition == core::Disposition::Delivered {
                                    "Delivered to host"
                                } else {
                                    "Filtered"
                                }
                            ),
                        );
                        mono(
                            ui,
                            &format!(
                                "{}:{} → {}:{} · {} payload bytes",
                                event.packet.source.address,
                                event.packet.source.port,
                                event.packet.destination.address,
                                event.packet.destination.port,
                                event.packet.payload_bytes
                            ),
                        );
                        lesson_presentation::prose(ui, &event.decision.detail);
                    }
                    if let Some(text) = &report.received {
                        ui.strong("Actual application receipt");
                        mono(ui, text);
                    } else {
                        ui.label("No application content received.");
                    }
                });
        }
    }
}
fn worked(ui: &mut egui::Ui, draft: &work::Draft) {
    for (i, l) in draft.listeners.iter().enumerate() {
        mono(
            ui,
            &format!(
                "Listener {}: {} port {}",
                i + 1,
                work::protocol_name(l.protocol),
                l.port
            ),
        );
    }
    for e in &draft.exchanges {
        mono(
            ui,
            &format!(
                "{}: {}, client {}, service {}",
                e.id,
                work::protocol_name(e.protocol),
                e.client_port,
                e.server_port
            ),
        );
    }
    for (router, text) in &draft.documents.routers {
        if !text.is_empty() {
            ui.strong(router);
            mono(ui, text);
        }
    }
}
impl LabApp {
    pub(super) fn transport_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            ui.label(e);
            return;
        }
        let engine::Goal::Transport { definition } = &source.goal else {
            return;
        };
        let engine::State::Transport { simulation } = &a.state else {
            return;
        };
        let visited = a.visited;
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
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
                    record(&mut self.metrics.transport, key, &r, ui);
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
                "IPv4 address and return-path foundations.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Separate practice values: follow both address and port pairs.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.transport, key, &r, ui);
                    if key == "try" {
                        self.metrics.example_action = Some(r.rect);
                    }
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
                &definition.assignment
            };
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let client = &task.transport.hosts[0];
            let server = &task.transport.hosts[1];
            controls::topology_pair(
                ui,
                [&client.id, &client.address.to_string(), "Supplied endpoint"],
                [&server.id, &server.address.to_string(), "Supplied endpoint"],
                palette(ui).accent,
                "Via simulated R1 · office ↔ branch",
            );
            echo::supplied(ui, task);
            let mut draft = scope.draft.clone();
            if !task.editable_listeners.is_empty() {
                settings(ui, task, &mut draft, &mut self.metrics.transport);
                if practice {
                    lesson_presentation::practice_step(ui, &source, 1);
                }
            } else {
                controls::collapsing_header(ui, "Supplied endpoints and conversations")
                    .show(ui, |ui| {
                        settings(ui, task, &mut draft, &mut self.metrics.transport)
                    });
                if practice {
                    lesson_presentation::practice_step(ui, &source, 1);
                }
            }
            for router in &task.editable_routers {
                ui.strong(format!("{router} · simulated ACL configuration"));
                let text = draft
                    .documents
                    .routers
                    .get_mut(router)
                    .expect("source-bound router");
                let height = (ui.text_style_height(&egui::TextStyle::Monospace) * 12.).min(320.);
                let mut editor_clip = egui::Rect::NOTHING;
                let output = controls::nested_scroll(
                    ui,
                    egui::ScrollArea::both()
                        .id_salt(("transport-editor", &source.id, practice, router))
                        .max_height(height)
                        .min_scrolled_height(height)
                        .auto_shrink([false, false]),
                    |ui| {
                        let response = ui.add(
                            egui::TextEdit::multiline(text)
                                .code_editor()
                                .char_limit(dlms_learning_lab::acl_editor::MAX_TEXT)
                                .desired_rows(12)
                                .desired_width(ui.available_width())
                                .id_salt(("transport-config", &source.id, practice, router)),
                        );
                        editor_clip = ui.clip_rect();
                        response
                    },
                );
                let response = output.inner;
                let visible = response
                    .rect
                    .intersect(output.inner_rect)
                    .intersect(editor_clip);
                self.metrics.transport.editor_clip = Some(editor_clip);
                self.metrics.transport.editor_region = Some(output.inner_rect);
                self.metrics.editor_viewport = Some(visible);
                self.metrics.focus = Some(response.id);
                self.metrics.transport.fields.push((
                    format!("config-{router}"),
                    visible,
                    response.id,
                ));
            }
            if let Err(e) = scope.edit(task, draft) {
                self.notice = Some(e);
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(active, primary(ui, "Check configuration"));
                record(&mut self.metrics.transport, "check", &r, ui);
                self.metrics.check = Some(r.rect);
                if r.clicked()
                    && let Err(e) = scope.check(task)
                {
                    self.notice = Some(e);
                }
                if practice {
                    let r = ui.add_enabled(active, button("Fill worked settings"));
                    record(&mut self.metrics.transport, "fill-example", &r, ui);
                    if r.clicked()
                        && let Err(e) = scope.edit(task, definition.worked_example.clone())
                    {
                        self.notice = Some(e);
                    }
                    let r = ui.add_enabled(active, button("Reset example only"));
                    record(&mut self.metrics.transport, "reset-example", &r, ui);
                    if r.clicked() {
                        *scope = work::Scope::new(task).expect("validated source");
                    }
                }
            });
            feedback(ui, task, scope, &mut self.metrics.transport);
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
            }
            ui.push_id(("transport-help", &source.id, practice, visited), |ui| {
                controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                    if !task.echoes.is_empty() {
                        lesson_presentation::prose(ui, echo::SCOPE);
                    }

                    if !practice {
                        let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && attempt.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.transport, "hint", &r, ui);
                        if r.clicked() {
                            attempt.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(attempt.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Show a worked repair").show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                            worked(ui, &definition.worked_assignment);
                        });
                    }
                    lesson_presentation::prose(ui, work::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for link in &source.sources {
                        ui.hyperlink_to(link, link);
                    }
                });
            });
        }
        if active && candidate != before {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::Transport {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        if view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source)
        {
            self.next_unit_lab(ui, i);
        }
    }
}
