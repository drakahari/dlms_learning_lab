//! An Apache configuration editor and named client trials over provided virtual fixtures.
use super::*;
use dlms_learning_lab::{
    apache_https::{
        config, source,
        workspace::{Action, Task, Work, trust_explanation},
    },
    engine::State,
};

type Metrics = super::linux_v6_page::Metrics;

fn apply(task: &Task, work: &mut Work, action: Action, notice: &mut Option<String>) {
    if let Err(error) = work.apply(task, action) {
        *notice = Some(error);
    }
}

fn supplied(ui: &mut egui::Ui, task: &Task) {
    controls::collapsing_header(ui, "Supplied certificate and matching key records")
        .default_open(false)
        .show(ui, |ui| {
            ui.strong("Certificate files · first name is the server certificate");
            for bundle in &task.inventory.bundles {
                lesson_presentation::monospace(ui, &bundle.path, true);
                let entry = bundle
                    .certificate_ids
                    .iter()
                    .map(|id| id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(" → ");
                lesson_presentation::monospace(ui, &entry, true);
            }
            ui.separator();
            ui.strong("Certificate identities");
            for cert in &task.inventory.certificates {
                lesson_presentation::monospace(ui, &cert.id.0, true);
                let dns = cert
                    .dns_names
                    .iter()
                    .map(|name| name.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{}. Issuer {}. Valid on exercise days {}–{}.",
                        if cert.is_ca {
                            "Issuing CA record".into()
                        } else {
                            format!("DNS name {dns}")
                        },
                        cert.issuer.0,
                        cert.first_day,
                        cert.last_day
                    ),
                );
                if !cert.is_ca {
                    lesson_presentation::monospace(
                        ui,
                        &format!("Public key ID · {}", cert.public_key_id.0),
                        true,
                    );
                }
            }
            ui.separator();
            ui.strong("Key files · public identifiers only");
            for key in &task.inventory.keys {
                lesson_presentation::monospace(ui, &key.path, true);
                lesson_presentation::monospace(
                    ui,
                    &format!("Public key ID · {}", key.matching_public_key_id.0),
                    true,
                );
            }
        });
}

fn editor(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Virtual Apache HTTPS configuration");
    let input = shared_libraries_page::configuration_input(
        ui,
        &work.draft,
        id.with("config"),
        config::CONFIG_LIMIT,
        ui.is_enabled(),
        "One directive per line",
    );
    metrics.record(ui, "config", &input.response);
    metrics
        .fields
        .push(("config".into(), input.response.rect, input.response.id));
    if let Some(contents) = input.value {
        apply(task, work, Action::Draft { contents }, notice);
    }
    if input.rejected {
        ui.label("The edit exceeded the configuration limit. Your earlier draft was kept.");
    }
    ui.horizontal_wrapped(|ui| {
        for (name, label, enabled, action) in [
            (
                "save",
                "Save configuration",
                work.draft != work.saved,
                Action::Save,
            ),
            (
                "apply",
                "Apply saved configuration",
                work.saved != work.applied,
                Action::Apply,
            ),
        ] {
            let response = ui.add_enabled(enabled, button(label));
            metrics.record(ui, name, &response);
            if response.clicked() {
                apply(task, work, action, notice);
            }
        }
    });
    ui.label(if work.draft != work.saved {
        "Draft has unsaved edits. Client trials use the applied configuration."
    } else if work.saved != work.applied {
        "Saved settings are ready to apply. Client trials still use the earlier configuration."
    } else {
        "Client trials use these saved and applied settings."
    });
}

