use super::*;
use dlms_learning_lab::{
    ipv4_routing::{self, Requirement, Topology},
    ipv4_routing_workspace::{Draft, RouteDraft},
};
use engine::State;
impl LabApp {
    pub(super) fn routing_v4_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.routing_v4_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_routing_v4_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .routing_v4_learning
            .as_mut()
        {
            l.view = view;
        }
        self.focus_input = true;
        self.help_open = false;
        self.notice = None;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn routing_v4_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.routing_v4_view(i);
        if !self.progress.attempts[&s.id].challenge && s.ipv4_routing_teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_routing_v4_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let lesson = s.ipv4_routing_teaching.as_ref().unwrap();
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A configuration example to try").strong());
                ui.label(&lesson.example_description);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_routing_v4_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_routing_v4_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = s.ipv4_routing_teaching.as_ref().unwrap();
                ui.label("Practice example · Not graded; your assignment stays separate.");
                ui.label(&lesson.example_description);
                let mut learning = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .routing_v4_learning
                    .clone()
                    .unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Load example settings"));
                    self.metrics.routing_load_example = Some(r.rect);
                    if r.clicked() {
                        learning.fill_example(lesson).expect("validated example");
                        self.dirty = true;
                    }
                    if ui.add(button("Reset example only")).clicked() {
                        learning = dlms_learning_lab::ipv4_routing_teaching::Learning::new(lesson);
                        learning.view = teaching::View::Example;
                        self.dirty = true;
                    }
                });
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    learning.view = teaching::View::Assignment;
                    self.set_routing_v4_view(i, teaching::View::Assignment);
                }
                ui.label(format!("Try a change: {}", lesson.try_change));
                let old = learning.draft.clone();
                configuration_editor(
                    ui,
                    i,
                    true,
                    &learning.topology,
                    &mut learning.draft,
                    &mut self.metrics,
                    &mut self.focus_input,
                );
                if learning.draft != old {
                    self.dirty = true;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), CONTROL],
                    primary(ui, "Apply and test example").wrap(),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let _ = learning.apply();
                    self.dirty = true;
                }
                if let Some(notice) = &learning.notice {
                    ui.colored_label(palette(ui).error, notice);
                }
                let results = learning.checks(lesson);
                if !results.is_empty() {
                    if results.iter().all(|r| r.0) {
                        ui.colored_label(
                            palette(ui).success,
                            "Example verified · This practice does not earn assignment credit.",
                        );
                    } else {
                        for (_, message) in results.iter().filter(|r| !r.0).take(2) {
                            ui.colored_label(palette(ui).error, format!("Retry · {message}"));
                        }
                    }
                    egui::CollapsingHeader::new("Example results")
                        .id_salt(("routing-v4-example-results", i))
                        .show(ui, |ui| {
                            for (pass, message) in results {
                                ui.label(format!(
                                    "{} · {message}",
                                    if pass { "Pass" } else { "Retry" }
                                ));
                            }
                        });
                }
                inspect_paths(ui, i, true, &learning.topology, &lesson.outcomes);
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .routing_v4_learning = Some(learning);
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let State::Ipv4Routing { topology } = self.progress.attempts[&s.id].state.clone()
                else {
                    return;
                };
                let mut draft = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .routing_v4_draft
                    .clone()
                    .unwrap();
                let before = draft.clone();
                configuration_editor(
                    ui,
                    i,
                    false,
                    &topology,
                    &mut draft,
                    &mut self.metrics,
                    &mut self.focus_input,
                );
                if before != draft {
                    self.dirty = true;
                }
                if before.devices != draft.devices {
                    self.checked = false;
                    self.notice = None;
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = false;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), CONTROL],
                    primary(ui, "Check configuration").wrap(),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let mut candidate = topology.clone();
                    match draft.apply(&mut candidate) {
                        Ok(()) => {
                            self.progress.attempts.get_mut(&s.id).unwrap().state =
                                State::Ipv4Routing {
                                    topology: candidate,
                                };
                            self.checked = true;
                            self.notice = None;
                        }
                        Err(e) => {
                            self.checked = false;
                            self.notice = Some(e);
                        }
                    }
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = self.checked;
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .routing_v4_draft = Some(draft);
                if let Some(notice) = &self.notice {
                    ui.colored_label(palette(ui).error, notice);
                }
                self.report_routing_v4(ui, i);
                if let (
                    State::Ipv4Routing { topology },
                    engine::Goal::Ipv4Routing { requirements },
                ) = (&self.progress.attempts[&s.id].state, &s.goal)
                {
                    inspect_paths(ui, i, false, topology, requirements);
                }
            }
        }
    }
    fn report_routing_v4(&mut self, ui: &mut egui::Ui, i: usize) {
        if !self.checked {
            return;
        }
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        if let State::Ipv4Routing { topology } = &a.state
            && a.workspace
                .as_ref()
                .and_then(|w| w.routing_v4_draft.as_ref())
                .is_some_and(|d| !d.matches(topology))
        {
            return;
        }
        let results = engine::checks(&a.state, &s.goal);
        let pass = progress::current_solved(a, s);
        self.metrics.completion = pass;
        if pass {
            ui.colored_label(
                palette(ui).success,
                "Objective complete · All current checks pass.",
            );
            ui.horizontal_wrapped(|ui| {
                if ui.add(button("Back to library")).clicked() {
                    self.selected = None;
                }
                self.next_unit_lab(ui, i);
            });
            egui::CollapsingHeader::new("Objective results")
                .id_salt(("routing-v4-results", i))
                .show(ui, |ui| {
                    for (_, message) in results {
                        ui.label(format!("Pass · {message}"));
                    }
                });
        } else {
            let failures: Vec<_> = results.iter().filter(|r| !r.0).collect();
            for (_, message) in failures.iter().take(2) {
                ui.colored_label(palette(ui).error, format!("Retry · {message}"));
            }
            if failures.len() > 2 {
                ui.label(format!(
                    "{} more checks need attention. Open Objective results for details.",
                    failures.len() - 2
                ));
            }
            egui::CollapsingHeader::new("Objective results")
                .id_salt(("routing-v4-results", i))
                .show(ui, |ui| {
                    for (okay, message) in results {
                        ui.label(format!(
                            "{} · {message}",
                            if okay { "Pass" } else { "Retry" }
                        ));
                    }
                });
        }
    }
}
fn configuration_editor(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    topology: &Topology,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    ui.label(
        RichText::new(if draft.matches(topology) {
            "Applied configuration · virtual topology"
        } else {
            if practice {
                "Practice draft · Apply and test example to use these edits"
            } else {
                "Draft changes · Check configuration applies and tests them"
            }
        })
        .small()
        .color(palette(ui).muted),
    );
    let width = ui.available_width();
    if width >= 850. {
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(250., 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(250.);
                    neighbors(ui, topology, draft);
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(width - 266., 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(width - 266.);
                    device_controls(ui, index, practice, topology, draft, metrics, focus);
                },
            );
        });
    } else {
        neighbors(ui, topology, draft);
        device_controls(ui, index, practice, topology, draft, metrics, focus);
    }
}
fn neighbors(ui: &mut egui::Ui, topology: &Topology, draft: &Draft) {
    let selected = &draft.selected_device;
    ui.label(RichText::new(format!("{selected} · connected neighbors")).strong());
    for (port, iface) in &draft.devices[selected].interfaces {
        let link = &topology.devices[selected].interfaces[port].link;
        egui::Frame::new()
            .fill(palette(ui).input)
            .inner_margin(7.)
            .corner_radius(6.)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.label(RichText::new(format!("{port} · {link}")).strong());
                ui.add(
                    egui::Label::new(format!(
                        "{selected}: {} · {}",
                        if iface.address.is_empty() {
                            "Unassigned"
                        } else {
                            &iface.address
                        },
                        if iface.up { "Up" } else { "Down" }
                    ))
                    .wrap(),
                );
                let mut found = false;
                for (peer, device) in &draft.devices {
                    if peer == selected {
                        continue;
                    }
                    for (peer_port, neighbor) in &device.interfaces {
                        if topology.devices[peer].interfaces[peer_port].link == *link {
                            found = true;
                            ui.add(
                                egui::Label::new(format!(
                                    "to {peer}/{peer_port}: {} · {}",
                                    if neighbor.address.is_empty() {
                                        "Unassigned"
                                    } else {
                                        &neighbor.address
                                    },
                                    if neighbor.up { "Up" } else { "Down" }
                                ))
                                .wrap(),
                            );
                        }
                    }
                }
                if !found {
                    ui.label("No other interface on this link");
                }
            });
    }
    egui::CollapsingHeader::new("All devices and cables")
        .id_salt(("routing-v4-all-devices", selected))
        .show(ui, |ui| {
            device_cards(ui, topology, draft, true);
        });
    ui.add_space(4.);
}
fn device_cards(ui: &mut egui::Ui, topology: &Topology, draft: &Draft, vertical: bool) {
    let count = if vertical {
        1.
    } else {
        draft.devices.len() as f32
    };
    let card_width = ((ui.available_width() - (count - 1.) * ui.spacing().item_spacing.x) / count)
        .clamp(150., 260.);
    ui.with_layout(
        if vertical {
            egui::Layout::top_down(egui::Align::Min)
        } else {
            egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true)
        },
        |ui| {
            for (name, device) in &draft.devices {
                egui::Frame::new()
                    .fill(palette(ui).input)
                    .inner_margin(8.)
                    .corner_radius(6.)
                    .show(ui, |ui| {
                        ui.set_width((card_width - 16.).min(ui.available_width()));
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.label(RichText::new(name).strong());
                            for (port, iface) in &device.interfaces {
                                let link = &topology.devices[name].interfaces[port].link;
                                ui.add(
                                    egui::Label::new(format!(
                                        "{port} · {link} · {}\n{}",
                                        if iface.up { "Up" } else { "Down" },
                                        if iface.address.is_empty() {
                                            "No assigned address"
                                        } else {
                                            &iface.address
                                        }
                                    ))
                                    .wrap(),
                                );
                            }
                        });
                    });
            }
        },
    );
    ui.add_space(4.);
}
fn device_controls(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    topology: &Topology,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Configure device");
        let width = 220_f32.min(ui.available_width());
        ui.allocate_ui_with_layout(
            egui::vec2(width, CONTROL),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                egui::ComboBox::from_id_salt(("routing-v4-device", index, practice))
                    .width(width)
                    .truncate()
                    .selected_text(&draft.selected_device)
                    .show_ui(ui, |ui| {
                        for name in topology.devices.keys() {
                            ui.selectable_value(&mut draft.selected_device, name.clone(), name);
                        }
                    });
            },
        );
        ui.selectable_value(&mut draft.show_routes, false, "Addresses & interfaces");
        ui.selectable_value(&mut draft.show_routes, true, "Static routes");
    });
    let device = draft.devices.get_mut(&draft.selected_device).unwrap();
    let name = draft.selected_device.clone();
    if draft.show_routes {
        routes(
            ui,
            (index, practice),
            &name,
            topology,
            device,
            metrics,
            focus,
        );
    } else {
        ui.label(RichText::new("Enter an address/prefix for each connected network. Link labels show which interfaces share a virtual cable.").small().color(palette(ui).muted));
        let before_focus = *focus;
        for (port, iface) in &mut device.interfaces {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(port).strong());
                ui.checkbox(&mut iface.up, "Interface up");
            });
            field(
                ui,
                "Address/prefix",
                &mut iface.address,
                32,
                (index, practice, &name, port, "address"),
                metrics,
                focus,
            );
        }
        if before_focus && *focus {
            ui.ctx().request_repaint();
        }
        if device.interfaces.len() > 1 || device.forwarding {
            let r = ui.checkbox(&mut device.forwarding, "Forward packets between networks");
            metrics.routing_forward = Some(r.rect);
        } else {
            let r = egui::CollapsingHeader::new("Router function (optional)")
                .id_salt(("routing-v4-forward", index, practice, &name))
                .show(ui, |ui| {
                    let r = ui.checkbox(&mut device.forwarding, "Allow this device to forward packets");
                    metrics.routing_forward = Some(r.rect);
                    ui.label("Ordinary workstations usually leave forwarding disabled. Enable it only when the assignment uses this device as a router, including forwarding on a shared interface.");
                });
            metrics.routing_router_options = Some(r.header_response.rect);
        }
    }
}

