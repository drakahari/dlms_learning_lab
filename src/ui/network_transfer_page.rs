//! Native file operations and network-policy editing; no host transfer client.
use super::*;
use dlms_learning_lab::{
    network_transfer as core, network_transfer_source as binding,
    network_transfer_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Rect, egui::Id)>,
    pub results: Option<egui::Rect>,
    pub diagram: Option<egui::Rect>,
    pub popup: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response) {
    m.controls.push((key.into(), r.rect));
}
fn select<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut T,
    options: &[(T, &str)],
    m: &mut Metrics,
) {
    controls::form_row(ui, label, |ui, width| {
        let selected = options
            .iter()
            .find(|(v, _)| v == value)
            .map_or("Choose a value", |(_, s)| *s);
        let response = controls::dropdown(ui, key, width)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (v, text) in options {
                    let r = ui.selectable_value(value, v.clone(), *text);
                    record(m, &format!("{key}:{text}"), &r);
                }
                m.popup = Some(controls::popup_content_clip(ui));
            });
        record(m, key, &response.response);
    });
}
fn text(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut String,
    m: &mut Metrics,
    focus: &mut bool,
    scope: egui::Id,
) {
    controls::form_row(ui, label, |ui, width| {
        let r = ui.add(
            controls::singleline_fitting(ui, value, width)
                .char_limit(120)
                .hint_text("Relative file name")
                .id(scope.with(key)),
        );
        if *focus && key == "local-path" && ui.is_enabled() {
            r.request_focus();
            *focus = false;
        }
        m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
        record(m, key, &r);
    });
}
fn request_form(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    focus: &mut bool,
    scope: egui::Id,
) {
    ui.strong("Transfer request");
    let c = &t.cases[0].context;
    let old = d.request.protocol;
    select(
        ui,
        "protocol",
        "Service",
        &mut d.request.protocol,
        &[
            (core::Protocol::Tftp, "TFTP"),
            (core::Protocol::Ftp, "FTP · passive"),
        ],
        m,
    );
    if old != d.request.protocol && d.request.protocol == core::Protocol::Tftp {
        d.request.login = None;
    }
    select(
        ui,
        "direction",
        "Direction",
        &mut d.request.direction,
        &[
            (core::Direction::Download, "Download · server to device"),
            (core::Direction::Upload, "Upload · device to server"),
        ],
        m,
    );
    let names: Vec<_> = c.servers.keys().cloned().collect();
    let options: Vec<_> = names.iter().map(|s| (s.clone(), s.as_str())).collect();
    select(ui, "server", "Server", &mut d.request.server, &options, m);
    if d.request.protocol == core::Protocol::Ftp {
        let names: Vec<_> = c
            .servers
            .get(&d.request.server)
            .map(|s| s.logins.keys().cloned().collect())
            .unwrap_or_default();
        let mut options = vec![(None, "Choose a supplied login")];
        options.extend(names.iter().map(|s| (Some(s.clone()), s.as_str())));
        select(
            ui,
            "login",
            "Training login",
            &mut d.request.login,
            &options,
            m,
        );
        if let Some(access) = d
            .request
            .login
            .as_ref()
            .and_then(|id| c.servers.get(&d.request.server)?.logins.get(id))
        {
            lesson_presentation::prose(
                ui,
                &format!(
                    "Server access · read: {}; write: {}",
                    if access.read.is_empty() {
                        "not allowed".into()
                    } else {
                        access.read.join(", ")
                    },
                    if access.write.is_empty() {
                        "not allowed".into()
                    } else {
                        access.write.join(", ")
                    }
                ),
            );
        }
    }
    text(
        ui,
        "local-path",
        "Device file",
        &mut d.request.local_path,
        m,
        focus,
        scope,
    );
    text(
        ui,
        "remote-path",
        "Server file",
        &mut d.request.remote_path,
        m,
        focus,
        scope,
    );
    let local = if d.request.local_path.is_empty() {
        "(enter device file)"
    } else {
        &d.request.local_path
    };
    let remote = if d.request.remote_path.is_empty() {
        "(enter server file)"
    } else {
        &d.request.remote_path
    };
    let local = format!("{}/{local}", c.client);
    let remote = format!("{}/{remote}", d.request.server);
    let (from, to) = if d.request.direction == core::Direction::Download {
        (remote, local)
    } else {
        (local, remote)
    };
    lesson_presentation::prose(
        ui,
        "This request reads from the source and writes to the destination:",
    );
    lesson_presentation::monospace(ui, &format!("{from} → {to}"), true);
}
fn policy_editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    scope: egui::Id,
) {
    for router in &t.editable_routers {
        ui.strong(format!("{router} · router policy"));
        lesson_presentation::prose(
            ui,
            "Edit the documented ACL subset. These rules are used by your next transfer and check; they do not change real equipment.",
        );
        let source = d.documents.routers.get_mut(router).unwrap();
        let height =
            ui.text_style_height(&egui::TextStyle::Monospace) * 12. + 2. * controls::PAD_Y as f32;
        let out = controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("transfer-policy-scroll", router))
                .max_height(height)
                .min_scrolled_height(height),
            |ui| {
                ui.add(
                    egui::TextEdit::multiline(source)
                        .code_editor()
                        .lock_focus(false)
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * controls::PAD_X as f32).max(24.),
                        )
                        .desired_rows(12)
                        .char_limit(dlms_learning_lab::acl_editor::MAX_TEXT)
                        .id(scope.with(("transfer-policy", router))),
                )
            },
        );
        let r = out.inner;
        let viewport = out.inner_rect.intersect(ui.clip_rect());
        // Reveal the logical viewport, not its currently visible fragment. A
        // parent clip can extend beyond the actual scroll region; using the
        // clipped fragment pins an offscreen target to that screen edge.
        m.controls
            .push((format!("policy-{router}"), out.inner_rect));
        m.editor = Some((r.rect, viewport, r.id));
    }
}
fn inputs(ui: &mut egui::Ui, t: &work::Task, m: &mut Metrics) {
    let response=controls::collapsing_header(ui,"Sample inputs · supplied endpoints and access").default_open(true).show(ui,|ui|{
        let first=&t.cases[0].context;
        lesson_presentation::prose(ui,&format!("The visible workspace uses “{}”. Check work repeats your transfers with each sample’s own files and endpoints.",t.cases[0].name));
        let endpoint = |server:&core::Server| {
            let mut values=vec![];
            if server.ftp { values.push(format!("FTP · TCP 21\nPassive data: {}",server.passive_port)); }
            if server.tftp { values.push(format!("TFTP · UDP 69\nTransfer port: {}",server.tftp_tid)); }
            values.join("\n")
        };
        if ui.available_width()>620. {
            let gap=controls::GAP;
            let sample=(ui.available_width()*0.28).max(200.);
            let value=(ui.available_width()-sample-gap*first.servers.len()as f32)/first.servers.len()as f32;
            egui::Grid::new("transfer-samples").striped(true).spacing(egui::vec2(gap,gap)).show(ui,|ui|{
                controls::cell(ui,sample,|ui|{ui.strong("Sample");});
                for host in first.servers.keys(){controls::cell(ui,value,|ui|{ui.strong(host);});}ui.end_row();
                for c in &t.cases {
                    controls::cell(ui,sample,|ui|{lesson_presentation::prose(ui,&c.name);});
                    for server in c.context.servers.values(){controls::cell(ui,value,|ui|{lesson_presentation::monospace(ui,&endpoint(server),true);});}ui.end_row();
                }
            });
        }else{for c in &t.cases{ui.strong(&c.name);for(host,server)in&c.context.servers{ui.strong(host);lesson_presentation::monospace(ui,&endpoint(server),true);}}}
        lesson_presentation::prose(ui,&format!("Client transfer port: {}. Choose a supplied FTP login in Transfer request to see its access. TFTP has no login; the recovery service is read-only.",first.client_port));
    });
    record(m, "inputs", &response.header_response);
}

