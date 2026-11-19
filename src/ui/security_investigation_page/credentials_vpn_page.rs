//! Credential policy trials and applied inner/outer VPN decisions.
use super::*;
use dlms_learning_lab::security_investigation::{
    credential_task::Input,
    credentials::{Evidence, SecondFactor},
    vpn_design::{Purpose, Role},
    vpn_design_task::{Draft, describe_path},
};

fn evidence(value: Evidence) -> &'static str {
    match value {
        Evidence::Missing => "No second proof",
        Evidence::MemorizedCode => "Memorized code supplied",
        Evidence::CertificateOnly => "Public certificate only",
        Evidence::CertificateWithVerifiedKey => "Certificate and verified key possession",
        Evidence::BiometricOnly => "Biometric observation only",
        Evidence::BiometricUnlockedVerifiedKey => "Biometric unlock plus verified key possession",
    }
}

fn practice_note(
    ui: &mut egui::Ui,
    scenario: Option<&Scenario>,
    index: usize,
    metrics: &mut Metrics,
) {
    if let Some(scenario) = scenario {
        let note = ui.scope(|ui| lesson_presentation::practice_step(ui, scenario, index));
        metrics.record(ui, &format!("practice-step-{}", index + 1), &note.response);
    }
}

pub(super) fn credentials_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::Credentials { supplied } = task else {
        return;
    };
    let Scope::Credentials { policy: before } = work.current else {
        return;
    };
    let mut policy = before;
    ui.strong("New password checks");
    practice_note(ui, guidance, 0, m);
    controls::form_row(ui, "Minimum length", |ui, _| {
        let r = ui.add(
            egui::DragValue::new(&mut policy.minimum_length)
                .range(8..=64)
                .suffix(" characters"),
        );
        m.record(ui, "minimum-length", &r);
    });
    let r = ui.checkbox(
        &mut policy.reject_listed_passwords,
        "Reject listed passwords",
    );
    m.record(ui, "blocklist", &r);
    ui.separator();
    ui.strong("Staff sign-in checks");
    practice_note(ui, guidance, 1, m);
    choose(
        ui,
        id,
        "Additional proof",
        &mut policy.second_factor,
        &[
            (SecondFactor::None, "No additional proof".into()),
            (SecondFactor::MemorizedCode, "Memorized code".into()),
            (
                SecondFactor::BoundAuthenticator,
                "Bound authenticator".into(),
            ),
        ],
        m,
        "factor",
    );
    practice_note(ui, guidance, 2, m);
    let r = ui.checkbox(
        &mut policy.replace_after_compromise,
        "Require replacement after compromise",
    );
    m.record(ui, "replace", &r);
    if policy != before {
        apply(task, work, Action::Credentials { policy }, notice);
    }
    ui.separator();
    let title = ui.strong("Supplied trials and actual outcomes");
    m.record(ui, "credential-trials", &title);
    for trial in &supplied.trials {
        let (actual, reason) = trial.test(policy);
        facts_row(ui, &trial.name, |ui| {
            match trial.input {
                Input::Proposal { supplied } => {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} characters · {}",
                            supplied.characters,
                            if supplied.listed {
                                "listed"
                            } else {
                                "unlisted"
                            }
                        ),
                    );
                }
                Input::SignIn { supplied } => {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {}",
                            if supplied.current_password_verified {
                                "Password matches"
                            } else {
                                "Password fails"
                            },
                            evidence(supplied.second_proof)
                        ),
                    );
                    if supplied.compromise_reported {
                        ui.strong("Confirmed compromise");
                    }
                }
            }
            lesson_presentation::rich(
                ui,
                RichText::new(format!("Observed · {}", actual.describe())).strong(),
            );
            lesson_presentation::prose(ui, reason);
        });
    }
}

