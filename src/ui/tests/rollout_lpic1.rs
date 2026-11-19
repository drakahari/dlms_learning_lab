//! Focused presentation regressions. These do not claim novice/native acceptance.
use super::*;

fn normalized(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}
#[test]
fn rollout_lpic1_inventory_preserves_every_original_requirement_clause() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../../../scenarios/presentation/lpic1.json")).unwrap();
    let rows = rows.as_array().unwrap();
    let (catalog, _) = ccna_pilot::catalog();
    let labs: Vec<_> = catalog.iter().filter(|s| s.track == "LPIC-1").collect();
    assert_eq!(rows.len(), labs.len());
    assert_eq!(rows.len(), 101); // Current registered LPIC-1 catalog; keep every per-lab assertion below.
    for s in labs {
        let matches: Vec<_> = rows.iter().filter(|r| r["id"] == s.id).collect();
        assert_eq!(matches.len(), 1, "missing/duplicate {}", s.id);
        let row = matches[0];
        assert_eq!(row["status"], "updated");
        let requirements = row["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r.as_str().unwrap())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            normalized(&requirements),
            normalized(&s.objective),
            "requirement altered: {}",
            s.id
        );
        let steps = row["practice"].as_array().unwrap();
        if let engine::Goal::SharedLibraries { definition } = &s.goal {
            // This native family presents its real typed worked actions, then
            // one change/observe exercise. Preserve the three-row contract for
            // every original lesson instead of adding redundant UI-only prose.
            let count = match s.id.as_str() {
                "lpic1-library-search" => 3,
                "lpic1-library-cache-handover" => 6,
                _ => panic!("Unreviewed shared-library teaching entry"),
            };
            assert_eq!(definition.lesson.worked.len(), count);
            assert_eq!(definition.lesson.concepts.len(), 5);
            assert_ne!(definition.initial, definition.lesson.initial);
            assert!(
                definition.lesson.worked.iter().all(
                    |step| !step.title.trim().is_empty() && !step.explanation.trim().is_empty()
                )
            );
            definition.verify_worked().unwrap();
            assert_eq!(steps.len(), 1);
        } else if let engine::Goal::FileIndex { definition } = &s.goal {
            assert_eq!(steps.len(), 8);
            assert_eq!(definition.teaching.try_steps.len(), 8);
            assert_ne!(
                definition.assignment.initial.context,
                definition.example.initial.context
            );
            definition.validate_reachability().unwrap();
        } else {
            assert_eq!(steps.len(), 3);
        }
        for step in steps {
            for key in ["title", "action", "observe"] {
                assert!(
                    !step[key].as_str().unwrap().trim().is_empty(),
                    "{} {key}",
                    s.id
                );
            }
        }
        for view in ["introduction", "example", "assignment"] {
            assert!(
                !row["review"][view].as_str().unwrap().trim().is_empty(),
                "{} {view}",
                s.id
            );
        }
    }
}
fn painted_text(ctx: &egui::Context, a: &mut LabApp) -> String {
    fn append(s: &egui::Shape, out: &mut String) {
        match s {
            egui::Shape::Text(t) => {
                out.push_str(&t.galley.job.text);
                out.push('\n');
            }
            egui::Shape::Vec(v) => {
                for s in v {
                    append(s, out)
                }
            }
            _ => {}
        }
    }
    let o = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1200., 2400.),
            )),
            ..Default::default()
        },
        |ctx| a.render(ctx),
    );
    let mut out = String::new();
    for s in o.shapes {
        append(&s.shape, &mut out);
    }
    out
}
#[test]
fn rollout_lpic1_terminal_and_account_views_teach_then_keep_independent_work() {
    let (ctx, mut a) = app();
    for id in [
        "lpic1-file-navigation",
        "lpic1-process-pause",
        "lpic1-account-onboarding",
        "lpic1-package-payload",
    ] {
        let i = a.scenarios.iter().position(|s| s.id == id).unwrap();
        a.launch(i);
        for view in [
            teaching::View::Introduction,
            teaching::View::Example,
            teaching::View::Assignment,
        ] {
            match id {
                "lpic1-file-navigation" => a.set_terminal_view(i, view),
                "lpic1-process-pause" => a.set_process_view(i, view),
                "lpic1-account-onboarding" => a.set_accounts_view(i, view),
                _ => a.set_debian_packages_view(i, view),
            }
            let text = painted_text(&ctx, &mut a);
            match view {
                teaching::View::Introduction => {
                    assert!(text.contains("WHAT YOU’LL LEARN"), "{id}");
                    assert!(text.contains("Worked example"), "{id}");
                }
                teaching::View::Example => {
                    let entry = lesson_presentation::entry(&a.scenarios[i]).unwrap();
                    assert!(text.contains(&entry.practice[0].title), "{id}");
                    assert!(
                        text.contains("Insert command"),
                        "{id} must retain terminal command practice"
                    );
                }
                teaching::View::Assignment => assert!(text.contains("Requirements"), "{id}"),
            }
            assert!(
                !engine::solved(&a.progress.attempts[id].state, &a.scenarios[i].goal),
                "presentation must not complete {id}"
            );
        }
    }
}
