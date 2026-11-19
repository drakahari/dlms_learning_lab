//! Native evidence review, edit/preview/apply controls and real virtual results.
use super::*;
use dlms_learning_lab::automation_workshop::{
    claim_review::{Assessment, Response},
    desired_config::DRAFT_LIMIT,
    plan_editor::Difference,
};

fn assessment_options() -> [(Option<Assessment>, String); 4] {
    [
        (None, "Select an assessment".into()),
        (
            Some(Assessment::Supported),
            "Supported by this observation".into(),
        ),
        (
            Some(Assessment::Contradicted),
            "Contradicted by this observation".into(),
        ),
        (Some(Assessment::Pending), "Evidence still pending".into()),
    ]
}

pub(super) fn claim_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::ClaimReview { supplied } = task else {
        return;
    };
    guide(ui, practice, 0);
    ui.strong("Supplied generated diagnosis");
    lesson_presentation::prose(ui, supplied.generated_claim.describe());
    let r = ui.add(button("Run incident probe"));
    m.record(ui, "incident-probe", &r);
    if r.clicked() {
        apply(task, work, Action::InspectClaim, notice);
    }
    let Scope::ClaimReview { review, .. } = &work.current else {
        return;
    };
    let before = *review;
    if before.inspected_incident {
        match supplied.baseline() {
            Ok(path) => {
                row(ui, "Independent observation", |ui| {
                    let desired = if path.desired { "Allow" } else { "Exclude" };
                    let installed = if path.installed { "Allow" } else { "Exclude" };
                    let carrier = if path.carrier.is_ok() {
                        "returned"
                    } else {
                        "failed"
                    };
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "Desired · {desired} | Installed · {installed} | Carrier · {carrier}"
                        ),
                    );
                });
            }
            Err(reason) => {
                lesson_presentation::prose(ui, &reason);
            }
        }
    } else {
        ui.label("Incident probe has not run.");
    }
    let mut assessment = before.diagnosis;
    select(
        ui,
        id,
        "diagnosis",
        "Diagnosis assessment",
        &mut assessment,
        &assessment_options(),
        m,
    );
    if assessment != before.diagnosis {
        apply(task, work, Action::AssessClaim { assessment }, notice);
    }

    ui.separator();
    guide(ui, practice, 1);
    ui.strong("Supplied traffic forecast");
    let earlier = supplied
        .forecast
        .prior_samples
        .map(|value| format!("{value}%"))
        .join(" · ");
    row(ui, "Earlier readings", |ui| {
        lesson_presentation::monospace(ui, &earlier, true);
    });
    row(ui, "Alert threshold", |ui| {
        lesson_presentation::monospace(
            ui,
            &format!("{}%", supplied.forecast.threshold_percent),
            true,
        );
    });
    row(ui, "Forecast", |ui| {
        lesson_presentation::monospace(
            ui,
            &format!("{}%", supplied.forecast.predicted_percent),
            true,
        );
    });
    let r = ui.add(button("Collect next reading"));
    m.record(ui, "next-reading", &r);
    if r.clicked() {
        apply(task, work, Action::ReadForecastSample, notice);
    }
    let Scope::ClaimReview { review, .. } = &work.current else {
        return;
    };
    let before = *review;
    row(ui, "Later observed reading", |ui| {
        if before.collected_next_sample {
            lesson_presentation::monospace(
                ui,
                &format!("{}%", supplied.forecast.next_sample),
                true,
            );
        } else {
            ui.label("Awaiting the next observation");
        }
    });
    let mut assessment = before.prediction;
    select(
        ui,
        id,
        "forecast",
        "Forecast assessment",
        &mut assessment,
        &assessment_options(),
        m,
    );
    if assessment != before.prediction {
        apply(task, work, Action::AssessForecast { assessment }, notice);
    }

    ui.separator();
    guide(ui, practice, 2);
    let Scope::ClaimReview { response, .. } = &work.current else {
        return;
    };
    let previous = *response;
    let mut selection = previous;
    let choices = [
        Response::NoChange,
        Response::RestoreCarrier,
        Response::PublishRules,
    ]
    .map(|response| (response, response.describe().into()));
    select(
        ui,
        id,
        "response",
        "Reviewed response",
        &mut selection,
        &choices,
        m,
    );
    if selection != previous {
        apply(
            task,
            work,
            Action::ClaimResponse {
                response: selection,
            },
            notice,
        );
    }
    let r = ui.add(button("Apply reviewed response"));
    m.record(ui, "apply-response", &r);
    if r.clicked() {
        apply(task, work, Action::ApplyClaimResponse, notice);
    }
    let Scope::ClaimReview { review, .. } = &work.current else {
        return;
    };
    ui.strong("Actual virtual exchanges");
    for exchange in std::iter::once(supplied.incident)
        .chain(supplied.controls.iter().map(|control| control.exchange))
    {
        let sides = format!(
            "{} to {}",
            supplied.topology.edges[exchange.from].name, supplied.topology.edges[exchange.to].name
        );
        row(ui, &sides, |ui| {
            match supplied.topology.trace(review.active, exchange) {
                Ok(trace) => {
                    let status = if trace.forwarded() {
                        "Forwarded"
                    } else {
                        "Did not forward"
                    };
                    ui.label(status);
                }
                Err(error) => {
                    lesson_presentation::prose(ui, &error);
                }
            }
        });
    }
}

