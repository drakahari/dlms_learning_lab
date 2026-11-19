use dlms_learning_lab::{
    ids_rule_workspace::{Action, Definition, Event, Session},
    script_teaching::View,
};
#[path = "support/ids_rule_fixture.rs"]
mod f;

fn definition() -> Definition {
    let assignment = f::task();
    let mut example = f::task();
    example.id = "mail-gateway-example".into();
    example.rule_path = "/etc/snort/rules/workshop.rules".into();
    example.initial_rule = example.initial_rule.replace("198.51.100.8", "203.0.113.20");
    for c in &mut example.cases {
        for e in &mut c.records {
            if e.destination.to_string() == "198.51.100.8" {
                e.destination = "203.0.113.20".parse().unwrap();
            }
            e.payload = e
                .payload
                .replace("POST", "PUT")
                .replace("/admin", "/settings");
        }
    }
    Definition {
        source_id: assignment.id.clone(),
        assignment,
        example,
        worked: vec![
            Action::Edit {
                text: f::SOLUTION
                    .replace("198.51.100.8", "203.0.113.20")
                    .replace("POST", "PUT")
                    .replace("/admin", "/settings"),
            },
            Action::Save,
            Action::Replay,
            Action::Check,
        ],
    }
}
fn solve(s: &mut Session, d: &Definition, example: bool) {
    let body = if example {
        match &d.worked[0] {
            Action::Edit { text } => text.clone(),
            _ => unreachable!(),
        }
    } else {
        f::ALTERNATIVE.into()
    };
    for a in [
        Action::Edit { text: body },
        Action::Save,
        Action::Replay,
        Action::Check,
    ] {
        s.action(d, &a, example).unwrap();
    }
}
#[test]
fn separate_practice_never_edits_or_awards_assignment_credit() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let assignment = s.assignment.clone();
    solve(&mut s, &d, true);
    assert!(s.example.checks(&d.example).iter().all(|r| r.passed));
    assert_eq!(s.assignment, assignment);
    assert!(!s.assignment.checks(&d.assignment).iter().all(|r| r.passed));
    let example = s.example.clone();
    solve(&mut s, &d, false);
    assert_eq!(s.example, example);
}
#[test]
fn unsaved_edit_and_saved_breakage_clear_success_and_survive_resume() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    assert!(s.assignment.checks(&d.assignment).iter().all(|r| r.passed));
    let saved = s.assignment.saved.clone();
    s.action(
        &d,
        &Action::Edit {
            text: "unfinished (".into(),
        },
        false,
    )
    .unwrap();
    assert_eq!(s.assignment.saved, saved);
    assert!(
        s.assignment.checks(&d.assignment)[0]
            .detail
            .contains("Save the virtual rule")
    );
    s.view = View::Example;
    s.action(&d, &Action::Select { case: 1 }, true).unwrap();
    s.action(
        &d,
        &Action::Edit {
            text: "separate unfinished example".into(),
        },
        true,
    )
    .unwrap();
    let mut restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    assert_eq!(restored, s);
    d.bound(&restored).unwrap();
    restored.shape().unwrap();
    restored.action(&d, &Action::Save, false).unwrap();
    assert!(restored.assignment.last_run.is_none());
    assert!(!restored.assignment.checked);
    restored.action(&d, &Action::Replay, false).unwrap();
    restored.action(&d, &Action::Check, false).unwrap();
    assert!(
        restored.assignment.checks(&d.assignment)[0]
            .detail
            .starts_with("Syntax error:")
    );
    solve(&mut restored, &d, false);
    assert!(
        restored
            .assignment
            .checks(&d.assignment)
            .iter()
            .all(|r| r.passed)
    );
}
#[test]
fn imported_observations_cannot_forge_credit_or_move_between_sources() {
    let d = definition();
    let s = Session::new(&d).unwrap();
    let mut forged = s.clone();
    forged.assignment.checked = true;
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.saved = f::SOLUTION.into();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.source_task = s.example.source_task.clone();
    assert!(d.bound(&forged).is_err());
    let mut forged = s.clone();
    forged.assignment.last_run = Some(dlms_learning_lab::ids_rule_workspace::Run {
        case: 0,
        rule: f::SOLUTION.into(),
    });
    assert!(d.bound(&forged).is_err());
    let mut swapped = s.clone();
    std::mem::swap(&mut swapped.assignment, &mut swapped.example);
    assert!(d.bound(&swapped).is_err());
    let mut changed = d.clone();
    changed.assignment.cases[0].records[0].source_port = 42000;
    assert!(changed.bound(&s).is_err());
}
#[test]
fn unsupported_replay_is_visible_and_does_not_award_credit() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let text = "alert tcp any any -> any any (sid:1;flow:established;)";
    for a in [
        Action::Edit { text: text.into() },
        Action::Save,
        Action::Replay,
        Action::Check,
    ] {
        s.action(&d, &a, false).unwrap();
    }
    let run = s.assignment.last_run.as_ref().unwrap();
    assert!(
        d.assignment
            .observe(&run.rule, run.case)
            .unwrap_err()
            .starts_with("Unsupported rule or input:")
    );
    assert!(!s.assignment.checks(&d.assignment)[0].passed);
    d.bound(&s).unwrap();
}
#[test]
fn rejection_is_atomic_and_history_limits_do_not_drop_work() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    for _ in 0..64 {
        s.action(&d, &Action::Check, false).unwrap();
    }
    let before = s.clone();
    assert!(s.action(&d, &Action::Replay, false).is_err());
    assert_eq!(s, before);
    assert!(s.action(&d, &Action::Select { case: 99 }, false).is_err());
    assert_eq!(s, before);
    assert!(
        s.action(
            &d,
            &Action::Edit {
                text: "x".repeat(2049)
            },
            false
        )
        .is_err()
    );
    assert_eq!(s, before);
    assert!(
        s.action(&d, &Action::Edit { text: "\0".into() }, false)
            .is_err()
    );
    assert_eq!(s, before);
    let mut forged = s;
    forged.assignment.history.push(Event::Check);
    assert!(d.bound(&forged).is_err());
}
#[test]
fn resetting_example_keeps_assignment_exact_and_worked_recipe_must_reach_goals() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    solve(&mut s, &d, true);
    let assignment = s.assignment.clone();
    s.reset_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    assert_eq!(s.example, Session::new(&d).unwrap().example);
    let mut bad = d.clone();
    bad.worked = vec![Action::Check];
    assert!(bad.validate().is_err());
    let mut bad = d.clone();
    bad.example.rule_path = d.assignment.rule_path.clone();
    assert!(bad.validate().is_err());
    let mut value = serde_json::to_value(s).unwrap();
    value["host_execution"] = true.into();
    assert!(serde_json::from_value::<Session>(value).is_err());
}
