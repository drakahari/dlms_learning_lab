//! Native logical DHCP controls. No Linux/IOS terminal, packets or connectivity claim.
use super::*;
use dlms_learning_lab::{
    dhcpv4::{self as net, State, Window},
    dhcpv4_teaching::Learning,
    dhcpv4_workspace::{self as work, Action, Draft, PoolDraft, RelayDraft, ReservationDraft},
};

impl LabApp {
    pub(super) fn dhcpv4_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.dhcpv4_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_dhcpv4_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.dhcpv4_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.notice = None;
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn dhcpv4_initial_selection(a: &mut Attempt, s: &Scenario) {
        let engine::Goal::DhcpV4 { requirements, .. } = &s.goal else {
            return;
        };
        let client = requirements.iter().find_map(|r| match r {
            work::Requirement::ClientConfiguration { expected } => Some(expected.client.clone()),
            _ => None,
        });
        if let (Some(client), Some(draft)) = (
            client,
            a.workspace.as_mut().and_then(|w| w.dhcpv4_draft.as_mut()),
        ) {
            draft.selected_client = client;
            select_current_pool(draft);
        }
        if let Some(l) = a
            .workspace
            .as_mut()
            .and_then(|w| w.dhcpv4_learning.as_mut())
        {
            select_current_pool(&mut l.draft);
        }
    }
    pub(super) fn dhcpv4_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let active = !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        if !active {
            self.focus_input = false;
        }
        ui.add_enabled_ui(active, |ui| self.dhcpv4_contents(ui, i));
    }
    fn dhcpv4_contents(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        ui.heading(&s.title);
        ui.label(RichText::new("Virtual DHCP · Supplied paths and client parameters; no packets, routed connectivity or DNS-service test.").small().color(palette(ui).muted));
        if !self.progress.attempts[&s.id].challenge && s.dhcpv4_teaching.is_some() {
            let view = self.dhcpv4_view(i);
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_dhcpv4_view(i, v);
                    }
                }
            });
        }
        // Recompute after a clicked tab; the released pointer belongs to the new view.
        match self.dhcpv4_view(i) {
            teaching::View::Introduction => {
                let lesson = s.dhcpv4_teaching.as_ref().unwrap();
                ui.label(&lesson.prerequisite_note);
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_dhcpv4_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_dhcpv4_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = s.dhcpv4_teaching.as_ref().unwrap();
                ui.label("Practice example · Ungraded; assignment policy, leases and draft stay separate.");
                ui.label(&lesson.example_description);
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dhcpv4_learning
                    .clone()
                    .unwrap();
                let before = l.clone();
                let old_action = l.selected_action;
                let action_picker = controls::dropdown(ui, ("dhcp-example-action", &s.id), 360.)
                    .selected_text(action_label(
                        &lesson.example_actions[l.selected_action].action,
                    ))
                    .show_ui(ui, |ui| {
                        for (n, a) in lesson.example_actions.iter().enumerate() {
                            let r = ui.selectable_value(
                                &mut l.selected_action,
                                n,
                                action_label(&a.action),
                            );
                            self.metrics.dhcp_example_actions.push((n, r.rect));
                        }
                        self.metrics.dhcp_popup = Some(controls::popup_content_clip(ui));
                    });
                self.metrics.dhcp_controls.push((
                    "example-action-menu".into(),
                    action_picker.response.rect,
                    ui.is_enabled(),
                ));
                if old_action != l.selected_action {
                    if let Some(client) =
                        action_client(&lesson.example_actions[l.selected_action].action)
                    {
                        l.draft.selected_client = client.into();
                        select_current_pool(&mut l.draft);
                    }
                    self.focus_input = true;
                }
                ui.label(&lesson.example_actions[l.selected_action].explanation);
                ui.horizontal_wrapped(|ui| {
                    if matches!(
                        lesson.example_actions[l.selected_action].action,
                        Action::ApplyPolicy { .. }
                    ) {
                        let r = ui.add(button("Fill these policy fields"));
                        self.metrics.insert = Some(r.rect);
                        if r.clicked() {
                            l.notice = l.fill(lesson).err();
                            select_current_pool(&mut l.draft);
                            self.focus_input = true;
                        }
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        l = Learning::new(lesson).expect("validated DHCP example");
                        l.view = teaching::View::Example;
                        select_current_pool(&mut l.draft);
                        self.focus_input = true;
                    }
                });
                ui.label(RichText::new("Fill changes only editor fields. Apply policy, then use the client controls explicitly.").small().color(palette(ui).muted));
                let edit = ui
                    .push_id(("dhcp-example", &s.id), |ui| {
                        editor(
                            ui,
                            &l.simulation,
                            &mut l.draft,
                            &mut l.notice,
                            &mut self.metrics,
                            &mut self.focus_input,
                        )
                    })
                    .inner;
                if edit.policy_changed {
                    l.checked = false;
                }
                if let Some(action) = edit.action {
                    let selected_pool = l.draft.selected_pool.clone();
                    match l.execute(lesson, &action) {
                        Ok(message) => l.notice = Some(message),
                        Err(error) => l.notice = Some(work::notice(error)),
                    }
                    if let Some(pool) =
                        selected_pool.filter(|pool| l.draft.pools.contains_key(pool))
                    {
                        l.draft.selected_pool = Some(pool);
                    }
                    l.checked = false;
                }
                let r = ui.add(primary(ui, "Check example · ungraded"));
                self.metrics.check = Some(r.rect);
                let requested = r.clicked() || check_key(ui, &self.metrics);
                if requested && ui.is_enabled() {
                    l.checked = true;
                }
                let results = controls::feedback(ui, requested || edit.attempted, |ui| {
                    if let Some(n) = &l.notice {
                        let r = ui.add(egui::Label::new(n).wrap());
                        self.metrics.dhcp_notice = Some(r.rect);
                    }
                    if l.checked {
                        let checks = l.checks(lesson);
                        if checks.iter().all(|c| c.0) {
                            ui.colored_label(
                                palette(ui).success,
                                "Example checks pass · No assignment credit",
                            );
                        }
                        self.metrics.dhcp_result_summary = Some(result_rows(ui, &checks));
                    }
                });
                self.metrics.feedback = Some(results.response.rect);
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                let assignment = r.clicked();
                self.dirty |= l != before;
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .dhcpv4_learning = Some(l);
                if assignment {
                    self.set_dhcpv4_view(i, teaching::View::Assignment);
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let engine::Goal::DhcpV4 { initial, .. } = &s.goal else {
                    unreachable!()
                };
                let a = &self.progress.attempts[&s.id];
                let engine::State::DhcpV4 { simulation } = &a.state else {
                    unreachable!()
                };
                let mut sim = simulation.clone();
                let mut draft = a.workspace.as_ref().unwrap().dhcpv4_draft.clone().unwrap();
                let old_sim = sim.clone();
                let old_draft = draft.clone();
                let mut checked = a.workspace.as_ref().unwrap().checked;
                let mut notice = self.notice.clone();
                let edit = ui
                    .push_id(("dhcp-assignment", &s.id), |ui| {
                        editor(
                            ui,
                            &sim,
                            &mut draft,
                            &mut notice,
                            &mut self.metrics,
                            &mut self.focus_input,
                        )
                    })
                    .inner;
                if edit.policy_changed {
                    checked = false;
                }
                if let Some(action) = edit.action {
                    notice = Some(match work::apply(initial, &mut sim, &action) {
                        Ok(text) => text,
                        Err(error) => work::notice(error),
                    });
                    checked = false;
                }
                {
                    let a = self.progress.attempts.get_mut(&s.id).unwrap();
                    a.state = engine::State::DhcpV4 {
                        simulation: sim.clone(),
                    };
                    let w = a.workspace.as_mut().unwrap();
                    w.dhcpv4_draft = Some(draft.clone());
                    w.checked = checked;
                }
                self.notice = notice;
                self.dirty |= sim != old_sim || draft != old_draft;
                let r = ui.add(primary(ui, "Check work"));
                self.metrics.check = Some(r.rect);
                let requested = r.clicked() || check_key(ui, &self.metrics);
                if requested && ui.is_enabled() {
                    self.progress
                        .attempts
                        .get_mut(&s.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = true;
                    checked = true;
                    self.dirty = true;
                }
                self.checked = checked;
                let feedback =
                    controls::feedback(ui, requested || edit.attempted, |ui| {
                        if let Some(n) = &self.notice {
                            let r = ui.add(egui::Label::new(n).wrap());
                            self.metrics.dhcp_notice = Some(r.rect);
                        }
                        if checked {
                            let rows =
                                progress::current_dhcpv4_checks(&self.progress.attempts[&s.id], &s);
                            let complete = rows.iter().all(|row| row.0);
                            self.metrics.completion = complete;
                            let completed = if complete {
                                Some(ui.colored_label(
                                palette(ui).success,
                                "Objective complete · Current clients and new-client checks pass.",
                            ).rect)
                            } else {
                                None
                            };
                            let summary = result_rows(ui, &rows);
                            self.metrics.dhcp_result_summary =
                                Some(completed.map_or(summary, |rect| rect.union(summary)));
                            if complete {
                                if !self.progress.attempts[&s.id].challenge {
                                    egui::CollapsingHeader::new("Why this configuration works")
                                        .show(ui, |ui| {
                                            ui.label(&s.explanation);
                                        });
                                }
                                self.next_unit_lab(ui, i);
                            }
                        }
                    });
                self.metrics.feedback = Some(feedback.response.rect);
            }
        }
    }
    pub(super) fn dhcpv4_help(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        let r = ui.add(button("Show next hint"));
        if r.clicked() {
            a.hints = (a.hints + 1).min(s.hints.len());
            self.dirty = true;
        }
        if a.hints > 0 {
            ui.label(&s.hints[a.hints - 1]);
        }
        egui::CollapsingHeader::new("Why these steps work").show(ui, |ui| {
            ui.label(&s.explanation);
        });
        egui::CollapsingHeader::new("Simulation scope & references").show(ui, |ui| {
            ui.label(work::HELP);
            ui.label(&s.simplifications);
            for source in &s.sources {
                ui.add(egui::Label::new(source).wrap());
            }
        });
    }
}
fn action_client(action: &Action) -> Option<&str> {
    match action {
        Action::Discover { client }
        | Action::Request { client, .. }
        | Action::Renew { client }
        | Action::Rebind { client }
        | Action::Release { client } => Some(client),
        _ => None,
    }
}
fn action_label(action: &Action) -> String {
    match action {
        Action::ApplyPolicy { .. } => "Apply the edited policy".into(),
        Action::Discover { client } => format!("Discover · {client}"),
        Action::Request { client, .. } => format!("Request · {client}"),
        Action::Renew { client } => format!("Renew · {client}"),
        Action::Rebind { client } => format!("Rebind · {client}"),
        Action::Release { client } => format!("Release · {client}"),
        Action::Advance { seconds } => format!("Advance {seconds} seconds"),
    }
}
fn select_current_pool(draft: &mut Draft) {
    let Some(client) = draft.context.clients.get(&draft.selected_client) else {
        return;
    };
    let segment = &draft.context.segments[&client.segment];
    let selector = segment.server_interface.or(segment.relay_interface);
    draft.selected_pool = draft
        .pools
        .iter()
        .find(|(_, p)| {
            let Ok(network) = p.network.parse() else {
                return false;
            };
            let Ok(prefix) = p.prefix.parse() else {
                return false;
            };
            let subnet = net::Subnet { network, prefix };
            selector.is_some_and(|a| subnet.contains(a))
        })
        .map(|(id, _)| id.clone())
        .or_else(|| draft.pools.keys().next().cloned());
}
fn check_key(ui: &egui::Ui, metrics: &Metrics) -> bool {
    ui.is_enabled()
        && metrics
            .dhcp_fields
            .iter()
            .any(|(_, _, id)| ui.memory(|m| m.has_focus(*id)))
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter))
}
fn result_rows(ui: &mut egui::Ui, rows: &[(bool, String)]) -> egui::Rect {
    let pass = rows.iter().filter(|r| r.0).count();
    let summary = ui
        .label(format!("{pass} of {} checks pass.", rows.len()))
        .rect;
    for (ok, message) in rows.iter().filter(|r| !r.0) {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{} · {message}",
                    if *ok { "Pass" } else { "Needs attention" }
                ))
                .color(palette(ui).error),
            )
            .wrap(),
        );
    }
    if pass > 0 {
        egui::CollapsingHeader::new(format!("View {pass} passing checks")).show(ui, |ui| {
            for (_, message) in rows.iter().filter(|r| r.0) {
                ui.add(egui::Label::new(format!("Pass · {message}")).wrap());
            }
        });
    }
    summary
}
#[derive(Default)]
struct Edit {
    attempted: bool,
    policy_changed: bool,
    action: Option<Action>,
}
fn field(
    ui: &mut egui::Ui,
    scope: egui::Id,
    key: &str,
    label: &str,
    value: &mut String,
    metrics: &mut Metrics,
    focus: &mut bool,
) {
    let height = controls::height(ui);
    ui.push_id(key, |ui| {
        controls::form_row(ui, label, |ui, width| {
            let id = scope.with(("value", key));
            let response = ui.add_sized(
                [width, height],
                controls::singleline(ui, value).id(id).char_limit(4096),
            );
            name_control(&response, label);
            metrics.dhcp_fields.push((key.into(), response.rect, id));
            metrics.dhcp_field_clips.insert(key.into(), ui.clip_rect());
            metrics.inputs.push(response.rect);
            if *focus && ui.is_enabled() {
                if ui.input(|i| i.pointer.any_down()) {
                    ui.ctx().request_repaint();
                } else {
                    response.request_focus();
                    *focus = false;
                }
            }
        });
    });
}