pub(super) fn config_body(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    practice: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::DesiredConfig { supplied } = task else {
        return;
    };
    let Scope::DesiredConfig { edit } = &work.current else {
        return;
    };
    let feedback_key = id.with("plan-preview-feedback");
    let mut rejection = ui
        .ctx()
        .data(|data| data.get_temp::<(String, String)>(feedback_key))
        .filter(|(draft, _)| draft == &edit.draft);
    guide(ui, practice, 0);
    controls::collapsing_header(ui, "Supplied recovery record")
        .id_salt(id.with("current-recovery"))
        .show(ui, |ui| {
            let record = &supplied.initial.profiles[&supplied.retained_profile];
            let pair = std::collections::BTreeMap::from([(&supplied.retained_profile, record)]);
            let json = serde_json::to_string_pretty(&pair).unwrap_or_default();
            lesson_presentation::monospace(ui, &json, true);
            lesson_presentation::prose(ui, "This profile is part of the current configuration.");
        });
    guide(ui, practice, 1);
    ui.strong("Proposed configuration");
    let input = shared_libraries_page::configuration_input(
        ui,
        &edit.draft,
        id.with("config"),
        DRAFT_LIMIT,
        ui.is_enabled(),
        "Edit the supplied profiles and bindings",
    );
    m.controls
        .push(("plan-draft".into(), input.viewport, input.clip));
    m.focus.push(("plan-draft".into(), input.response.id));
    if let Some(text) = input.value {
        rejection = None;
        apply(task, work, Action::ChangePlanDraft { text }, notice);
    }
    if input.rejected {
        ui.label("The text exceeds this editor’s limit. Your previous draft was kept.");
    }
    ui.horizontal_wrapped(|ui| {
        let preview = ui.add(button("Preview plan"));
        m.record(ui, "preview-plan", &preview);
        if preview.clicked() {
            rejection = work
                .apply(task, Action::PreviewConfig)
                .err()
                .and_then(|error| {
                    let Scope::DesiredConfig { edit } = &work.current else {
                        return None;
                    };
                    Some((edit.draft.clone(), error))
                });
        }
        let reviewed = matches!(&work.current,Scope::DesiredConfig { edit } if edit.previewed);
        let publish = ui.add_enabled(reviewed, button("Apply plan"));
        m.record(ui, "apply-plan", &publish);
        if publish.clicked() {
            apply(task, work, Action::ApplyConfig, notice);
        }
    });
    let Scope::DesiredConfig { edit } = &work.current else {
        return;
    };
    ui.strong("Current preview");
    if let Some((_, error)) = &rejection {
        let feedback = lesson_presentation::prose(ui, error);
        m.record(ui, "plan-error", &feedback);
    } else if edit.previewed
        && let Ok(plan) = supplied.plan(&edit.applied, &edit.draft)
    {
        if plan.changes.is_empty() {
            ui.label("No remaining changes.");
        }
        for entry in plan.changes {
            let change = match entry.difference {
                Difference::Create => "Create",
                Difference::Update => "Update",
                Difference::Remove => "Remove",
            };
            row(ui, change, |ui| {
                lesson_presentation::monospace(ui, &entry.resource, true);
            });
        }
    } else {
        ui.label("Select Preview plan to compare this draft with the applied settings.");
    }

    ui.ctx().data_mut(|data| {
        if let Some(rejection) = rejection {
            data.insert_temp(feedback_key, rejection);
        } else {
            data.remove::<(String, String)>(feedback_key);
        }
    });
    ui.separator();
    guide(ui, practice, 2);
    ui.strong("Actual virtual path results");
    if let Ok(fabric) = supplied.topology.project(&edit.applied) {
        for case in &supplied.paths {
            row(
                ui,
                &format!("{} and {}", case.from, case.to),
                |ui| match fabric.visibility(&case.from, &case.to) {
                    Ok(seen) => {
                        let result = if seen.mutual() {
                            "Both directions work"
                        } else if seen.isolated() {
                            "Both directions remain separate"
                        } else {
                            "Only one direction works"
                        };
                        ui.label(result);
                    }
                    Err(error) => {
                        lesson_presentation::prose(ui, &error);
                    }
                },
            );
        }
    }
}
