//! Native rollout previews, installed policy, and actual virtual path observations.
mod evidence_and_plan;
mod host_security;
mod site_paths;
mod vlan_gateway;
mod nfs_operator_page;
mod route_reading;
mod ospf_broadcast;
pub(super) use vlan_gateway::last_vlan_observation;

use super::*;
use dlms_learning_lab::{
    automation_workshop::{
        change_rollout::{Change, PortSettings},
        source,
        workspace::{Action, Scope, Task, Work},
    },
    engine::State,
};

type Metrics = super::linux_v6_page::Metrics;

fn guide(ui: &mut egui::Ui, lesson: Option<&Scenario>, index: usize) {
    if let Some(lesson) = lesson {
        lesson_presentation::practice_step(ui, lesson, index);
    }
}

fn apply(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = work.apply(task, action) {
        *notice = Some(error);
    }
}

fn row(ui: &mut egui::Ui, label: &str, body: impl FnOnce(&mut egui::Ui)) {
    controls::form_row(ui, label, |ui, width| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(width);
                body(ui);
            },
        );
    });
}

fn select<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    id: egui::Id,
    key: &str,
    label: &str,
    value: &mut T,
    choices: &[(T, String)],
    m: &mut Metrics,
) {
    controls::form_row(ui, label, |ui, width| {
        let text = choices
            .iter()
            .find(|(candidate, _)| candidate == value)
            .map_or("Select", |(_, label)| label.as_str());
        let menu = controls::dropdown(ui, id.with(key), width)
            .selected_text(text)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, (item, label)) in choices.iter().enumerate() {
                    let r = ui.selectable_value(value, item.clone(), label);
                    m.record(ui, &format!("{key}-{index}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &menu.response);
        m.record(ui, key, &menu.response);
    });
}

fn status(settings: PortSettings) -> String {
    format!(
        "{} · VLAN {}",
        if settings.up { "Up" } else { "Down" },
        settings.vlan
    )
}

fn setting_fields(ui: &mut egui::Ui, settings: &mut PortSettings, metrics: &mut Metrics) {
    controls::form_row(ui, "VLAN", |ui, _| {
        let r = ui.add(egui::DragValue::new(&mut settings.vlan).range(1..=4094));
        metrics.record(ui, "rollout-vlan", &r);
    });
    let r = ui.checkbox(&mut settings.up, "Port up");
    metrics.record(ui, "rollout-up", &r);
}

fn rollout_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Rollout { supplied } = task else {
        return;
    };
    let Scope::Rollout { draft, records, .. } = &work.current else {
        return;
    };
    let current = match supplied.replay(records) {
        Ok(current) => current,
        Err(error) => {
            lesson_presentation::prose(ui, &error);
            return;
        }
    };

    ui.strong("Work-order ports");
    for desk in &supplied.approved {
        let position = &supplied.spec.ports[&desk.slot];
        row(ui, &desk.host, |ui| {
            lesson_presentation::monospace(
                ui,
                &format!("{}/{}", position.switch, position.port),
                true,
            );
        });
    }
    row(ui, "Reserve port", |ui| {
        lesson_presentation::monospace(ui, &supplied.reserved.slot, true);
    });
    row(ui, "Required service", |ui| {
        lesson_presentation::monospace(ui, &supplied.intended_peer, true);
    });
    row(ui, "Separate service", |ui| {
        lesson_presentation::monospace(ui, &supplied.isolated_peer, true);
    });

    ui.separator();
    guide(ui, practice, 0);
    let mut next = draft.clone();
    let modes = [
        (0, "Device port edit".into()),
        (1, "Shared profile change".into()),
        (2, "Bind to profile".into()),
        (3, "Reconcile".into()),
    ];
    let previous = match next {
        Change::Individual { .. } => 0,
        Change::SharedProfile { .. } => 1,
        Change::Rebind { .. } => 2,
        Change::Reconcile => 3,
    };
    let mut mode = previous;
    select(
        ui,
        id,
        "rollout-method",
        "Change method",
        &mut mode,
        &modes,
        m,
    );
    let slots: Vec<_> = supplied
        .spec
        .ports
        .keys()
        .map(|name| (name.clone(), name.clone()))
        .collect();
    let profiles: Vec<_> = current
        .inventory
        .desired
        .profiles
        .keys()
        .map(|name| (name.clone(), name.clone()))
        .collect();
    if mode != previous {
        let slot = supplied.approved[0].slot.clone();
        let profile = current.inventory.desired.bindings[&slot].profile.clone();
        let settings = current
            .inventory
            .settings(&slot)
            .unwrap_or(PortSettings { up: false, vlan: 1 });
        next = match mode {
            0 => Change::Individual { slot, settings },
            1 => Change::SharedProfile { profile, settings },
            2 => Change::Rebind { slot, profile },
            _ => Change::Reconcile,
        };
    }
    match &mut next {
        Change::Individual { slot, settings } => {
            select(ui, id, "rollout-slot", "Port resource", slot, &slots, m);
            setting_fields(ui, settings, m);
        }
        Change::SharedProfile { profile, settings } => {
            select(ui, id, "rollout-profile", "Profile", profile, &profiles, m);
            setting_fields(ui, settings, m);
        }
        Change::Rebind { slot, profile } => {
            select(ui, id, "rollout-slot", "Port resource", slot, &slots, m);
            select(ui, id, "rollout-profile", "Profile", profile, &profiles, m);
        }
        Change::Reconcile => {
            lesson_presentation::prose(
                ui,
                "Reapply the controller's desired profiles and clear direct device overrides.",
            );
        }
    }
    if next != *draft {
        apply(
            task,
            work,
            Action::RolloutDraft {
                draft: next.clone(),
            },
            notice,
        );
    }
    ui.horizontal_wrapped(|ui| {
        let preview = ui.add(button("Preview"));
        m.record(ui, "rollout-preview", &preview);
        if preview.clicked() {
            apply(task, work, Action::Preview, notice);
        }
        let enabled = matches!(&work.current, Scope::Rollout { preview: true, .. });
        let apply_change = ui.add_enabled(enabled, primary(ui, "Apply this change"));
        m.record(ui, "rollout-apply", &apply_change);
        if apply_change.clicked() {
            apply(task, work, Action::Apply, notice);
        }
    });
    let Scope::Rollout {
        draft,
        records,
        preview,
    } = &work.current
    else {
        return;
    };
    let Ok(current) = supplied.replay(records) else {
        return;
    };
    if *preview && let Ok(plan) = current.inventory.preview(&supplied.spec, draft) {
        ui.strong("Preview result");
        if plan.affected.is_empty() {
            lesson_presentation::prose(
                ui,
                "This change leaves the current settings and bindings intact.",
            );
        }
        for item in plan.affected {
            row(ui, &item.slot, |ui| {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Before · {}. After · {}.",
                        status(item.before),
                        status(item.after)
                    ),
                );
                if item.origin_changed {
                    ui.label("The source of this port's settings changes.");
                }
            });
        }
    }

    ui.separator();
    guide(ui, practice, 1);
    ui.strong("Actual port and reachability observations");
    let mut verify = None;
    for (index, desk) in supplied.approved.iter().enumerate() {
        row(ui, &desk.slot, |ui| {
            if let Ok(settings) = current.inventory.settings(&desk.slot) {
                ui.label(status(settings));
            }
            let path =
                current
                    .inventory
                    .visible(&supplied.spec, &desk.host, &supplied.intended_peer);
            lesson_presentation::prose(
                ui,
                match path {
                    Ok(true) => "The required service is reachable in both directions.",
                    Ok(false) => "The required service is unreachable with the current settings.",
                    Err(ref error) => error,
                },
            );
            let r = ui.add(button("Verify pilot"));
            m.record(ui, &format!("rollout-verify-{index}"), &r);
            if r.clicked() {
                verify = Some(index);
            }
        });
    }
    if let Some(index) = verify {
        apply(task, work, Action::VerifyPilot { index }, notice);
    }
    row(ui, &supplied.reserved.slot, |ui| {
        if let Ok(settings) = current.inventory.settings(&supplied.reserved.slot) {
            ui.label(status(settings));
        }
    });
    ui.label(if current.pilot_verified {
        "Pilot verified in the recorded rollout."
    } else {
        "Pilot needs one approved controller-managed path while the other approved port remains unchanged."
    });
    guide(ui, practice, 2);
}

