use dlms_learning_lab::{
    auth_filter_workspace::{Action, Definition, Event, Session},
    script_teaching::View,
};
#[path = "support/auth_filter_fixture.rs"]
mod f;
fn definition() -> Definition {
    let assignment = f::task(true);
    let mut example = f::task(true);
    example.id = "workshop-example".into();
    example.log_path = "/var/log/workshop.log".into();
    example.filter_path = "/etc/fail2ban/filter.d/workshop.conf".into();
    example.initial_filter = example.initial_filter.replace("portal", "workshop");
    for case in &mut example.cases {
        for line in &mut case.lines {
            *line = line
                .replace("portal", "workshop")
                .replace("alex", "tariq")
                .replace("192.0.2.9", "203.0.113.42");
        }
    }
    Definition {
        source_id: assignment.id.clone(),
        assignment,
        example,
        worked: vec![
            Action::Edit {
                text: f::canonical(true).replace("portal", "workshop"),
            },
            Action::Save,
            Action::Run,
            Action::Check,
        ],
    }
}
fn action(s: &mut Session, d: &Definition, a: Action, example: bool) {
    s.action(d, &a, example).unwrap();
}
fn solve(s: &mut Session, d: &Definition, example: bool) {
    let body = if example {
        f::alternative(true).replace("portal", "workshop")
    } else {
        f::canonical(true)
    };
    action(s, d, Action::Edit { text: body }, example);
    action(s, d, Action::Save, example);
    action(s, d, Action::Run, example);
    action(s, d, Action::Check, example);
}
#[test]
fn native_editor_example_has_its_own_saved_filter_and_no_assignment_credit() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.clone();
    solve(&mut s, &d, true);
    assert!(s.example.checks(&d.example).iter().all(|c| c.passed));
    assert_eq!(s.assignment, original);
    assert!(!s.assignment.checks(&d.assignment).iter().all(|c| c.passed));
    s.view = View::Assignment;
    d.bound(&s).unwrap();
}
#[test]
fn unsaved_and_saved_breakage_invalidate_current_success_and_repair_is_possible() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    assert!(s.assignment.checks(&d.assignment).iter().all(|c| c.passed));
    action(
        &mut s,
        &d,
        Action::Edit {
            text: "[Definition]\nfailregex=invalid missing tag\n".into(),
        },
        false,
    );
    let rows = s.assignment.checks(&d.assignment);
    assert!(!rows[0].passed);
    assert!(rows[0].detail.contains("Save virtual filter"));
    let bytes = serde_json::to_vec(&s).unwrap();
    let mut restored: Session = serde_json::from_slice(&bytes).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(s, restored);
    action(&mut restored, &d, Action::Save, false);
    action(&mut restored, &d, Action::Check, false);
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
            .all(|c| c.passed)
    );
}
#[test]
fn partial_work_and_distinct_unrun_commands_resume_exactly_before_execution() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    action(
        &mut s,
        &d,
        Action::Edit {
            text: f::canonical(false),
        },
        false,
    );
    action(&mut s, &d, Action::Save, false);
    action(&mut s, &d, Action::Check, false);
    let result = s.assignment.checks(&d.assignment);
    assert!(result.iter().any(|c| c.passed));
    assert!(result.iter().any(|c| !c.passed));
    action(
        &mut s,
        &d,
        Action::Command {
            input: "cat 'unfinished assignment".into(),
        },
        false,
    );
    action(
        &mut s,
        &d,
        Action::Command {
            input: "fail2ban-regex 'unfinished example".into(),
        },
        true,
    );
    action(
        &mut s,
        &d,
        Action::Edit {
            text: "# separate unfinished practice\n".into(),
        },
        true,
    );
    let restored: Session = serde_json::from_slice(&serde_json::to_vec(&s).unwrap()).unwrap();
    d.bound(&restored).unwrap();
    assert_eq!(restored, s);
    assert!(restored.assignment.console.is_empty());
    assert!(restored.example.console.is_empty());
    assert_ne!(restored.example.command, restored.assignment.command);
}
#[test]
fn utility_output_uses_real_virtual_lines_and_flags_and_read_only_queries_preserve_success() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    let input = format!(
        "fail2ban-regex {} {} --usedns no --print-all-matched --print-all-ignored --print-all-missed",
        d.assignment.log_path, d.assignment.filter_path
    );
    action(&mut s, &d, Action::Command { input }, false);
    action(&mut s, &d, Action::Run, false);
    assert!(
        s.assignment
            .console
            .contains("4 lines, 1 ignored, 1 matched, 2 missed")
    );
    for line in &d.assignment.cases[0].lines {
        assert!(s.assignment.console.contains(line));
    }
    let record = s.assignment.last_run.clone();
    action(
        &mut s,
        &d,
        Action::Command {
            input: format!("cat {}", d.assignment.filter_path),
        },
        false,
    );
    action(&mut s, &d, Action::Run, false);
    assert!(s.assignment.console.contains(&f::canonical(true)));
    assert_eq!(s.assignment.last_run, record);
    assert!(s.assignment.checks(&d.assignment).iter().all(|c| c.passed));
    action(&mut s, &d, Action::Select { case: 1 }, false);
    assert!(s.assignment.checks(&d.assignment).iter().all(|c| c.passed));
}
#[test]
fn unsupported_or_host_escape_inputs_never_change_virtual_files_or_fabricate_a_test() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    let original = s.assignment.saved.clone();
    for input in [
        "cat /etc/passwd",
        "cat /var/log/../secrets",
        "systemctl restart fail2ban",
        "fail2ban-client status",
        "cat\n/var/log/portal.log",
        "cat /var/log/portal.log; touch /tmp/never",
        "$(id)",
    ] {
        action(
            &mut s,
            &d,
            Action::Command {
                input: input.into(),
            },
            false,
        );
        action(&mut s, &d, Action::Run, false);
        assert_eq!(s.assignment.saved, original);
        assert!(s.assignment.last_run.is_none());
        assert!(!s.assignment.checks(&d.assignment).iter().all(|c| c.passed));
    }
    assert!(s.assignment.console.contains("no host path is read"));
    assert!(s.assignment.console.contains("Unsupported"));
    d.bound(&s).unwrap();
}
#[test]
fn replay_rejects_forged_saved_results_histories_or_changed_source() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    for variant in 0..5 {
        let mut forged = s.clone();
        match variant {
            0 => forged.assignment.saved.push(' '),
            1 => forged.assignment.console.push('x'),
            2 => forged.assignment.history.pop().map(|_| ()).unwrap(),
            3 => forged
                .assignment
                .last_run
                .as_mut()
                .unwrap()
                .filter
                .push(' '),
            _ => forged.assignment.source_task.push(' '),
        };
        assert!(d.bound(&forged).is_err());
    }
    let mut changed = d.clone();
    changed.assignment.cases[0].expected[0] = dlms_learning_lab::auth_filter_task::Expected::Missed;
    assert!(changed.bound(&s).is_err());
    let mut oversized = s.clone();
    oversized.assignment.history = vec![Event::Check; 65];
    assert!(d.bound(&oversized).is_err());
    let mut oversized = s.clone();
    oversized.assignment.console = "x".repeat(32769);
    assert!(d.bound(&oversized).is_err());
}
#[test]
fn example_reset_preserves_assignment_and_invalid_actions_are_atomic() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    solve(&mut s, &d, false);
    solve(&mut s, &d, true);
    let assignment = s.assignment.clone();
    s.reset_example(&d).unwrap();
    assert_eq!(s.assignment, assignment);
    assert!(!s.example.checked);
    assert!(!s.example.checks(&d.example).iter().all(|c| c.passed));
    let before = s.clone();
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
    assert!(s.action(&d, &Action::Select { case: 100 }, false).is_err());
    assert_eq!(s, before);
}