fn prefix_fields(
    ui: &mut egui::Ui,
    id: egui::Id,
    profile: usize,
    draft: &mut Draft,
    m: &mut Metrics,
) {
    for (key, label, value) in [
        ("left", "Left inner range", &mut draft.left_traffic),
        ("right", "Right inner range", &mut draft.right_traffic),
    ] {
        let name = format!("vpn-{profile}-{key}-prefix");
        controls::form_row(ui, label, |ui, _| {
            let input = shared_libraries_page::fitting_input(
                ui,
                value,
                id.with(&name),
                32,
                ui.is_enabled(),
                "Network/prefix",
            );
            m.record(ui, &name, &input.response);
            if let Some(next) = input.value {
                *value = next;
            }
            if input.rejected {
                ui.label("Use a single IPv4 network/prefix within 32 bytes.");
            }
        });
    }
}

pub(super) fn vpn_ui(
    ui: &mut egui::Ui,
    id: egui::Id,
    task: &Task,
    work: &mut Work,
    guidance: Option<&Scenario>,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let Task::VpnDesign { supplied } = task else {
        return;
    };
    let net = supplied.topology;
    let branch = net.branch_gateway.to_string();
    let office = net.headquarters_gateway.to_string();
    let remote = net.remote_client.to_string();
    let branch_lan = net.branch_lan.text();
    let office_lan = net.headquarters_lan.text();
    controls::topology_chain(
        ui,
        &[
            ["Branch gateway", &branch, &branch_lan],
            ["Office gateway", &office, &office_lan],
        ],
        "Each box shows its outer gateway address and the inner LAN.",
    );
    controls::form_row(ui, "Remote device", |ui, _| {
        lesson_presentation::monospace(
            ui,
            &format!("{remote} outer · {}/32 inner", net.remote_inner),
            true,
        );
    });
    controls::form_row(ui, "Office service for remote access", |ui, _| {
        lesson_presentation::monospace(ui, &net.remote_service.text(), true);
    });

    let Scope::VpnDesign { drafts: before, .. } = &work.current else {
        return;
    };
    let before = before.clone();
    let mut drafts = before.clone();
    let roles = [
        (Role::BranchGateway, "Branch gateway".into()),
        (Role::HeadquartersGateway, "Office gateway".into()),
        (Role::RemoteClient, "Remote device".into()),
    ];
    let purposes = [
        (Purpose::SiteToSite, "Site-to-site".into()),
        (Purpose::RemoteAccess, "Remote access".into()),
    ];
    for (profile, draft) in drafts.iter_mut().enumerate() {
        ui.separator();
        ui.strong(format!("Profile {}", profile + 1));
        practice_note(ui, guidance, profile, m);
        choose(
            ui,
            id,
            "Purpose",
            &mut draft.purpose,
            &purposes,
            m,
            &format!("vpn-{profile}-purpose"),
        );
        choose(
            ui,
            id,
            "Left endpoint",
            &mut draft.endpoints.left,
            &roles,
            m,
            &format!("vpn-{profile}-left"),
        );
        choose(
            ui,
            id,
            "Right endpoint",
            &mut draft.endpoints.right,
            &roles,
            m,
            &format!("vpn-{profile}-right"),
        );
        prefix_fields(ui, id, profile, draft, m);
    }
    if drafts != before {
        apply(task, work, Action::VpnDrafts { drafts }, notice);
    }
    let r = ui.add(button("Apply profiles"));
    m.record(ui, "apply-vpn", &r);
    if r.clicked() {
        apply(task, work, Action::ApplyVpn, notice);
    }
    let Scope::VpnDesign { drafts, applied } = &work.current else {
        return;
    };
    let current = drafts
        .iter()
        .zip(applied)
        .all(|(draft, profile)| draft.profile().ok() == Some(*profile));
    ui.label(if current {
        "These profiles are applied."
    } else {
        "Trials use the earlier applied profiles."
    });
    ui.separator();
    practice_note(ui, guidance, 2, m);
    let heading = ui.strong("Actual path trials");
    m.record(ui, "vpn-trials", &heading);
    for trial in &supplied.trials {
        facts_row(ui, &trial.name, |ui| {
            lesson_presentation::monospace(
                ui,
                &format!("{} → {}", trial.traffic.source, trial.traffic.destination),
                true,
            );
            let observation = net
                .trace(applied, trial.traffic)
                .map(describe_path)
                .unwrap_or_else(|error| error);
            lesson_presentation::prose(ui, &observation);
        });
    }
}
