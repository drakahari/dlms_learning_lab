//! Two complete source tasks; actual Fedora/native/package acceptance is separate.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    lvm_workspace::{self as w, config_scan as f},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt},
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-lvm-filter-policy", "lpic2-lvm-devices-file"];
fn labs() -> &'static Vec<Scenario> {
    static LABS: OnceLock<Vec<Scenario>> = OnceLock::new();
    LABS.get_or_init(|| {
        serde_json::from_str::<Pack>(include_str!("../scenarios/lvm-config-lessons-v1.json"))
            .unwrap()
            .scenarios
    })
}
fn d(s: &Scenario) -> &w::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(a: &mut Attempt) -> &mut w::Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    for op in &d(s).config_scan_task.as_ref().unwrap().reference {
        sim(&mut a).config_action(d(s), op, false).unwrap();
    }
    assert!(
        progress::current_solved(&a, s),
        "{:?}",
        d(s).checks(sim(&mut a))
    );
    a
}
fn config(flag: bool, rules: &[&str]) -> f::Text {
    f::Text(format!(
        "devices {{use_devicesfile={} filter=[{}]}}",
        u8::from(flag),
        rules
            .iter()
            .map(|s| serde_json::to_string(s).unwrap())
            .collect::<Vec<_>>()
            .join(",")
    ))
}
fn selection(paths: &[&str]) -> f::Selection {
    f::Selection {
        entries: paths.iter().map(|p| f::DeviceName((*p).into())).collect(),
    }
}
#[test]
fn both_distinct_authored_tasks_have_complete_reachable_teaching_and_no_solved_starter() {
    assert_eq!(
        labs().iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        IDS
    );
    for s in labs() {
        w::source(s).unwrap();
        d(s).validate().unwrap();
        assert_eq!(d(s).lesson.concepts.len(), 4);
        assert!(!progress::current_solved(&Attempt::new(s, 1), s));
        let t = d(s).config_scan_task.as_ref().unwrap();
        let p = d(s).lesson.config_scan_practice.as_ref().unwrap();
        assert!(t.context.cases.len() >= 2);
        assert!(p.context.cases.len() >= 2);
        assert!(
            t.reference
                .iter()
                .filter_map(|op| if let f::Operation::SaveConfig { text } = op {
                    Some(text)
                } else {
                    None
                })
                .all(|value| value != &t.context.initial_config)
        );
        assert!(s.simplifications.contains("not real hardware IDs"));
        assert!(s.simplifications.contains("Sources checked2026-11-07"));
        assert_eq!(
            s.solution,
            t.reference.iter().map(|op| op.label()).collect::<Vec<_>>()
        );
        let mut a = Attempt::new(s, 1);
        let original = a.state.clone();
        for step in &p.worked {
            sim(&mut a)
                .config_action(d(s), &step.operation, true)
                .unwrap();
        }
        assert!(!progress::current_solved(&a, s));
        let State::Lvm {
            simulation: initial,
        } = &original
        else {
            panic!()
        };
        assert_eq!(
            sim(&mut a).config_scan_runtime.as_ref().unwrap().assignment,
            initial.config_scan_runtime.as_ref().unwrap().assignment
        );
        assert!(d(s).practice_checks(sim(&mut a)).iter().all(|r| r.0));
        solve(s);
    }
    assert!(
        d(&labs()[0])
            .config_scan_task
            .as_ref()
            .unwrap()
            .context
            .initial_file
            .is_none()
    );
    assert!(
        d(&labs()[1])
            .config_scan_task
            .as_ref()
            .unwrap()
            .context
            .selection_editable
    );
}
#[test]
fn filter_equivalents_pass_and_shadowing_lookalikes_default_allow_and_single_case_shortcuts_fail() {
    let s = &labs()[0];
    for rules in [
        vec!["a|^/dev/data-(main|spare)$|", "r|.*|"],
        vec!["a#^/dev/data-spare$#", "a#^/dev/data-main$#", "r#.*#"],
    ] {
        let mut a = Attempt::new(s, 1);
        sim(&mut a)
            .config_action(
                d(s),
                &f::Operation::SaveConfig {
                    text: config(false, &rules),
                },
                false,
            )
            .unwrap();
        sim(&mut a)
            .config_action(d(s), &f::Operation::Scan, false)
            .unwrap();
        assert!(progress::current_solved(&a, s));
    }
    for rules in [
        vec!["r|.*|", "a|^/dev/data-(main|spare)$|"],
        vec!["a|^/dev/data-main$|", "r|.*|"],
        vec!["a|^/dev/data-main|", "a|^/dev/data-spare|", "r|.*|"],
        vec!["a|^/dev/data-(main|spare)$|"],
        vec!["a|.*|"],
    ] {
        let mut a = solve(s);
        sim(&mut a)
            .config_action(
                d(s),
                &f::Operation::SaveConfig {
                    text: config(false, &rules),
                },
                false,
            )
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        sim(&mut a)
            .config_action(d(s), &f::Operation::Scan, false)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        let rows = d(s).checks(sim(&mut a));
        assert!(rows.iter().any(|r| !r.0 && r.1.contains("Needs to be")));
    }
}
#[test]
fn devices_file_cannot_be_bypassed_with_correct_filters_and_both_config_and_roster_matter() {
    let s = &labs()[1];
    let mut a = Attempt::new(s, 1);
    sim(&mut a)
        .config_action(d(s), &f::Operation::Scan, false)
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        d(s).checks(sim(&mut a))
            .iter()
            .any(|r| !r.0 && r.1.contains("Use the provided devices file"))
    );
    sim(&mut a)
        .config_action(
            d(s),
            &f::Operation::SaveSelection {
                selection: selection(&["/dev/pool-b", "/dev/pool-a"]),
            },
            false,
        )
        .unwrap();
    sim(&mut a)
        .config_action(d(s), &f::Operation::Scan, false)
        .unwrap();
    assert!(!progress::current_solved(&a, s));
    sim(&mut a)
        .config_action(
            d(s),
            &f::Operation::SaveConfig {
                text: config(true, &["r|.*|"]),
            },
            false,
        )
        .unwrap();
    sim(&mut a)
        .config_action(d(s), &f::Operation::Scan, false)
        .unwrap();
    assert!(progress::current_solved(&a, s));
    let reports = sim(&mut a)
        .config_scan_runtime
        .as_ref()
        .unwrap()
        .assignment
        .last_results(&d(s).config_scan_task.as_ref().unwrap().context)
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(
        reports
            .iter()
            .all(|r| r.mode == dlms_learning_lab::lvm::config_scan::Mode::DevicesFile)
    );
    for paths in [
        &[][..],
        &["/dev/pool-a"][..],
        &["/dev/pool-a", "/dev/pool-b", "/dev/import-copy"][..],
    ] {
        sim(&mut a)
            .config_action(
                d(s),
                &f::Operation::SaveSelection {
                    selection: selection(paths),
                },
                false,
            )
            .unwrap();
        sim(&mut a)
            .config_action(d(s), &f::Operation::Scan, false)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn partial_drafts_failed_runs_resume_and_selected_reset_preserve_truth_and_example_work() {
    for s in labs() {
        let mut a = solve(s);
        let original = sim(&mut a).clone();
        sim(&mut a)
            .config_edit(d(s), "# unfinished assignment\ndevices {".into(), false)
            .unwrap();
        sim(&mut a)
            .config_edit(d(s), "# unfinished example\ndevices {".into(), true)
            .unwrap();
        assert!(!progress::current_solved(&a, s));
        let bytes = serde_json::to_vec(&a).unwrap();
        let mut a: Attempt = serde_json::from_slice(&bytes).unwrap();
        w::context(&a, s).unwrap();
        assert_eq!(
            sim(&mut a)
                .config_scan_runtime
                .as_ref()
                .unwrap()
                .assignment
                .draft
                .0,
            "# unfinished assignment\ndevices {"
        );
        sim(&mut a).reset_example(d(s)).unwrap();
        assert!(!progress::current_solved(&a, s));
        assert_eq!(
            sim(&mut a)
                .config_scan_runtime
                .as_ref()
                .unwrap()
                .assignment
                .saved,
            original
                .config_scan_runtime
                .as_ref()
                .unwrap()
                .assignment
                .saved
        );
        let reset = Attempt::new(s, 2);
        assert!(!progress::current_solved(&reset, s));
        assert_eq!(
            sim(&mut a)
                .config_scan_runtime
                .as_ref()
                .unwrap()
                .assignment
                .draft
                .0,
            "# unfinished assignment\ndevices {"
        );
        let bad = f::Operation::SaveConfig {
            text: f::Text("devices {\nfilter=[\"a|.*|\"]\n}".into()),
        };
        sim(&mut a).config_action(d(s), &bad, false).unwrap();
        assert!(
            sim(&mut a)
                .config_action(d(s), &f::Operation::Scan, false)
                .unwrap_err()
                .contains("Unsupported:")
        );
        let bytes = serde_json::to_vec(&a).unwrap();
        let a: Attempt = serde_json::from_slice(&bytes).unwrap();
        w::context(&a, s).unwrap();
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn current_catalog_contains_new_tasks_and_preserves_all331prior_values_and113unit_values() {
    let old = pack::accepted331().unwrap();
    let current = pack::accepted333().unwrap();
    assert_eq!((old.len(), current.len()), (331, 333));
    assert_eq!(
        serde_json::to_value(&current[..old.len()]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        current[331..]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    let old_units = course::accepted331(&old).unwrap();
    let units = course::accepted333(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (113, 114));
    let retained = units
        .iter()
        .filter(|u| u.id != "lvm-device-selection")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "lvm-device-selection")
        .unwrap();
    assert_eq!(units[at - 1].id, "lvm-application-references");
    assert_eq!(units[at].labs, IDS);
    for (track, count) in [
        ("CCNA", 74),
        ("LPIC-1", 114),
        ("LPIC-2", 68),
        ("Bash/Shell", 77),
    ] {
        let actual = current.iter().filter(|s| s.track == track).count();
        assert_eq!(actual, count, "{track}");
    }
}
#[test]
fn malformed_or_mixed_authored_profiles_and_forged_initial_progress_are_rejected() {
    for s in labs() {
        let mut bad = s.clone();
        let Goal::Lvm { definition } = &mut bad.goal else {
            panic!()
        };
        definition.config_scan_task.as_mut().unwrap().context.cases[0].expected[0].path =
            "/dev/not-supplied".into();
        assert!(w::source(&bad).is_err());
        let mut bad = s.clone();
        bad.initial = solve(s).state;
        assert!(w::source(&bad).is_err());
        let mut a = Attempt::new(s, 1);
        let before = a.state.clone();
        for input in [
            "sh -c pvs",
            "lvmdevices --adddev /dev/pool-a",
            "pvs",
            "simulated-lvm-reference {}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
            assert_eq!(a.state, before);
        }
    }
}
#[test]
fn taught_unmatched_default_is_actually_observed_and_renderer_owns_section_numbering() {
    for s in labs() {
        for concept in &d(s).lesson.concepts {
            assert!(
                !concept.title.chars().next().unwrap().is_ascii_digit(),
                "Renderer numbers sections: {}",
                concept.title
            );
        }
        assert!(!d(s).lesson.description.starts_with("Worked example:"));
    }
    let p = d(&labs()[0]).lesson.config_scan_practice.as_ref().unwrap();
    let mut live = f::Live::new(&p.context).unwrap();
    for step in &p.worked {
        live.apply(&p.context, &step.operation).unwrap();
    }
    for op in &p.experiment {
        live.apply(&p.context, op).unwrap();
    }
    let reports = live.last_results(&p.context).unwrap().unwrap().unwrap();
    assert!(reports.iter().any(|r| {
        r.records.iter().any(|r| {
            r.allowed && r.reason == dlms_learning_lab::lvm::config_scan::Reason::NoFilterMatched
        })
    }));
    assert!(
        live.checks(&p.context)
            .iter()
            .any(|r| !r.0 && r.1.contains("no filter matched, so the default allows it"))
    );
}
