use dlms_learning_lab::{
    engine::{self, Goal, State},
    linux_logging as core, linux_logging_workspace as w, pack,
    progress::{self, Attempt, Progress},
};
const BASE: &str = include_str!("../scenarios/starter-v1.json");
fn labs() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        BASE,
        include_str!("../scenarios/linux-logging-lessons-v1.json"),
    ])
    .unwrap()
    .into_iter()
    .filter(|s| s.id.starts_with("lpic1-syslog-"))
    .collect()
}

fn parts(s: &pack::Scenario) -> (&w::Definition, &w::Session) {
    let (Goal::LinuxLogging { definition }, State::LinuxLogging { simulation }) =
        (&s.goal, &s.initial)
    else {
        panic!()
    };
    (definition, simulation)
}
fn solved(s: &pack::Scenario) -> w::Session {
    let (d, initial) = parts(s);
    let mut x = initial.clone();
    x.execute_reference(d, &s.solution[0]).unwrap();
    x
}
#[test]
fn logging_four_full_solutions_and_examples_are_independent() {
    for s in labs() {
        let (d, initial) = parts(&s);
        assert!(!engine::solved(&s.initial, &s.goal));
        let x = solved(&s);
        assert!(d.checks(&x, false).iter().all(|r| r.0));
        let mut example = initial.clone();
        example.practice.draft = d.example_worked.clone();
        example.run(d, true).unwrap();
        assert!(d.checks(&example, true).iter().all(|r| r.0));
        assert_eq!(example.assignment, initial.assignment);
        assert!(!d.checks(&example, false).iter().all(|r| r.0));
    }
}
#[test]
fn logging_reordered_case_comment_and_split_exact_rules_are_legitimate_alternatives() {
    for s in labs() {
        let (d, initial) = parts(&s);
        let raw = s.solution[0].strip_prefix(w::PREFIX).unwrap();
        let mut lines = raw.lines().map(str::to_string).collect::<Vec<_>>();
        lines.reverse();
        let mut x = initial.clone();
        x.assignment.draft = format!("# different supported form\n{}\n", lines.join("\n"));
        x.run(d, false).unwrap();
        assert!(d.checks(&x, false).iter().all(|r| r.0));
    }
    let s = labs().remove(0);
    let (d, initial) = parts(&s);
    let mut x = initial.clone();
    x.assignment.draft = core::SEVERITIES[..5]
        .iter()
        .map(|sev| format!("*.={sev} /var/log/operations.log\n"))
        .collect();
    x.run(d, false).unwrap();
    assert!(d.checks(&x, false).iter().all(|r| r.0));
}
#[test]
fn logging_unwanted_levels_leaks_duplicates_and_missing_streams_fail_beyond_displayed_events() {
    for s in labs() {
        let (d, initial) = parts(&s);
        for text in [
            "*.* /var/log/operations.log\n",
            "",
            "authpriv.* /var/log/private.log\n",
        ] {
            let mut x = initial.clone();
            x.assignment.draft = text.into();
            x.run(d, false).unwrap();
            assert!(d.checks(&x, false).iter().any(|r| !r.0), "{} {text}", s.id);
        }
        let mut x = solved(&s);
        let extra = x.assignment.draft.lines().next().unwrap().to_string();
        x.assignment.draft.push_str(&format!("{extra}\n"));
        x.run(d, false).unwrap();
        let rows = d.checks(&x, false);
        assert!(rows.iter().any(|r| !r.0 && r.1.contains("actual")));
    }
}
#[test]
fn logging_stale_actual_replay_is_retained_but_never_current_success() {
    for s in labs() {
        let (d, _) = parts(&s);
        let mut x = solved(&s);
        let old = x.assignment.last_run.clone();
        x.assignment.draft.push_str("# changed pending text\n");
        assert_eq!(x.assignment.last_run, old);
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
        x.run(d, false).unwrap();
        assert!(d.checks(&x, false).iter().all(|r| r.0));
        x.assignment
            .draft
            .push_str("*.debug /var/log/operations.log\n");
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
        x.run(d, false).unwrap();
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
    }
}
#[test]
fn logging_syntax_and_unsupported_features_get_diagnostics_and_do_not_fake_applied_state() {
    let s = labs().remove(0);
    let (d, initial) = parts(&s);
    for text in [
        "daemon /var/log/operations.log",
        "*.debug;mail.err /var/log/operations.log",
        "*.info @192.0.2.5",
        "$IncludeConfig /etc/rsyslog.d/*",
        "*.err /etc/passwd",
    ] {
        let mut x = initial.clone();
        x.assignment.draft = text.into();
        let e = x.run(d, false).unwrap_err();
        assert!(!e.is_empty());
        assert_eq!(x.assignment.last_run, None);
        assert!(x.assignment.diagnostic.is_some());
        assert!(!d.checks(&x, false).iter().all(|r| r.0));
        let mut state = s.initial.clone();
        assert!(
            engine::execute_for_goal(&mut state, &s.goal, &format!("{}{text}", w::PREFIX)).is_err()
        );
        assert_eq!(state, s.initial);
    }
}
#[test]
fn logging_resume_preserves_separate_drafts_and_tested_source_and_reset_is_selected_only() {
    let all = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        let mut x = solved(s);
        x.assignment.draft.push_str("# unfinished\n");
        x.practice.draft = "daemon.err /var/log/service.log\n".into();
        a.state = State::LinuxLogging {
            simulation: Box::new(x),
        };
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let got = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&got).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    assert!(progress::save(&path, &p).is_err());
    let before = p.attempts[&all[1].id].clone();
    p.attempts
        .insert(all[0].id.clone(), Attempt::new(&all[0], 2));
    assert_eq!(
        serde_json::to_value(&p.attempts[&all[1].id]).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    assert!(p.favorites.contains(&all[0].id));
}
#[test]
fn logging_invalid_packs_foreign_metadata_and_oversized_optional_text_refuse() {
    let raw = include_str!("../scenarios/linux-logging-lessons-v1.json");
    for n in 0..6 {
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let s = &mut v["scenarios"][0];
        match n {
            0 => s["goal"]["definition"]["destinations"] = serde_json::json!(["/etc/passwd"]),
            1 => s["goal"]["definition"]["initial"]["policy"] = serde_json::json!([]),
            2 => {
                s["initial"]["simulation"]["assignment"]["draft"] =
                    serde_json::json!("# foreign initial")
            }
            3 => {
                s["initial"]["simulation"]["assignment"]["last_run"] =
                    serde_json::json!("a".repeat(8193))
            }
            4 => {
                s["initial"]["simulation"]["assignment"]["diagnostic"] =
                    serde_json::json!("a".repeat(1025))
            }
            _ => {
                s["goal"]["definition"]["example_worked"] =
                    serde_json::json!("*.debug /var/log/operations.log")
            }
        };
        assert!(pack::load_catalog(&[BASE, &v.to_string()]).is_err(), "{n}");
    }
    let all = labs();
    let s = &all[0];
    let mut a = Attempt::new(s, 1);
    a.workspace = Some(dlms_learning_lab::workspace::WorkspaceData::new());
    a.workspace.as_mut().unwrap().command = "foreign".into();
    assert!(w::context(&a, s).is_err());
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
#[test]
fn logging_matrix_feedback_reports_missing_urgent_and_extra_low_priority_correctly() {
    let s = labs().remove(0);
    let (d, initial) = parts(&s);
    for text in [
        "*.=warning /var/log/operations.log\n",
        "*.notice /var/log/operations.log\n",
    ] {
        let mut x = initial.clone();
        x.assignment.draft = text.into();
        x.run(d, false).unwrap();
        let rows = d.checks(&x, false);
        assert!(rows.iter().any(|r| !r.0
            && r.1.contains(if text.contains('=') {
                "emerg(0)"
            } else {
                "notice(5)"
            })));
    }
}

#[test]
fn logging_current_catalog_preserves_every231_lab_and62_unit_object() {
    let prior = pack::accepted231().unwrap();
    let now = pack::accepted235().unwrap();
    assert_eq!(prior.len(), 231);
    assert_eq!(now.len(), 235);
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(&now[..231]).unwrap()
    );
    let old = dlms_learning_lab::course::accepted231(&prior).unwrap();
    let units = dlms_learning_lab::course::accepted235(&now).unwrap();
    assert_eq!(old.len(), 62);
    assert_eq!(units.len(), 63);
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&units[..62]).unwrap()
    );
    assert_eq!(
        units.last().unwrap().labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
}
