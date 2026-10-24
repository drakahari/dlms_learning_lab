//! Native configuration of the bounded OSPF snapshot; no device session or packets.
use super::*;
use dlms_learning_lab::{
    ospf::{self, NetworkType},
    ospf_teaching::Learning,
    ospf_workspace::{self as model_ospf, Draft},
};
use std::collections::{BTreeMap, BTreeSet};

impl LabApp {
    /// Set only a new/reset attempt's view; never replace a resumed selection.
    pub(super) fn ospf_initial_selection(a: &mut Attempt, s: &pack::Scenario) {
        let engine::Goal::Ospf { requirements, .. } = &s.goal else {
            return;
        };
        let Some(draft) = a.workspace.as_mut().and_then(|w| w.ospf_draft.as_mut()) else {
            return;
        };
        if let Some(at) = requirements.iter().find_map(|r| match r {
            model_ospf::Requirement::Compatibility { local, .. } => Some(local.clone()),
            model_ospf::Requirement::RemoteRoute { router, .. } => {
                let ports = &draft.routers.get(router)?.interfaces;
                // Select context, never infer a solution from required next hops.
                let port = ports.keys().next()?.clone();
                Some(ospf::Endpoint::new(router, &port))
            }
            _ => None,
        }) {
            draft.selected = at;
        }
    }
    pub(super) fn ospf_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.ospf_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_ospf_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.ospf_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn ospf_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let lesson = s
            .ospf_teaching
            .as_ref()
            .expect("validated OSPF introduction");
        ui.heading(&s.title);
        ui.label(RichText::new("Virtual router settings · Configured compatibility and SPF candidates; no live protocol or packets.").small().color(palette(ui).muted));
        if !self.progress.attempts[&s.id].challenge {
            let view = self.ospf_view(i);
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_ospf_view(i, v);
                    }
                }
            });
        }
        match self.ospf_view(i) {
            teaching::View::Introduction => {
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for c in &lesson.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                for a in &lesson.example_actions {
                    ui.label(&a.explanation);
                }
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_ospf_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_ospf_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                ui.label("Practice example · Ungraded; your assignment settings and draft stay separate.");
                ui.label(&lesson.example_description);
                let mut l = self.progress.attempts[&s.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .ospf_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&l).unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        l.fill_example(lesson).expect("validated worked settings");
                        self.focus_input = true;
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        l = Learning::new(lesson).unwrap();
                        l.view = teaching::View::Example;
                        self.focus_input = true;
                    }
                });
                ui.label(RichText::new("Fill changes only the example fields. Check example applies and evaluates them.").small().color(palette(ui).muted));
                ui.push_id(("ospf-example", &s.id), |ui| {
                    if ospf_configuration(
                        ui,
                        &l.simulation,
                        &mut l.draft,
                        &mut self.metrics,
                        &mut self.focus_input,
                        &lesson.requirements,
                    ) {
                        l.checked = false;
                    }
                    let r = ui.add(primary(ui, "Check example"));
                    self.metrics.check = Some(r.rect);
                    let requested = r.clicked() || check_shortcut(ui);
                    if requested && l.apply_check(lesson).is_ok() {
                        l.draft.notice = None;
                    }
                    let result = controls::feedback(ui, requested, |ui| {
                        if l.checked {
                            if !l.draft.matches(&l.simulation)
                                && let Some(message) = &l.draft.notice
                            {
                                ui.colored_label(palette(ui).error, message);
                            }
                            let checks = l.checks(lesson);
                            if checks.iter().all(|c| c.0) {
                                ui.colored_label(
                                    palette(ui).success,
                                    "Example checks pass · No assignment credit",
                                );
                            }
                            ospf_results(ui, &checks);
                        }
                    });
                    self.metrics.feedback = Some(result.response.rect);
                });
                ui.label(format!("Try a change: {}", lesson.try_change.description));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    l.view = teaching::View::Assignment;
                    self.set_ospf_view(i, teaching::View::Assignment);
                }
                self.dirty |= before != serde_json::to_vec(&l).unwrap();
                self.progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .ospf_learning = Some(l);
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let engine::Goal::Ospf { requirements, .. } = &s.goal else {
                    unreachable!()
                };
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let engine::State::Ospf { simulation } = &mut a.state else {
                    unreachable!()
                };
                let w = a.workspace.as_mut().unwrap();
                let mut draft = w.ospf_draft.clone().expect("prepared OSPF draft");
                let before = serde_json::to_vec(&draft).unwrap();
                ui.push_id(("ospf-assignment", &s.id), |ui| {
                    if ospf_configuration(
                        ui,
                        simulation,
                        &mut draft,
                        &mut self.metrics,
                        &mut self.focus_input,
                        requirements,
                    ) {
                        w.checked = false;
                    }
                });
                self.dirty |= before != serde_json::to_vec(&draft).unwrap();
                let r = ui.add(primary(ui, "Check configuration"));
                self.metrics.check = Some(r.rect);
                let requested = r.clicked() || check_shortcut(ui);
                if requested {
                    let engine::Goal::Ospf { initial, .. } = &s.goal else {
                        unreachable!()
                    };
                    if draft.apply(initial, simulation).is_ok() {
                        draft.notice = None;
                    }
                    w.checked = true;
                    self.dirty = true;
                }
                w.ospf_draft = Some(draft);
                self.checked = w.checked;
                let result = controls::feedback(ui, requested, |ui| {
                    if !self.checked {
                        return;
                    }
                    let a = &self.progress.attempts[&s.id];
                    let complete = progress::current_solved(a, &s);
                    self.metrics.completion = complete;
                    if complete {
                        ui.colored_label(
                            palette(ui).success,
                            "Objective complete · All current configuration checks pass.",
                        );
                    }
                    let draft = a.workspace.as_ref().unwrap().ospf_draft.as_ref().unwrap();
                    let engine::State::Ospf { simulation } = &a.state else {
                        unreachable!()
                    };
                    if !draft.matches(simulation) {
                        if let Some(message) = &draft.notice {
                            ui.colored_label(palette(ui).error, message);
                        }
                        ui.colored_label(
                            palette(ui).error,
                            "Draft not applied · Correct the field values and check again.",
                        );
                        return;
                    }
                    ospf_results(ui, &engine::checks(&a.state, &s.goal));
                    if complete {
                        if !a.challenge {
                            ui.label(&s.explanation);
                        }
                        self.next_unit_lab(ui, i);
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
            }
        }
    }
    pub(super) fn ospf_help(&mut self, ui: &mut egui::Ui, i: usize) {
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
        egui::CollapsingHeader::new("Why these settings work").show(ui, |ui| {
            ui.label(&s.explanation);
        });
        egui::CollapsingHeader::new("Worked reference · reveal on request").show(ui, |ui| {
            let mut reference = s.initial.clone();
            for action in &s.solution { engine::execute_for_goal(&mut reference, &s.goal, action).expect("validated reference"); }
            let engine::State::Ospf { simulation } = reference else { unreachable!() };
            for (router, c) in &simulation.configuration().routers {
                ui.label(RichText::new(format!("{router}: process {} · {}, area {}, router ID {}", c.process.process_id, if c.process.enabled {"Enabled"} else {"Disabled"}, c.process.area, c.process.router_id.map_or("Unconfigured".into(), |n| n.to_string()))).strong());
                for (port, p) in &c.interfaces {
                    ui.label(format!("{port}: {} · {} · {} · cost {} · Hello {}s / Dead {}s", if p.enabled {"OSPF enabled"} else {"OSPF disabled"}, if p.passive {"Passive"} else {"Active"}, match p.network_type {NetworkType::PointToPoint=>"Point to point",NetworkType::Broadcast=>"Broadcast"}, p.cost, p.hello_seconds, p.dead_seconds));
                }
            }
            ui.label("Reference values only; your draft is unchanged. Equivalent settings are accepted where the task allows them.");
        });
        egui::CollapsingHeader::new("Simulation scope & references").show(ui, |ui| {
            ui.label(model_ospf::HELP);
            ui.label(&s.simplifications);
            for source in &s.sources {
                ui.add(egui::Label::new(source).wrap());
            }
        });
    }
}
fn check_shortcut(ui: &egui::Ui) -> bool {
    ui.is_enabled() && ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Enter))
}
fn ospf_results(ui: &mut egui::Ui, checks: &[(bool, String)]) {
    let passed = checks.iter().filter(|c| c.0).count();
    if passed < checks.len() {
        ui.label(format!(
            "{passed} of {} current objective checks pass.",
            checks.len()
        ));
    }
    for (_, message) in checks.iter().filter(|c| !c.0) {
        ui.add(
            egui::Label::new(
                RichText::new(format!("Needs attention · {message}")).color(palette(ui).error),
            )
            .wrap(),
        );
    }
    if passed > 0 {
        egui::CollapsingHeader::new(format!("View {passed} passing objective checks")).show(
            ui,
            |ui| {
                for (_, message) in checks.iter().filter(|c| c.0) {
                    ui.add(egui::Label::new(format!("Pass · {message}")).wrap());
                }
            },
        );
    }
}
fn ospf_configuration(
    ui: &mut egui::Ui,
    state: &ospf::State,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
    requirements: &[model_ospf::Requirement],
) -> bool {
    ospf_diagram(ui, state, metrics);
    let before = draft.routers.clone();
    let row = ui.text_style_height(&egui::TextStyle::Body);
    let label_w = (row * 5.).min(ui.available_width() * 0.35);
    let value_w = (row * 10. + 24.).min((ui.available_width() - label_w - controls::GAP).max(60.));
    let picker_w = label_w + controls::GAP + value_w;
    ui.horizontal_wrapped(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(picker_w, controls::height(ui)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                controls::label(ui, "Router", label_w);
                let old = draft.selected.router.clone();
                let r = egui::ComboBox::from_id_salt("ospf-router")
                    .selected_text(&old)
                    .truncate()
                    .width(value_w)
                    .show_ui(ui, |ui| {
                        for id in draft.routers.keys() {
                            let r = ui.selectable_value(&mut draft.selected.router, id.clone(), id);
                            metrics.ospf_router_items.push((id.clone(), r.rect));
                        }
                    });
                metrics.ospf_router = Some(r.response.rect);
                name_control(&r.response, "Virtual router to configure");
                if old != draft.selected.router {
                    draft.selected.interface = draft.routers[&draft.selected.router]
                        .interfaces
                        .keys()
                        .next()
                        .unwrap()
                        .clone();
                    *focus = true;
                }
            },
        );
        ui.allocate_ui_with_layout(
            egui::vec2(picker_w, controls::height(ui)),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                controls::label(ui, "Interface", label_w);
                let old = draft.selected.interface.clone();
                let r = egui::ComboBox::from_id_salt("ospf-interface")
                    .selected_text(&old)
                    .truncate()
                    .width(value_w)
                    .show_ui(ui, |ui| {
                        for id in draft.routers[&draft.selected.router].interfaces.keys() {
                            let r =
                                ui.selectable_value(&mut draft.selected.interface, id.clone(), id);
                            metrics.ospf_interface_items.push((id.clone(), r.rect));
                        }
                    });
                metrics.ospf_interface = Some(r.response.rect);
                name_control(&r.response, "Virtual interface to configure");
                if old != draft.selected.interface {
                    *focus = true;
                }
            },
        );
    });
    // The same child hierarchy at both widths retains widget focus/undo identity.
    let total = ui.available_width();
    let wide = total > row * 45.;
    let left = if wide {
        (total - controls::GAP) * 0.64
    } else {
        total
    };
    let right = if wide {
        total - left - controls::GAP
    } else {
        total
    };
    let scope = ui.id().with("ospf-field-values");
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("ospf-workbench")
            .layout(if wide {
                egui::Layout::left_to_right(egui::Align::Min)
            } else {
                egui::Layout::top_down(egui::Align::Min)
            }),
        |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left, 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(left);
                    ospf_fields(ui, draft, metrics, focus, scope);
                },
            );
            ui.allocate_ui_with_layout(
                egui::vec2(right, 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(right);
                    ospf_current(ui, state, &draft.selected, requirements);
                },
            );
        },
    );
    if !draft.matches(state) {
        ui.label(
            RichText::new(
                "Unapplied field changes · Check configuration applies all router fields together.",
            )
            .small()
            .color(palette(ui).muted),
        );
    }
    before != draft.routers
}
fn ospf_fields(
    ui: &mut egui::Ui,
    draft: &mut Draft,
    metrics: &mut Metrics,
    focus: &mut bool,
    scope: egui::Id,
) {
    let router = draft.selected.router.clone();
    let port = draft.selected.interface.clone();
    let r = draft.routers.get_mut(&router).unwrap();
    let row = ui.text_style_height(&egui::TextStyle::Body);
    let total = ui.available_width();
    let paired = total >= 2. * (row * 13. + 36.) + controls::GAP;
    let width = if paired {
        (total - controls::GAP) / 2.
    } else {
        total
    };
    let label_w = (row * 5.).min(width * 0.35);
    let value_w = (row * 10. + 24.).min((width - label_w - controls::GAP).max(40.));
    let mut first = true;
    ui.scope_builder(
        egui::UiBuilder::new()
            .id_salt("ospf-setting-groups")
            .layout(if paired {
                egui::Layout::left_to_right(egui::Align::Min)
            } else {
                egui::Layout::top_down(egui::Align::Min)
            }),
        |ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(width);
                    ui.label(RichText::new("Process settings").strong());
                    egui::Grid::new(("ospf-process", &router))
                        .num_columns(2)
                        .min_row_height(controls::height(ui))
                        .spacing(egui::vec2(controls::GAP, 8.))
                        .show(ui, |ui| {
                            controls::label(ui, "Process", label_w);
                            controls::cell(ui, value_w, |ui| {
                                let x = ui.checkbox(&mut r.enabled, "Enabled");
                                name_control(&x, format!("{router}: OSPF process enabled"));
                                metrics
                                    .ospf_toggles
                                    .push((format!("{router}:enabled"), x.rect));
                            });
                            ui.end_row();
                            ospf_field(
                                ui,
                                "Router ID",
                                &mut r.router_id,
                                "32-bit identity",
                                &format!("{router}:router-id"),
                                label_w,
                                value_w,
                                metrics,
                                focus,
                                &mut first,
                                scope,
                            );
                            ui.end_row();
                            ospf_field(
                                ui,
                                "Process ID",
                                &mut r.process_id,
                                "Local number",
                                &format!("{router}:process-id"),
                                label_w,
                                value_w,
                                metrics,
                                focus,
                                &mut first,
                                scope,
                            );
                            ui.end_row();
                            ospf_field(
                                ui,
                                "Area",
                                &mut r.area,
                                "0",
                                &format!("{router}:area"),
                                label_w,
                                value_w,
                                metrics,
                                focus,
                                &mut first,
                                scope,
                            );
                            ui.end_row();
                        });
                },
            );
            let p = r.interfaces.get_mut(&port).unwrap();
            ui.allocate_ui_with_layout(
                egui::vec2(width, 0.),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ui.set_width(width);
                    ui.label(RichText::new(format!("{router} / {port}")).strong());
                    egui::Grid::new(("ospf-port", &router, &port))
                        .num_columns(2)
                        .min_row_height(controls::height(ui))
                        .spacing(egui::vec2(controls::GAP, 8.))
                        .show(ui, |ui| {
                            controls::label(ui, "Interface", label_w);
                            controls::cell(ui, value_w, |ui| {
                                let r = ui.checkbox(&mut p.up, "Up");
                                name_control(&r, format!("{router}/{port}: interface up"));
                                metrics
                                    .ospf_toggles
                                    .push((format!("{router}/{port}:up"), r.rect));
                            });
                            ui.end_row();
                            for (label, value, hint, key) in [
                                (
                                    "IP address",
                                    &mut p.address,
                                    "Numbered IPv4 address",
                                    "address",
                                ),
                                ("Prefix", &mut p.prefix, "1–30", "prefix"),
                            ] {
                                ospf_field(
                                    ui,
                                    label,
                                    value,
                                    hint,
                                    &format!("{router}/{port}:{key}"),
                                    label_w,
                                    value_w,
                                    metrics,
                                    focus,
                                    &mut first,
                                    scope,
                                );
                                ui.end_row();
                            }
                            controls::label(ui, "OSPF", label_w);
                            controls::cell(ui, value_w, |ui| {
                                let r = ui.checkbox(&mut p.enabled, "Enabled");
                                name_control(&r, format!("{router}/{port}: OSPF enabled"));
                                metrics
                                    .ospf_toggles
                                    .push((format!("{router}/{port}:enabled"), r.rect));
                            });
                            ui.end_row();
                            controls::label(ui, "Peering", label_w);
                            controls::cell(ui, value_w, |ui| {
                                let r = ui.checkbox(&mut p.passive, "Passive · advertise only");
                                name_control(
                                    &r,
                                    format!(
                                        "{router}/{port}: passive advertisement without peering"
                                    ),
                                );
                                metrics
                                    .ospf_toggles
                                    .push((format!("{router}/{port}:passive"), r.rect));
                            });
                            ui.end_row();
                            controls::label(ui, "Network type", label_w);
                            controls::cell(ui, value_w, |ui| {
                                let r = egui::ComboBox::from_id_salt(("ospf-type", &router, &port))
                                    .truncate()
                                    .selected_text(match p.network_type {
                                        NetworkType::PointToPoint => "Point to point",
                                        NetworkType::Broadcast => "Broadcast (passive only)",
                                    })
                                    .width(value_w)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(
                                            &mut p.network_type,
                                            NetworkType::PointToPoint,
                                            "Point to point",
                                        );
                                        ui.selectable_value(
                                            &mut p.network_type,
                                            NetworkType::Broadcast,
                                            "Broadcast · passive LAN only",
                                        );
                                    });
                                name_control(
                                    &r.response,
                                    format!("{router}/{port}: OSPF network type"),
                                );
                                metrics.ospf_type = Some(r.response.rect);
                            });
                            ui.end_row();
                            for (label, value, hint, key) in [
                                ("Cost", &mut p.cost, "1–65535", "cost"),
                                (
                                    "Hello (s)",
                                    &mut p.hello_seconds,
                                    "Positive seconds",
                                    "hello",
                                ),
                                ("Dead (s)", &mut p.dead_seconds, "Positive seconds", "dead"),
                            ] {
                                ospf_field(
                                    ui,
                                    label,
                                    value,
                                    hint,
                                    &format!("{router}/{port}:{key}"),
                                    label_w,
                                    value_w,
                                    metrics,
                                    focus,
                                    &mut first,
                                    scope,
                                );
                                ui.end_row();
                            }
                        });
                },
            );
        },
    );
}
#[allow(clippy::too_many_arguments)]
fn ospf_field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    hint: &str,
    key: &str,
    label_w: f32,
    value_w: f32,
    metrics: &mut Metrics,
    focus: &mut bool,
    first: &mut bool,
    scope: egui::Id,
) {
    controls::label(ui, label, label_w);
    controls::cell(ui, value_w, |ui| {
        let r = ui.add_sized(
            [value_w, controls::height(ui)],
            controls::singleline(ui, value)
                .id(scope.with(key))
                .hint_text(hint)
                .char_limit(model_ospf::MAX_TEXT),
        );
        name_control(&r, format!("{key} — virtual OSPF setting"));
        metrics.ospf_fields.push((key.into(), r.rect, r.id));
        if r.changed() {
            super::workspace::bound(value, model_ospf::MAX_TEXT);
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
    });
}
fn ospf_current(
    ui: &mut egui::Ui,
    state: &ospf::State,
    at: &ospf::Endpoint,
    requirements: &[model_ospf::Requirement],
) {
    ui.label(RichText::new("Applied snapshot").strong());
    let snapshot = state.snapshot().expect("validated bounded OSPF state");
    let r = &snapshot.routers[&at.router];
    let link = r.links.iter().find(|l| l.local == *at).unwrap();
    let port = &state.configuration().routers[&at.router].interfaces[&at.interface];
    ui.label(format!(
        "{} / {}: {}",
        at.router,
        at.interface,
        if port.passive {
            "Passive · peering off by configuration"
        } else if link.compatible {
            "Compatible parameters"
        } else {
            "No compatible peer"
        }
    ));
    if let Some(peer) = &link.peer {
        ui.label(format!("Direct peer: {} / {}", peer.router, peer.interface));
    }
    // Current evidence is visible before checking; it does not reveal a repair.
    for issue in r.issues.iter().take(3) {
        ui.add(
            egui::Label::new(model_ospf::describe_condition(&ospf::Condition::Router {
                router: at.router.clone(),
                issue: issue.clone(),
            }))
            .wrap(),
        );
    }
    for condition in link
        .conditions
        .iter()
        .filter(|c| c.blocking() && !matches!(c, ospf::Condition::Router { .. }))
        .take(3)
    {
        ui.add(egui::Label::new(model_ospf::describe_condition(condition)).wrap());
    }
    let mut wanted = Vec::new();
    for requirement in requirements {
        match requirement {
            model_ospf::Requirement::RemoteRoute {
                router,
                destination,
                ..
            }
            | model_ospf::Requirement::AbsentRemoteRoute {
                router,
                destination,
            } if router == &at.router && !wanted.contains(destination) => wanted.push(*destination),
            _ => {}
        }
    }
    if !wanted.is_empty() {
        ui.label(RichText::new("Routes requested by this task").strong());
        for destination in &wanted {
            match r.route(*destination) {
                Some(route) => ospf_route(ui, route),
                None => {
                    ui.label(format!(
                        "{} · No current remote SPF candidate",
                        destination.text()
                    ));
                }
            }
        }
    } else {
        ui.label("No specific remote-prefix check for this selected router.");
    }
    egui::CollapsingHeader::new(format!("Other current SPF candidates · {}", r.routes.len())).show(
        ui,
        |ui| {
            if r.routes.is_empty() {
                ui.label("None currently");
            }
            for route in &r.routes {
                if !wanted.contains(&route.destination) {
                    ospf_route(ui, route);
                }
            }
        },
    );
}
fn ospf_route(ui: &mut egui::Ui, route: &ospf::Route) {
    ui.label(format!(
        "{} · cost {}",
        route.destination.text(),
        route.cost
    ));
    ui.add(
        egui::Label::new(format!(
            "Via {}",
            route
                .next_hops
                .iter()
                .map(|h| format!("{} ({})", h.interface, h.via))
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .wrap(),
    );
}

fn ospf_diagram(ui: &mut egui::Ui, state: &ospf::State, metrics: &mut Metrics) {
    let topology = state.topology();
    let mut peers: BTreeMap<&str, BTreeSet<&str>> = topology
        .devices
        .keys()
        .map(|n| (n.as_str(), BTreeSet::new()))
        .collect();
    let mut segments: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (n, d) in &topology.devices {
        for p in d.interfaces.values() {
            segments.entry(&p.link).or_default().push(n);
        }
    }
    for members in segments.values() {
        if let [a, b] = members.as_slice() {
            peers.get_mut(a).unwrap().insert(b);
            peers.get_mut(b).unwrap().insert(a);
        }
    }
    let first = peers.iter().find(|(_, p)| p.len() == 1).map(|(n, _)| *n);
    let mut path = vec![];
    if peers.len() <= 4
        && peers.values().all(|p| p.len() <= 2)
        && segments.values().all(|members| members.len() <= 2)
        && let Some(mut at) = first
    {
        let mut previous = None;
        loop {
            path.push(at);
            let next = peers[at].iter().copied().find(|n| Some(*n) != previous);
            let Some(next) = next else {
                break;
            };
            if path.contains(&next) {
                break;
            }
            previous = Some(at);
            at = next;
        }
    }
    if path.len() == peers.len() && path.len() >= 2 {
        let labels: Vec<_> = path
            .iter()
            .map(|n| {
                let Some(router) = state.configuration().routers.get(*n) else {
                    let address = topology.devices[*n]
                        .interfaces
                        .values()
                        .next()
                        .and_then(|p| p.address.as_ref())
                        .map_or("No address".into(), |a| {
                            format!("{}/{}", a.address, a.prefix)
                        });
                    return [(*n).to_owned(), address, "Read-only endpoint".into()];
                };
                let c = &router.process;
                [
                    (*n).to_owned(),
                    c.router_id
                        .map_or("No router ID".into(), |n| format!("Router ID {n}")),
                    if c.enabled {
                        "Process enabled".into()
                    } else {
                        "Process disabled".into()
                    },
                ]
            })
            .collect();
        let cards: Vec<_> = labels
            .iter()
            .map(|a| [a[0].as_str(), a[1].as_str(), a[2].as_str()])
            .collect();
        let r = controls::topology_chain(
            ui,
            &cards,
            "Router links · Stub networks are shown in interface settings; no live protocol exchange.",
        );
        metrics.ospf_diagram = Some(r.rect);
    } else {
        let r = egui::CollapsingHeader::new("Links and outgoing costs · applied settings")
            .default_open(true)
            .show(ui, |ui| {
                for link in segments.into_keys() {
                    let endpoints: Vec<_> = topology
                        .devices
                        .iter()
                        .flat_map(|(r, d)| {
                            d.interfaces
                                .iter()
                                .filter(|(_, p)| p.link == link)
                                .map(move |(p, i)| (r, p, i))
                        })
                        .collect();
                    if let [(a, ap, _), (b, bp, _)] = endpoints.as_slice() {
                        let ac = state
                            .configuration()
                            .routers
                            .get(*a)
                            .map(|r| &r.interfaces[*ap]);
                        let bc = state
                            .configuration()
                            .routers
                            .get(*b)
                            .map(|r| &r.interfaces[*bp]);
                        if let (Some(ac), Some(bc)) = (ac, bc) {
                            ui.add(
                                egui::Label::new(format!(
                                    "{a}/{ap} <-> {b}/{bp} · costs {a} to {b}: {}; {b} to {a}: {}",
                                    ac.cost, bc.cost
                                ))
                                .wrap(),
                            );
                        } else {
                            ui.add(
                                egui::Label::new(format!(
                                    "{a}/{ap} — {b}/{bp} · LAN attachment, not an OSPF peer"
                                ))
                                .wrap(),
                            );
                        }
                    } else if let [(r, p, i)] = endpoints.as_slice() {
                        if let Some(c) = state.configuration().routers.get(*r) {
                            ui.add(
                                egui::Label::new(format!(
                                    "{r}/{p} · attached {} · LAN cost {}",
                                    i.address.as_ref().map_or("No address".into(), |a| format!(
                                        "{}/{}",
                                        a.address, a.prefix
                                    )),
                                    c.interfaces[*p].cost
                                ))
                                .wrap(),
                            );
                        }
                    } else {
                        ui.label(format!(
                            "{link}: {} attachments · inspect selected interfaces",
                            endpoints.len()
                        ));
                    }
                }
            });
        metrics.ospf_diagram = Some(r.header_response.rect);
    }
}
