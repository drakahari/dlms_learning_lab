//! Native evidence and policy workspaces using only the supplied model.
use super::*;
mod credentials_vpn_page;
use dlms_learning_lab::{
    engine::State,
    security_investigation::{
        access_plan::{Access, Reports},
        change_evidence::{self, RecordKind},
        source,
        workspace::{Action, Scope, Task, Work},
    },
};
type Metrics = super::linux_v6_page::Metrics;

fn guide(ui: &mut egui::Ui, s: Option<&Scenario>, n: usize) {
    if let Some(s) = s {
        lesson_presentation::practice_step(ui, s, n);
    }
}
fn apply(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = work.apply(task, action) {
        *notice = Some(error);
    }
}

fn choose<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    selected: &mut T,
    choices: &[(T, String)],
    m: &mut Metrics,
    key: &str,
) {
    controls::form_row(ui, label, |ui, width| {
        let text = choices
            .iter()
            .find(|(value, _)| value == selected)
            .map_or("Select", |(_, label)| label.as_str());
        let response = controls::dropdown(ui, id.with(key), width)
            .selected_text(text)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (n, (value, option)) in choices.iter().enumerate() {
                    let r = ui.selectable_value(selected, *value, option);
                    m.record(ui, &format!("{key}-{n}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &response.response);
        m.record(ui, key, &response.response);
    });
}

fn facts_row<R>(
    ui: &mut egui::Ui,
    label: &str,
    facts: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    controls::form_row(ui, label, |ui, width| {
        ui.allocate_ui_with_layout(
            egui::vec2(width, 0.),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(width);
                facts(ui)
            },
        )
        .inner
    })
}

fn sentence(record: &RecordKind) -> String {
    match record {
        RecordKind::TicketApproved { revision } => format!("Ticket approved revision {revision}"),
        RecordKind::RevisionApplied { revision } => format!("Applied revision {revision}"),
        RecordKind::AdmissionObserved { trial, admitted } => format!(
            "Client trial: {trial}. {}.",
            if *admitted {
                "Reached sign-in boundary"
            } else {
                "Denied before sign-in"
            }
        ),
        RecordKind::SuccessfulSignin { trial } => format!("Successful sign-in recorded: {trial}"),
        RecordKind::UnrelatedMaintenance => "Routine maintenance note".into(),
    }
}

fn approved_clients(
    ui: &mut egui::Ui,
    task: &dlms_learning_lab::security_investigation::workspace::ChangeTask,
) {
    ui.strong("Approved management clients");
    for case in task.trials.iter().filter(|case| case.intended_admission) {
        if let Some(endpoint) = task
            .network
            .clients
            .iter()
            .find(|client| client.id == case.client)
            && let Some(address) = task
                .network
                .topology
                .devices
                .get(&endpoint.binding.device)
                .and_then(|device| device.interfaces.get(&endpoint.binding.interface))
                .and_then(|port| port.address.as_ref())
        {
            controls::form_row(ui, &case.name, |ui, _| {
                lesson_presentation::monospace(ui, &address.address.to_string(), true)
            });
        }
    }
}

fn change_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Change { supplied } = task else {
        return;
    };
    controls::form_row(ui, "Affected relay", |ui, _| {
        lesson_presentation::monospace(ui, &supplied.network.management.device, true)
    });
    ui.strong("Supplied change journal");
    for record in &supplied.journal {
        let label = format!("Record {} · minute {}", record.id, record.minute);
        facts_row(ui, &label, |ui| {
            lesson_presentation::rich(ui, RichText::new(&record.device).strong());
            lesson_presentation::prose(ui, &sentence(&record.kind));
        });
    }
    ui.separator();
    guide(ui, guidance, 1);
    let Scope::Change { report, .. } = &work.current else {
        return;
    };
    let before = report.clone();
    let mut report = before.clone();
    let mut records = vec![(None, "Select a journal record".into())];
    records.extend(supplied.journal.iter().map(|row| {
        (
            Some(row.id),
            format!("{} · {}", row.id, sentence(&row.kind)),
        )
    }));
    ui.strong("Evidence report");
    for (label, value, key) in [
        ("Approval record", &mut report.approval, "approval"),
        (
            "Applied revision record",
            &mut report.application,
            "applied-record",
        ),
        (
            "Restricted client observation",
            &mut report.observed_extra_access,
            "observation",
        ),
    ] {
        choose(ui, id, label, value, &records, m, key);
    }
    choose(
        ui,
        id,
        "Successful sign-in for that client",
        &mut report.successful_use_recorded,
        &[
            (None, "Select an answer".into()),
            (Some(true), "Recorded".into()),
            (Some(false), "No matching record".into()),
        ],
        m,
        "signin",
    );
    if report != before {
        apply(task, work, Action::Report { report }, notice);
    }
    ui.separator();
    guide(ui, guidance, 2);
    approved_clients(ui, supplied);
    let Scope::Change { draft, .. } = &work.current else {
        return;
    };
    ui.strong("Virtual management policy");
    let input = shared_libraries_page::configuration_input(
        ui,
        draft,
        id.with("policy"),
        8192,
        ui.is_enabled(),
        "Edit the supplied standard source list",
    );
    m.record(ui, "policy", &input.response);
    if let Some(contents) = input.value {
        apply(task, work, Action::Draft { contents }, notice);
    }
    if input.rejected {
        ui.label("The draft exceeded the editor limit. Your previous text was kept.");
    }
    let button = ui.add(button("Apply virtual policy"));
    m.record(ui, "apply-policy", &button);
    if button.clicked() {
        apply(task, work, Action::Apply, notice);
    }
    let Scope::Change { draft, applied, .. } = &work.current else {
        return;
    };
    ui.label(if draft == applied {
        "The trials use this applied policy."
    } else {
        "The draft has edits. The trials use the earlier policy."
    });
    ui.strong("Actual client outcomes");
    match supplied
        .parse(applied)
        .and_then(|p| change_evidence::trials(&supplied.network, &p, &supplied.trials))
    {
        Ok(results) => {
            for (client, result) in supplied.trials.iter().zip(results) {
                facts_row(ui, &client.name, |ui| {
                    lesson_presentation::prose(
                        ui,
                        if result.reached {
                            "Reached the sign-in boundary"
                        } else {
                            "Denied before sign-in"
                        },
                    );
                    lesson_presentation::prose(ui, &result.detail);
                });
            }
        }
        Err(error) => {
            lesson_presentation::prose(ui, &error);
        }
    }
}

fn facility_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Facility { supplied } = task else {
        return;
    };
    let Scope::Facility { plan: before } = work.current else {
        return;
    };
    let mut plan = before;
    let choices = [
        (Access::Denied, "No access".into()),
        (Access::StaffedOnly, "Staffed hours".into()),
        (Access::AnyWindow, "Any window".into()),
        (Access::StaffedWithEscort, "Staffed with escort".into()),
    ];
    choose(
        ui,
        id,
        "Network operator",
        &mut plan.operator,
        &choices,
        m,
        "operator",
    );
    choose(
        ui,
        id,
        "Facilities technician",
        &mut plan.technician,
        &choices,
        m,
        "technician",
    );
    guide(ui, guidance, 1);
    choose(
        ui,
        id,
        "Registered visitor",
        &mut plan.visitor,
        &choices,
        m,
        "visitor",
    );
    ui.separator();
    guide(ui, guidance, 2);
    let r = ui.checkbox(&mut plan.record_denials, "Keep denied access records");
    m.record(ui, "record-denials", &r);
    choose(
        ui,
        id,
        "Staff report destination",
        &mut plan.suspicious_contact_reports,
        &[
            (Reports::FacilityDesk, "Facility desk".into()),
            (Reports::SecurityDesk, "Security desk".into()),
            (Reports::BothDesks, "Both desks".into()),
        ],
        m,
        "report-desk",
    );
    if plan != before {
        apply(task, work, Action::Facility { plan }, notice);
    }
    ui.separator();
    ui.strong("Actual access trials");
    for (index, case) in supplied.trials.iter().enumerate() {
        let result = plan.entry(case.trial);
        let status = if result.recorded {
            "Event recorded."
        } else {
            "No denied-event record."
        };
        facts_row(ui, &case.label, |ui| {
            let r = lesson_presentation::prose(ui, &format!("{} {status}", result.access.detail()));
            m.record(ui, &format!("facility-trial-text-{index}"), &r);
        });
    }
    let destination = if plan.report_reaches_security() {
        "The supplied staff report reaches the security desk."
    } else {
        "The supplied staff report reaches the facility desk."
    };
    lesson_presentation::prose(ui, destination);
}
fn results(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &Work,
    example: bool,
    m: &mut Metrics,
) -> bool {
    let outcomes = work.outcomes(task);
    let passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    let solved = passed == outcomes.len();
    let headline = ui.strong(if !work.checked {
        "Not checked".into()
    } else if solved && example {
        "Example verified".into()
    } else if solved {
        "Assignment complete".into()
    } else {
        format!("Needs work · {passed}/{} requirements met", outcomes.len())
    });
    m.record(ui, "result", &headline);
    m.result = Some((passed, outcomes.len(), headline.rect));
    if !work.checked {
        return false;
    }
    for outcome in outcomes.iter().filter(|o| !o.passed) {
        ui.strong(format!("Needs work · {}", outcome.label));
        lesson_presentation::prose(ui, &outcome.detail);
    }
    controls::collapsing_header(
        ui,
        if solved {
            "Objective results"
        } else {
            "Requirements already met"
        },
    )
    .id_salt(id.with("details"))
    .show(ui, |ui| {
        for outcome in outcomes.iter().filter(|o| o.passed) {
            ui.strong(format!("Passed · {}", outcome.label));
            lesson_presentation::prose(ui, &outcome.detail);
        }
    });
    solved
}

