//! Whole-catalog presentation checks, not engine/certification/novice acceptance.
use super::*;
fn text_shapes(s: &egui::Shape, out: &mut Vec<(String, egui::Rect, bool)>) {
    match s {
        egui::Shape::Text(t) => out.push((
            t.galley.job.text.clone(),
            egui::Rect::from_min_size(t.pos, t.galley.size()),
            t.galley.job.justify,
        )),
        egui::Shape::Vec(v) => {
            for s in v {
                text_shapes(s, out)
            }
        }
        _ => {}
    }
}
fn paint(
    ctx: &egui::Context,
    a: &mut LabApp,
    events: Vec<egui::Event>,
) -> Vec<(String, egui::Rect, bool)> {
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1500., 6000.),
            )),
            events,
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut out = vec![];
    for s in output.shapes {
        text_shapes(&s.shape, &mut out);
    }
    out
}
fn settle(ctx: &egui::Context, a: &mut LabApp) -> Vec<(String, egui::Rect, bool)> {
    let mut out = vec![];
    for _ in 0..4 {
        out = paint(ctx, a, vec![]);
    }
    out
}
fn click_text(ctx: &egui::Context, a: &mut LabApp, labels: &[&str], id: &str) {
    let rows = settle(ctx, a);
    let rect = labels
        .iter()
        .find_map(|label| rows.iter().find(|(text, _, _)| text == label).map(|r| r.1))
        .unwrap_or_else(|| {
            panic!(
                "{id} missing navigation {labels:?}; captions={:?}",
                rows.iter().map(|r| &r.0).collect::<Vec<_>>()
            )
        });
    for pressed in [true, false] {
        paint(
            ctx,
            a,
            vec![
                egui::Event::PointerMoved(rect.center()),
                egui::Event::PointerButton {
                    pos: rect.center(),
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ],
        );
    }
    settle(ctx, a);
}
#[test]
fn rollout_catalog_every_registered_lab_has_a_concrete_three_screen_content_review() {
    let (catalog, _) = ccna_pilot::catalog();
    let entries = lesson_presentation::entries();
    assert_eq!(entries.len(), catalog.len());
    let mut raw = Vec::<serde_json::Value>::new();
    for source in [
        include_str!("../../../scenarios/presentation/ccna.json"),
        include_str!("../../../scenarios/presentation/lpic1.json"),
        include_str!("../../../scenarios/presentation/lpic2.json"),
        include_str!("../../../scenarios/presentation/bash.json"),
    ] {
        raw.extend(serde_json::from_str::<Vec<serde_json::Value>>(source).unwrap());
    }
    // Unit spacing (16MiB / 16 MiB) is presentation; every non-whitespace character must stay exact.
    let norm = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    for s in &catalog {
        let e = entries.get(&s.id).unwrap();
        assert_eq!(
            norm(&e.requirements.join(" ")),
            norm(&s.objective),
            "requirement changed: {}",
            s.id
        );
        let row = raw.iter().find(|r| r["id"] == s.id).unwrap();
        assert!(
            matches!(row["status"].as_str(), Some("updated" | "already meets")),
            "pending/blocked: {}",
            s.id
        );
        for view in ["introduction", "example", "assignment"] {
            assert!(
                row["review"][view].as_str().is_some_and(|v| v.len() > 40),
                "missing actual content review {} {view}",
                s.id
            );
        }
    }
}
#[test]
fn rollout_shared_columns_keep_titles_left_aligned_and_range_cells_equal() {
    let (ctx, mut a) = app();
    let mut cells = vec![];
    let o = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1300., 900.),
            )),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(2, |cols| {
                    lesson_presentation::lead(
                        &mut cols[0],
                        "OBJECTIVE",
                        "A multiword heading must retain ordinary word spacing",
                    );
                    lesson_presentation::section(
                        &mut cols[1],
                        1,
                        "A longer calculation title stays left aligned",
                        "Use mask 255.255.255.192 to find a block.",
                    );
                });
                cells = lesson_presentation::range_blocks(
                    ui,
                    &[
                        ("0–63".into(), false),
                        ("64–127".into(), true),
                        ("128–191".into(), false),
                        ("192–255".into(), false),
                    ],
                );
            });
        },
    );
    let mut texts = vec![];
    for s in o.shapes {
        text_shapes(&s.shape, &mut texts);
    }
    for (text, _, justify) in texts {
        if text.contains("heading") || text.contains("calculation title") {
            assert!(!justify, "stretched {text}");
        }
    }
    assert_eq!(cells.len(), 4);
    for c in &cells {
        assert!((c.width() - cells[0].width()).abs() < 1.5);
        assert!((c.height() - cells[0].height()).abs() < 1.5);
        assert!((c.top() - cells[0].top()).abs() < 1.5);
    }
    let spacings = cells
        .windows(2)
        .map(|w| w[1].center().x - w[0].center().x)
        .collect::<Vec<_>>();
    assert!(spacings.iter().all(|v| (v - spacings[0]).abs() < 1.5));
    a.selected = None;
}
fn assignment_state(state: &engine::State) -> serde_json::Value {
    let mut value = serde_json::to_value(state).unwrap();
    // These models store their navigation view inside the session. Allow only
    // that field to change; simulation, checks, answers and drafts stay exact.
    let key = if matches!(state, engine::State::MagicSubnet { .. }) {
        Some("session")
    } else if matches!(
        state,
        engine::State::Transport { .. }
            | engine::State::FileIndex { .. }
            | engine::State::SharedLibraries { .. }
            | engine::State::Raid1 { .. }
            | engine::State::StartupFiles { .. }
            | engine::State::FsIntegrity { .. }
            | engine::State::UserMounts { .. }
            | engine::State::Environment { .. }
            | engine::State::DebianSources { .. }
            | engine::State::ClientNames { .. }
            | engine::State::RemovableMounts { .. }
            | engine::State::NmProfiles { .. }
            | engine::State::DhcpConfig { .. }
            | engine::State::Qos { .. }
            | engine::State::LinuxLogging { .. }
            | engine::State::HostNetwork { .. }
            | engine::State::Repositories { .. }
            | engine::State::Capacity { .. }
            | engine::State::Lvm { .. }
            | engine::State::Ntp { .. }
            | engine::State::PartitionFs { .. }
            | engine::State::Hsrp { .. }
            | engine::State::Syslog { .. }
            | State::Links { .. }
    ) {
        Some("simulation")
    } else {
        None
    };
    if let Some(key) = key {
        value[key].as_object_mut().unwrap().remove("view");
    }
    value
}
#[test]
fn rollout_catalog_all_three_actual_views_render_without_changing_assignment_state() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let mut receipts = serde_json::Map::new();
    for i in 0..a.scenarios.len() {
        let s = a.scenarios[i].clone();
        println!("PRESENTATION_VIEWS {}", s.id);
        a.launch(i);
        let before = assignment_state(&a.progress.attempts[&s.id].state);
        // Use actual navigation widgets, including guarded exceptional workspaces.
        let mut initial = settle(&ctx, &mut a);
        if initial.iter().any(|r| r.0 == "Open workspace") {
            click_text(&ctx, &mut a, &["Open workspace"], &s.id);
            initial = settle(&ctx, &mut a);
        }
        if initial.iter().any(|r| r.0 == "Introduction") {
            click_text(&ctx, &mut a, &["Introduction"], &s.id);
        }
        let intro = settle(&ctx, &mut a)
            .iter()
            .map(|r| r.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            intro.contains("WHAT YOU’LL LEARN"),
            "{} intro not opened: {intro}",
            s.id
        );
        click_text(
            &ctx,
            &mut a,
            &[
                "Try example · ungraded",
                "Try example · Ungraded",
                "Separate example · ungraded",
                "Separate practice · ungraded",
                "Try this example",
                "Try the separate example",
                "Try the separate example · ungraded",
                "Try example",
            ],
            &s.id,
        );
        let example = settle(&ctx, &mut a)
            .iter()
            .map(|r| r.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let entry = lesson_presentation::entry(&s).unwrap();
        if s.id.starts_with("ccna-magic-") {
            assert!(example.contains("GUIDED PRACTICE"), "{}", s.id);
        } else if s.id == "ccna-ipv4" {
            assert!(example.contains("What to do"));
        } else {
            assert!(
                example.contains(&entry.practice[0].title),
                "{} practice guidance absent: {example}",
                s.id
            );
        }
        click_text(
            &ctx,
            &mut a,
            &[
                "Assignment",
                "Start assignment",
                "Open full assignment",
                "Open assignment",
                "Go to assignment",
            ],
            &s.id,
        );
        let assignment = settle(&ctx, &mut a)
            .iter()
            .map(|r| r.0.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            assignment.contains("Requirements")
                || s.id.starts_with("ccna-magic-") && assignment.contains("Given address:")
                || s.id == "ccna-ipv4" && assignment.contains("Workstation 2"),
            "{} assignment not reached: {assignment}",
            s.id
        );
        assert_eq!(
            assignment_state(&a.progress.attempts[&s.id].state),
            before,
            "presentation altered assignment state: {}",
            s.id
        );
        receipts.insert(
            s.id.clone(),
            serde_json::json!({"introduction":intro,"example":example,"assignment":assignment}),
        );
    }
    assert_eq!(receipts.len(), 290);
    assert_eq!(
        receipts
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>(),
        a.scenarios.iter().map(|s| s.id.clone()).collect(),
        "Every registered ID needs all three actual rendered views"
    );
    if std::env::var("DLMS_TEACHING_RENDER_RECEIPTS").as_deref() == Ok("1") {
        std::fs::write(
            ".build-artifacts/teaching-rollout/rendered-views.json",
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn rollout_small_worked_script_reserves_its_measured_height_and_diagnoses_real_changes() {
    let ctx = egui::Context::default();
    let source = "badge() {\n  printf 'Visitor: %s\\n' \"$1\"\n}\nbadge Mina\nbadge 'Rowan Reed'";
    let _=ctx.run(egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(600.,160.))),..Default::default()},|ctx|{
        egui::CentralPanel::default().show(ctx,|ui|{
            ui.add_space(100.);
            let font=egui::TextStyle::Monospace.resolve(ui.style());let color=palette(ui).text;
            let measured=ui.fonts_mut(|f|f.layout_no_wrap(source.into(),font,color).size().y);
            let rect=script_learning::worked_script(ui,"fixture",source);
            assert!(rect.height()>=measured+19.,"worked code was compressed into an unnecessary tiny viewport: {rect:?}, text={measured}");
        });
    });
    let entries = lesson_presentation::entries();
    for row in serde_json::from_str::<Vec<serde_json::Value>>(include_str!(
        "../../../scenarios/presentation/lpic1.json"
    ))
    .unwrap()
    {
        let id = row["id"].as_str().unwrap();
        let step = &entries[id].practice[2];
        assert!(
            !step.observe.contains("repair the changed setting"),
            "generic/inaccurate observation {id}"
        );
    }
    assert!(
        entries["lpic1-file-navigation"].practice[2]
            .observe
            .contains("/home/alex")
    );
    assert!(
        entries["lpic1-file-cleanup"].practice[2]
            .observe
            .contains("no repair is needed")
    );
}

