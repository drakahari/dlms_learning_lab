//! Source-bound native template adapter regressions; no host account execution.
use dlms_learning_lab::{
    accounts::{Entry, Kind, Requirement, SkeletonLeaf, State},
    accounts_teaching::{self, Learning, Lesson},
    accounts_template::{self, Drafts, Step, Teaching},
    pack::CommandExample,
};
use std::collections::BTreeMap;
fn old_lesson() -> Lesson {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/accounts-lifecycle-v1.json")).unwrap();
    serde_json::from_value(v["scenarios"][0]["accounts_teaching"].clone()).unwrap()
}
fn source(name: &str) -> State {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/accounts-lifecycle-v1.json")).unwrap();
    let old: State =
        serde_json::from_value(v["scenarios"][0]["initial"]["simulation"].clone()).unwrap();
    let mut p = old.policy().clone();
    let mut c = old.configuration().clone();
    p.fixture = name.into();
    c.entries.insert(
        "/etc/skel/private-note".into(),
        Entry {
            uid: 0,
            gid: 0,
            mode: 0o600,
            data: Kind::Text {
                contents: "Synthetic note not for a new home\n".into(),
            },
        },
    );
    p.editable_skeleton_leaves = BTreeMap::from([
        (
            "/etc/skel/.profile".into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: false,
            },
        ),
        (
            "/etc/skel/private-note".into(),
            SkeletonLeaf {
                uid: 0,
                gid: 0,
                allow_remove: true,
            },
        ),
    ]);
    State::new(p, c).unwrap()
}
fn save(contents: &str) -> Step {
    Step::Save {
        path: "/etc/skel/.profile".into(),
        contents: contents.into(),
        mode: 0o640,
    }
}
fn teaching() -> Teaching {
    Teaching {
        assignment: vec![save("Assignment-only default\n")],
        example: vec![
            save("Worked default\n"),
            Step::Remove {
                path: "/etc/skel/private-note".into(),
            },
        ],
        try_steps: vec![save("Trial default\n")],
        repair: vec![save("Worked default\n")],
    }
}
fn text_entry(contents: &str, uid: u32, gid: u32) -> Entry {
    Entry {
        uid,
        gid,
        mode: 0o640,
        data: Kind::Text {
            contents: contents.into(),
        },
    }
}
fn lesson() -> Lesson {
    let mut l = old_lesson();
    l.fixture = source("template-example");
    l.template_teaching = Some(teaching());
    l.example = vec!["useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash -m probe".into()];
    l.example_commands = vec![CommandExample {
        command: l.example[0].clone(),
        explanation: "Create a home from the saved virtual template".into(),
        simulator_only: false,
    }];
    l.outcomes = vec![
        Requirement::Entry {
            path: "/etc/skel/.profile".into(),
            expected: text_entry("Worked default\n", 0, 0),
        },
        Requirement::Entry {
            path: "/home/probe/.profile".into(),
            expected: text_entry("Worked default\n", 3201, 1000),
        },
        Requirement::AbsentPath {
            path: "/etc/skel/private-note".into(),
        },
    ];
    l.try_actions = vec!["cat /etc/skel/.profile".into()];
    l.try_outcomes = vec![Requirement::Entry {
        path: "/etc/skel/.profile".into(),
        expected: text_entry("Trial default\n", 0, 0),
    }];
    l.repair = vec!["id probe".into()];
    l
}
fn run_reference(l: &Lesson) -> Learning {
    let mut learn = Learning::new(l);
    let t = l.template_teaching.as_ref().unwrap();
    accounts_template::apply_phase(&l.fixture, &mut learn.simulation, &t.example).unwrap();
    for cmd in &l.example {
        learn.command = cmd.clone();
        learn.execute(l).unwrap();
    }
    // Deliberate explicit refresh after author-controlled work; never fill answers at construction.
    learn.template_drafts = Some(Drafts::new(&l.fixture, &learn.simulation).unwrap());
    learn
}
#[test]
fn old_none_wire_behavior_and_old_validation_are_unchanged() {
    let l = old_lesson();
    assert!(l.template_teaching.is_none());
    l.validate().unwrap();
    let v = serde_json::to_value(&l).unwrap();
    assert!(v.get("template_teaching").is_none());
    let learn = Learning::new(&l);
    assert!(learn.template_drafts.is_none());
    assert!(learn.valid_for(&l));
    let raw = serde_json::to_string(&learn).unwrap();
    assert!(!raw.contains("template_drafts"));
    let d: Learning = serde_json::from_str(&raw).unwrap();
    assert!(d.valid_for(&l));
    assert_eq!(
        serde_json::to_value(&d).unwrap(),
        serde_json::to_value(&learn).unwrap()
    );
}
#[test]
fn typed_phases_precede_real_commands_and_lesson_validation_reaches_actual_home() {
    let l = lesson();
    l.validate().unwrap();
    let learn = run_reference(&l);
    assert!(learn.checks(&l).iter().all(|x| x.0));
    assert_eq!(
        learn.simulation.configuration().entries["/home/probe/.profile"],
        text_entry("Worked default\n", 3201, 1000)
    );
    assert!(
        !learn
            .simulation
            .configuration()
            .entries
            .contains_key("/home/probe/private-note")
    );
}
#[test]
fn assignment_reference_is_explicit_never_called_by_new_and_accepts_held_assignment() {
    let l = lesson();
    let assignment = source("template-assignment");
    l.validate_assignment_templates(&assignment).unwrap();
    let before = assignment.clone();
    let learn = Learning::new(&l);
    assert_eq!(learn.simulation, l.fixture);
    assert_eq!(assignment, before);
    let worked = l
        .template_teaching
        .as_ref()
        .unwrap()
        .assignment_reference(&assignment)
        .unwrap();
    assert_eq!(
        worked.configuration().entries["/etc/skel/.profile"],
        text_entry("Assignment-only default\n", 0, 0)
    );
    assert_ne!(worked, learn.simulation);
}
#[test]
fn selected_save_canonicalizes_only_that_mode_and_preserves_other_pending_edits() {
    let s = source("template-drafts");
    let mut c = s.clone();
    let mut d = Drafts::new(&s, &c).unwrap();
    d.selected = "/etc/skel/.profile".into();
    let row = d.leaves.get_mut(&d.selected).unwrap();
    row.contents = "Saved\n".into();
    row.mode = "640".into();
    d.leaves.get_mut("/etc/skel/private-note").unwrap().contents = "Unsaved other\n".into();
    let other = d.leaves["/etc/skel/private-note"].clone();
    assert!(!d.matches_state(&s, &c));
    d.save_selected(&s, &mut c).unwrap();
    assert_eq!(d.leaves[&d.selected].mode, "0640");
    assert_eq!(d.leaves["/etc/skel/private-note"], other);
    assert!(!d.matches_state(&s, &c));
    assert_eq!(
        c.configuration().entries["/etc/skel/private-note"],
        s.configuration().entries["/etc/skel/private-note"]
    );
}
#[test]
fn invalid_octal_and_nul_raw_drafts_roundtrip_but_save_is_atomic() {
    let s = source("template-invalid");
    for (contents, mode) in [
        ("Pending\n", "08"),
        ("Bad\0text", "0644"),
        ("Pending", "1644"),
        ("Pending", ""),
    ] {
        let mut c = s.clone();
        let mut d = Drafts::new(&s, &c).unwrap();
        d.selected = "/etc/skel/.profile".into();
        let row = d.leaves.get_mut(&d.selected).unwrap();
        row.contents = contents.into();
        row.mode = mode.into();
        assert!(d.valid_for(&s, &c));
        let raw = serde_json::to_string(&d).unwrap();
        let mut resumed = Drafts::from_json(&s, &c, &raw).unwrap();
        assert_eq!(d, resumed);
        assert!(!resumed.matches_state(&s, &c));
        assert!(resumed.save_selected(&s, &mut c).is_err());
        assert_eq!(c, s);
        assert_eq!(resumed, d);
    }
}
#[test]
fn removed_leaf_draft_uses_source_text_not_an_invented_answer_and_can_recreate() {
    let s = source("template-removal");
    let mut c = s.clone();
    let mut d = Drafts::new(&s, &c).unwrap();
    d.selected = "/etc/skel/private-note".into();
    d.leaves.get_mut(&d.selected).unwrap().included = false;
    d.save_selected(&s, &mut c).unwrap();
    assert!(d.matches_state(&s, &c));
    let mut resumed = Drafts::new(&s, &c).unwrap();
    let key = "/etc/skel/private-note";
    assert!(!resumed.leaves[key].included);
    assert_eq!(
        resumed.leaves[key].contents,
        "Synthetic note not for a new home\n"
    );
    resumed.selected = key.into();
    resumed.leaves.get_mut(key).unwrap().included = true;
    resumed.save_selected(&s, &mut c).unwrap();
    assert_eq!(
        c.configuration().entries[key],
        s.configuration().entries[key]
    );
}
#[test]
fn removal_of_required_leaf_is_saved_as_pending_work_then_refused_without_mutation() {
    let s = source("template-required");
    let mut c = s.clone();
    let mut d = Drafts::new(&s, &c).unwrap();
    d.selected = "/etc/skel/.profile".into();
    d.leaves.get_mut(&d.selected).unwrap().included = false;
    assert!(d.valid_for(&s, &c));
    assert!(!d.matches_state(&s, &c));
    let before = d.clone();
    assert!(d.save_selected(&s, &mut c).is_err());
    assert_eq!(d, before);
    assert_eq!(c, s);
}
#[test]
fn source_policy_and_full_initial_identity_are_not_imported_authority() {
    let s = source("template-source");
    let c = s.clone();
    let d = Drafts::new(&s, &c).unwrap();
    let mut foreign = d.clone();
    foreign.source = source("template-foreign");
    assert!(!foreign.valid_for(&s, &c));
    assert!(Drafts::from_json(&s, &c, &serde_json::to_string(&foreign).unwrap()).is_err());
    let mut config = s.configuration().clone();
    config.entries.get_mut("/etc/skel/.profile").unwrap().mode = 0o600;
    let changed = State::new(s.policy().clone(), config).unwrap();
    foreign.source = changed;
    assert!(!foreign.valid_for(&s, &c));
    let mut p = s.policy().clone();
    p.editable_skeleton_leaves.clear();
    foreign.source = State::new(p, s.configuration().clone()).unwrap();
    assert!(!foreign.valid_for(&s, &c));
}
#[test]
fn keys_selection_version_and_erased_context_refuse_and_preserve_actual_work() {
    let s = source("template-keys");
    let c = s.clone();
    let d = Drafts::new(&s, &c).unwrap();
    for kind in 0..5 {
        let mut bad = d.clone();
        match kind {
            0 => {
                bad.leaves.remove("/etc/skel/.profile");
            }
            1 => {
                bad.leaves.insert(
                    "/etc/foreign".into(),
                    bad.leaves["/etc/skel/.profile"].clone(),
                );
            }
            2 => bad.selected = "/home/pax/.profile".into(),
            3 => bad.version = 2,
            _ => bad.selected = "/etc/skel/../skel/.profile".into(),
        }
        assert!(!bad.valid_for(&s, &c));
        assert!(Drafts::from_json(&s, &c, &serde_json::to_string(&bad).unwrap()).is_err());
    }
}
#[test]
fn pending_practice_survives_real_investigation_and_resume_without_completion_credit() {
    let l = lesson();
    let mut learn = run_reference(&l);
    let d = learn.template_drafts.as_mut().unwrap();
    d.selected = "/etc/skel/.profile".into();
    d.leaves.get_mut(&d.selected).unwrap().contents = "Unsaved new work\n".into();
    let before = d.clone();
    learn.command = "id probe".into();
    learn.execute(&l).unwrap();
    assert_eq!(learn.template_drafts.as_ref().unwrap(), &before);
    let checks = learn.checks(&l);
    assert!(checks[..checks.len() - 1].iter().all(|x| x.0));
    assert!(!checks.last().unwrap().0);
    let raw = serde_json::to_string(&learn).unwrap();
    let resumed: Learning = serde_json::from_str(&raw).unwrap();
    assert!(resumed.valid_for(&l));
    assert!(!resumed.checks(&l).iter().all(|x| x.0));
    let reset = Learning::new(&l);
    assert!(
        reset
            .template_drafts
            .as_ref()
            .unwrap()
            .matches_state(&l.fixture, &reset.simulation)
    );
    assert_eq!(reset.simulation, l.fixture);
}
#[test]
fn erasing_learning_drafts_or_crossing_sources_does_not_restore_success() {
    let l = lesson();
    let mut learn = run_reference(&l);
    learn.template_drafts = None;
    assert!(!learn.valid_for(&l));
    assert!(!learn.checks(&l)[0].0);
    let mut learn = run_reference(&l);
    learn.template_drafts.as_mut().unwrap().source = source("other-template-source");
    assert!(!learn.valid_for(&l));
    let mut unannotated = l.clone();
    unannotated.template_teaching = None;
    assert!(unannotated.validate().is_err());
    assert!(
        unannotated
            .validate_assignment_templates(&l.fixture)
            .is_err()
    );
}
#[test]
fn changed_template_is_copied_once_and_equivalent_explicit_default_k_is_supported() {
    let l = lesson();
    for flags in ["-m", "-m -k /etc/skel"] {
        let mut c = l.fixture.clone();
        accounts_template::apply_phase(
            &l.fixture,
            &mut c,
            &l.template_teaching.as_ref().unwrap().example,
        )
        .unwrap();
        let old_entries = c.configuration().entries.clone();
        accounts_teaching::execute_in_context(
            &l.fixture,
            &mut c,
            &format!("useradd -N -u 3201 -g staff -d /home/probe -s /bin/bash {flags} probe"),
        )
        .unwrap();
        for (p, e) in old_entries {
            assert_eq!(c.configuration().entries.get(&p), Some(&e));
        }
        accounts_template::apply_phase(&l.fixture, &mut c, &[save("Future default\n")]).unwrap();
        assert_eq!(
            c.configuration().entries["/home/probe/.profile"],
            text_entry("Worked default\n", 3201, 1000)
        );
    }
}
#[test]
fn malformed_reference_phases_and_late_failure_are_atomic() {
    let s = source("template-reference");
    let mut c = s.clone();
    for steps in [
        vec![
            save("x"),
            Step::Remove {
                path: "/etc/skel/.profile".into(),
            },
        ],
        vec![
            Step::Remove {
                path: "/etc/skel/private-note".into(),
            },
            Step::Remove {
                path: "/etc/skel/private-note".into(),
            },
        ],
        vec![Step::Save {
            path: "/etc/skel/../skel/.profile".into(),
            contents: "x".into(),
            mode: 0o644,
        }],
        vec![save("x"); 5],
    ] {
        assert!(accounts_template::apply_phase(&s, &mut c, &steps).is_err());
        assert_eq!(c, s);
    }
}
#[test]
fn decoder_bounds_duplicate_leaves_unknown_fields_and_phase_counts_are_strict() {
    let s = source("template-decode");
    let c = s.clone();
    let d = Drafts::new(&s, &c).unwrap();
    let mut value = serde_json::to_value(&d).unwrap();
    value["leaves"]["/etc/skel/.profile"]["contents"] =
        serde_json::json!("x".repeat(accounts_template::MAX_TEXT_BYTES + 1));
    assert!(Drafts::from_json(&s, &c, &serde_json::to_string(&value).unwrap()).is_err());
    let mut value = serde_json::to_value(&d).unwrap();
    value["leaves"]["/etc/skel/.profile"]["mode"] = serde_json::json!("00644");
    assert!(Drafts::from_json(&s, &c, &serde_json::to_string(&value).unwrap()).is_err());
    let key = serde_json::to_string("/etc/skel/.profile").unwrap();
    let row = serde_json::to_string(&d.leaves["/etc/skel/.profile"]).unwrap();
    let duplicate = format!(
        "{{\"version\":1,\"source\":{},\"selected\":{key},\"leaves\":{{{key}:{row},{key}:{row}}}}}",
        serde_json::to_string(&s).unwrap()
    );
    assert!(Drafts::from_json(&s, &c, &duplicate).is_err());
    let mut value = serde_json::to_value(teaching()).unwrap();
    value["example"] = serde_json::json!([save("a"), save("b"), save("c"), save("d"), save("e")]);
    assert!(serde_json::from_value::<Teaching>(value).is_err());
    let raw =
        r#"{"action":"save","path":"/etc/skel/.profile","contents":"x","mode":420,"uid":999}"#;
    assert!(serde_json::from_str::<Step>(raw).is_err());
}
#[test]
fn escaped_full_size_raw_drafts_fit_documented_cap_and_oversized_envelopes_refuse() {
    let s = source("template-escaped");
    let c = s.clone();
    let mut d = Drafts::new(&s, &c).unwrap();
    for row in d.leaves.values_mut() {
        row.contents = "\0".repeat(accounts_template::MAX_TEXT_BYTES);
    }
    let raw = serde_json::to_string(&d).unwrap();
    assert!(raw.len() < accounts_template::MAX_JSON_BYTES);
    assert_eq!(Drafts::from_json(&s, &c, &raw).unwrap(), d);
    assert!(Drafts::from_json(&s, &c, &" ".repeat(accounts_template::MAX_JSON_BYTES + 1)).is_err());
}
#[test]
fn editor_bound_does_not_silently_truncate_a_larger_core_valid_source() {
    let s = source("template-large-source");
    let mut config = s.configuration().clone();
    config.entries.get_mut("/etc/skel/.profile").unwrap().data = Kind::Text {
        contents: "x".repeat(accounts_template::MAX_TEXT_BYTES + 1),
    };
    let large = State::new(s.policy().clone(), config).unwrap();
    assert!(large.valid_initial());
    assert!(Drafts::new(&large, &large).is_err());
    assert!(teaching().validate_example(&large).is_err());
}