fn toggle(ui: &mut egui::Ui, key: &str, label: &str, value: &mut bool, metrics: &mut Metrics) {
    let r = ui.checkbox(value, label);
    name_control(&r, label);
    metrics
        .dhcp_controls
        .push((key.into(), r.rect, ui.is_enabled()));
}
fn focus_new_field(ui: &mut egui::Ui, scope: egui::Id, key: &str) {
    let id = scope.with(("value", key));
    ui.memory_mut(|m| m.request_focus(id));
    ui.ctx().request_repaint();
}
fn action_button(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    enabled: bool,
    action: Action,
    metrics: &mut Metrics,
    chosen: &mut Option<Action>,
) {
    let r = ui.add_enabled(enabled, button(label));
    name_control(&r, label);
    metrics
        .dhcp_controls
        .push((key.into(), r.rect, enabled && ui.is_enabled()));
    if r.clicked() && ui.is_enabled() {
        *chosen = Some(action);
    }
}
fn editor(
    ui: &mut egui::Ui,
    sim: &State,
    draft: &mut Draft,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> Edit {
    let before = draft.clone();
    // Independent of responsive child columns: text focus/Undo survives reflow.
    let scope = ui.id();
    if !draft.same_shape(sim) {
        ui.label("Invalid DHCP editing context; previous state retained.");
        return Edit::default();
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Client");
        let old = draft.selected_client.clone();
        let response = controls::dropdown(ui, "dhcp-client", 280.)
            .selected_text(&draft.selected_client)
            .show_ui(ui, |ui| {
                for id in draft.context.clients.keys() {
                    let r = ui.selectable_value(&mut draft.selected_client, id.clone(), id);
                    metrics.dhcp_clients.push((id.clone(), r.rect));
                }
                metrics.dhcp_popup = Some(controls::popup_content_clip(ui));
            });
        metrics.dhcp_client = Some(response.response.rect);
        if old != draft.selected_client {
            select_current_pool(draft);
            *focus = true;
        }
    });
    let client = draft.selected_client.clone();
    let segment_id = draft.context.clients[&client].segment.clone();
    let segment = &draft.context.segments[&segment_id];
    let server = sim.topology().server.to_string();
    let server_state = if sim.policy().server_enabled {
        "Server enabled"
    } else {
        "Server disabled"
    };
    let path = if segment.server_interface.is_some() {
        "Direct segment"
    } else {
        "Relay required"
    };
    let network = format!("{}/{}", segment.subnet.network, segment.subnet.prefix);
    let state = match sim.window(&client) {
        Ok(Window::Unconfigured) => "Unconfigured",
        Ok(Window::Selecting) => "Selecting · Offer only",
        Ok(Window::Bound) => "Bound",
        Ok(Window::Renewal) => "Renewal window",
        Ok(Window::Rebinding) => "Rebinding window",
        Err(_) => "Invalid state",
    };
    let lease = sim
        .leases()
        .get(&client)
        .map_or("No client lease".into(), |b| b.address.to_string());
    let graph = controls::topology_chain(
        ui,
        &[
            ["Logical DHCP server", &server, server_state],
            [&segment_id, &network, path],
            [&client, &lease, state],
        ],
        "Supplied server, client subnet and current lease",
    );
    metrics.dhcp_topology = Some(graph.rect);
    let (policy_edit, client_edit) = if ui.available_width() >= 720. {
        ui.columns(2, |columns| {
            let policy_edit = policy_panel(&mut columns[0], draft, scope, notice, metrics, focus);
            let client_edit = client_panel(&mut columns[1], sim, draft, scope, notice, metrics);
            (policy_edit, client_edit)
        })
    } else {
        let client_edit = client_panel(ui, sim, draft, scope, notice, metrics);
        ui.separator();
        let policy_edit = policy_panel(ui, draft, scope, notice, metrics, focus);
        (policy_edit, client_edit)
    };
    let mut attempted = policy_edit.attempted || client_edit.attempted;
    let chosen = policy_edit.action.or(client_edit.action);
    if !draft.valid() {
        attempted = true;
        *draft = before.clone();
        *notice=Some("Editing limit exceeded: fields allow64 UTF8 bytes and the complete draft allows256KiB; the previous draft is retained.".into());
    }
    Edit {
        attempted: attempted || chosen.is_some(),
        policy_changed: draft.server_enabled != before.server_enabled
            || draft.pools != before.pools
            || draft.relays != before.relays,
        action: chosen,
    }
}
fn client_panel(
    ui: &mut egui::Ui,
    sim: &State,
    draft: &mut Draft,
    scope: egui::Id,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
) -> Edit {
    ui.scope_builder(
        egui::UiBuilder::new().id(scope.with("client-tools")),
        |ui| {
            let client = draft.selected_client.clone();
            let status = ui.vertical(|ui| {
                ui.label(format!("Simulated time: {} seconds", sim.now()));
                binding(
                    ui,
                    "Client-held lease",
                    sim.leases().get(&client),
                    sim.now(),
                );
                if let Some(o) = sim.offers().get(&client) {
                    ui.label(format!(
                        "Offer only: {} · expires in {}s · lease {}s",
                        o.address,
                        o.expires_at.saturating_sub(sim.now()),
                        o.lease_seconds
                    ));
                    ui.label(format!(
                        "Offered subnet {}/{} · router {} · DNS {}",
                        o.parameters.subnet.network,
                        o.parameters.subnet.prefix,
                        option_ip(o.parameters.router),
                        ips(&o.parameters.dns)
                    ));
                } else {
                    ui.label("No pending offer");
                }
                egui::CollapsingHeader::new("Inspect server binding").show(ui, |ui| {
                    binding(ui, "Server binding", sim.bindings().get(&client), sim.now());
                });
            });
            metrics.dhcp_status = Some(status.response.rect);
            ui.label(
                RichText::new("Client actions · Explicit exchanges using the applied policy")
                    .strong(),
            );
            // Keep one action queue for the actual frame; no automatic request/renewal.
            let mut chosen = None;
            let mut attempted = false;
            let mut clock_focus = false;
            ui.horizontal_wrapped(|ui| {
                let held = sim.leases().contains_key(&client);
                action_button(
                    ui,
                    "discover",
                    "Discover",
                    !held,
                    Action::Discover {
                        client: client.clone(),
                    },
                    metrics,
                    &mut chosen,
                );
                let offer = sim.offers().get(&client);
                action_button(
                    ui,
                    "request",
                    "Request",
                    offer.is_some(),
                    Action::Request {
                        client: client.clone(),
                        token: offer.map_or(0, |o| o.token),
                    },
                    metrics,
                    &mut chosen,
                );
                action_button(
                    ui,
                    "renew",
                    "Renew",
                    held,
                    Action::Renew {
                        client: client.clone(),
                    },
                    metrics,
                    &mut chosen,
                );
                action_button(
                    ui,
                    "rebind",
                    "Rebind",
                    matches!(sim.window(&client), Ok(Window::Rebinding)),
                    Action::Rebind {
                        client: client.clone(),
                    },
                    metrics,
                    &mut chosen,
                );
                action_button(
                    ui,
                    "release",
                    "Release",
                    held,
                    Action::Release {
                        client: client.clone(),
                    },
                    metrics,
                    &mut chosen,
                );
            });
            field(
                ui,
                scope,
                "time",
                "Advance seconds",
                &mut draft.advance_seconds,
                metrics,
                &mut clock_focus,
            );
            ui.horizontal_wrapped(|ui| {
                let r = ui.add(button("Advance time"));
                metrics
                    .dhcp_controls
                    .push(("advance".into(), r.rect, ui.is_enabled()));
                if r.clicked() && ui.is_enabled() {
                    attempted = true;
                    match draft.seconds() {
                        Ok(seconds) => chosen = Some(Action::Advance { seconds }),
                        Err(e) => *notice = Some(work::notice(e)),
                    }
                }
                ui.label("Clock only; no automatic requests.");
            });
            Edit {
                attempted: attempted || chosen.is_some(),
                action: chosen,
                ..Default::default()
            }
        },
    )
    .inner
}