fn diagram(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let c = &t.cases[0].context;
    let address = |host: &str| {
        let interface = &c.network.host_endpoints[host];
        c.network.topology.devices[host].interfaces[interface]
            .address
            .as_ref()
            .unwrap()
            .address
            .to_string()
    };
    let client_ip = address(&c.client);
    let server_ip = address(&d.request.server);
    let server = &c.servers[&d.request.server];
    let service = match (server.ftp, server.tftp) {
        (true, true) => "FTP and TFTP supplied",
        (true, false) => "FTP server",
        (false, true) => "TFTP recovery server",
        _ => "No supplied service",
    };
    let r = controls::topology_chain(
        ui,
        &[
            [&c.client, &client_ip, "File-transfer client"],
            [&d.request.server, &server_ip, service],
        ],
        "The supplied routed path carries the selected transfer; no real network is contacted.",
    );
    m.diagram = Some(r.rect);
}
fn files(ui: &mut egui::Ui, t: &work::Task, a: &work::Attempt, m: &mut Metrics) {
    let response=controls::collapsing_header(ui,"Inspect files · current sample").show(ui,|ui|{
        lesson_presentation::prose(ui,"This is supplied lab evidence and the result of your copies, not a TFTP directory-listing command.");
        match t.replay(0,&a.history) {Ok(w)=>for(host,files)in&w.files{ui.strong(host);for(path,value)in files{let r=controls::collapsing_header(ui,&format!("{path} · {} bytes",value.len())).show(ui,|ui|{lesson_presentation::code_value(ui,("transfer-file",host,path),value,ui.available_width());});record(m,&format!("file:{host}/{path}"),&r.header_response);}},Err(e)=>{lesson_presentation::prose(ui,&e);}}
    });
    record(m, "files", &response.header_response);
}
fn actual(ui: &mut egui::Ui, t: &work::Task, a: &work::Attempt, m: &mut Metrics) {
    ui.strong("Actual transfer · current sample");
    let Some(result) = a.last_report(t) else {
        lesson_presentation::prose(ui, "No transfer result for these inputs yet.");
        return;
    };
    match result {
        Err(e) => {
            ui.colored_label(palette(ui).error, "Transfer not performed");
            lesson_presentation::prose(ui, &e);
        }
        Ok(report) => {
            if !report.success {
                ui.colored_label(
                    palette(ui).error,
                    format!("Transfer stopped · {}", report.stage.label()),
                );
            }
            lesson_presentation::prose(ui, &report.detail);
            let r = controls::collapsing_header(ui, "Packet and file stages").show(ui, |ui| {
                for event in &report.events {
                    ui.strong(format!(
                        "{} · {}",
                        if event.delivered {
                            "Delivered"
                        } else {
                            "Filtered"
                        },
                        event.label
                    ));
                    if let Some(p) = event.packet {
                        lesson_presentation::monospace(
                            ui,
                            &format!(
                                "{}:{} → {}:{} · {}",
                                p.source.address,
                                p.source.port,
                                p.destination.address,
                                p.destination.port,
                                match p.protocol {
                                    dlms_learning_lab::transport_core::Protocol::Tcp => "TCP",
                                    dlms_learning_lab::transport_core::Protocol::Udp => "UDP",
                                }
                            ),
                            true,
                        );
                    }
                    if let Some(block) = event.block {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "Block {block} · {} file-data bytes",
                                event.packet.map_or(0, |p| p.payload_bytes)
                            ),
                        );
                    }
                    lesson_presentation::prose(ui, &event.detail);
                }
            });
            record(m, "stages", &r.header_response);
        }
    }
}
fn results(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &work::Attempt,
    s: &Scenario,
    m: &mut Metrics,
    practice: bool,
) {
    if !a.checked {
        return;
    }
    let cases: Vec<_> = (0..t.cases.len()).map(|i| t.case_checks(i, a)).collect();
    let passed = cases.iter().filter(|rows| rows.iter().all(|r| r.0)).count();
    let r = ui.group(|ui| {
        let pass = passed == cases.len();
        let title = ui.strong(if pass {
            if practice {
                "Example checks pass"
            } else {
                "Assignment complete"
            }
        } else {
            "File handover needs attention"
        });
        record(m, "result-title", &title);
        lesson_presentation::prose(
            ui,
            &format!("{passed} of {} supplied cases pass.", cases.len()),
        );
        if pass {
            lesson_presentation::prose(ui, &s.explanation);
        }
        let mut failure_shown = false;
        for (case, rows) in t.cases.iter().zip(cases) {
            let pass = rows.iter().all(|r| r.0);
            let first_failure = !pass && !failure_shown;
            failure_shown |= !pass;
            controls::collapsing_header(
                ui,
                &format!(
                    "{} · {}",
                    if pass { "Passed" } else { "Needs attention" },
                    case.name
                ),
            )
            .default_open(first_failure)
            .show(ui, |ui| {
                for (ok, why) in rows {
                    let reason = lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {why}",
                            if ok { "Verified" } else { "Needs attention" }
                        ),
                    );
                    if !ok && !m.controls.iter().any(|r| r.0 == "failure-reason") {
                        record(m, "failure-reason", &reason);
                    }
                }
            });
        }
    });
    m.results = Some(r.response.rect);
}
impl LabApp {
    pub(super) fn network_transfer_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let s = self.scenarios[i].clone();
        let original = &self.progress.attempts[&s.id];
        if let Err(e) = binding::context(original, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (
            engine::Goal::NetworkTransfer { definition },
            engine::State::NetworkTransfer { simulation },
        ) = (&s.goal, &original.state)
        else {
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
        lesson_presentation::rich(ui, RichText::new(&s.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    record(&mut self.metrics.network_transfer, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &s, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(ui, "Different training files and endpoints");
            lesson_presentation::worked_rows(ui, &s);
            // The worked table teaches the method; it does not execute or populate
            // the independent request. The normal shared diagrams retain alignment.
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "example", "Try this example"),
                    (
                        teaching::View::Assignment,
                        "assignment",
                        "Go straight to assignment",
                    ),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.network_transfer, key, &r);
                    if v == teaching::View::Example {
                        self.metrics.example_action = Some(r.rect);
                    } else {
                        self.metrics.assignment_action = Some(r.rect);
                    }
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if !practice {
                lesson_presentation::assignment(ui, &s);
            } else {
                lesson_presentation::practice_step(ui, &s, 0);
            }
            let task = next.task(definition, practice).clone();
            let a = next.attempt_mut(practice);
            let mut draft = a.draft.clone();
            let scope = egui::Id::new(("network-transfer-fields", &s.id, practice));
            ui.push_id((&s.id,practice),|ui|{ui.add_enabled_ui(active,|ui|{
                diagram(ui,&task,&draft,&mut self.metrics.network_transfer);
                inputs(ui,&task,&mut self.metrics.network_transfer);
                ui.separator();
                let column_min=controls::form_label_width(ui)+controls::button_width(ui,"Download · server to device")+controls::GAP;
                if ui.available_width()>=2.*column_min+controls::GAP&&!task.editable_routers.is_empty(){
                    ui.columns(2,|columns|{request_form(&mut columns[0],&task,&mut draft,&mut self.metrics.network_transfer,&mut self.focus_input,scope);policy_editor(&mut columns[1],&task,&mut draft,&mut self.metrics.network_transfer,scope);});
                }else{request_form(ui,&task,&mut draft,&mut self.metrics.network_transfer,&mut self.focus_input,scope);policy_editor(ui,&task,&mut draft,&mut self.metrics.network_transfer,scope);}
                if draft!=a.draft && let Err(e)=a.edit(&task,draft){self.notice=Some(e);}
                if practice{lesson_presentation::practice_step(ui,&s,1);}
                let check_label=if practice{"Check example"}else{"Check work"};
                ui.horizontal_wrapped(|ui|{
                    let r=ui.add_enabled(active,primary(ui,"Transfer file"));record(&mut self.metrics.network_transfer,"transfer",&r);
                    if r.clicked() && let Err(e)=a.transfer(&task) && a.last_report(&task).is_none(){self.notice=Some(e);}
                    let r=ui.add_enabled(active,button(check_label));record(&mut self.metrics.network_transfer,"check",&r);self.metrics.check=Some(r.rect);
                    if r.clicked() && let Err(e)=a.check(&task){self.notice=Some(e);}
                    if practice{
                        let r=ui.add_enabled(active,button("Fill worked transfer"));record(&mut self.metrics.network_transfer,"worked",&r);
                        if r.clicked(){let recipe=definition.worked_example.iter().find(|d|d.request.protocol==a.draft.request.protocol&&d.request.server==a.draft.request.server).or_else(||definition.worked_example.iter().find(|d|d.request.protocol==a.draft.request.protocol)).unwrap_or(&definition.worked_example[0]);if let Err(e)=a.edit(&task,recipe.clone()){self.notice=Some(e);}}
                        let r=ui.add_enabled(active,button("Reset example only"));record(&mut self.metrics.network_transfer,"reset-example",&r);if r.clicked(){*a=work::Attempt::new(&task).unwrap();}
                    }
                });
                lesson_presentation::prose(ui,&format!("Transfer file makes the copy. {check_label} checks copied files and the current network rules; it does not make another copy."));
                actual(ui,&task,a,&mut self.metrics.network_transfer);
                files(ui,&task,a,&mut self.metrics.network_transfer);
                results(ui,&task,a,&s,&mut self.metrics.network_transfer,practice);
            });});
            if practice {
                lesson_presentation::practice_step(ui, &s, 2);
                let r = ui.add_enabled(active, button("Open full assignment"));
                record(&mut self.metrics.network_transfer, "assignment", &r);
                if r.clicked() {
                    next.view = teaching::View::Assignment;
                    self.lesson_scroll_reset = true;
                    self.focus_input = true;
                }
            }
            controls::collapsing_header(ui, "Optional help and simulation scope").show(ui, |ui| {
                if !practice {
                    let a = self.progress.attempts.get_mut(&s.id).unwrap();
                    if ui
                        .add_enabled(active && a.hints < s.hints.len(), button("Show a hint"))
                        .clicked()
                    {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for h in s.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, h);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals reasoning").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &s.explanation);
                        },
                    );
                }
                lesson_presentation::prose(ui, &s.simplifications);
                for link in &s.sources {
                    ui.hyperlink_to(link, link);
                }
            });
        }
        if next != before && active {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::NetworkTransfer {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&s.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&s.id], &s);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