pub(super) fn field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    limit: usize,
    id: impl std::hash::Hash,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    ui.horizontal(|ui| {
        let label_width = (ui.available_width() * 0.4).clamp(90., 165.);
        ui.allocate_ui_with_layout(
            egui::vec2(label_width, CONTROL),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(label_width);
                ui.add(egui::Label::new(label).wrap());
            },
        );
        let r = ui.add_sized(
            [
                (ui.available_width() - 8.).min(380.).min(
                    (ui.ctx().content_rect().right() - ui.next_widget_position().x - 32.).max(30.),
                ),
                CONTROL,
            ],
            egui::TextEdit::singleline(value)
                .id_salt(id)
                .char_limit(limit),
        );
        r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, true, label));
        metrics.inputs.push(r.rect);
        metrics.focus.get_or_insert(r.id);
        if *focus && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
            r.request_focus();
            *focus = false;
        }
        if r.changed() {
            workspace::bound(value, limit);
        }
    });
}
fn routes(
    ui: &mut egui::Ui,
    key: (usize, bool),
    name: &str,
    topology: &Topology,
    device: &mut dlms_learning_lab::ipv4_routing_workspace::DeviceDraft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let (index, practice) = key;
    ui.label("Static routes · Destination network, neighboring next hop and outgoing interface");
    let mut remove = None;
    if ui.available_width() >= 650. {
        remove = route_table(ui, key, name, device, metrics, focus);
    } else {
        for (n, route) in device.routes.iter_mut().enumerate() {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("Route {}", n + 1)).strong());
                if ui.add(button("Remove route")).clicked() {
                    remove = Some(n);
                }
            });
            field(
                ui,
                "Destination/prefix",
                &mut route.destination,
                32,
                (index, practice, name, n, "prefix"),
                metrics,
                focus,
            );
            field(
                ui,
                "Next-hop address",
                &mut route.next_hop,
                32,
                (index, practice, name, n, "via"),
                metrics,
                focus,
            );
            ui.horizontal_wrapped(|ui| {
                ui.label("Outgoing interface");
                let ports: Vec<_> = device.interfaces.keys().cloned().collect();
                interface_picker(
                    ui,
                    (index, practice, name, n),
                    &mut route.interface,
                    &ports,
                    140.,
                    metrics,
                );
                ui.label("Distance");
                let r = ui.add_sized(
                    [60., CONTROL],
                    egui::TextEdit::singleline(&mut route.distance)
                        .id_salt(("routing-v4-distance", index, practice, name, n))
                        .char_limit(3),
                );
                metrics.inputs.push(r.rect);
            });
            if Draft::new(topology).devices[name].routes.get(n) == Some(route)
                && let Some(status) = topology.route_status(name, n).ok()
            {
                ui.label(
                    RichText::new(format!("Applied row: {}", status.explanation()))
                        .small()
                        .color(palette(ui).muted),
                );
            }
        }
    }
    if let Some(n) = remove {
        device.routes.remove(n);
    }
    let r = ui.add_enabled(device.routes.len() < 32, button("Add static route"));
    metrics.routing_add = Some(r.rect);
    if r.clicked() {
        device.routes.push(RouteDraft {
            destination: "0.0.0.0/0".into(),
            interface: device.interfaces.keys().next().unwrap().clone(),
            next_hop: String::new(),
            distance: "1".into(),
        });
    }
    egui::CollapsingHeader::new("Connected networks and route rules").id_salt(("routing-v4-connected",index,practice,name)).show(ui,|ui|{
        for (port,iface) in &topology.devices[name].interfaces {
            if let Some(address)=&iface.address {
                ui.label(format!("{} on {port} · {}",ipv4_routing::Prefix::new(address.address,address.prefix).unwrap().text(),if iface.up && address.usable(){"Available"}else{"Unavailable: interface down or reserved address"}));
            }
        }
        for (n,_) in topology.devices[name].routes.iter().enumerate() {
            if let Ok(status)=topology.route_status(name,n) {ui.label(format!("Applied route {}: {}",n+1,status.explanation()));}
        }
        ui.label("A more-specific destination prefix wins first. Lower distance chooses between equally specific eligible routes. Supply the immediate neighboring next hop; recursive and interface-only routes are outside this model. An absent neighbor alone does not activate a backup route.");
    });
}
fn route_table(
    ui: &mut egui::Ui,
    key: (usize, bool),
    name: &str,
    device: &mut dlms_learning_lab::ipv4_routing_workspace::DeviceDraft,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> Option<usize> {
    let (index, practice) = key;
    let mut remove = None;
    egui::Grid::new(("routing-v4-table", index, practice, name))
        .spacing(egui::vec2(8., 6.))
        .show(ui, |ui| {
            for title in [
                "Destination/prefix",
                "Next hop",
                "Interface",
                "Distance",
                "",
            ] {
                ui.label(title);
            }
            ui.end_row();
            for (n, route) in device.routes.iter_mut().enumerate() {
                for (value, width, salt) in [
                    (&mut route.destination, 180., "prefix"),
                    (&mut route.next_hop, 160., "via"),
                ] {
                    let r = ui.add_sized(
                        [width, CONTROL],
                        egui::TextEdit::singleline(value)
                            .id_salt((index, practice, name, n, salt))
                            .char_limit(32),
                    );
                    r.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::WidgetType::TextEdit,
                            true,
                            format!(
                                "{name} route {} {}",
                                n + 1,
                                if salt == "prefix" {
                                    "destination/prefix"
                                } else {
                                    "next hop"
                                }
                            ),
                        )
                    });
                    metrics.inputs.push(r.rect);
                    metrics.focus.get_or_insert(r.id);
                    if *focus && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released())
                    {
                        r.request_focus();
                        *focus = false;
                    }
                    if r.changed() {
                        workspace::bound(value, 32);
                    }
                }
                let ports: Vec<_> = device.interfaces.keys().cloned().collect();
                interface_picker(
                    ui,
                    (index, practice, name, n),
                    &mut route.interface,
                    &ports,
                    110.,
                    metrics,
                );
                let r = ui.add_sized(
                    [64., CONTROL],
                    egui::TextEdit::singleline(&mut route.distance)
                        .id_salt(("routing-v4-distance", index, practice, name, n))
                        .char_limit(3),
                );
                metrics.inputs.push(r.rect);
                if ui.add_sized([80., CONTROL], button("Remove")).clicked() {
                    remove = Some(n);
                }
                ui.end_row();
            }
        });
    remove
}
fn interface_picker(
    ui: &mut egui::Ui,
    key: (usize, bool, &str, usize),
    selected: &mut String,
    ports: &[String],
    width: f32,
    metrics: &mut Metrics,
) {
    let before = selected.clone();
    let open_key = ui.make_persistent_id(("routing-v4-interface-menu-open", key));
    let phase = ui.data(|d| d.get_temp::<u8>(open_key).unwrap_or(0));
    let r = egui::ComboBox::from_id_salt(("routing-v4-route-interface", key))
        .width(width)
        .truncate()
        .selected_text(&*selected)
        .show_ui(ui, |ui| {
            let mut current = None;
            for port in ports {
                let active = *selected == *port;
                let r = ui.selectable_value(selected, port.clone(), port);
                if active {
                    current = Some((r.id, r.enabled()));
                }
                metrics.routing_port_options.push((port.clone(), r.id));
            }
            current
        });
    let open = r.inner.is_some();
    // A new popup first measures disabled contents. Pointer release can also clear
    // focus. Wait for enabled options and a quiet pointer frame, then focus once.
    // 0=closed, 1=pending, 2=initialized; keyboard navigation must not be stolen.
    let phase: u8 = if !open {
        0
    } else if phase < 2 {
        if !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
            if let Some(Some((option, true))) = r.inner {
                ui.memory_mut(|m| m.request_focus(option));
                2
            } else {
                1
            }
        } else {
            1
        }
    } else {
        2
    };
    ui.data_mut(|d| d.insert_temp(open_key, phase));
    if *selected != before && !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
        r.response.request_focus();
    }
    r.response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::ComboBox,
            true,
            format!("{} route {} outgoing interface", key.2, key.3 + 1),
        )
    });
    metrics.routing_route_interface = Some(r.response.rect);
    metrics.routing_picker_id = Some(r.response.id);
}
fn inspect_paths(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    topology: &Topology,
    requirements: &[Requirement],
) {
    egui::CollapsingHeader::new("Inspect applied forwarding paths").id_salt(("routing-v4-inspect",index,practice)).show(ui,|ui|{
        ui.label("These checks inspect the applied simulation. Check configuration applies pending edits. Failure checks use a separate copy of your configuration; your saved interface settings stay unchanged.");
        for requirement in requirements {
            if !requirement.valid_for(topology){ui.label("Required device or interface is missing.");continue;}
            if let Requirement::Reachable{label,from,target_device,target_interface,when,..}=requirement {
                ui.label(RichText::new(label).strong());
                let mut candidate=topology.clone();
                for c in when {candidate.devices.get_mut(&c.device).unwrap().interfaces.get_mut(&c.interface).unwrap().up=c.up;}
                match candidate.verify_endpoint(from,target_device,target_interface) {
                    Ok(probe)=>{
                        let target=candidate.devices[target_device].interfaces[target_interface].address.as_ref().unwrap().address;
                        ui.label(format!("Reply verified · {from} ({}) to {target_device} ({target})",probe.source));
                        for (heading,path) in [("Forward",probe.forward),("Reply",probe.reply)] {
                            for hop in path.hops {
                                ui.add(egui::Label::new(format!("{heading}: {} / {} to {} via {} / {}",hop.device,hop.interface,hop.next_hop,hop.peer,hop.peer_interface)).wrap());
                            }
                        }
                    },
                    Err(error)=>{ui.colored_label(palette(ui).error,format!("Not verified · {error}"));}
                }
            }
        }
    });
}