fn policy_panel(
    ui: &mut egui::Ui,
    draft: &mut Draft,
    scope: egui::Id,
    notice: &mut Option<String>,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> Edit {
    ui.scope_builder(
        egui::UiBuilder::new().id(scope.with("policy-tools")),
        |ui| {
            let segment_id = draft.context.clients[&draft.selected_client]
                .segment
                .clone();
            let mut chosen = None;
            let mut attempted = false;
            ui.label(RichText::new("Policy draft · Apply does not update held leases").strong());
            toggle(
                ui,
                "server-enabled",
                "Enable DHCP server",
                &mut draft.server_enabled,
                metrics,
            );
            ui.horizontal_wrapped(|ui| {
                ui.label("Pool");
                let response = controls::dropdown(ui, "dhcp-pool", 280.)
                    .selected_text(draft.selected_pool.as_deref().unwrap_or("No pool selected"))
                    .show_ui(ui, |ui| {
                        for id in draft.pools.keys() {
                            let r =
                                ui.selectable_value(&mut draft.selected_pool, Some(id.clone()), id);
                            metrics.dhcp_pools.push((id.clone(), r.rect));
                        }
                        metrics.dhcp_popup = Some(controls::popup_content_clip(ui));
                    });
                metrics.dhcp_pool = Some(response.response.rect);
                let add = ui.add_enabled(draft.pools.len() < net::MAX_POOLS, button("Add pool"));
                if add.clicked() {
                    let id = (1..=9)
                        .map(|n| format!("pool-{n}"))
                        .find(|id| !draft.pools.contains_key(id))
                        .unwrap();
                    draft.pools.insert(
                        id.clone(),
                        PoolDraft {
                            network: String::new(),
                            prefix: String::new(),
                            first: String::new(),
                            last: String::new(),
                            excluded: vec![],
                            reservations: vec![],
                            router: String::new(),
                            dns: vec![],
                            lease_seconds: String::new(),
                        },
                    );
                    draft.selected_pool = Some(id);
                    *focus = true;
                }
            });
            if let Some(pool) = draft.selected_pool.clone() {
                let mut remove_pool = false;
                ui.push_id(&pool, |ui| {
                    let scope = scope.with(("pool", &pool));
                    let p = draft.pools.get_mut(&pool).unwrap();
                    field(
                        ui,
                        scope,
                        "network",
                        "Pool network",
                        &mut p.network,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "prefix",
                        "Prefix length",
                        &mut p.prefix,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "first",
                        "Range first address",
                        &mut p.first,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "last",
                        "Range last address",
                        &mut p.last,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "router",
                        "Router option (optional)",
                        &mut p.router,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "lease-seconds",
                        "Lease seconds",
                        &mut p.lease_seconds,
                        metrics,
                        focus,
                    );
                    let dns = egui::CollapsingHeader::new(format!("DNS servers ({})", p.dns.len()))
                        .id_salt("dhcp-dns")
                        .show(ui, |ui| {
                            let mut remove = None;
                            for (n, value) in p.dns.iter_mut().enumerate() {
                                field(
                                    ui,
                                    scope,
                                    &format!("dns-{n}"),
                                    "DNS address",
                                    value,
                                    metrics,
                                    focus,
                                );
                                if ui.add(button("Remove this DNS server")).clicked() {
                                    remove = Some(n);
                                }
                            }
                            if let Some(n) = remove {
                                p.dns.remove(n);
                            }
                            if ui
                                .add_enabled(p.dns.len() < 4, button("Add DNS server"))
                                .clicked()
                            {
                                p.dns.push(String::new());
                                focus_new_field(ui, scope, &format!("dns-{}", p.dns.len() - 1));
                            }
                        });
                    metrics.dhcp_controls.push((
                        "dns-header".into(),
                        dns.header_response.rect,
                        ui.is_enabled(),
                    ));
                    let exclusions = egui::CollapsingHeader::new(format!(
                        "Excluded addresses ({})",
                        p.excluded.len()
                    ))
                    .id_salt("dhcp-exclusions")
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("excluded")
                            .max_height(controls::height(ui) * 5.)
                            .show(ui, |ui| {
                                let mut remove = None;
                                for (n, value) in p.excluded.iter_mut().enumerate() {
                                    field(
                                        ui,
                                        scope,
                                        &format!("excluded-{n}"),
                                        "Excluded address",
                                        value,
                                        metrics,
                                        focus,
                                    );
                                    let r = ui.add(button("Remove this exclusion"));
                                    metrics.dhcp_controls.push((
                                        format!("remove-exclusion-{n}"),
                                        r.rect,
                                        ui.is_enabled(),
                                    ));
                                    if r.clicked() {
                                        remove = Some(n);
                                    }
                                }
                                if let Some(n) = remove {
                                    p.excluded.remove(n);
                                }
                            });
                        let r = ui.add_enabled(
                            p.excluded.len() < net::MAX_RANGE,
                            button("Add excluded address"),
                        );
                        metrics.dhcp_controls.push((
                            "add-exclusion".into(),
                            r.rect,
                            ui.is_enabled() && p.excluded.len() < net::MAX_RANGE,
                        ));
                        if r.clicked() {
                            p.excluded.push(String::new());
                            focus_new_field(
                                ui,
                                scope,
                                &format!("excluded-{}", p.excluded.len() - 1),
                            );
                        }
                    });
                    metrics.dhcp_controls.push((
                        "excluded-header".into(),
                        exclusions.header_response.rect,
                        ui.is_enabled(),
                    ));
                    let reservations = egui::CollapsingHeader::new(format!(
                        "Reservations ({})",
                        p.reservations.len()
                    ))
                    .id_salt("dhcp-reservations")
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .id_salt("reservations")
                            .max_height(controls::height(ui) * 6.)
                            .show(ui, |ui| {
                                let mut remove = None;
                                for (n, value) in p.reservations.iter_mut().enumerate() {
                                    field(
                                        ui,
                                        scope,
                                        &format!("reserved-client-{n}"),
                                        "Supplied client ID",
                                        &mut value.client,
                                        metrics,
                                        focus,
                                    );
                                    field(
                                        ui,
                                        scope,
                                        &format!("reserved-address-{n}"),
                                        "Reserved address",
                                        &mut value.address,
                                        metrics,
                                        focus,
                                    );
                                    let r = ui.add(button("Remove this reservation"));
                                    metrics.dhcp_controls.push((
                                        format!("remove-reservation-{n}"),
                                        r.rect,
                                        ui.is_enabled(),
                                    ));
                                    if r.clicked() {
                                        remove = Some(n);
                                    }
                                }
                                if let Some(n) = remove {
                                    p.reservations.remove(n);
                                }
                            });
                        let r = ui.add_enabled(
                            p.reservations.len() < net::MAX_CLIENTS,
                            button("Add reservation"),
                        );
                        metrics.dhcp_controls.push((
                            "add-reservation".into(),
                            r.rect,
                            ui.is_enabled() && p.reservations.len() < net::MAX_CLIENTS,
                        ));
                        if r.clicked() {
                            p.reservations.push(ReservationDraft {
                                client: String::new(),
                                address: String::new(),
                            });
                            focus_new_field(
                                ui,
                                scope,
                                &format!("reserved-client-{}", p.reservations.len() - 1),
                            );
                        }
                    });
                    metrics.dhcp_controls.push((
                        "reservations-header".into(),
                        reservations.header_response.rect,
                        ui.is_enabled(),
                    ));
                    remove_pool = ui.add(button("Remove selected pool")).clicked();
                });
                if remove_pool {
                    draft.pools.remove(&pool);
                    draft.selected_pool = draft.pools.keys().next().cloned();
                }
            }
            ui.label(RichText::new(format!("Relay policy · {segment_id}")).strong());
            let supplied = &draft.context.segments[&segment_id];
            ui.label(format!(
                "Supplied receiving interface: {} · client-facing relay address: {}",
                option_ip(supplied.server_interface),
                option_ip(supplied.relay_interface)
            ));
            ui.label(format!(
                "Supplied relay path: {} · client unicast path: {}",
                if supplied.relay_to_server {
                    "Available"
                } else {
                    "Unavailable"
                },
                if supplied.client_unicast_to_server {
                    "Available"
                } else {
                    "Unavailable"
                }
            ));
            ui.push_id(("relay-record", &segment_id), |ui| {
                let scope = scope.with(("relay", &segment_id));
                if let Some(relay) = draft.relays.get_mut(&segment_id) {
                    toggle(
                        ui,
                        "relay-enabled",
                        "Enable this relay",
                        &mut relay.enabled,
                        metrics,
                    );
                    field(
                        ui,
                        scope,
                        "relay-target",
                        "Relay server target",
                        &mut relay.target,
                        metrics,
                        focus,
                    );
                    field(
                        ui,
                        scope,
                        "relay-giaddr",
                        "Relay client-facing address",
                        &mut relay.giaddr,
                        metrics,
                        focus,
                    );
                    if ui.add(button("Remove relay policy")).clicked() {
                        draft.relays.remove(&segment_id);
                    }
                } else if ui.add(button("Add relay policy")).clicked() {
                    draft.relays.insert(
                        segment_id.clone(),
                        RelayDraft {
                            enabled: false,
                            target: String::new(),
                            giaddr: String::new(),
                        },
                    );
                }
            });
            let r = ui.add(button("Apply policy"));
            metrics
                .dhcp_controls
                .push(("apply".into(), r.rect, ui.is_enabled()));
            if r.clicked() && ui.is_enabled() {
                attempted = true;
                match draft.snapshot() {
                    Ok(policy) => chosen = Some(Action::ApplyPolicy { policy }),
                    Err(e) => *notice = Some(work::notice(e)),
                }
            }
            Edit {
                attempted: attempted || chosen.is_some(),
                action: chosen,
                ..Default::default()
            }
        },
    )
    .inner
}

fn option_ip(address: Option<std::net::Ipv4Addr>) -> String {
    address.map_or("Not supplied".into(), |a| a.to_string())
}
fn ips(addresses: &[std::net::Ipv4Addr]) -> String {
    if addresses.is_empty() {
        "None supplied".into()
    } else {
        addresses
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}
fn binding(ui: &mut egui::Ui, label: &str, b: Option<&net::Binding>, now: u64) {
    if let Some(b) = b {
        ui.label(format!(
            "{label}: {} · {}s remaining · grant {}s",
            b.address,
            b.remaining(now),
            b.granted_seconds
        ));
        ui.label(format!(
            "Held subnet {}/{} · router {} · DNS {}",
            b.parameters.subnet.network,
            b.parameters.subnet.prefix,
            option_ip(b.parameters.router),
            ips(&b.parameters.dns)
        ));
        ui.label(format!(
            "Granted at {} · renew at {} · rebind at {} · expires at {}",
            b.acquired_at, b.renew_at, b.rebind_at, b.expires_at
        ));
    } else {
        ui.label(format!("{label}: None"));
    }
}
