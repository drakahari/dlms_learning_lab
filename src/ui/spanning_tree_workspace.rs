//! Native settings for the ideal P2P steady-state calculation, not an IOS session.
use super::*;
use dlms_learning_lab::{
    spanning_tree::{self as tree, PortState, Role, Unavailable},
    spanning_tree_teaching::Learning,
    spanning_tree_workspace::{self as work, Draft, Requirement},
};

impl LabApp {
    pub(super) fn spanning_tree_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.spanning_tree_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_spanning_tree_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.spanning_tree_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.help_open = false;
        self.notice = None;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn spanning_tree_initial_selection(a: &mut Attempt, s: &Scenario) {
        let engine::Goal::SpanningTree { requirements, .. } = &s.goal else {
            return;
        };
        let bridge = requirements.iter().find_map(|r| match r {
            Requirement::Root { bridge, .. } | Requirement::RootCost { bridge, .. } => Some(bridge),
            Requirement::RootPort { endpoint } | Requirement::PortRole { endpoint, .. } => {
                Some(&endpoint.bridge)
            }
            Requirement::ForwardingPath { from, .. } => Some(from),
            _ => None,
        });
        if let (Some(b), Some(d)) = (
            bridge,
            a.workspace
                .as_mut()
                .and_then(|w| w.spanning_tree_draft.as_mut()),
        ) {
            stp_select_bridge(d, b);
        }
    }
    pub(super) fn spanning_tree_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let active = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if !active {
            self.focus_input = false;
        }
        ui.add_enabled_ui(active, |ui| self.spanning_tree_contents(ui, i));
    }
    fn spanning_tree_contents(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        ui.heading(&s.title);
        ui.label(RichText::new("Ideal point-to-point spanning tree · Current calculated roles and paths; no BPDUs, convergence or packet test.").small().color(palette(ui).muted));
        if !self.progress.attempts[&s.id].challenge && s.spanning_tree_teaching.is_some() {
            let view = self.spanning_tree_view(i);
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_spanning_tree_view(i, v);
                    }
                }
            });
        }
        match self.spanning_tree_view(i) {
            teaching::View::Introduction => {
                let lesson = s.spanning_tree_teaching.as_ref().expect("prepared lesson");
                ui.label(&lesson.prerequisite_note);
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different graph to try").strong());
                for a in &lesson.example_actions {
                    ui.label(RichText::new(&a.title).strong());
                    ui.label(&a.explanation);
                }
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try the separate example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_spanning_tree_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Open full assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_spanning_tree_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = s.spanning_tree_teaching.as_ref().expect("prepared lesson");
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .spanning_tree_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&l).unwrap();
                ui.label(
                    "Practice graph · Ungraded; assignment settings and drafts stay separate.",
                );
                ui.push_id(("stp-example", &s.id), |ui| {
                    controls::dropdown(ui, "worked-action", ui.available_width()).selected_text(&lesson.example_actions[l.selected_action].title).show_ui(ui, |ui| {
                        for (n, a) in lesson.example_actions.iter().enumerate() { ui.selectable_value(&mut l.selected_action, n, &a.title); }
                    });
                    ui.label(&lesson.example_actions[l.selected_action].explanation);
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(button("Insert worked settings")); self.metrics.insert = Some(r.rect);
                        if r.clicked() {
                            if let Err(e) = l.fill_example(lesson) { l.notice = Some(work::notice(&e)); }
                            self.focus_input = true;
                        }
                        let r = ui.add(button("Reset example only")); self.metrics.example_reset = Some(r.rect);
                        if r.clicked() { l = Learning::new(lesson).unwrap(); l.view = teaching::View::Example; self.focus_input = true; }
                    });
                    ui.label(RichText::new("Insert fills all worked settings in this example draft. Apply changes the example; Check example evaluates its current result.").small().color(palette(ui).muted));
                    stp_workbench(ui, &l.simulation, &mut l.draft, &mut self.metrics, &mut self.focus_input);
                    let mut check_example = false;
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(button("Apply settings")); self.metrics.stp_apply = Some(r.rect);
                        if r.clicked() { let _ = l.apply(lesson); }
                        let r = ui.add(primary(ui,"Check example")); self.metrics.check = Some(r.rect);
                        check_example = r.clicked() || stp_check_shortcut(ui);
                        if check_example { l.checked = true; }
                    });
                    if let Some(n) = l.draft.notice.as_ref().or(l.notice.as_ref()) { ui.label(n); }
                    let feedback = controls::feedback(ui, check_example, |ui| {
                        if l.checked {
                            let rows = l.checks(lesson);
                            if rows.iter().all(|r| r.0) { ui.label(RichText::new("Example checks pass · No assignment credit").color(palette(ui).success)); }
                            stp_results(ui, &rows);
                        }
                    });
                    self.metrics.feedback = Some(feedback.response.rect);
                });
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    l.view = teaching::View::Assignment;
                    self.set_spanning_tree_view(i, teaching::View::Assignment);
                }
                self.dirty |= before != serde_json::to_vec(&l).unwrap();
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .spanning_tree_learning = Some(l);
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let engine::State::SpanningTree { simulation } = &mut a.state else {
                    unreachable!()
                };
                let engine::Goal::SpanningTree { initial, .. } = &s.goal else {
                    unreachable!()
                };
                let w = a.workspace.as_mut().unwrap();
                let mut draft = w.spanning_tree_draft.clone().expect("prepared graph draft");
                let before = serde_json::to_vec(&draft).unwrap();
                let mut check = false;
                ui.push_id(("stp-assignment", &s.id), |ui| {
                    stp_workbench(
                        ui,
                        simulation,
                        &mut draft,
                        &mut self.metrics,
                        &mut self.focus_input,
                    );
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(button("Apply settings"));
                        self.metrics.stp_apply = Some(r.rect);
                        if r.clicked() {
                            let _ = draft.apply(initial, simulation);
                            w.checked = false;
                            self.dirty = true; // Applied state can change while draft/notice bytes stay equal.
                        }
                        let r = ui.add(primary(ui, "Check work"));
                        self.metrics.check = Some(r.rect);
                        check = r.clicked() || stp_check_shortcut(ui);
                    });
                });
                if let Some(n) = &draft.notice {
                    ui.label(n);
                }
                self.dirty |= before != serde_json::to_vec(&draft).unwrap() || check;
                if check {
                    w.checked = true;
                }
                w.spanning_tree_draft = Some(draft);
                self.checked = w.checked;
                let feedback = controls::feedback(ui, check, |ui| {
                    if self.checked {
                        let a = &self.progress.attempts[&s.id];
                        let rows = progress::current_spanning_tree_checks(a, &s);
                        let passed = rows.iter().all(|r| r.0);
                        self.metrics.completion = passed;
                        if passed {
                            ui.label(
                                RichText::new("Objective complete · All current graph checks pass")
                                    .color(palette(ui).success),
                            );
                        }
                        stp_results(ui, &rows);
                        if passed {
                            if !a.challenge {
                                ui.label(&s.explanation);
                            }
                            self.next_unit_lab(ui, i);
                        }
                    }
                });
                self.metrics.feedback = Some(feedback.response.rect);
            }
        }
    }
    pub(super) fn spanning_tree_help(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        if ui.add(button("Show next hint")).clicked() {
            a.hints = (a.hints + 1).min(s.hints.len());
            self.dirty = true;
        }
        if a.hints > 0 {
            ui.label(&s.hints[a.hints - 1]);
        }
        egui::CollapsingHeader::new("Why these settings work")
            .id_salt(("stp-why", &s.id, a.challenge))
            .show(ui, |ui| {
                ui.label(&s.explanation);
            });
        egui::CollapsingHeader::new("Worked reference · reveal on request").id_salt(("stp-worked", &s.id, a.challenge)).show(ui, |ui| {
            let mut state = s.initial.clone();
            for action in &s.solution { engine::execute_for_goal(&mut state,&s.goal,action).expect("validated reference"); }
            let engine::State::SpanningTree { simulation } = state else { unreachable!() };
            for (b,c) in &simulation.configuration().bridges {
                ui.label(format!("{b}: priority {}, {}", c.priority, if c.online {"Available"}else{"Offline"}));
                for (p,v) in &c.ports { ui.label(format!("{p}: priority {}, local cost {}, {}",v.priority,v.cost,if v.up {"Up"}else{"Down"})); }
            }
            ui.label("Reference settings only; your draft stays unchanged. Other settings are accepted wherever the task permits.");
        });
        egui::CollapsingHeader::new("Simulation scope & references").show(ui,|ui| {
            ui.label("Fixed ideal point-to-point steady-state calculation. No BPDUs, timers, convergence, packets, PVST or guard behavior.");
            ui.label(&s.simplifications); for source in &s.sources { ui.add(egui::Label::new(source).wrap()); }
        });
    }
}
fn stp_check_shortcut(ui: &egui::Ui) -> bool {
    ui.is_enabled() && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter))
}
fn stp_results(ui: &mut egui::Ui, rows: &[(bool, String)]) {
    let pass = rows.iter().filter(|r| r.0).count();
    if pass < rows.len() {
        ui.label(format!("{pass} of {} current checks pass.", rows.len()));
    }
    for (_, m) in rows.iter().filter(|r| !r.0) {
        ui.add(
            egui::Label::new(
                RichText::new(format!("Needs attention · {m}")).color(palette(ui).error),
            )
            .wrap(),
        );
    }
    if pass > 0 {
        egui::CollapsingHeader::new(format!("View {pass} passing checks")).show(ui, |ui| {
            for (_, m) in rows.iter().filter(|r| r.0) {
                ui.add(egui::Label::new(format!("Pass · {m}")).wrap());
            }
        });
    }
}
fn stp_select_bridge(d: &mut Draft, b: &str) {
    d.selected_bridge = b.into();
    d.selected_port = d.bridges[b].ports.keys().next().cloned();
}
fn stp_workbench(
    ui: &mut egui::Ui,
    state: &tree::State,
    d: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let scope = ui.id();
    egui::CollapsingHeader::new("Physical links · applied")
        .default_open(true)
        .show(ui, |ui| {
            let height = ui.text_style_height(&egui::TextStyle::Body) * 5.;
            egui::ScrollArea::vertical()
                .id_salt(scope.with("physical-links"))
                .max_height(height)
                .show(ui, |ui| {
                    let current = state.configuration();
                    for (name, link) in &state.topology().links {
                        ui.add(
                            egui::Label::new(format!(
                                "{name}: {}/{} — {}/{} · {}",
                                link.a.bridge,
                                link.a.port,
                                link.b.bridge,
                                link.b.port,
                                if current.links[name] {
                                    "Link up"
                                } else {
                                    "Link down"
                                }
                            ))
                            .wrap(),
                        );
                    }
                });
        });
    ui.horizontal_wrapped(|ui| {
        ui.label("Bridge");
        let old = d.selected_bridge.clone();
        let r = controls::dropdown(ui, "stp-bridge", ui.available_width())
            .selected_text(&d.selected_bridge)
            .show_ui(ui, |ui| {
                for b in d.bridges.keys() {
                    let r = ui.selectable_value(&mut d.selected_bridge, b.clone(), b);
                    metrics.stp_bridge_items.push((b.clone(), r.rect));
                }
            });
        metrics.stp_bridge = Some(r.response.rect);
        if old != d.selected_bridge {
            d.selected_port = d.bridges[&d.selected_bridge].ports.keys().next().cloned();
            *focus = false;
        }
        ui.label("Port");
        if d.bridges[&d.selected_bridge].ports.is_empty() {
            ui.label("No supplied ports");
        } else {
            let r = controls::dropdown(ui, "stp-port", ui.available_width())
                .selected_text(d.selected_port.as_deref().unwrap_or("Select port"))
                .show_ui(ui, |ui| {
                    for p in d.bridges[&d.selected_bridge].ports.keys() {
                        let r = ui.selectable_value(&mut d.selected_port, Some(p.clone()), p);
                        metrics.stp_port_items.push((p.clone(), r.rect));
                    }
                });
            metrics.stp_port = Some(r.response.rect);
        }
    });
    let width = ui.available_width();
    let minimum = ui.text_style_height(&egui::TextStyle::Body) * 18.;
    if width >= minimum * 2. + controls::GAP {
        ui.columns(2, |col| {
            stp_configuration(&mut col[0], d, metrics, focus, scope);
            stp_current(&mut col[1], state, d, scope);
        });
    } else {
        stp_configuration(ui, d, metrics, focus, scope);
        ui.separator();
        stp_current(ui, state, d, scope);
    }
    if !d.matches(state) {
        ui.label(RichText::new("Pending settings · Apply before checking; observations still show the applied configuration.").color(palette(ui).muted));
    }
}
fn stp_configuration(
    ui: &mut egui::Ui,
    d: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
    scope: egui::Id,
) {
    ui.label(RichText::new("Settings draft").strong());
    let bridge = d.selected_bridge.clone();
    let port = d.selected_port.clone();
    let b = d.bridges.get_mut(&bridge).unwrap();
    let mut first = true;
    let mut rejected = false;
    let label_w =
        (ui.text_style_height(&egui::TextStyle::Body) * 5.5).min(ui.available_width() * 0.43);
    let value_w = (ui.available_width() - label_w - controls::GAP).max(24.);
    egui::Grid::new(scope.with("settings-grid"))
        .num_columns(2)
        .min_row_height(controls::height(ui))
        .spacing(egui::vec2(controls::GAP, 8.))
        .show(ui, |ui| {
            controls::label(ui, "Bridge priority", label_w);
            rejected |= stp_field(
                ui,
                &mut b.priority,
                &format!("{bridge}:priority"),
                "Multiples of 4096",
                value_w,
                metrics,
                focus,
                &mut first,
                scope,
            );
            ui.end_row();
            controls::label(ui, "Bridge", label_w);
            let r = ui.checkbox(&mut b.online, "Available");
            name_control(&r, format!("{bridge}: bridge available"));
            metrics
                .stp_toggles
                .push((format!("{bridge}:online"), r.rect));
            ui.end_row();
            if let Some(port) = &port {
                let p = b.ports.get_mut(port).unwrap();
                for (label, value, key, hint) in [
                    (
                        "Port priority",
                        &mut p.priority,
                        "priority",
                        "0–240, steps of 16",
                    ),
                    (
                        "Local port cost",
                        &mut p.cost,
                        "cost",
                        "Positive whole number",
                    ),
                ] {
                    controls::label(ui, label, label_w);
                    rejected |= stp_field(
                        ui,
                        value,
                        &format!("{bridge}/{port}:{key}"),
                        hint,
                        value_w,
                        metrics,
                        focus,
                        &mut first,
                        scope,
                    );
                    ui.end_row();
                }
                controls::label(ui, "Port", label_w);
                let r = ui.checkbox(&mut p.up, "Up");
                name_control(&r, format!("{bridge}/{port}: port up"));
                metrics
                    .stp_toggles
                    .push((format!("{bridge}/{port}:up"), r.rect));
                ui.end_row();
            }
        });
    if rejected {
        d.notice = Some("Keep each field within 64 UTF8 bytes and use text without control characters; previous text retained.".into());
    }
    ui.label(RichText::new("Lower bridge priority wins before MAC. Port cost is local to the receiving bridge; availability is physical, not a way to disable STP while forwarding.").small().color(palette(ui).muted));
    if d.links.is_empty() {
        ui.label("No supplied links");
    } else {
        ui.label("Physical link");
        let r = controls::dropdown(ui, "stp-link", ui.available_width())
            .selected_text(d.selected_link.as_deref().unwrap_or("Select link"))
            .show_ui(ui, |ui| {
                for l in d.links.keys() {
                    let r = ui.selectable_value(&mut d.selected_link, Some(l.clone()), l);
                    metrics.stp_link_items.push((l.clone(), r.rect));
                }
            });
        metrics.stp_link = Some(r.response.rect);
        if let Some(l) = &d.selected_link {
            let link = &d.context.links[l];
            ui.add(
                egui::Label::new(format!(
                    "{}/{} — {}/{}",
                    link.a.bridge, link.a.port, link.b.bridge, link.b.port
                ))
                .wrap(),
            );
            let r = ui.checkbox(d.links.get_mut(l).unwrap(), "Link up");
            name_control(&r, format!("{l}: link up"));
            metrics.stp_toggles.push((format!("{l}:up"), r.rect));
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn stp_field(
    ui: &mut egui::Ui,
    value: &mut String,
    key: &str,
    hint: &str,
    width: f32,
    metrics: &mut Metrics,
    focus: &mut bool,
    first: &mut bool,
    scope: egui::Id,
) -> bool {
    let before = value.clone();
    let r = ui.add_sized(
        [width, controls::height(ui)],
        controls::singleline(ui, value)
            .id(scope.with(key))
            .hint_text(hint),
    );
    name_control(&r, format!("{key} — virtual spanning-tree setting"));
    metrics.stp_fields.push((key.into(), r.rect, r.id));
    let rejected = value.len() > work::MAX_TEXT || value.chars().any(char::is_control);
    if rejected {
        *value = before;
    }
    if *first {
        metrics.focus = Some(r.id);
    }
    if *first && *focus && ui.is_enabled() {
        if !ui.input(|i| i.pointer.any_pressed() || i.pointer.any_released()) {
            r.request_focus();
            *focus = false;
        } else {
            ui.ctx().request_repaint();
        }
    }
    *first = false;
    rejected
}
fn stp_current(ui: &mut egui::Ui, state: &tree::State, d: &Draft, scope: egui::Id) {
    ui.label(RichText::new("Applied steady-state result").strong());
    let snapshot = state.snapshot().expect("validated ideal graph");
    let b = &snapshot.bridges()[&d.selected_bridge];
    ui.label(format!(
        "{}: bridge ID {} + MAC {}",
        d.selected_bridge,
        b.id.priority,
        b.id.mac.text()
    ));
    ui.label(format!(
        "System-ID extension: {} (included in bridge ID above)",
        state.topology().system_id_extension
    ));
    ui.label(match &b.root_bridge {
        Some(root) => format!(
            "Root: {root} · root cost {} · root port {}",
            b.root_cost.unwrap(),
            b.root_port.as_deref().unwrap_or("None · root bridge")
        ),
        None => "Bridge offline · no current root or path".into(),
    });
    if let Some(port) = &d.selected_port {
        let p = &b.ports[port];
        let role = match p.role {
            Role::Root => "Root",
            Role::Designated => "Designated",
            Role::Alternate => "Alternate",
            Role::Disabled => "Disabled",
        };
        let state = match p.state {
            PortState::Forwarding => "Forwarding",
            PortState::Discarding => "Discarding",
        };
        ui.label(format!("{port}: {role} · {state}"));
        ui.label(format!(
            "Port ID: priority {} / number {}",
            p.id.priority, p.id.number
        ));
        if let Some(peer) = &p.peer {
            ui.label(format!("Peer: {} / {}", peer.bridge, peer.port));
        }
        if let Some(reason) = p.unavailable {
            ui.label(match reason {
                Unavailable::BridgeOffline => "Bridge is offline",
                Unavailable::LocalPortDown => "Local port is down",
                Unavailable::NoLink => "No supplied link",
                Unavailable::LinkDown => "Physical link is down",
                Unavailable::PeerBridgeOffline => "Peer bridge is offline",
                Unavailable::PeerPortDown => "Peer port is down",
            });
        }
        if let Some(l) = &p.link {
            ui.label(if snapshot.links()[l].forwarding {
                "Both ends forward on this link"
            } else {
                "This link is not a forwarding-tree edge"
            });
        }
        egui::CollapsingHeader::new("Election comparison values").show(ui, |ui| {
            if let Some(c) = p.candidate {
                ui.label(format!(
                    "Root ID {} / {} · local rootward total {}",
                    c.root.priority,
                    c.root.mac.text(),
                    c.cost
                ));
                ui.label(format!(
                    "Sender bridge {} / {} · sender port {} · receiving port {}",
                    c.designated_bridge.priority,
                    c.designated_bridge.mac.text(),
                    c.designated_port.encoded(),
                    c.receiving_port.encoded()
                ));
            } else {
                ui.label("No operational received candidate at this port.");
            }
        });
    }
    let key = scope.with("path-destination");
    let saved = ui.ctx().data(|data| data.get_temp::<String>(key));
    let mut to = saved
        .filter(|b| b != &d.selected_bridge && state.topology().bridges.contains_key(b))
        .or_else(|| {
            state
                .topology()
                .bridges
                .keys()
                .find(|b| *b != &d.selected_bridge)
                .cloned()
        });
    if let Some(target) = to.as_mut() {
        ui.label("Current forwarding-tree path to");
        controls::dropdown(ui, "stp-destination", ui.available_width())
            .selected_text(target.as_str())
            .show_ui(ui, |ui| {
                for b in state
                    .topology()
                    .bridges
                    .keys()
                    .filter(|b| *b != &d.selected_bridge)
                {
                    ui.selectable_value(target, b.clone(), b);
                }
            });
        match snapshot
            .forwarding_path(&d.selected_bridge, target)
            .expect("known graph vertices")
        {
            Some(path) => {
                ui.add(egui::Label::new(path.join(" -> ")).wrap());
            }
            None => {
                ui.label("No current forwarding-tree path");
            }
        }
        ui.ctx()
            .data_mut(|data| data.insert_temp(key, target.clone()));
    }
}