fn intent_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Intent { supplied } = task else {
        return;
    };
    let Scope::Intent { request, allow, .. } = &work.current else {
        return;
    };
    let mut proposed = *request;
    let mut choice = *allow;
    let options: Vec<_> = supplied
        .spec
        .edges
        .iter()
        .enumerate()
        .map(|(n, edge)| (n, edge.name.clone()))
        .collect();
    if let Some(lesson) = practice {
        guide(ui, Some(lesson), 0);
        if let Scope::Intent { current, .. } = &work.current
            && let Ok(trace) = supplied.spec.trace(*current, supplied.trials[0].exchange())
        {
            let desired = if trace.desired { "Allow" } else { "Exclude" };
            let installed = if trace.installed { "Allow" } else { "Exclude" };
            let transport = if trace.carrier.is_ok() {
                "Returned"
            } else {
                "Failed"
            };
            row(ui, "First actual exchange", |ui| {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Desired · {desired} | Installed · {installed} | Underlay · {transport}"
                    ),
                );
            });
        }
        ui.separator();
    }
    ui.strong("Controller and device policy");
    guide(ui, practice, 1);
    select(
        ui,
        id,
        "intent-from",
        "From",
        &mut proposed.from,
        &options,
        m,
    );
    select(ui, id, "intent-to", "To", &mut proposed.to, &options, m);
    select(
        ui,
        id,
        "intent-allow",
        "Action",
        &mut choice,
        &[(true, "Allow".into()), (false, "Exclude".into())],
        m,
    );
    if proposed != *request || choice != *allow {
        apply(
            task,
            work,
            Action::IntentDraft {
                request: proposed,
                allow: choice,
            },
            notice,
        );
    }
    ui.horizontal_wrapped(|ui| {
        for (label, key, action) in [
            ("Submit intent", "intent-submit", Action::Submit),
            ("Publish rules", "intent-publish", Action::Publish),
        ] {
            let r = ui.add(button(label));
            m.record(ui, key, &r);
            if r.clicked() {
                apply(task, work, action, notice);
            }
        }
    });

    ui.separator();
    guide(ui, practice, 2);
    let Scope::Intent { current, .. } = &work.current else {
        return;
    };
    let mut up = current.uplink_up;
    let r = ui.checkbox(&mut up, "Service uplink up");
    m.record(ui, "intent-uplink", &r);
    if r.changed() {
        apply(task, work, Action::Uplink { up }, notice);
    }
    let Scope::Intent { current, .. } = &work.current else {
        return;
    };
    ui.separator();
    ui.strong("Actual path observations");
    for trial in &supplied.trials {
        let label = format!(
            "{} to {}",
            supplied.spec.edges[trial.from].name, supplied.spec.edges[trial.to].name
        );
        row(ui, &label, |ui| {
            match supplied.spec.trace(*current, trial.exchange()) {
                Ok(trace) => {
                    let desired = if trace.desired { "Allow" } else { "Exclude" };
                    let installed = if trace.installed { "Allow" } else { "Exclude" };
                    lesson_presentation::prose(
                        ui,
                        &format!("Desired · {desired} | Installed · {installed}"),
                    );
                    if let Err(error) = &trace.carrier {
                        lesson_presentation::prose(ui, &format!("Underlay · {error}"));
                    } else {
                        lesson_presentation::prose(ui, "Underlay · returned");
                    }
                    ui.strong(if trace.forwarded() {
                        "Exchange forwarded"
                    } else {
                        "Exchange did not forward"
                    });
                }
                Err(error) => {
                    lesson_presentation::prose(ui, &error);
                }
            }
        });
    }
}