fn clients(ui: &mut egui::Ui, id: egui::Id, task: &Task, work: &Work, m: &mut Metrics) {
    ui.strong("Virtual client trials");
    let store = id.with("client-trial");
    let mut selected = ui.data(|d| d.get_temp::<usize>(store)).unwrap_or(0);
    if selected >= task.checks.len() {
        selected = 0;
    }
    let response = controls::dropdown(ui, id.with("client"), ui.available_width().min(480.))
        .selected_text(&task.checks[selected].label)
        .show_ui(ui, |ui| {
            for (index, case) in task.checks.iter().enumerate() {
                let response = ui.selectable_value(&mut selected, index, &case.label);
                m.record(ui, &format!("trial-{index}"), &response);
            }
        });
    m.record(ui, "client", &response.response);
    m.record(
        ui,
        &format!("selected-trial-{selected}"),
        &response.response,
    );
    ui.data_mut(|d| d.insert_temp(store, selected));
    let case = &task.checks[selected];
    let client = &case.client;
    for (label, value) in [
        ("Name to verify", client.hostname.0.clone()),
        (
            "SNI sent",
            client
                .sni
                .as_ref()
                .map_or("No SNI", |name| &name.0)
                .to_owned(),
        ),
        (
            "Trusted anchor IDs",
            client
                .trusted
                .iter()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ),
        ("Exercise day", client.day.to_string()),
    ] {
        controls::form_row(ui, label, |ui, _| {
            lesson_presentation::monospace(ui, &value, true)
        });
    }
    ui.strong("Actual result from the applied configuration");
    match work.inspect(task, case) {
        Err(error) => {
            lesson_presentation::prose(ui, &error);
        }
        Ok(connection) => {
            let result = connection.verification;
            let selected = format!(
                "{} · {}",
                connection.selected.0,
                if connection.used_default {
                    "default host"
                } else {
                    "SNI match"
                }
            );
            for (label, value) in [
                ("Selected host", selected),
                ("Certificate served", result.presented.0),
            ] {
                controls::form_row(ui, label, |ui, _| {
                    lesson_presentation::monospace(ui, &value, true)
                });
            }
            ui.label(if result.hostname_matches {
                "DNS name matches"
            } else {
                "DNS name does not match"
            });
            lesson_presentation::prose(ui, &trust_explanation(&result.trust));
        }
    }
}
fn results(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &Work,
    example: bool,
    m: &mut Metrics,
) -> bool {
    let outcomes = work.checks(task);
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
        lesson_presentation::prose(ui, &outcome.feedback);
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
            lesson_presentation::prose(ui, &outcome.feedback);
        }
    });
    solved
}

impl LabApp {
    pub(super) fn apache_https_lesson(&mut self, ui: &mut egui::Ui, index: usize) {
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
        let State::ApacheHttps { simulation } = &original.state else {
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
        let metrics = &mut self.metrics.apache_https;
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
            lesson_presentation::worked(ui, "Example with different virtual host names");
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
            let id = egui::Id::new(("apache-https", &scenario.id, practice));
            if practice {
                lesson_presentation::practice_step(ui, &scenario, 0);
            } else {
                let assignment = lesson_presentation::assignment_responses(ui, &scenario);
                metrics.record(ui, "assignment-content", &assignment.goal);
            }
            supplied(ui, task);
            ui.separator();
            if practice {
                lesson_presentation::practice_step(ui, &scenario, 1);
            }
            let mut reset = false;
            ui.add_enabled_ui(active, |ui| {
                editor(ui, id, task, work, metrics, &mut self.notice);
                ui.separator();
                if practice {
                    lesson_presentation::practice_step(ui, &scenario, 2);
                }
                clients(ui, id, task, work, metrics);
                ui.separator();
                let check = ui.add(primary(ui, "Check configuration"));
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
            controls::collapsing_header(ui, "Supported operations, hints and references").id_salt(id.with("help")).show(ui, |ui| {
                lesson_presentation::prose(ui, "Use one supported directive per line within a <VirtualHost *:443> block. Put comments on their own lines. File paths refer only to supplied certificate and key records.");
                if ui.add_enabled(active, button("Show a hint")).clicked() {
                    let a = self.progress.attempts.get_mut(&scenario.id).unwrap();
                    a.hints = (a.hints + 1).min(scenario.hints.len());
                    self.dirty = true;
                }
                for hint in scenario.hints.iter().take(self.progress.attempts[&scenario.id].hints) {
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
                        State::ApacheHttps {
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
