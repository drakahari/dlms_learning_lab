use dlms_learning_lab::capacity_planning_workspace::*;
fn definition() -> Definition {
    serde_json::from_str(include_str!("fixtures/capacity-planning-workspace-v1.json")).unwrap()
}
fn solved(d: &Definition) -> Draft {
    let mut p = Draft::new(&d.initial);
    for op in &d.reference {
        p.apply(&d.initial, op).unwrap();
    }
    assert!(p.checked && d.initial.assess(&p).passed());
    p
}
#[test]
fn worked_and_independent_plans_reach_different_real_workload_outcomes() {
    let d = definition();
    d.validate().unwrap();
    let mut s = Session::new(&d);
    for op in &d.example_reference {
        s.practice.apply(&d.example, op).unwrap();
    }
    assert!(d.checks(&s, true).iter().all(|r| r.0));
    assert_eq!(s.assignment, Draft::new(&d.initial));
    assert!(d.checks(&s, false).iter().all(|r| !r.0));
    let p = solved(&d);
    let a = d.initial.assess(&p);
    assert_eq!(a.cases.len(), 3);
    assert_eq!(
        a.cases
            .iter()
            .map(|c| c.minimum_free_kib)
            .collect::<Vec<_>>(),
        // Before arrival, day20 leaves1000-730=270 and1000-740=260.
        // The busier case's minimum is its final1400-1240=160.
        vec![270, 160, 260]
    );
}
#[test]
fn different_recent_windows_quotes_dates_reserves_and_number_formats_can_pass() {
    let d = definition();
    for first in [1, 2] {
        for day in [14, 15, 16] {
            let mut p = solved(&d);
            for op in [
                Action::Window { first, last: 3 },
                Action::Offer { id: "fast".into() },
                Action::OrderDay {
                    value: format!(" 0{day} "),
                },
                Action::Reserve {
                    value: "0200".into(),
                },
                Action::Test,
            ] {
                p.apply(&d.initial, &op).unwrap();
            }
            assert!(
                d.initial.assess(&p).passed(),
                "first{first} day{day}: {:?}",
                d.initial.assess(&p).checks
            );
        }
    }
}
#[test]
fn a_passing_case_or_an_affordable_but_late_upgrade_does_not_pass_the_plan() {
    let d = definition();
    let mut p = solved(&d);
    p.apply(&d.initial, &Action::OrderDay { value: "20".into() })
        .unwrap();
    p.apply(&d.initial, &Action::Test).unwrap();
    let a = d.initial.assess(&p);
    assert!(!a.passed());
    assert!(a.cases.iter().any(|c| c.first_failure.is_none()));
    assert!(a.cases.iter().any(|c| c.first_failure.is_some()));
    assert!(
        a.checks
            .iter()
            .any(|r| !r.0 && r.1.contains("reserve first fails on day"))
    );
    assert!(
        a.cases
            .iter()
            .all(|c| c.days.last().unwrap().capacity_kib == 1400)
    );
    // The slow quote is worse: all three cases breach reserve before arrival.
    p = solved(&d);
    p.apply(&d.initial, &Action::Offer { id: "slow".into() })
        .unwrap();
    let late = d.initial.assess(&p);
    assert!(late.cases.iter().all(|c| c.first_failure.is_some()));
}
#[test]
fn over_budget_and_old_baseline_fail_even_when_every_capacity_case_passes() {
    let d = definition();
    let mut p = solved(&d);
    p.apply(
        &d.initial,
        &Action::Offer {
            id: "costly".into(),
        },
    )
    .unwrap();
    p.apply(&d.initial, &Action::Test).unwrap();
    let a = d.initial.assess(&p);
    assert!(a.cases.iter().all(|c| c.first_failure.is_none()));
    assert!(!a.passed());
    assert!(a.checks.iter().any(|r| !r.0 && r.1.contains("Budget:")));
    p = solved(&d);
    p.apply(&d.initial, &Action::Window { first: 0, last: 3 })
        .unwrap();
    p.apply(&d.initial, &Action::Test).unwrap();
    let a = d.initial.assess(&p);
    assert!(a.cases.iter().all(|c| c.first_failure.is_none()));
    assert!(!a.passed());
    assert!(a.checks[0].1.contains("old operating period"));
}
#[test]
fn blank_malformed_wrong_units_past_dates_and_stale_windows_have_specific_feedback() {
    let d = definition();
    let blank = Draft::new(&d.initial);
    let a = d.initial.assess(&blank);
    assert!(a.checks.iter().any(|r| r.1.contains("choose its starting")));
    assert!(
        a.checks
            .iter()
            .any(|r| r.1.contains("Reserve: enter a value"))
    );
    assert!(a.cases.is_empty());
    let mut p = solved(&d);
    for (value, expected) in [
        ("100 GiB", "unit suffix"),
        ("-100", "whole number"),
        ("999999999999999999999999", "size limit"),
        ("99", "below the required"),
        ("4398046511105", "size limit"),
        ("1001", "reserve first fails"),
    ] {
        p.apply(
            &d.initial,
            &Action::Reserve {
                value: value.into(),
            },
        )
        .unwrap();
        let a = d.initial.assess(&p);
        assert!(!a.passed());
        assert!(
            a.checks.iter().any(|r| !r.0 && r.1.contains(expected)),
            "{value}: {:?}",
            a.checks
        );
    }
    p = solved(&d);
    p.apply(&d.initial, &Action::OrderDay { value: "13".into() })
        .unwrap();
    assert!(
        d.initial
            .assess(&p)
            .checks
            .iter()
            .any(|r| !r.0 && r.1.contains("cannot backdate"))
    );
    p = solved(&d);
    p.apply(&d.initial, &Action::Window { first: 1, last: 2 })
        .unwrap();
    assert!(
        d.initial.assess(&p).checks[0]
            .1
            .contains("latest supplied observation")
    );
    p.apply(&d.initial, &Action::Window { first: 3, last: 1 })
        .unwrap();
    assert!(
        d.initial.assess(&p).checks[0]
            .1
            .contains("increasing date order")
    );
}
#[test]
fn editing_clears_success_until_tested_again_and_case_selection_is_read_only() {
    let d = definition();
    let mut s = Session::new(&d);
    s.assignment = solved(&d);
    assert!(d.checks(&s, false).iter().all(|r| r.0));
    s.assignment.selected_case = 2;
    assert!(d.checks(&s, false).iter().all(|r| r.0));
    s.assignment
        .apply(&d.initial, &Action::Reserve { value: "0".into() })
        .unwrap();
    assert!(!s.assignment.checked);
    assert!(d.checks(&s, false).iter().all(|r| !r.0));
    s.assignment
        .apply(
            &d.initial,
            &Action::Reserve {
                value: "100".into(),
            },
        )
        .unwrap();
    assert!(d.initial.assess(&s.assignment).passed());
    assert!(
        d.checks(&s, false).iter().all(|r| !r.0),
        "A valid edit alone must not reuse an earlier successful test"
    );
    s.assignment.apply(&d.initial, &Action::Test).unwrap();
    assert!(d.checks(&s, false).iter().all(|r| r.0));
}
#[test]
fn separate_drafts_case_selection_and_checked_outcomes_survive_validated_roundtrip() {
    let d = definition();
    let mut s = Session::new(&d);
    s.assignment = solved(&d);
    s.practice.inputs.reserve_kib = "trying a value".into();
    s.assignment.selected_case = 2;
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&bytes).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(d.checks(&restored, false).iter().all(|r| r.0));
    let kept = restored.practice.clone();
    restored.assignment = Draft::new(&d.initial);
    assert_eq!(restored.practice, kept);
    assert!(d.checks(&restored, false).iter().all(|r| !r.0));
    restored.version = 2;
    assert!(d.bound(&restored).is_err());
    restored = s;
    restored.assignment.task_id = d.example.id.clone();
    assert!(d.bound(&restored).is_err());
}
#[test]
fn invalid_actions_are_atomic_and_forged_unknown_or_overlong_saved_fields_fail() {
    let d = definition();
    let mut p = solved(&d);
    let old = p.clone();
    for action in [
        Action::Window { first: 0, last: 99 },
        Action::Offer {
            id: "not-a-quote".into(),
        },
        Action::Reserve {
            value: "x".repeat(25),
        },
    ] {
        assert!(p.apply(&d.initial, &action).is_err());
        assert_eq!(p, old);
    }
    let mut s = Session::new(&d);
    s.assignment.inputs.offer = Some("not-a-quote".into());
    assert!(d.bound(&s).is_err());
    let text = serde_json::to_string(&Session::new(&d)).unwrap();
    assert!(
        serde_json::from_str::<Session>(&text.replace(
            "\"offer\":null",
            &format!("\"offer\":\"{}\"", "x".repeat(49))
        ))
        .is_err()
    );
    assert!(
        serde_json::from_str::<Session>(
            &text.replace("\"version\":1", "\"version\":1,\"execute\":\"sh\"")
        )
        .is_err()
    );
}
#[test]
fn invalid_policies_cases_starters_and_unchecked_or_wrong_references_are_not_authorable() {
    let d = definition();
    for n in 0..8 {
        let mut bad = d.clone();
        match n {
            0 => bad.reference.pop().map(|_| ()).unwrap(),
            1 => bad.initial.policy.minimum_window_days = 99,
            2 => bad.initial.offers[1].id = bad.initial.offers[0].id.clone(),
            3 => bad.initial.demands[1].id = bad.initial.demands[0].id.clone(),
            4 => bad.initial.starter = solved(&d).inputs,
            5 => bad.example.series = bad.initial.series.clone(),
            6 => bad.reference[1] = Action::Offer { id: "slow".into() },
            _ => {
                bad.initial.demands[0].changes =
                    vec![dlms_learning_lab::resource_measurements::capacity::Change {
                        day: 13,
                        added_kib: 10,
                    }]
            }
        };
        assert!(bad.validate().is_err(), "invalid case {n}");
    }
}
