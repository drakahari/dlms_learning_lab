#[path = "support/nm_workspace.rs"]
mod fixture;
use dlms_learning_lab::{nm_workspace as w, script_teaching::View};
use fixture::*;
#[test]
fn checks_real_current_and_future_settings_with_legitimate_alternatives() {
    let d = definition();
    d.validate().unwrap();
    for t in [&d.initial, &d.example] {
        let rows = t.run(&solved(t)).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(w::CaseResult::passed));
        for r in rows {
            assert!(r.diagnostic.is_none());
            assert_eq!(r.checks.len(), 5);
            assert_eq!(r.current.unwrap().selected().len(), 2);
            assert_eq!(r.future.unwrap().selected().len(), 2);
        }
        let mut alternative = solved(t);
        let app = alternative.get_mut("app.nmconnection").unwrap();
        *app = app.replace("route-metric=100", "route-metric=50");
        assert!(
            t.run(&alternative)
                .unwrap()
                .iter()
                .all(w::CaseResult::passed)
        );
    }
}
#[test]
fn partial_success_and_context_dependent_failures_cannot_hide_behind_passing_rows() {
    let t = task("network-assignment", 20);
    let rows = t.run(&t.starter).unwrap();
    assert!(rows.iter().all(|r| !r.passed()));
    assert!(rows[0].checks.iter().any(|r| r.0));
    assert!(
        rows[0]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("10.20.0.99"))
    );
    let mut fs = solved(&t);
    for text in fs.values_mut() {
        *text = text
            .replace("route-metric=100", "route-metric=-1")
            .replace("route-metric=200", "route-metric=-1");
    }
    let rows = t.run(&fs).unwrap();
    assert!(rows[0].passed());
    assert!(!rows[1].passed());
    assert!(
        rows[1]
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("enp2s0"))
    );
}
#[test]
fn practice_success_does_not_award_assignment_credit_and_edits_clear_success() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.practice.replace(&d.example_worked).unwrap();
    s.practice.run(&d.example).unwrap();
    assert!(s.practice.passed(&d.example));
    assert!(!d.solved(&s));
    assert!(d.checks(&s)[0].1.contains("Not checked"));
    s.assignment.replace(&solved(&d.initial)).unwrap();
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
    let before = s.assignment.draft["app.nmconnection"].clone();
    s.assignment
        .edit(
            "app.nmconnection",
            &before.replace("10.20.0.20/24", "10.20.0.99/24"),
        )
        .unwrap();
    assert!(s.assignment.tested.is_none());
    assert!(!d.solved(&s));
    s.assignment.edit("app.nmconnection", &before).unwrap();
    assert!(!d.solved(&s));
    s.assignment.run(&d.initial).unwrap();
    assert!(d.solved(&s));
}
#[test]
fn stop_unfinished_or_changed_runs_never_publish_credit() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.assignment.replace(&solved(&d.initial)).unwrap();
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
        .edit("app.nmconnection", "# different unfinished draft")
        .unwrap();
    assert!(s.assignment.finish(&d.initial, &run).is_err());
    assert!(!d.solved(&s));
}
#[test]
fn draft_resume_selection_reset_and_reference_are_separate_and_source_bound() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    s.view = View::Example;
    s.practice.select("management.nmconnection").unwrap();
    s.practice
        .edit("app.nmconnection", "# unfinished example")
        .unwrap();
    let reference = format!(
        "{}{}",
        w::PREFIX,
        serde_json::to_string(&solved(&d.initial)).unwrap()
    );
    let text = s.execute_reference(&d, &reference).unwrap();
    assert!(text.contains("2/2"));
    assert!(d.solved(&s));
    let saved = serde_json::to_string(&s).unwrap();
    let restored = w::Session::from_json(&saved, &d).unwrap();
    assert_eq!(restored, s);
    assert!(d.solved(&restored));
    assert_eq!(
        restored.practice.draft["app.nmconnection"],
        "# unfinished example"
    );
    let mut reset = w::Session::new(&d).unwrap();
    assert!(!d.solved(&reset));
    assert_eq!(reset.assignment.draft, d.initial.starter);
    reset.origin = "other-source".into();
    assert!(w::Session::from_json(&serde_json::to_string(&reset).unwrap(), &d).is_err());
    assert_eq!(s.practice.selected, "management.nmconnection");
    assert_eq!(s.practice.draft, restored.practice.draft);
}
#[test]
fn parse_unsupported_and_identity_changes_remain_explicit_failures() {
    let t = task("network-assignment", 20);
    for bad in ["[connection]\nunknown=thing\n", "not a keyfile"] {
        let mut fs = solved(&t);
        fs.insert("app.nmconnection".into(), bad.into());
        let rows = t.run(&fs).unwrap();
        assert!(
            rows.iter()
                .all(|r| !r.passed() && r.diagnostic.is_some() && r.checks.iter().any(|r| !r.0))
        );
    }
    let mut fs = solved(&t);
    let a = fs.get_mut("app.nmconnection").unwrap();
    *a = a.replace("000000000020", "000000000099");
    let rows = t.run(&fs).unwrap();
    assert!(rows.iter().all(|r| !r.passed() && !r.checks[0].0));
    let mut fs = solved(&t);
    fs.insert("../host.nmconnection".into(), "ignored".into());
    assert!(t.run(&fs).is_err());
}
#[test]
fn malformed_sources_and_progress_limits_refuse_before_replacing_drafts() {
    let d = definition();
    let mut s = w::Session::new(&d).unwrap();
    let before = s.clone();
    assert!(s.assignment.edit("../../real-file", "not read").is_err());
    assert!(
        s.assignment
            .edit("app.nmconnection", &"x".repeat(16385))
            .is_err()
    );
    assert_eq!(s, before);
    let mut bad = d.clone();
    bad.example = d.initial.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0].requirements.clear();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.cases[0].initially_selected = vec!["unknown".into()];
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(&s).unwrap();
    value["version"] = 2.into();
    assert!(w::Session::from_json(&value.to_string(), &d).is_err());
    value = serde_json::to_value(&s).unwrap();
    value["assignment"]["selected"] = "../foreign".into();
    assert!(w::Session::from_json(&value.to_string(), &d).is_err());
    assert!(w::Definition::from_json(&" ".repeat(w::MAX_DEFINITION + 1)).is_err());
    assert!(w::Session::from_json(&" ".repeat(w::MAX_SESSION + 1), &d).is_err());
}
#[test]
fn deserialization_rejects_duplicate_and_excess_maps_and_long_identity_values() {
    let env = serde_json::to_string(&environment()).unwrap();
    assert!(
        serde_json::from_str::<dlms_learning_lab::nm_policy::Environment>(
            &env.replace("\"enp1s0\":100", "\"enp1s0\":100,\"enp1s0\":100")
        )
        .is_err()
    );
    assert!(
        serde_json::from_str::<dlms_learning_lab::nm_policy::Environment>(
            &env.replace("\"enp1s0\":100", "\"third\":1,\"enp1s0\":100")
        )
        .is_err()
    );
    let d = definition();
    let mut v = serde_json::to_value(&d).unwrap();
    v["initial"]["cases"][0]["initially_selected"][0] = "x".repeat(37).into();
    assert!(serde_json::from_value::<w::Definition>(v).is_err());
    let s = w::Session::new(&d).unwrap();
    let mut v = serde_json::to_value(&s).unwrap();
    for n in 0..9 {
        v["assignment"]["draft"][format!("extra{n}.nmconnection")] = "".into();
    }
    assert!(serde_json::from_value::<w::Session>(v).is_err());
}