#[test]
fn rollout_navigation_normalization_retains_actual_startup_drafts_and_values() {
    let ctx = egui::Context::default();
    let mut a = LabApp::from_context_with_pilot(&ctx, true, false, true);
    a.smoke = false;
    let i = a
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-shell-startup-priority")
        .unwrap();
    a.launch(i);
    let state = &a.progress.attempts["lpic1-shell-startup-priority"].state;
    let before = assignment_state(state);
    let mut changed = state.clone();
    if let State::StartupFiles { simulation } = &mut changed {
        simulation.view = teaching::View::Example;
    } else {
        panic!()
    };
    assert_eq!(before, assignment_state(&changed));
    let State::StartupFiles { simulation } = &mut changed else {
        panic!()
    };
    simulation
        .assignment
        .files
        .0
        .get_mut("~/.bash_profile")
        .unwrap()
        .text
        .push_str("# different assignment draft\n");
    assert_ne!(
        before,
        assignment_state(&changed),
        "Draft changes must never be hidden as navigation"
    );
    let mut changed = state.clone();
    let State::StartupFiles { simulation } = &mut changed else {
        panic!()
    };
    simulation.assignment.case = 1;
    assert_ne!(
        before,
        assignment_state(&changed),
        "Selected sample is preserved by the unchanged full-state assertion"
    );
}