fn result(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &Work,
    practice: bool,
    metrics: &mut Metrics,
) -> bool {
    let outcomes = work.outcomes(task);
    let passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    let count = outcomes.len();
    let complete = work.checked && passed == count;
    let headline = ui.strong(if !work.checked {
        "Not checked".into()
    } else if complete && practice {
        "Example verified".into()
    } else if complete {
        "Assignment complete".into()
    } else {
        format!("Needs work · {passed}/{count} requirements met")
    });
    metrics.record(ui, "result", &headline);
    metrics.result = Some((passed, count, headline.rect));
    if work.checked {
        for outcome in outcomes.iter().filter(|row| !row.passed) {
            ui.strong(&outcome.name);
            lesson_presentation::prose(ui, &outcome.detail);
        }
        controls::collapsing_header(ui, "Requirements met")
            .id_salt(id.with("met"))
            .show(ui, |ui| {
                for outcome in outcomes.iter().filter(|row| row.passed) {
                    ui.strong(&outcome.name);
                    lesson_presentation::prose(ui, &outcome.detail);
                }
            });
    }
    complete
}

impl LabApp {
    pub(super) fn automation_workshop_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let before_header = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if before_header != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let attempt = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario)
            .and_then(|d| source::context(attempt, &scenario).map(|()| d))
        {
            Ok(definition) => definition,
            Err(error) => {
                lesson_presentation::prose(ui, &error);
                return;
            }
        };
        let State::AutomationWorkshop { simulation } = &attempt.state else {
            return;
        };
        let original = (**simulation).clone();
        let mut session = original.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            session.view
        };
        let m = &mut self.metrics.automation_workshop;
        lesson_presentation::rich(ui, RichText::new(&scenario.title).size(22.).strong());
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (candidate, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == candidate));
                    m.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = candidate;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            session.view = view;
        }
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            m.record(ui, "introduction-content", &intro.objective);
            let example_title = match &definition.assignment {
                Task::Rollout { .. } => "Example rollout at the supply room",
                Task::Intent { .. } => "Example observations from a different fabric",
                Task::ClaimReview { .. } => "Example network diagnosis and forecast",
                Task::DesiredConfig { .. } => "Example plan and recovery record",
                Task::SitePaths { .. } => "Example path trials",
                Task::SocketRetirement { .. } => "Example relay retirement",
                Task::KeyedTransfer { .. } => "Example verified zone refresh",
                Task::AccessDefault { .. } => "Example bench desk membership",
                Task::RoutedVlans { .. } => "Example routing and return path",
                Task::NfsExportRetirement { .. } => "Example retiring a training share",
                Task::RouteReading { .. } => "Example preferred route and fallback",
                Task::OspfBroadcast { .. } => "Example broadcast segment and backup promotion",
            };
            lesson_presentation::worked(ui, example_title);
            lesson_presentation::wrapped_worked_commands(ui, &scenario);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            m.record(ui, "try-example", &r);
            if r.clicked() {
                session.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (task, work) = if practice {
                (&definition.example, &mut session.example.work)
            } else {
                (&definition.assignment, &mut session.assignment.work)
            };
            let id = egui::Id::new(("automation-workshop", &scenario.id, practice));
            if !practice {
                let heading = lesson_presentation::assignment_responses(ui, &scenario);
                m.record(ui, "assignment-content", &heading.goal);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                let teaching = practice.then_some(&scenario);
                match task {
                    Task::Rollout { .. } => {
                        rollout_body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::Intent { .. } => {
                        intent_body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::ClaimReview { .. } => evidence_and_plan::claim_body(
                        ui,
                        id,
                        task,
                        work,
                        teaching,
                        m,
                        &mut self.notice,
                    ),
                    Task::DesiredConfig { .. } => evidence_and_plan::config_body(
                        ui,
                        id,
                        task,
                        work,
                        teaching,
                        m,
                        &mut self.notice,
                    ),
                    Task::SitePaths { .. } => {
                        site_paths::body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::SocketRetirement { .. } => host_security::socket_body(
                        ui,
                        id,
                        task,
                        work,
                        teaching,
                        m,
                        &mut self.notice,
                    ),
                    Task::KeyedTransfer { .. } => {
                        host_security::dns_body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::AccessDefault { .. } => {
                        vlan_gateway::defaults_body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::RoutedVlans { .. } => {
                        vlan_gateway::routed_body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::NfsExportRetirement { .. } => nfs_operator_page::body(ui,id,task,work,teaching,m,&mut self.notice),
                    Task::RouteReading { .. } => {
                        route_reading::body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                    Task::OspfBroadcast { .. } => {
                        ospf_broadcast::body(ui, id, task, work, teaching, m, &mut self.notice)
                    }
                }
                ui.separator();
                let check = ui.add(primary(ui, "Check workspace"));
                m.record(ui, "check", &check);
                if check.clicked() {
                    apply(task, work, Action::Check, &mut self.notice);
                }
                if result(ui, id, task, work, practice, m) && !practice {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = ui.add(button("Reset example"));
                    m.record(ui, "reset-example", &r);
                    reset = r.clicked();
                }
            });
            if reset && let Err(error) = session.reset_example(definition) {
                self.notice = Some(error);
            }
            controls::collapsing_header(ui, "Hints and model scope")
                .id_salt(id.with("help"))
                .show(ui, |ui| {
                    if ui.add_enabled(active, button("Show a hint")).clicked() {
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
                    lesson_presentation::prose(ui, &scenario.simplifications);
                    for source in &scenario.sources {
                        ui.hyperlink_to(source, source);
                    }
                });
        }
        if active && session != original {
            match session.bind(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state =
                        State::AutomationWorkshop {
                            simulation: Box::new(session),
                        };
                    self.completion_index.forget(&scenario.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(error) => self.notice = Some(error),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&scenario.id], &scenario);
        if self.metrics.completion {
            self.next_unit_lab(ui, index);
        }
        Self::workspace_footer(ui);
    }
}