#[test]
fn failed_practice_save_keeps_raw_work_and_actual_state_then_retry_invalidates_old_outcome() {
    let l = lesson();
    let mut learn = run_reference(&l);
    let before = learn.simulation.clone();
    learn.checked = true;
    let d = learn.template_drafts.as_mut().unwrap();
    d.selected = "/etc/skel/.profile".into();
    let row = d.leaves.get_mut(&d.selected).unwrap();
    row.contents = "Future default\n".into();
    row.mode = "08".into();
    let raw = learn.template_drafts.clone();
    assert!(learn.save_template(&l).is_err());
    assert!(!learn.checked);
    assert_eq!(learn.simulation, before);
    assert_eq!(learn.template_drafts, raw);
    learn
        .template_drafts
        .as_mut()
        .unwrap()
        .leaves
        .get_mut("/etc/skel/.profile")
        .unwrap()
        .mode = "640".into();
    learn.save_template(&l).unwrap();
    assert!(
        learn
            .template_drafts
            .as_ref()
            .unwrap()
            .matches_state(&l.fixture, &learn.simulation)
    );
    assert!(!learn.checks(&l).iter().all(|x| x.0));
    assert_eq!(
        learn.simulation.configuration().entries["/home/probe/.profile"],
        text_entry("Worked default\n", 3201, 1000)
    );
}
