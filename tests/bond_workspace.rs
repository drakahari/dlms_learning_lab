//! No registered lesson yet: exercise explicit current membership in bounded sessions.
#[path = "support/bond_workspace.rs"]
mod f;
use dlms_learning_lab::{nm_workspace as w, script_teaching::View};
#[test]
fn three_cases_report_actual_partial_progress_and_alternate_primary_is_valid() {
    let d = f::definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let rows = t.run(&t.starter).unwrap();
        assert_eq!(rows.iter().filter(|r| r.passed()).count(), 0);
        assert!(rows.iter().all(|r| r.checks.iter().any(|c| c.0)));
        let rows = t.run(&f::partial(t)).unwrap();
        assert_eq!(
            rows.iter().map(w::CaseResult::passed).collect::<Vec<_>>(),
            [true, false, true]
        );
        assert!(
            rows[1]
                .checks
                .iter()
                .any(|c| !c.0 && c.1.contains("no selected monitored member"))
        );
        assert!(
            t.run(&f::solved(t))
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
        let mut alt = f::solved(t);
        let b = alt.get_mut("bond.nmconnection").unwrap();
        *b = b
            .replace("primary=enp1s0", "primary=enp2s0")
            .replace("mode=active-backup", "mode=1");
        assert!(t.run(&alt).unwrap().iter().all(w::CaseResult::passed));
    }
}
#[test]
fn example_credit_is_separate_and_changes_clear_assignment_proof_until_rechecked() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    assert_eq!(s.assignment.selected, "bond.nmconnection");
    let initial = s.assignment.clone();
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert!(!d.solved(&s));
    assert_eq!(s.assignment, initial);
    s.assignment.replace(&f::solved(&d.initial)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    s.assignment.replace(&f::partial(&d.initial)).unwrap();
    assert!(!d.solved(&s));
    assert!(s.assignment.tested.is_none());
    s.assignment.replace(&f::solved(&d.initial)).unwrap();
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
}
#[test]
fn source_bound_resume_reset_and_draft_selection_are_preserved() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.view = View::Assignment;
    s.assignment.replace(&f::solved(&d.initial)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    s.practice.select("backup.nmconnection").unwrap();
    s.practice
        .edit("backup.nmconnection", "# unfinished independent practice")
        .unwrap();
    let restored = w::Session::from_json(&serde_json::to_string(&s).unwrap(), &d).unwrap();
    assert_eq!(restored, s);
    assert!(d.solved(&restored));
    let reset = w::Session::new(&d).unwrap();
    assert_eq!(reset.assignment.draft, d.initial.starter);
    assert!(!d.solved(&reset));
    let mut wrong = serde_json::to_value(&s).unwrap();
    wrong["origin"] = "different-source".into();
    assert!(w::Session::from_json(&wrong.to_string(), &d).is_err());
}
#[test]
fn explicit_current_controls_and_disjoint_practice_identities_are_required() {
    let d = f::definition();
    let mut bad = d.clone();
    bad.initial.cases[0].current_controls = None;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0]
        .current_controls
        .as_mut()
        .unwrap()
        .members[1]
        .0 = "invented".into();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0].environment.bond = None;
    assert!(bad.validate().is_err());
    let mut bad = d;
    for c in &mut bad.example.cases {
        let b = c.environment.bond.as_mut().unwrap();
        for x in std::iter::once(&mut b.profiles.controller).chain(&mut b.profiles.members) {
            x.uuid = x.uuid.replace("10000030", "10000020");
        }
        for text in c.current_files.values_mut() {
            *text = text.replace("10000030", "10000020");
        }
    }
    for fs in [&mut bad.example.starter, &mut bad.example_worked] {
        for text in fs.values_mut() {
            *text = text.replace("10000030", "10000020");
        }
    }
    assert!(
        bad.validate()
            .unwrap_err()
            .contains("control-profile identities")
    );
}
#[test]
fn malformed_or_unsupported_control_settings_produce_diagnostics_and_never_award_credit() {
    let d = f::definition();
    for bad in [
        "[bond\nmode=active-backup",
        "[bond]\nmode=802.3ad\n",
        "[bond]\nmiimon=0\n",
    ] {
        let mut fs = f::solved(&d.initial);
        fs.insert("bond.nmconnection".into(), bad.into());
        let rows = d.initial.run(&fs).unwrap();
        assert!(rows.iter().all(|r| !r.passed() && r.diagnostic.is_some()));
    }
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.tested = Some(s.assignment.draft.clone());
    assert!(!d.solved(&s));
}
#[test]
fn stop_mid_run_edits_and_escape_paths_remain_bounded_and_do_not_publish_results() {
    let d = f::definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&f::solved(&d.initial)).unwrap();
    let mut run = s.assignment.begin(&d.initial).unwrap();
    assert!(!run.tick());
    run.cancel();
    for _ in 0..w::MAX_TICKS {
        assert!(!run.tick());
    }
    assert!(run.results().is_none());
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    let mut run = s.assignment.begin(&d.initial).unwrap();
    for _ in 0..w::MAX_TICKS {
        if run.tick() {
            break;
        }
    }
    assert!(run.done());
    s.assignment
        .edit("bond.nmconnection", "# changed during run")
        .unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
    let before = s.clone();
    assert!(
        s.assignment
            .edit("../../host-profile", "never opened")
            .is_err()
    );
    assert!(
        s.assignment
            .edit("bond.nmconnection", &"x".repeat(16385))
            .is_err()
    );
    assert_eq!(before, s);
}

#[test]
fn exact_source_validation_reuse_never_hides_changed_source_or_learner_state() {
    let original = f::definition();
    original.validate().unwrap();
    original.validate().unwrap();
    let mut broken = original.clone();
    broken
        .example_worked
        .get_mut("bond.nmconnection")
        .unwrap()
        .push_str("\n[bond]\nmode=802.3ad\n");
    assert!(
        broken.validate().is_err(),
        "changed worked output must be revalidated"
    );
    let mut broken = original.clone();
    broken.initial.instructions.clear();
    assert!(broken.validate().is_err());
    let mut broken = original.clone();
    broken.initial.cases[0].current_controls = None;
    assert!(broken.validate().is_err());
    let mut another = original.clone();
    another
        .example_worked
        .get_mut("bond.nmconnection")
        .unwrap()
        .push_str("\n# another valid source value\n");
    another.validate().unwrap();
    original.validate().unwrap();
    let mut session = w::Session::new(&original).unwrap();
    session
        .assignment
        .replace(&f::solved(&original.initial))
        .unwrap();
    session.assignment.run(&original.initial).unwrap();
    assert!(original.solved(&session));
    session
        .assignment
        .replace(&f::partial(&original.initial))
        .unwrap();
    assert!(!original.solved(&session));
    session.assignment.tested = Some(session.assignment.draft.clone());
    assert!(
        !original.solved(&session),
        "forged current proof cannot cache a grade"
    );
}
