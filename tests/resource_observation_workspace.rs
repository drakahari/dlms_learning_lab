use dlms_learning_lab::resource_observation_workspace::{
    Action, Definition, Draft, Metric, Probe, Selection, Session,
};
fn def() -> Definition {
    serde_json::from_str(include_str!(
        "fixtures/resource-investigation-workspace-v1.json"
    ))
    .unwrap()
}
fn solved(d: &Definition) -> Draft {
    let mut s = Draft::new(&d.initial);
    for a in &d.reference {
        s.apply(&d.initial, a).unwrap()
    }
    assert!(s.check(&d.initial).unwrap().iter().all(|r| r.0));
    s
}
fn run(s: &mut Draft, d: &Definition, start: usize, command: &str) {
    s.apply(
        &d.initial,
        &Action::Run {
            probe: Probe {
                start,
                command: command.into(),
            },
        },
    )
    .unwrap()
}
fn value(s: &mut Draft, d: &Definition, rule: &str, v: &str) {
    s.apply(
        &d.initial,
        &Action::Value {
            rule: rule.into(),
            value: v.into(),
        },
    )
    .unwrap()
}
#[test]
fn authored_recordings_have_reachable_but_separate_actual_outcomes() {
    let d = def();
    d.validate().unwrap();
    let mut s = Session::new(&d);
    assert!(d.initial.check(&s.assignment).iter().all(|r| !r.0));
    for a in &d.example_reference {
        s.practice.apply(&d.example, a).unwrap()
    }
    assert!(s.practice.check(&d.example).unwrap().iter().all(|r| r.0));
    assert_eq!(s.assignment, Draft::new(&d.initial));
    assert!(!s.assignment.checked);
    d.bound(&s).unwrap();
    assert!(solved(&d).report(&d.initial).unwrap().contains("verified"));
}
#[test]
fn explicit_units_optional_flags_different_command_order_and_leading_zeroes_are_equivalent() {
    let d = def();
    let mut s = Draft::new(&d.initial);
    run(&mut s, &d, 1, "vmstat -S K 1 2");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "quiet".into(),
            row: 1,
        },
    )
    .unwrap();
    value(&mut s, &d, "quiet", "020");
    run(&mut s, &d, 0, "vmstat 1 2");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "peak".into(),
            row: 1,
        },
    )
    .unwrap();
    value(&mut s, &d, "peak", "80");
    run(&mut s, &d, 2, " cat\t/proc/meminfo ");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "available".into(),
            row: 0,
        },
    )
    .unwrap();
    value(&mut s, &d, "available", " 3584 ");
    assert!(s.check(&d.initial).unwrap().iter().all(|r| r.0));
}
#[test]
fn boot_average_wrong_interval_and_wrong_metric_evidence_fail_without_losing_other_rows() {
    let d = def();
    let mut s = solved(&d);
    run(&mut s, &d, 0, "vmstat 1 3");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "peak".into(),
            row: 0,
        },
    )
    .unwrap();
    let c = s.check(&d.initial).unwrap();
    assert!(!c[0].0 && c[0].1.contains("since-boot"));
    assert!(c[1].0 && c[2].0);
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "peak".into(),
            row: 2,
        },
    )
    .unwrap();
    let c = s.check(&d.initial).unwrap();
    assert!(!c[0].0 && c[0].1.contains("higher"));
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "available".into(),
            row: 2,
        },
    )
    .unwrap();
    let c = s.check(&d.initial).unwrap();
    assert!(!c[2].0 && c[2].1.contains("MemAvailable"));
}
#[test]
fn blank_format_wrong_value_and_free_versus_available_receive_distinct_feedback() {
    let d = def();
    let mut s = solved(&d);
    for (v, why) in [("", "enter"), ("80%", "digits only"), ("35", "since-boot")] {
        value(&mut s, &d, "peak", v);
        assert!(!s.checked);
        let c = s.check(&d.initial).unwrap();
        assert!(!c[0].0 && c[0].1.contains(why));
        assert!(c[1].0 && c[2].0);
    }
    value(&mut s, &d, "available", "1500");
    let c = s.check(&d.initial).unwrap();
    assert!(!c[2].0 && c[2].1.contains("not MemFree"));
}
#[test]
fn editing_a_verified_claim_clears_current_check_and_investigation_alone_keeps_valid_report() {
    let d = def();
    let mut s = solved(&d);
    run(&mut s, &d, 0, "vmstat");
    assert!(s.checked && d.initial.check(&s).iter().all(|r| r.0));
    value(&mut s, &d, "peak", "1");
    assert!(!s.checked);
    assert!(!d.initial.check(&s)[0].0);
    assert!(s.report(&d.initial).unwrap().contains("needs correction"));
}
#[test]
fn failed_commands_and_out_of_range_rows_never_attach_fake_evidence() {
    let d = def();
    let mut s = Draft::new(&d.initial);
    run(&mut s, &d, 0, "cat /etc/passwd");
    let before = s.clone();
    assert!(
        s.apply(
            &d.initial,
            &Action::Attach {
                rule: "peak".into(),
                row: 0
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert!(!d.initial.check(&s)[0].0);
    run(&mut s, &d, 0, "vmstat 1 2");
    let before = s.clone();
    assert!(
        s.apply(
            &d.initial,
            &Action::Attach {
                rule: "peak".into(),
                row: 2
            }
        )
        .is_err()
    );
    assert_eq!(s, before);
}
#[test]
fn retained_evidence_survives_bounded_history_rollover_and_resume() {
    let d = def();
    let mut session = Session::new(&d);
    session.assignment = solved(&d);
    session.practice.input = "vmstat 1 3".into();
    for _ in 0..20 {
        run(&mut session.assignment, &d, 0, "vmstat")
    }
    assert_eq!(session.assignment.history.len(), 8);
    let saved = serde_json::to_string(&session).unwrap();
    let loaded: Session = serde_json::from_str(&saved).unwrap();
    d.bound(&loaded).unwrap();
    assert_eq!(loaded, session);
    assert!(d.initial.check(&loaded.assignment).iter().all(|r| r.0));
    session.practice = Draft::new(&d.example);
    assert_eq!(loaded.assignment, session.assignment);
    assert!(session.practice.history.is_empty());
}
#[test]
fn forged_extra_duplicate_unknown_or_unreachable_saved_claims_are_rejected() {
    let d = def();
    let good = solved(&d);
    let mut bad = good.clone();
    bad.claims.push(bad.claims[0].clone());
    assert!(bad.bound(&d.initial).is_err());
    let mut bad = good.clone();
    bad.claims[0].rule = "unknown".into();
    assert!(bad.bound(&d.initial).is_err());
    let mut bad = good.clone();
    bad.claims[0].evidence.as_mut().unwrap().row = 7;
    assert!(bad.bound(&d.initial).is_err());
    let mut bad = good.clone();
    bad.claims[0].evidence.as_mut().unwrap().probe.command = "cat /home/personal".into();
    assert!(bad.bound(&d.initial).is_err());
    let mut bad = good.clone();
    bad.claims[0].value = "1\n80".into();
    assert!(bad.bound(&d.initial).is_err());
    let text = serde_json::to_string(&good).unwrap().replacen(
        "\"task_id\"",
        "\"host_path\":\"/etc/passwd\",\"task_id\"",
        1,
    );
    assert!(serde_json::from_str::<Draft>(&text).is_err());
}
#[test]
fn invalid_pack_selection_and_worked_solutions_never_count_as_reachable() {
    let d = def();
    let mut bad = d.clone();
    bad.example.id = bad.initial.id.clone();
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.rules[0].selection = Selection::Snapshot {
        from_ms: 3000,
        to_ms: 4000,
    };
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.rules[0].metric = Metric::AvailableKib;
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.reference[2] = Action::Value {
        rule: "peak".into(),
        value: "1".into(),
    };
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.initial.rules[0].selection = Selection::Extremum {
        from_ms: 3000,
        to_ms: 5000,
        step_ms: 500,
        highest: true,
    };
    assert!(bad.validate().is_err());
}
#[test]
fn tied_displayed_peaks_accept_either_legitimate_interval() {
    let mut d = def();
    d.initial.rules.truncate(1);
    let o = &mut d.initial.recording.observations[2];
    o.counters.cpu.ticks = [340, 70, 140, 280, 100, 35, 35, 0];
    o.guest_ticks = [70, 0];
    let mut s = Draft::new(&d.initial);
    run(&mut s, &d, 1, "vmstat 1 2");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "peak".into(),
            row: 1,
        },
    )
    .unwrap();
    value(&mut s, &d, "peak", "80");
    assert!(s.check(&d.initial).unwrap()[0].0);
    run(&mut s, &d, 0, "vmstat 1 2");
    s.apply(
        &d.initial,
        &Action::Attach {
            rule: "peak".into(),
            row: 1,
        },
    )
    .unwrap();
    assert!(s.check(&d.initial).unwrap()[0].0);
}
