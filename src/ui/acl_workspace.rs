//! Focused native policy editing. No device session or host command execution.
use super::*;
use dlms_learning_lab::{
    acl, acl_editor as editor,
    acl_routing::Network,
    acl_teaching::Learning,
    acl_workspace::{self as policy, Draft},
};
impl LabApp {
    pub(super) fn acl_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.acl_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_acl_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.acl_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn acl_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s.acl_teaching.as_ref().expect("validated ACL teaching");
        let view = self.acl_view(i);
        ui.heading(&s.title);
        if !self.progress.attempts[&s.id].challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_acl_view(i, v);
                    }
                }
            });
        }
        let view = self.acl_view(i);
        match view {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                for (router, text) in &lesson.example_documents.routers {
                    if !text.trim().is_empty() {
                        ui.label(format!("Virtual router: {router}"));
                        ui.add(egui::Label::new(RichText::new(text).monospace()).wrap());
                    }
                }
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_acl_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_acl_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Ungraded; your assignment draft stays separate.");
                ui.label(&lesson.example_description);
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&l).unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        l.fill_example(lesson).expect("validated example");
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        l = Learning::new(lesson).unwrap();
                        l.view = teaching::View::Example;
                    }
                });
                ui.push_id(("acl-example", &s.id), |ui| {
                    let (changed, shortcut) = configuration(
                        ui,
                        &l.network,
                        &mut l.draft,
                        &mut self.metrics,
                        &mut self.focus_input,
                        !l.checked,
                    );
                    if changed {
                        l.checked = false;
                    }
                    let r = ui.add(primary(ui, "Check example"));
                    self.metrics.check = Some(r.rect);
                    let requested = r.clicked() || shortcut;
                    if requested {
                        match l.apply_check(lesson) {
                            Ok(_) => l.draft.notice = None,
                            Err(e) => l.draft.notice = Some(notice(e)),
                        }
                    }
                    let result = controls::feedback(ui, requested, |ui| {
                        if l.checked {
                            let checks = l.checks(lesson);
                            if !checks.is_empty() && checks.iter().all(|c| c.0) {
                                ui.colored_label(
                                    palette(ui).success,
                                    "Example checks pass · No assignment credit",
                                );
                            }
                            if let Some(error) = l
                                .draft
                                .notice
                                .as_ref()
                                .filter(|_| !l.draft.matches(&l.network))
                            {
                                ui.colored_label(
                                    palette(ui).error,
                                    format!("Needs attention · {error}"),
                                );
                            } else {
                                results(ui, &checks);
                            }
                        }
                    });
                    self.metrics.feedback = Some(result.response.rect);
                });
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    l.view = teaching::View::Assignment;
                    self.set_acl_view(i, teaching::View::Assignment);
                }
                if serde_json::to_vec(&l).unwrap() != before {
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .acl_learning = Some(l);
            }
            teaching::View::Assignment => {
                self.checked = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .checked;
                ui.label(&s.objective);
                if let engine::Goal::Ipv4Acl { definition } = &s.goal {
                    ui.push_id(("acl-required-traffic", &s.id), |ui| {
                        traffic_table(ui, definition)
                    });
                }
                let engine::State::Ipv4Acl { mut network } =
                    self.progress.attempts[&s.id].state.clone()
                else {
                    unreachable!()
                };
                let engine::Goal::Ipv4Acl { definition } = &s.goal else {
                    unreachable!()
                };
                let mut draft = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .acl_draft
                    .clone()
                    .unwrap_or_else(|| Draft::new(&network).unwrap());
                let before = serde_json::to_vec(&(&network, &draft)).unwrap();
                let (changed, shortcut) = ui
                    .push_id(("acl-assignment", &s.id), |ui| {
                        configuration(
                            ui,
                            &network,
                            &mut draft,
                            &mut self.metrics,
                            &mut self.focus_input,
                            !self.checked,
                        )
                    })
                    .inner;
                if changed {
                    self.checked = false;
                }
                let r = ui.add(primary(ui, "Check configuration"));
                self.metrics.check = Some(r.rect);
                let requested = r.clicked() || shortcut;
                if requested {
                    self.checked = true;
                    match draft.apply(&mut network) {
                        Ok(()) => draft.notice = None,
                        Err(e) => draft.notice = Some(notice(e)),
                    }
                }
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                a.state = engine::State::Ipv4Acl {
                    network: network.clone(),
                };
                let w = a.workspace.as_mut().unwrap();
                w.checked = self.checked;
                w.acl_draft = Some(draft.clone());
                if serde_json::to_vec(&(&network, &draft)).unwrap() != before || requested {
                    self.dirty = true;
                }
                let result = controls::feedback(ui, requested, |ui| {
                    if self.checked {
                        let checks = policy::checks(&network, definition, &draft.documents);
                        let complete = progress::current_solved(&self.progress.attempts[&s.id], &s);
                        self.metrics.completion = complete;
                        if complete {
                            ui.colored_label(
                                palette(ui).success,
                                "Objective complete · The required traffic matches the task.",
                            );
                            results(ui, &checks);
                            if !self.progress.attempts[&s.id].challenge {
                                ui.label(&s.explanation);
                            }
                            self.next_unit_lab(ui, i);
                        } else if let Some(error) =
                            draft.notice.as_ref().filter(|_| !draft.matches(&network))
                        {
                            ui.colored_label(
                                palette(ui).error,
                                format!("Needs attention · {error}"),
                            );
                        } else {
                            results(ui, &checks);
                        }
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
            }
        }
    }
    pub(super) fn acl_help(&mut self, ui: &mut egui::Ui, i: usize) {
        self.metrics.help_visible = true;
        let s = self.scenarios[i].clone();
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let r = ui.add_enabled(
            a.hints < s.hints.len(),
            button(if a.hints == 0 {
                "Show a hint"
            } else {
                "Next hint"
            }),
        );
        self.metrics.hint = Some(r.rect);
        if r.clicked() {
            a.hints += 1;
            self.dirty = true;
        }
        if a.hints > 0 {
            ui.label(&s.hints[a.hints - 1]);
        }
        egui::CollapsingHeader::new("Why these rules work").show(ui, |ui| {
            ui.label(&s.explanation);
        });
        egui::CollapsingHeader::new("Worked reference · reveal on request").show(ui,|ui|{
            for action in &s.solution {
                if let Some(json) = action.strip_prefix("acl-author ")
                    && let Ok(policy::Action::ApplyDocuments{documents})=serde_json::from_str(json) {
                    for (router,text) in &documents.routers {
                        if !text.trim().is_empty() {
                            ui.label(format!("Virtual router: {router}"));
                            ui.add(egui::Label::new(RichText::new(text).monospace()).wrap());
                        }
                    }
                }
            }
            ui.label("Reading the reference does not change your draft. Edit the document, then check the configuration.");
        });
        egui::CollapsingHeader::new("Traffic details, supported editor syntax & sources").show(ui,|ui|{
            if let engine::Goal::Ipv4Acl{definition}=&s.goal{for case in &definition.cases{ui.label(format!("{}: {} to {} · {}",case.label,case.flow.source.device,case.flow.destination.device,protocol(case.flow.protocol)));}}
            ui.label(policy::HELP);ui.label("Use complete ip access-list standard|extended NAME and interface NAME sections. Close a section with exit. Rules support permit/deny, any, host ADDRESS or ADDRESS WILDCARD; TCP/UDP ports support eq or range with numbers. Every rule in one list must use a sequence number, or none may use one.");ui.label(&s.simplifications);for source in &s.sources{ui.add(egui::Label::new(source).wrap());}
        });
    }
}
fn traffic_table(ui: &mut egui::Ui, definition: &editor::Definition) {
    egui::CollapsingHeader::new(format!("Required traffic · {} checks",definition.cases.len())).default_open(true).show(ui,|ui| {
        ui.label(RichText::new("Each row is a separate one-way packet; reverse rows do not simulate an established session.").small().color(palette(ui).muted));
        let wide=ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*34.;
        let space=controls::GAP;
        let width=ui.available_width();
        let flow_width=(width-space*(if wide{2.}else{1.}))*if wide{0.30}else{0.58};
        let service_width=(width-2.*space)*0.32;
        let result_width=if wide {width-flow_width-service_width-2.*space}else{width-flow_width-space};
        egui::Grid::new("required-acl-traffic").min_row_height(controls::table_height(ui)).num_columns(if wide{3}else{2}).spacing(egui::vec2(space,4.)).striped(true).show(ui,|ui| {
            controls::table_text(ui,RichText::new(if wide{"From / To"}else{"Packet"}).strong(),flow_width);
            if wide {controls::table_text(ui,RichText::new("Protocol and ports").strong(),service_width);}
            controls::table_text(ui,RichText::new("Required result").strong(),result_width);ui.end_row();
            for case in &definition.cases {
                let flow=format!("{} to {}",case.flow.source.device,case.flow.destination.device);
                let service=protocol(case.flow.protocol);
                controls::table_text(ui,if wide{flow}else{format!("{flow}\n{service}")},flow_width);
                if wide {controls::table_text(ui,service,service_width);}
                controls::table_text(ui,match &case.expectation {
                    editor::Expectation::Allowed{}=>"Allow".into(),
                    editor::Expectation::Denied{expected}=>format!("Block at {}/{} {}",expected.at.device,expected.at.interface,match expected.at.direction { dlms_learning_lab::acl_routing::Direction::Inbound=>"in",dlms_learning_lab::acl_routing::Direction::Outbound=>"out" }),
                },result_width);ui.end_row();
            }
        });
    });
}
fn notice(text: String) -> String {
    let mut text = text;
    super::workspace::bound(&mut text, 512);
    text
}
fn results(ui: &mut egui::Ui, checks: &[(bool, String)]) {
    for (passed, text) in checks {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{} · {text}",
                    if *passed { "Pass" } else { "Needs attention" }
                ))
                .color(if *passed {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            )
            .wrap(),
        );
    }
}
fn protocol(p: acl::PacketProtocol) -> String {
    match p {
        acl::PacketProtocol::Icmp {} => "ICMP".into(),
        acl::PacketProtocol::Tcp {
            source_port,
            destination_port,
        } => format!("TCP source {source_port}, destination {destination_port}"),
        acl::PacketProtocol::Udp {
            source_port,
            destination_port,
        } => format!("UDP source {source_port}, destination {destination_port}"),
        acl::PacketProtocol::Other { number } => format!("IP protocol {number}"),
    }
}
fn configuration(
    ui: &mut egui::Ui,
    network: &Network,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
    show_notice: bool,
) -> (bool, bool) {
    diagram(ui, network, metrics);
    let selected_before = draft.selected_router.clone();
    ui.horizontal_wrapped(|ui| {
        ui.label("Virtual router");
        let width = 220_f32.min(ui.available_width());
        controls::cell(ui, width, |ui| {
            let r = egui::ComboBox::from_id_salt("acl-router")
                .width(width)
                .truncate()
                .selected_text(draft.selected_router.as_deref().unwrap_or("Choose router"))
                .show_ui(ui, |ui| {
                    for router in draft.documents.routers.keys() {
                        let option = ui.selectable_value(
                            &mut draft.selected_router,
                            Some(router.clone()),
                            router,
                        );
                        metrics.acl_router_items.push((router.clone(), option.rect));
                    }
                });
            name_control(&r.response, "Selected virtual router");
            metrics.acl_router = Some(r.response.rect);
        });
    });
    if selected_before != draft.selected_router {
        *focus = true;
    }
    ui.label(RichText::new("Each document replaces that router’s ACL lists and bindings. Check applies every draft; omitted sections are removed.").small().color(palette(ui).muted));
    let Some(router) = draft.selected_router.clone() else {
        ui.label("This topology has no editable virtual routers.");
        return (false, false);
    };
    let label = ui.label(RichText::new(format!("{router} · complete ACL document")).strong());
    let other_bytes: usize = draft
        .documents
        .routers
        .iter()
        .filter(|(name, _)| *name != &router)
        .map(|(_, text)| text.len())
        .sum();
    let remaining = editor::MAX_DRAFT_TOTAL.saturating_sub(other_bytes);
    let text = draft.documents.routers.get_mut(&router).unwrap();
    let before = text.clone();
    let id = ui.make_persistent_id(("acl-document", &router));
    let shortcut = ui.is_enabled()
        && ui.memory(|m| m.has_focus(id))
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    let h =
        ui.text_style_height(&egui::TextStyle::Monospace) * 8. + 2. * f32::from(controls::PAD_Y);
    let width = (ui.available_width()
        - 2. * f32::from(controls::PAD_X)
        - ui.spacing().scroll.allocated_width())
    .max(1.);
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let colour = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout(text.as_str().into(), font.clone(), colour, f32::INFINITY))
    };
    let area = egui::ScrollArea::both()
        .id_salt(("acl-scroll", &router))
        .max_height(h)
        .min_scrolled_height(h)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(text)
                    .id(id)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width(width)
                    .desired_rows(8)
                    .char_limit(editor::MAX_TEXT),
            )
            .labelled_by(label.id)
        });
    name_control(&area.inner, "Virtual router ACL configuration document");
    metrics.editor_viewport = Some(area.inner_rect);
    metrics.focus = Some(area.inner.id);
    if *focus && ui.is_enabled() {
        // The tab's release event must finish before moving focus to its editor.
        if ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
            ui.ctx().request_repaint();
        } else {
            area.inner.request_focus();
            *focus = false;
        }
    }
    if area.inner.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        area.inner.surrender_focus();
    }
    let changed = area.inner.changed();
    if text.len() > editor::MAX_TEXT || text.len() > remaining {
        *text = before;
        draft.notice = Some(
            "This edit exceeds the 64 KiB document or 256 KiB total draft limit. The previous draft is retained.".into(),
        );
    } else if changed {
        draft.notice = None;
    }
    if show_notice && let Some(message) = &draft.notice {
        ui.label(message);
    }
    ui.label(
        RichText::new(
            "Tab moves to the next control · Ctrl+Enter checks · Esc releases editor focus",
        )
        .small()
        .color(palette(ui).muted),
    );
    (changed, shortcut)
}
fn diagram(ui: &mut egui::Ui, network: &Network, metrics: &mut Metrics) {
    let routers: Vec<_> = network
        .topology
        .devices
        .iter()
        .filter(|(name, _)| !network.host_endpoints.contains_key(*name))
        .collect();
    if routers.len() == 1
        && network.host_endpoints.len() == 2
        && network.topology.devices.len() == 3
    {
        let (router, device) = routers[0];
        let hosts: Vec<_> = network.host_endpoints.iter().collect();
        if device.interfaces.len() == 2
            && hosts.iter().all(|(h, _)| {
                let endpoint = &network.topology.devices[*h];
                endpoint.interfaces.len() == 1
                    && endpoint
                        .interfaces
                        .values()
                        .all(|p| device.interfaces.values().any(|q| p.link == q.link))
            })
            && network.topology.devices[hosts[0].0]
                .interfaces
                .values()
                .next()
                .unwrap()
                .link
                != network.topology.devices[hosts[1].0]
                    .interfaces
                    .values()
                    .next()
                    .unwrap()
                    .link
        {
            let info = |name: &str| {
                let d = &network.topology.devices[name];
                d.interfaces
                    .iter()
                    .map(|(p, i)| {
                        format!(
                            "{p}: {}",
                            i.address
                                .as_ref()
                                .map(|a| format!("{}/{}", a.address, a.prefix))
                                .unwrap_or_else(|| "No address".into())
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" · ")
            };
            let a = info(hosts[0].0);
            let b = info(router);
            let c = info(hosts[1].0);
            let links = device
                .interfaces
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" / ");
            let response = controls::topology_chain(
                ui,
                &[
                    [hosts[0].0.as_str(), &a, "Endpoint"],
                    [router, &b, "Virtual router"],
                    [hosts[1].0.as_str(), &c, "Endpoint"],
                ],
                &format!("Supplied virtual links · {router}: {links}"),
            );
            metrics.acl_diagram = Some(response.rect);
            return;
        }
    }
    let info = |name: &str| {
        network.topology.devices[name]
            .interfaces
            .iter()
            .map(|(port, i)| {
                format!(
                    "{port}: {}",
                    i.address
                        .as_ref()
                        .map(|a| format!("{}/{}", a.address, a.prefix))
                        .unwrap_or_else(|| "No address".into())
                )
            })
            .collect::<Vec<_>>()
            .join(" · ")
    };
    let connected = |a: &str, b: &str| {
        network.topology.devices[a].interfaces.values().any(|p| {
            network.topology.devices[b]
                .interfaces
                .values()
                .any(|q| p.link == q.link)
        })
    };
    let hosts: Vec<_> = network.host_endpoints.keys().map(String::as_str).collect();
    if routers.len() == 1 && hosts.len() == 3 && network.topology.devices.len() == 4 {
        let router = routers[0].0.as_str();
        if hosts.iter().all(|h| connected(h, router)) {
            let addresses: Vec<_> = hosts.iter().map(|h| info(h)).collect();
            let hub = info(router);
            let peers: Vec<_> = hosts
                .iter()
                .zip(&addresses)
                .map(|(name, address)| [*name, address.as_str(), "Endpoint"])
                .collect();
            let diagram=ui.scope(|ui|controls::topology_star(
                ui,
                [router, &hub, "Virtual router"],
                &peers,
                "Shared virtual segments connect these endpoints to the router; this is not a physical cabling diagram.",
            ));
            if diagram.inner {
                metrics.acl_diagram = Some(diagram.response.rect);
                return;
            }
        }
    }
    if routers.len() == 2 && hosts.len() == 2 && network.topology.devices.len() == 4 {
        let first = hosts[0];
        let last = hosts[1];
        let r0 = routers[0].0.as_str();
        let r1 = routers[1].0.as_str();
        let path = [(r0, r1), (r1, r0)].into_iter().find_map(|(r, other)| {
            (connected(first, r)
                && !connected(first, other)
                && connected(r, other)
                && connected(other, last)
                && !connected(r, last))
            .then_some([first, r, other, last])
        });
        if let Some(names) = path {
            let addresses: Vec<_> = names.iter().map(|n| info(n)).collect();
            let cards: Vec<_> = names
                .iter()
                .zip(&addresses)
                .enumerate()
                .map(|(i, (name, address))| {
                    [
                        *name,
                        address.as_str(),
                        if i == 0 || i == 3 {
                            "Endpoint"
                        } else {
                            "Virtual router"
                        },
                    ]
                })
                .collect();
            let response = controls::topology_chain(
                ui,
                &cards,
                "Supplied virtual path · Router interface labels and addresses stay fixed in this policy task.",
            );
            metrics.acl_diagram = Some(response.rect);
            return;
        }
    }
    ui.label(RichText::new("Supplied virtual links and addresses").strong());
    ui.label(
        RichText::new("Ports with the same link name share a virtual segment.")
            .small()
            .color(palette(ui).muted),
    );
    let width = ui.available_width();
    let name_width = (width * 0.22).clamp(60., 160.).min(width / 2.);
    let area = egui::Grid::new("acl-topology-table")
        .min_row_height(controls::table_height(ui))
        .num_columns(2)
        .spacing(egui::vec2(controls::GAP, 6.))
        .striped(true)
        .show(ui, |ui| {
            for (name, device) in &network.topology.devices {
                controls::table_text(ui, RichText::new(name).strong(), name_width);
                let rows = device
                    .interfaces
                    .iter()
                    .map(|(port, i)| {
                        format!(
                            "{port}: {} · {} · link {}",
                            i.address
                                .as_ref()
                                .map(|a| format!("{}/{}", a.address, a.prefix))
                                .unwrap_or_else(|| "No address".into()),
                            if i.up { "Up" } else { "Down" },
                            i.link
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                controls::table_text(ui, rows, (width - name_width - controls::GAP).max(1.));
                ui.end_row();
            }
        });
    metrics.acl_diagram = Some(area.response.rect);
}