impl LabApp {
    pub(super) fn security_investigation_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, index);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let scenario = self.scenarios[index].clone();
        let original = &self.progress.attempts[&scenario.id];
        let definition = match source::source(&scenario) {
            Ok(d) => d,
            Err(error) => {
                lesson_presentation::prose(ui, &error);
                return;
            }
        };
        if let Err(error) = source::context(original, &scenario) {
            lesson_presentation::prose(ui, &error);
            return;
        }
        let State::SecurityInvestigation { simulation } = &original.state else {
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
        let metrics = &mut self.metrics.security_investigation;
        lesson_presentation::rich(ui, RichText::new(&scenario.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (candidate, name, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == candidate));
                    metrics.record(ui, name, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = candidate;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &scenario, "", Vec::<(String, String)>::new());
            metrics.record(ui, "introduction-content", &intro.objective);
            let label = match &definition.assignment {
                Task::Change { .. } | Task::Facility { .. } => {
                    "Example with different records and roles"
                }
                Task::Credentials { .. } => "Example staff policy",
                Task::VpnDesign { .. } => "Example gateway and remote client paths",
            };
            lesson_presentation::worked(ui, label);
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &scenario));
            metrics.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            metrics.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let practice = view == teaching::View::Example;
            let (task, work) = if practice {
                (&definition.example, &mut next.example.work)
            } else {
                (&definition.assignment, &mut next.assignment.work)
            };
            let id = egui::Id::new(("security-investigation", &scenario.id, practice));
            if !practice {
                let assignment = lesson_presentation::assignment_responses(ui, &scenario);
                metrics.record(ui, "assignment-content", &assignment.goal);
            } else if matches!(task, Task::Change { .. } | Task::Facility { .. }) {
                lesson_presentation::practice_step(ui, &scenario, 0);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                let guidance = practice.then_some(&scenario);
                match task {
                    Task::Change { .. } => {
                        change_ui(ui, id, task, work, guidance, metrics, &mut self.notice)
                    }
                    Task::Facility { .. } => {
                        facility_ui(ui, id, task, work, guidance, metrics, &mut self.notice)
                    }
                    Task::Credentials { .. } => credentials_vpn_page::credentials_ui(
                        ui,
                        id,
                        task,
                        work,
                        guidance,
                        metrics,
                        &mut self.notice,
                    ),
                    Task::VpnDesign { .. } => credentials_vpn_page::vpn_ui(
                        ui,
                        id,
                        task,
                        work,
                        guidance,
                        metrics,
                        &mut self.notice,
                    ),
                }
                ui.separator();
                let check = ui.add(primary(ui, "Check workspace"));
                metrics.record(ui, "check", &check);
                if check.clicked() {
                    apply(task, work, Action::Check, &mut self.notice);
                }
                if results(ui, id, task, work, practice, metrics) && !practice {
                    lesson_presentation::prose(ui, &scenario.explanation);
                }
                if practice {
                    let r = ui.add(button("Reset example"));
                    metrics.record(ui, "reset-example", &r);
                    reset = r.clicked();
                }
            });
            if reset && let Err(error) = next.reset_example(definition) {
                self.notice = Some(error);
            }
            controls::collapsing_header(ui, "Supported operations, hints and references")
                .id_salt(id.with("help"))
                .show(ui, |ui| {
                    let subset = match task {
                        Task::Change { .. } => "Select records from the supplied journal and edit the provided standard source list. Apply the virtual policy to see the supplied client trials.",
                        Task::Facility { .. } => "The virtual profile checks registered roles, staffed hours and the presence of an escort. The supplied report can go to either desk or both.",
                        Task::Credentials { .. } => "Set policy thresholds and see outcomes for the supplied proposal and sign-in trials. No password or actual identity is needed.",
                        Task::VpnDesign { .. } => "Edit the two endpoint roles and inner ranges. Apply the profiles to test the supplied paths. Use network/prefix notation for each range.",
                    };
                    lesson_presentation::prose(ui, subset);
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
                    for url in &scenario.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }
        if active && next != before {
            match next.bind(definition) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&scenario.id).unwrap().state =
                        State::SecurityInvestigation {
                            simulation: Box::new(next),
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
