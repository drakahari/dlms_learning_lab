//! Opt-in native configuration review. Existing package activities render nothing here.
use super::*;
use dlms_learning_lab::{debian_packages as core, debian_packages_workspace as work};
#[derive(Clone, Debug)]
pub(super) enum Edit {
    Review,
    Choose(String, Option<core::ConffileDecision>),
    Apply,
}
pub(super) fn edit(
    d: &work::Definition,
    s: &core::State,
    draft: &mut work::Draft,
    e: Edit,
) -> Result<Option<core::Action>, String> {
    let package = d
        .requirements
        .iter()
        .find_map(|r| {
            if let work::Requirement::ConfigurationUpgrade { outcome } = r {
                Some(outcome.package.as_str())
            } else {
                None
            }
        })
        .ok_or("This activity has no configuration upgrade")?;
    match e {
        Edit::Review => {
            work::prepare_configuration(d, s, draft, package)?;
            Ok(None)
        }
        Edit::Choose(path, choice) => {
            work::choose_configuration(d, s, draft, &path, choice)?;
            Ok(None)
        }
        Edit::Apply => {
            let a = draft.action()?;
            if !matches!(a, core::Action::UpgradeConffiles { .. }) {
                return Err("Prepare a configuration review first".into());
            }
            Ok(Some(a))
        }
    }
}
fn text(ui: &mut egui::Ui, title: &str, value: &str, key: impl std::hash::Hash) {
    ui.label(RichText::new(title).strong());
    egui::ScrollArea::both()
        .id_salt(key)
        .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 5.)
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(if value.is_empty() {
                        "(empty file)"
                    } else {
                        value
                    })
                    .monospace(),
                )
                .extend()
                .selectable(true),
            );
        });
}
pub(super) fn panel(
    ui: &mut egui::Ui,
    d: &work::Definition,
    state: &core::State,
    draft: &work::Draft,
    scope: (&str, bool),
    metrics: &mut Metrics,
) -> Vec<Edit> {
    let Some(goal) = d.requirements.iter().find_map(|r| {
        if let work::Requirement::ConfigurationUpgrade { outcome } = r {
            Some(outcome)
        } else {
            None
        }
    }) else {
        return vec![];
    };
    let mut edits = vec![];
    ui.separator();
    ui.heading("Configuration upgrade");
    ui.label("Compare the package defaults with your site settings, then prepare and apply an explicit choice.");
    let prepared = draft
        .conffile_upgrade
        .as_ref()
        .filter(|u| u.package == goal.package);
    let source_matches = prepared.is_some_and(|u| {
        u.incoming_archive == goal.incoming_archive
            && state
                .catalog()
                .archives
                .get(&u.expected_archive)
                .is_some_and(|old| {
                    state
                        .catalog()
                        .archives
                        .get(&u.incoming_archive)
                        .is_some_and(|new| {
                            old.name == goal.package
                                && new.name == goal.package
                                && new.version.compare(&old.version).is_gt()
                        })
                })
    });
    let applied = source_matches
        && prepared.is_some_and(|u| {
            state.runtime().records.get(&u.package).is_some_and(|r| {
                r.status == core::Status::Installed && r.archive == u.incoming_archive
            })
        });
    let review = ui.add_enabled(
        !applied,
        button(if prepared.is_some() {
            "Review again"
        } else {
            "Review configuration upgrade"
        }),
    );
    name_control(
        &review,
        "Prepare configuration choices without changing files",
    );
    metrics
        .package_configuration
        .push(("review".into(), review.rect));
    if review.clicked() {
        edits.push(Edit::Review);
    }
    let Some(u) = prepared else {
        ui.label("No review prepared. Inspect package versions and refresh supplied metadata if needed, then choose Review configuration upgrade.");
        return edits;
    };
    if !source_matches {
        ui.colored_label(palette(ui).error,"Review needs updating: the saved target differs from this activity. Choose Review again.");
        return edits;
    }
    let freshness = state.conffile_review_current(u);
    let fresh = freshness.is_ok();
    if applied {
        ui.label("Target version is installed. Inspect actual files below and Check work.");
        ui.label(if scope.1 {
            "Use Reset example only below to try different choices."
        } else {
            "Use Reset at the top of this lab to try different choices; other labs stay saved."
        });
    } else if let Err(e) = &freshness {
        ui.colored_label(
            palette(ui).error,
            format!("Review needs updating: {}", e.message),
        );
    }
    let Some(old) = state.catalog().archives.get(&u.expected_archive) else {
        return edits;
    };
    let Some(new) = state.catalog().archives.get(&u.incoming_archive) else {
        return edits;
    };
    ui.label(
        RichText::new(format!(
            "{}: {} → {}",
            u.package,
            old.version.as_str(),
            new.version.as_str()
        ))
        .monospace(),
    );
    let mut ready = true;
    for row in &u.files {
        let (Some(previous), Some(incoming)) = (old.files.get(&row.path), new.files.get(&row.path))
        else {
            continue;
        };
        let outcome =
            core::conffile_disposition(&previous.contents, &row.expected_local, &incoming.contents);
        let conflict = outcome == core::ConffileDisposition::Conflict;
        ready &= !conflict || row.decision.is_some();
        let status = match outcome {
            core::ConffileDisposition::MatchesIncoming => "Already matches incoming default",
            core::ConffileDisposition::UpdateUnmodified => {
                "Untouched setting: update automatically"
            }
            core::ConffileDisposition::KeepLocalOnly => "Only local settings changed: keep them",
            core::ConffileDisposition::Conflict => "Both changed: choose which settings to use",
        };
        let label = format!("{} · {status}", row.path);
        egui::CollapsingHeader::new(label)
            .id_salt((scope, "configuration-row", &row.path))
            .default_open(conflict && !applied)
            .show(ui, |ui| {
                let values = [
                    ("Previous default", previous.contents.as_str()),
                    ("Reviewed local", row.expected_local.as_str()),
                    ("Incoming default", incoming.contents.as_str()),
                ];
                if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 38. {
                    ui.columns(3, |columns| {
                        for (n, (label, value)) in values.iter().enumerate() {
                            text(&mut columns[n], label, value, (scope, &row.path, n));
                        }
                    });
                } else {
                    for (n, (label, value)) in values.iter().enumerate() {
                        text(ui, label, value, (scope, &row.path, n));
                        ui.add_space(controls::GAP);
                    }
                }
                if conflict {
                    let mut value = row.decision;
                    let width = ui.available_width();
                    ui.add_enabled_ui(fresh && !applied, |ui| {
                        controls::cell(ui, width, |ui| {
                            let combo = egui::ComboBox::from_id_salt((
                                scope,
                                "configuration-choice",
                                &row.path,
                            ))
                            .width(width)
                            .truncate()
                            .selected_text(match value {
                                None => "Choose a result…",
                                Some(core::ConffileDecision::KeepLocal) => {
                                    "Keep current site settings"
                                }
                                Some(core::ConffileDecision::InstallNew) => {
                                    "Use incoming package default"
                                }
                            })
                            .show_ui(ui, |ui| {
                                for (choice, label, key) in [
                                    (None, "Choose a result…", "none"),
                                    (
                                        Some(core::ConffileDecision::KeepLocal),
                                        "Keep current site settings",
                                        "keep",
                                    ),
                                    (
                                        Some(core::ConffileDecision::InstallNew),
                                        "Use incoming package default",
                                        "new",
                                    ),
                                ] {
                                    let r = ui.selectable_value(&mut value, choice, label);
                                    metrics
                                        .package_configuration
                                        .push((format!("{}:{key}", row.path), r.rect));
                                }
                            });
                            name_control(
                                &combo.response,
                                format!("Configuration choice for {}", row.path),
                            );
                            metrics
                                .package_configuration
                                .push((format!("choice:{}", row.path), combo.response.rect));
                        });
                    });
                    if value != row.decision {
                        edits.push(Edit::Choose(row.path.clone(), value));
                    }
                }
            });
    }
    if !applied {
        ui.label("A review is a draft. Applying changes only the virtual package and files; Check work verifies the result.");
    }
    let apply = ui.add_enabled(
        fresh && ready && !applied,
        primary(ui, "Apply reviewed upgrade"),
    );
    metrics
        .package_configuration
        .push(("apply".into(), apply.rect));
    name_control(&apply, "Apply the reviewed virtual configuration upgrade");
    if apply.clicked() {
        edits.push(Edit::Apply);
    }
    if fresh && !ready {
        ui.label("Choose a result for each file marked Both changed, then apply the upgrade.");
    }
    edits
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (work::Definition, core::State) {
        let mut v: serde_json::Value = serde_json::from_str(include_str!(
            "../../tests/fixtures/debian-packages-declared.json"
        ))
        .unwrap();
        let s = &mut v["state"];
        s["runtime"]["cached"] = "current".into();
        for suffix in [".dpkg-old", ".dpkg-dist", ".dpkg-new", ".dpkg-tmp"] {
            s["catalog"]["extra_paths"]
                .as_array_mut()
                .unwrap()
                .push(format!("/etc/museum/report.conf{suffix}").into());
        }
        let s: core::State = serde_json::from_value(s.clone()).unwrap();
        let d = work::Definition {
            initial: s.clone(),
            requirements: vec![work::Requirement::ConfigurationUpgrade {
                outcome: work::ConfigurationUpgradeGoal {
                    package: "museum-report".into(),
                    incoming_archive: "report-2".into(),
                    choices: vec![work::ConfigurationTarget {
                        path: "/etc/museum/report.conf".into(),
                        decision: core::ConffileDecision::KeepLocal,
                    }],
                },
            }],
        };
        d.validate().unwrap();
        (d, s)
    }
    fn frame(
        ctx: &egui::Context,
        d: &work::Definition,
        s: &core::State,
        draft: &work::Draft,
        events: Vec<egui::Event>,
        width: f32,
    ) -> (Vec<Edit>, Metrics, String) {
        let mut metrics = Metrics::default();
        let mut edits = vec![];
        let output = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(width, 3000.),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    edits = panel(ui, d, s, draft, ("technical-conffile", false), &mut metrics);
                });
            },
        );
        fn collect(shape: &egui::Shape, s: &mut String) {
            match shape {
                egui::Shape::Text(t) => {
                    s.push_str(&t.galley.job.text);
                    s.push('\n');
                }
                egui::Shape::Vec(items) => {
                    for i in items {
                        collect(i, s)
                    }
                }
                _ => {}
            }
        }
        let mut text = String::new();
        for shape in output.shapes {
            collect(&shape.shape, &mut text);
        }
        (edits, metrics, text)
    }
    fn pointer(pos: egui::Pos2, down: bool) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: down,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }
    #[test]
    fn decoded_reversed_review_is_stale_and_review_again_remains_clickable() {
        let (d, s) = fixture();
        let mut draft = work::Draft::new(&s).unwrap();
        work::prepare_configuration(&d, &s, &mut draft, "museum-report").unwrap();
        let u = draft.conffile_upgrade.as_mut().unwrap();
        u.expected_archive = "report-2".into();
        u.incoming_archive = "report-1".into();
        let mut draft: work::Draft =
            serde_json::from_slice(&serde_json::to_vec(&draft).unwrap()).unwrap();
        assert!(draft.valid_for(&s));
        let ctx = egui::Context::default();
        let _ = frame(&ctx, &d, &s, &draft, vec![], 1200.);
        let (_, m, text) = frame(&ctx, &d, &s, &draft, vec![], 1200.);
        assert!(text.contains("Review needs updating"), "{text}");
        assert!(!text.contains("Target version is installed"));
        let pos = m
            .package_configuration
            .iter()
            .find(|r| r.0 == "review")
            .unwrap()
            .1
            .center();
        let _ = frame(&ctx, &d, &s, &draft, pointer(pos, true), 1200.);
        let (edits, _, _) = frame(&ctx, &d, &s, &draft, pointer(pos, false), 1200.);
        assert!(edits.iter().any(|e| matches!(e, Edit::Review)));
        for e in edits {
            assert!(edit(&d, &s, &mut draft, e).unwrap().is_none());
        }
        let u = draft.conffile_upgrade.unwrap();
        assert_eq!(u.expected_archive, "report-1");
        assert_eq!(u.incoming_archive, "report-2");
        assert!(u.files.iter().all(|r| r.decision.is_none()));
    }
    #[test]
    fn columns_reflow_and_legacy_activities_gain_no_controls() {
        let (d, s) = fixture();
        let mut draft = work::Draft::new(&s).unwrap();
        work::prepare_configuration(&d, &s, &mut draft, "museum-report").unwrap();
        for light in [false, true] {
            let theme = theme::inventory()
                .iter()
                .find(|t| t.light == light)
                .unwrap();
            for scale in [1., 1.5, 2.] {
                for width in [760., 1280.] {
                    let ctx = egui::Context::default();
                    theme::apply(&ctx, &theme.id);
                    ctx.set_zoom_factor(scale);
                    let _ = frame(&ctx, &d, &s, &draft, vec![], width);
                    let (_, metrics, text) = frame(&ctx, &d, &s, &draft, vec![], width);
                    for label in [
                        "Previous default",
                        "Reviewed local",
                        "Incoming default",
                        "Both changed",
                    ] {
                        assert!(
                            text.contains(label),
                            "{}/{scale}/{width}: {label}: {text}",
                            theme.id
                        );
                    }
                    assert!(metrics.package_configuration.iter().any(|r| r.0 == "apply"));
                }
            }
        }
        let old = work::Definition {
            initial: s.clone(),
            requirements: vec![work::Requirement::Installed {
                package: "museum-report".into(),
                constraint: None,
            }],
        };
        let ctx = egui::Context::default();
        let (edits, metrics, text) = frame(&ctx, &old, &s, &draft, vec![], 1200.);
        assert!(edits.is_empty() && metrics.package_configuration.is_empty() && text.is_empty());
    }
}
#[cfg(test)]
mod lesson_views {
    use super::*;
    fn rendered(ctx: &egui::Context, a: &mut LabApp) -> String {
        let o = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280., 6000.),
                )),
                ..Default::default()
            },
            |ctx| a.render(ctx),
        );
        fn collect(s: &egui::Shape, t: &mut String) {
            match s {
                egui::Shape::Text(s) => {
                    t.push_str(&s.galley.job.text);
                    t.push('\n');
                }
                egui::Shape::Vec(v) => {
                    for s in v {
                        collect(s, t)
                    }
                }
                _ => {}
            }
        }
        let mut t = String::new();
        for s in o.shapes {
            collect(&s.shape, &mut t);
        }
        t
    }
    #[test]
    fn conffile_each_lesson_teaches_all_concepts_and_three_actions_before_independent_assignment() {
        let ctx = egui::Context::default();
        let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
        a.smoke = false;
        for id in [
            "lpic1-conffile-retain",
            "lpic1-conffile-adopt",
            "lpic1-conffile-handover",
        ] {
            let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
            a.launch(i);
            a.set_debian_packages_view(i, teaching::View::Introduction);
            let _ = rendered(&ctx, &mut a);
            let intro = rendered(&ctx, &mut a);
            let s = a.scenarios[i].clone();
            let entry = lesson_presentation::entry(&s).unwrap();
            for c in &entry.concepts {
                assert!(intro.contains(&c.title), "{id}: {} missing", c.title);
            }
            a.set_debian_packages_view(i, teaching::View::Example);
            let _ = rendered(&ctx, &mut a);
            let example = rendered(&ctx, &mut a);
            assert_eq!(entry.practice.len(), 3);
            for step in &entry.practice {
                assert!(
                    example.contains(&step.title) && example.contains(&step.action),
                    "{id}: {} missing: {example}",
                    step.title
                );
            }
            assert!(example.contains("Configuration upgrade"));
            assert_eq!(a.progress.attempts[id].state, s.initial);
            assert!(!progress::current_solved(&a.progress.attempts[id], &s));
            a.set_debian_packages_view(i, teaching::View::Assignment);
            let _ = rendered(&ctx, &mut a);
            let text = rendered(&ctx, &mut a);
            assert!(
                text.contains("Requirements")
                    && text.contains("Review configuration upgrade")
                    && text.contains("Check work")
            );
            assert_eq!(a.progress.attempts[id].state, s.initial);
            assert!(
                a.progress.attempts[id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .debian_packages_draft
                    .as_ref()
                    .unwrap()
                    .conffile_upgrade
                    .is_none()
            );
        }
    }
}
