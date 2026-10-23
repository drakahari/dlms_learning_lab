use super::*;
use dlms_learning_lab::{
    ipv6::{self, Requirement, Topology},
    ipv6_workspace::{Draft, RouteDraft},
};
use engine::State;
impl LabApp {
    pub(super) fn ipv6_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            return teaching::View::Assignment;
        }
        a.workspace
            .as_ref()
            .and_then(|w| w.ipv6_learning.as_ref())
            .map_or(teaching::View::Assignment, |l| l.view)
    }
    pub(super) fn set_ipv6_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .ipv6_learning
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
    pub(super) fn ipv6_learning_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let view = self.ipv6_view(i);
        if !self.progress.attempts[&s.id].challenge && s.ipv6_teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_ipv6_view(i, v);
                    }
                }
            });
        }
        match view {
            teaching::View::Introduction => {
                let lesson = s.ipv6_teaching.as_ref().unwrap();
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
                        self.set_ipv6_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_ipv6_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = s.ipv6_teaching.as_ref().unwrap();
                ui.label("Practice example · Not graded; your assignment stays separate.");
                ui.label(&lesson.example_description);
                let mut learning = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ipv6_learning
                    .clone()
                    .unwrap();
                ui.horizontal_wrapped(|ui| {
                    if ui.add(button("Load example settings")).clicked() {
                        learning.fill_example(lesson).expect("validated example");
                        self.dirty = true;
                    }
                    if ui.add(button("Reset example only")).clicked() {
                        learning = dlms_learning_lab::ipv6_teaching::Learning::new(lesson);
                        learning.view = teaching::View::Example;
                        self.dirty = true;
                    }
                });
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    learning.view = teaching::View::Assignment;
                    self.set_ipv6_view(i, teaching::View::Assignment);
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
                    [240_f32.min(ui.available_width()), controls::height(ui)],
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
                for (pass, message) in learning.checks(lesson) {
                    ui.colored_label(
                        if pass {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                        format!("{} {message}", if pass { "Pass ·" } else { "Retry ·" }),
                    );
                }
                inspect_paths(ui, i, true, &learning.topology, &lesson.outcomes);
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .ipv6_learning = Some(learning);
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let State::Ipv6 { topology } = self.progress.attempts[&s.id].state.clone() else {
                    return;
                };
                let mut draft = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ipv6_draft
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
                    [240_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Check configuration").wrap(),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    let mut candidate = topology.clone();
                    match draft.apply(&mut candidate) {
                        Ok(()) => {
                            self.progress.attempts.get_mut(&s.id).unwrap().state = State::Ipv6 {
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
                    .ipv6_draft = Some(draft);
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if let Some(notice) = &self.notice {
                        ui.colored_label(palette(ui).error, notice);
                    }
                    self.report_ipv6(ui, i);
                });
                self.metrics.feedback = Some(result.response.rect);
                if let (State::Ipv6 { topology }, engine::Goal::Ipv6 { requirements }) =
                    (&self.progress.attempts[&s.id].state, &s.goal)
                {
                    inspect_paths(ui, i, false, topology, requirements);
                }
            }
        }
    }
    fn report_ipv6(&mut self, ui: &mut egui::Ui, i: usize) {
        if !self.checked {
            return;
        }
        let s = &self.scenarios[i];
        let a = &self.progress.attempts[&s.id];
        let pass = progress::current_solved(a, s);
        self.metrics.completion = pass;
        if pass {
            ui.colored_label(
                palette(ui).success,
                "Objective complete · All current checks pass.",
            );
            self.next_unit_lab(ui, i);
        } else {
            for (okay, message) in engine::checks(&a.state, &s.goal) {
                if !okay {
                    ui.colored_label(palette(ui).error, format!("Retry · {message}"));
                }
            }
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
                    device_cards(ui, topology, draft, true);
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
        device_cards(ui, topology, draft, false);
        device_controls(ui, index, practice, topology, draft, metrics, focus);
    }
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
                controls::topology_frame(ui).show(ui, |ui| {
                    ui.set_width((card_width - 24.).min(ui.available_width()));
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
            egui::vec2(width, controls::height(ui)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_width(width);
                egui::ComboBox::from_id_salt(("ipv6-device", index, practice))
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
        ui.label(RichText::new("Manual assignment includes an on-link prefix by default; optional settings can change that declaration.").small().color(palette(ui).muted));
        let before_focus = *focus;
        for (port, iface) in &mut device.interfaces {
            ui.horizontal_wrapped(|ui| {
                controls::label(ui, RichText::new(port).strong(), 165.);
                ui.checkbox(&mut iface.up, "Interface up");
            });
            field(
                ui,
                "Manual address/prefix",
                &mut iface.address,
                64,
                (index, practice, &name, port, "address"),
                metrics,
                focus,
            );
            ui.label(
                RichText::new(format!("Link-local: {}", iface.link_local))
                    .small()
                    .color(palette(ui).muted),
            );
        }
        if before_focus && *focus {
            ui.ctx().request_repaint();
        }
        if device.interfaces.len() > 1 || device.forwarding {
            ui.checkbox(&mut device.forwarding, "Forward packets between networks");
        }
        egui::CollapsingHeader::new("On-link and link-local settings").id_salt(("ipv6-advanced",index,practice,&name)).show(ui,|ui|{
            ui.label("A manually connected prefix is an explicit on-link declaration in this model. These settings do not simulate router advertisements.");
            for (port,iface) in &mut device.interfaces {ui.label(port);ui.checkbox(&mut iface.on_link,"Manually treat the assigned prefix as on-link");field(ui,"Link-local address",&mut iface.link_local,64,(index,practice,&name,port,"link-local"),metrics,focus);}
        });
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
            egui::vec2(label_width, controls::height(ui)),
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
                controls::height(ui),
            ],
            controls::singleline(ui, value)
                .id_salt(id)
                .char_limit(limit),
        );
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
    device: &mut dlms_learning_lab::ipv6_workspace::DeviceDraft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let (index, practice) = key;
    ui.label("Static routes · Destination network, neighboring next hop and outgoing interface");
    let mut remove = None;
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
            64,
            (index, practice, name, n, "prefix"),
            metrics,
            focus,
        );
        field(
            ui,
            "Next-hop address",
            &mut route.next_hop,
            64,
            (index, practice, name, n, "via"),
            metrics,
            focus,
        );
        ui.horizontal_wrapped(|ui| {
            ui.label("Outgoing interface");
            egui::ComboBox::from_id_salt(("ipv6-route-interface", index, practice, name, n))
                .width(140.)
                .truncate()
                .selected_text(&route.interface)
                .show_ui(ui, |ui| {
                    for port in device.interfaces.keys() {
                        ui.selectable_value(&mut route.interface, port.clone(), port);
                    }
                });
            ui.label("Distance");
            let r = ui.add_sized(
                [60., controls::height(ui)],
                controls::singleline(ui, &mut route.distance)
                    .id_salt(("ipv6-distance", index, practice, name, n))
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
    if let Some(n) = remove {
        device.routes.remove(n);
    }
    if ui
        .add_enabled(device.routes.len() < 32, button("Add static route"))
        .clicked()
    {
        device.routes.push(RouteDraft {
            destination: "::/0".into(),
            interface: device.interfaces.keys().next().unwrap().clone(),
            next_hop: String::new(),
            distance: "1".into(),
        });
    }
    egui::CollapsingHeader::new("Derived connected prefixes").id_salt(("ipv6-connected",index,practice,name)).show(ui,|ui|{
        for (port,iface) in &topology.devices[name].interfaces {if let Some(address)=&iface.address && address.on_link {ui.label(format!("{} on {port} · {}",ipv6::Prefix::new(address.address,address.prefix).unwrap().text(),if iface.up{"Active"}else{"Interface down"}));}}
        ui.label("Lower distance selects between equally specific active routes. A more-specific destination prefix wins first. Blank next hop means directly on-link, not an automatic gateway.");
    });
}
fn inspect_paths(
    ui: &mut egui::Ui,
    index: usize,
    practice: bool,
    topology: &Topology,
    requirements: &[Requirement],
) {
    egui::CollapsingHeader::new("Inspect applied forwarding paths").id_salt(("ipv6-inspect",index,practice)).show(ui,|ui|{
        ui.label("These probes inspect the applied simulation. Pending field edits are applied by the Check action.");
        for requirement in requirements {if !requirement.valid_for(topology){ui.label("Required device or interface is missing.");continue;}
        if let Requirement::Reachable{label,from,target,scope,when,..}=requirement {
            ui.label(RichText::new(label).strong());let mut candidate=topology.clone();for c in when {candidate.devices.get_mut(&c.device).unwrap().interfaces.get_mut(&c.interface).unwrap().up=c.up;}
            match candidate.ping(from,*target,scope.as_deref()) {Ok(ping)=>{ui.label(format!("Reply received · source {} → {target}",ping.source));for (heading,path) in [("Forward",ping.forward),("Reply",ping.reply)] {for hop in path.hops{ui.add(egui::Label::new(format!("{heading}: {} / {} → {} via {} / {}",hop.device,hop.interface,hop.next_hop,hop.peer,hop.peer_interface)).wrap());}}},Err(error)=>{ui.colored_label(palette(ui).error,error);}}
        }}
    });
}