#[test]
fn authored_examples_must_be_reachable_and_cannot_alias_assignment_files() {
    let mut d = definition();
    d.worked.clear();
    assert!(d.validate().is_err());
    let mut d = definition();
    d.worked = vec![Action::Check];
    assert!(d.validate().is_err());
    let mut d = definition();
    d.example.log_path = d.assignment.filter_path.clone();
    assert!(d.validate().is_err());
}

#[test]
fn unbound_import_checks_shape_but_current_credit_still_requires_source() {
    let d = definition();
    let mut s = Session::new(&d).unwrap();
    s.shape().unwrap();
    solve(&mut s, &d, false);
    s.shape().unwrap();
    let mut malformed = s.clone();
    malformed.assignment.source_task = "{}".into();
    assert!(malformed.shape().is_err());
    let mut forged = s.clone();
    forged.assignment.saved.push('x');
    assert!(forged.shape().is_err());
    let mut unrelated = d.clone();
    unrelated.assignment.cases[0].label.push_str(" changed");
    assert!(unrelated.bound(&s).is_err());
}

fn scenario() -> dlms_learning_lab::pack::Scenario {
    let d = definition();
    serde_json::from_value(serde_json::json!({
        "id":d.source_id,"version":1,"track":"LPIC-2","topic":"Filter fixture",
        "unit":"linux-auth-log-filters","title":"Synthetic source-binding fixture",
        "summary":"Controlled test only, not a catalog lesson.","objective":"Classify supplied lines.",
        "difficulty":"Intermediate","minutes":20,"steps":["Edit","Test","Check"],
        "guide":[],"explanation":"Fixed test context.","hints":[],"sources":["https://www.fail2ban.org/"],
        "simplifications":"Fixed in-process fixture.",
        "initial":dlms_learning_lab::engine::State::AuthFilter{simulation:Box::new(Session::new(&d).unwrap())},
        "goal":dlms_learning_lab::engine::Goal::AuthFilter{definition:Box::new(d)},"solution":[]
    })).unwrap()
}
fn route(
    state: &mut dlms_learning_lab::engine::State,
    s: &dlms_learning_lab::pack::Scenario,
    action: Action,
    practice: bool,
) {
    let input = format!(
        "{}{}",
        dlms_learning_lab::auth_filter_source::PREFIX,
        serde_json::json!({"practice":practice,"action":action})
    );
    dlms_learning_lab::engine::execute_for_goal(state, &s.goal, &input).unwrap();
}
#[test]
fn typed_engine_progress_and_source_binding_preserve_independent_work() {
    use dlms_learning_lab::{auth_filter_source as binding, engine, progress};
    let s = scenario();
    binding::source(&s).unwrap();
    let mut attempt = progress::Attempt::new(&s, 1);
    assert!(!progress::current_solved(&attempt, &s));
    for action in &definition().worked {
        route(&mut attempt.state, &s, action.clone(), true);
    }
    assert!(!progress::current_solved(&attempt, &s));
    for a in [
        Action::Edit {
            text: f::canonical(true),
        },
        Action::Save,
        Action::Run,
        Action::Check,
    ] {
        route(&mut attempt.state, &s, a, false);
    }
    assert!(engine::solved(&attempt.state, &s.goal));
    assert!(progress::current_solved(&attempt, &s));
    binding::context(&attempt, &s).unwrap();
    route(
        &mut attempt.state,
        &s,
        Action::Edit {
            text: f::canonical(false),
        },
        false,
    );
    assert!(!progress::current_solved(&attempt, &s));
    assert!(engine::execute_for_goal(&mut attempt.state, &s.goal, "touch /etc/never").is_err());
}
#[test]
fn ordinary_progress_save_requires_catalog_and_roundtrip_retains_partial_drafts() {
    use dlms_learning_lab::{
        auth_filter_source as binding,
        engine::State,
        progress::{self, Attempt, Progress},
    };
    let source = scenario();
    let mut a = Attempt::new(&source, 1);
    for action in [
        Action::Edit {
            text: f::canonical(false),
        },
        Action::Save,
        Action::Check,
        Action::Command {
            input: "cat 'unfinished assignment".into(),
        },
    ] {
        route(&mut a.state, &source, action, false);
    }
    route(
        &mut a.state,
        &source,
        Action::Edit {
            text: "# independent unfinished example\n".into(),
        },
        true,
    );
    route(
        &mut a.state,
        &source,
        Action::Command {
            input: "cat 'unfinished example".into(),
        },
        true,
    );
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), a);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("filter-progress.json");
    assert!(
        temp.path().starts_with(std::env::var_os("TMPDIR").unwrap()),
        "Runner must keep test artifacts on project disk"
    );
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    progress::save_with_catalog(&path, &p, std::slice::from_ref(&source)).unwrap();
    let original_bytes = std::fs::read(&path).unwrap();
    let refused = progress::save_with_catalog(&path, &p, &[]).unwrap_err();
    assert!(
        refused.contains("authentication-filter"),
        "Wrong activity diagnostic: {refused}"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original_bytes,
        "Unknown source must preserve the original saved progress"
    );
    let restored = progress::load(&path, std::slice::from_ref(&source)).unwrap();
    let a = &restored.attempts[&source.id];
    binding::context(a, &source).unwrap();
    let State::AuthFilter { simulation } = &a.state else {
        panic!()
    };
    let State::AuthFilter { simulation: before } = &p.attempts[&source.id].state else {
        panic!()
    };
    assert_eq!(simulation, before);
    assert!(!progress::current_solved(a, &source));
    let mut changed = source.clone();
    let dlms_learning_lab::engine::Goal::AuthFilter { definition } = &mut changed.goal else {
        panic!()
    };
    definition.assignment.cases[0].label.push_str(" changed");
    assert!(binding::context(a, &changed).is_err());
}