#[test]
fn rollout_navigation_normalization_retains_actual_library_recipes_and_drafts() {
    let ctx = egui::Context::default();
    let mut app = LabApp::from_context_with_pilot(&ctx, true, false, true);
    app.smoke = false;
    let i = app
        .scenarios
        .iter()
        .position(|s| s.id == "lpic1-library-search")
        .unwrap();
    app.launch(i);
    let state = &app.progress.attempts["lpic1-library-search"].state;
    let before = assignment_state(state);
    let mut changed = state.clone();
    let State::SharedLibraries { simulation } = &mut changed else {
        panic!()
    };
    simulation.view = dlms_learning_lab::shared_libraries_workspace::View::Example;
    assert_eq!(before, assignment_state(&changed));
    let State::SharedLibraries { simulation } = &mut changed else {
        panic!()
    };
    simulation.assignment.launch_drafts[0].value = "/opt/site/lib".into();
    assert_ne!(
        before,
        assignment_state(&changed),
        "Pending application edits must not be hidden as navigation"
    );
    let mut changed = state.clone();
    let State::SharedLibraries { simulation } = &mut changed else {
        panic!()
    };
    simulation.assignment.recipes[0].environment = Some("/opt/common/lib".into());
    assert_ne!(
        before,
        assignment_state(&changed),
        "Applied settings must stay in the exact navigation comparison"
    );
}
