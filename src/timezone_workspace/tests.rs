use super::*;
use crate::timezones::Civil;
fn moment(id: &str, m: u8) -> Moment {
    Moment {
        id: id.into(),
        label: if m == 1 { "Winter" } else { "Summer" }.into(),
        epoch: Civil::new(2026, m, 15, 12, 0, 0).unwrap().epoch().unwrap(),
    }
}
fn date(id: &str, m: &str, scope: Scope, expected: &str) -> Case {
    Case::Date {
        id: id.into(),
        moment: m.into(),
        scope,
        expected: expected.into(),
    }
}
fn task(id: &str) -> Task {
    let winter = moment("winter", 1);
    let summer = moment("summer", 7);
    Task {
        id: id.into(),
        initial: Snapshot {
            configured: "UTC".into(),
            environment: Environment { tz: None },
            clock: winter.epoch,
        },
        facts: Facts {
            moments: vec![winter, summer],
        },
        cases: vec![
            Case::Configured {
                id: "saved-zone".into(),
                expected: Zone::Berlin,
            },
            date(
                "current-winter",
                "winter",
                Scope::Current {},
                "2026-01-15 13:00:00 +0100 CET",
            ),
            date(
                "current-summer",
                "summer",
                Scope::Current {},
                "2026-07-15 14:00:00 +0200 CEST",
            ),
            date(
                "login-winter",
                "winter",
                Scope::Login {},
                "2026-01-15 13:00:00 +0100 CET",
            ),
            date(
                "login-summer",
                "summer",
                Scope::Login {},
                "2026-07-15 14:00:00 +0200 CEST",
            ),
        ],
        reference: vec![Command::new("timedatectl set-timezone Europe/Berlin")],
    }
}
fn override_task() -> Task {
    let mut t = task("timezone-override-task");
    t.initial.configured = "Europe/Berlin".into();
    t.initial.environment.tz = Some("UTC".into());
    t.cases.push(date(
        "future-utc",
        "winter",
        Scope::Handover {
            system_zone: Zone::Utc,
        },
        "2026-01-15 12:00:00 +0000 UTC",
    ));
    t.cases.push(date(
        "future-new-york",
        "summer",
        Scope::Handover {
            system_zone: Zone::NewYork,
        },
        "2026-07-15 08:00:00 -0400 EDT",
    ));
    t.reference = vec![Command::new("unset TZ")];
    t
}
fn definition() -> Definition {
    let independent = task("timezone-default-task");
    let mut example = task("timezone-different-example");
    for m in &mut example.facts.moments {
        m.epoch += 86_400;
        m.id = format!("demo-{}", m.id);
        m.label = format!("Different {}", m.label);
    }
    example.initial.clock = example.facts.moments[0].epoch;
    for c in &mut example.cases {
        if let Case::Date {
            moment, expected, ..
        } = c
        {
            *moment = format!("demo-{moment}");
            *expected = expected.replace("-15 ", "-16 ");
        }
    }
    Definition {
        initial: independent,
        example,
    }
}
fn run(d: &mut Draft, t: &Task, raw: &str) -> std::result::Result<Output, Error> {
    d.input = raw.into();
    d.run(t).unwrap()
}
fn solve(d: &mut Draft, t: &Task) {
    for c in &t.reference {
        run(d, t, &c.text).unwrap();
    }
    assert!(d.check(t).unwrap().iter().all(|r| r.correct));
}
#[test]
fn real_system_and_process_goals_reach_multiple_actual_outcomes() {
    for t in [task("timezone-test-task"), override_task()] {
        t.validate().unwrap();
        let initial = t.initial.world().unwrap();
        assert!(t.checks(&initial).iter().any(|r| !r.correct));
        let good = t.replay(&t.reference).unwrap();
        assert!(t.checks(&good).iter().all(|r| r.correct));
        assert_eq!(good.clock(), initial.clock());
    }
}
#[test]
fn supported_zone_aliases_and_literal_command_forms_pass_without_string_matching() {
    let mut t = task("timezone-utc-alias");
    t.initial.configured = "Europe/Berlin".into();
    t.reference = vec![Command::new("timedatectl set-timezone UTC")];
    for c in &mut t.cases {
        match c {
            Case::Configured { expected, .. } => *expected = Zone::Utc,
            Case::Date {
                moment, expected, ..
            } => {
                *expected = if moment == "winter" {
                    "2026-01-15 12:00:00 +0000 UTC"
                } else {
                    "2026-07-15 12:00:00 +0000 UTC"
                }
                .into()
            }
        }
    }
    t.validate().unwrap();
    let world = t
        .replay(&[Command::new("timedatectl set-timezone 'Etc/UTC'")])
        .unwrap();
    assert!(t.checks(&world).iter().all(|r| r.correct));
    let t = override_task();
    let world = t.replay(&[Command::new("unset 'TZ'")]).unwrap();
    assert!(t.checks(&world).iter().all(|r| r.correct));
}
#[test]
fn process_only_changes_do_not_hide_wrong_saved_defaults_or_missing_login_work() {
    let t = task("timezone-partial-task");
    let current = t
        .replay(&[Command::new("export TZ=Europe/Berlin")])
        .unwrap();
    let checks = t.checks(&current);
    assert_eq!(checks.iter().filter(|c| !c.correct).count(), 3);
    assert!(
        checks
            .iter()
            .any(|r| r.correct && r.label.starts_with("Current process"))
    );
    assert!(
        checks
            .iter()
            .filter(|r| !r.correct)
            .all(|r| r.feedback.contains("default"))
    );
}
#[test]
fn hardcoded_process_region_and_empty_tz_fail_changed_default_contexts() {
    let t = override_task();
    let pinned = t
        .replay(&[Command::new("export TZ=Europe/Berlin")])
        .unwrap();
    let checks = t.checks(&pinned);
    assert_eq!(checks.iter().filter(|r| !r.correct).count(), 2);
    assert!(
        checks
            .iter()
            .filter(|r| !r.correct)
            .all(|r| r.label.starts_with("After the supplied zone handover"))
    );
    let empty = t.replay(&[Command::new("export TZ=''")]).unwrap();
    let checks = t.checks(&empty);
    assert!(
        checks
            .iter()
            .any(|r| !r.correct && r.feedback.contains("empty TZ value"))
    );
    assert!(
        checks
            .iter()
            .any(|r| r.correct && r.label.starts_with("Separate login"))
    );
}
#[test]
fn correct_printed_text_or_a_utc_view_does_not_award_state_credit() {
    let t = task("timezone-output-shortcut");
    let mut d = Draft::new(&t);
    assert_eq!(
        run(&mut d, &t, "date '+all requirements pass'")
            .unwrap()
            .stdout,
        "all requirements pass\n"
    );
    run(&mut d, &t, "date -u '+%F %T %z %Z'").unwrap();
    assert!(d.check(&t).unwrap().iter().any(|r| !r.correct));
    assert!(!d.checked);
    assert_eq!(d.world(&t).unwrap(), t.initial.world().unwrap());
}
#[test]
fn practice_is_separate_and_roundtrip_rederives_real_work_and_output() {
    let def = definition();
    def.validate().unwrap();
    let mut s = Session::new(&def);
    solve(&mut s.practice, &def.example);
    assert!(!s.solved(&def));
    assert_eq!(s.assignment, Draft::new(&def.initial));
    s.assignment.input = "timedatectl set-timezone Europe/Berlin".into();
    let raw = serde_json::to_string(&s).unwrap();
    let loaded: Session = serde_json::from_str(&raw).unwrap();
    loaded.validate(&def).unwrap();
    assert_eq!(loaded, s);
    assert_eq!(
        loaded.practice.last_output(&def.example).unwrap(),
        s.practice.last_output(&def.example).unwrap()
    );
    assert!(!loaded.solved(&def));
}
#[test]
fn changes_invalidate_current_success_queries_and_unsupported_input_do_not_fake_changes() {
    let def = definition();
    let mut s = Session::new(&def);
    solve(&mut s.assignment, &def.initial);
    assert!(s.solved(&def));
    let good = s.assignment.world(&def.initial).unwrap();
    run(&mut s.assignment, &def.initial, "date -u +%s").unwrap();
    assert!(s.solved(&def));
    assert_eq!(
        run(&mut s.assignment, &def.initial, "printenv TZ")
            .unwrap()
            .status,
        1
    );
    assert!(s.solved(&def));
    assert_eq!(
        run(&mut s.assignment, &def.initial, "sh script.sh")
            .unwrap_err()
            .kind,
        ErrorKind::Unsupported
    );
    assert_eq!(s.assignment.world(&def.initial).unwrap(), good);
    assert!(s.solved(&def));
    run(
        &mut s.assignment,
        &def.initial,
        "timedatectl set-timezone UTC",
    )
    .unwrap();
    assert!(!s.assignment.checked && !s.assignment.attempted && !s.solved(&def));
    assert!(
        s.assignment
            .check(&def.initial)
            .unwrap()
            .iter()
            .any(|r| !r.correct)
    );
}
#[test]
fn reset_only_selected_draft_preserves_other_work() {
    let def = definition();
    let mut s = Session::new(&def);
    solve(&mut s.practice, &def.example);
    solve(&mut s.assignment, &def.initial);
    let p = s.practice.clone();
    s.reset_assignment(&def);
    assert_eq!(s.practice, p);
    assert_eq!(s.assignment, Draft::new(&def.initial));
    solve(&mut s.assignment, &def.initial);
    let a = s.assignment.clone();
    s.reset_practice(&def);
    assert_eq!(s.assignment, a);
    assert_eq!(s.practice, Draft::new(&def.example));
}
#[test]
fn forged_completion_task_output_and_future_versions_refuse() {
    let def = definition();
    let virgin = Session::new(&def);
    let mut bad = virgin.clone();
    bad.assignment.checked = true;
    bad.assignment.attempted = true;
    assert!(bad.validate(&def).is_err());
    let mut bad = virgin.clone();
    bad.version = 2;
    assert!(bad.validate(&def).is_err());
    let mut bad = virgin.clone();
    bad.assignment.task_id = "other-source".into();
    assert!(bad.validate(&def).is_err());
    let mut good = virgin.clone();
    solve(&mut good.assignment, &def.initial);
    let mut bad = good.clone();
    bad.assignment.last_run.as_mut().unwrap().command.text = "date +%s".into();
    assert!(bad.validate(&def).is_err());
    let mut bad = good.clone();
    bad.assignment.last_run.as_mut().unwrap().before = 999;
    assert!(bad.validate(&def).is_err());
    let mut bad = good.clone();
    bad.assignment.checked = false;
    assert!(bad.validate(&def).is_err());
    let mut value = serde_json::to_value(&virgin).unwrap();
    value["assignment"]["world"] = serde_json::json!({"configured":"Europe/Berlin"});
    assert!(serde_json::from_value::<Session>(value).is_err());
}
#[test]
fn malformed_or_unreachable_source_cannot_be_admitted() {
    let original = task("timezone-invalid-source");
    let mut t = original.clone();
    t.facts.moments[1] = t.facts.moments[0].clone();
    assert!(t.validate().is_err());
    let mut t = original.clone();
    t.initial.clock += 1;
    assert!(t.validate().is_err());
    let mut t = original.clone();
    t.reference = vec![Command::new("date '+all requirements pass'")];
    assert!(t.validate().is_err());
    let mut t = original.clone();
    t.reference = vec![Command::new("sh repair.sh")];
    assert!(t.validate().is_err());
    let mut t = original.clone();
    if let Case::Date { expected, .. } = &mut t.cases[1] {
        *expected = "made-up output".into();
    }
    assert!(t.validate().is_err());
    let mut t = original.clone();
    for c in &mut t.cases {
        if let Case::Date { scope, .. } = c {
            *scope = Scope::Current {};
        }
    }
    assert!(t.validate().is_err());
    let mut t = original.clone();
    t.facts.moments[1].epoch = timezones::AFTER;
    assert!(t.validate().is_err());
    let mut t = original;
    let mut world = t.initial.world().unwrap();
    world
        .command("timedatectl set-timezone Europe/Berlin")
        .unwrap();
    t.initial.configured = "Europe/Berlin".into();
    assert!(t.validate().is_err());
}
#[test]
fn bounded_history_errors_restore_without_unlimited_execution_or_mutation() {
    let t = task("timezone-history-bounds");
    let mut d = Draft::new(&t);
    for _ in 0..MAX_HISTORY {
        run(&mut d, &t, "date +%s").unwrap();
    }
    let before = d.world(&t).unwrap();
    assert_eq!(
        run(&mut d, &t, "timedatectl set-timezone Europe/Berlin")
            .unwrap_err()
            .kind,
        ErrorKind::Limit
    );
    assert_eq!(d.history.len(), MAX_HISTORY);
    assert_eq!(d.world(&t).unwrap(), before);
    d.validate(&t).unwrap();
    assert_eq!(
        d.last_output(&t).unwrap().unwrap().unwrap_err().kind,
        ErrorKind::Limit
    );
    let mut value = serde_json::to_value(&d).unwrap();
    value["history"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"text":"date"}));
    assert!(serde_json::from_value::<Draft>(value).is_err());
}

fn scenario() -> crate::pack::Scenario {
    let mut d = definition();
    d.initial.id = "timezone-capability-binding-fixture".into();
    let mut solution = d
        .initial
        .reference
        .iter()
        .map(|c| c.text.clone())
        .collect::<Vec<_>>();
    solution.push("@timezone-check".into());
    let data = serde_json::json!({"schema_version":1,"scenarios":[{
        "id":d.initial.id,"version":1,"track":"LPIC-1","topic":"Time-zone display",
        "title":"Repair supplied time-zone policy","summary":"Unregistered technical binding fixture, not authored curriculum credit.",
        "objective":"Set the supplied default zone and verify current and separate-login timestamps without changing the UTC instant.",
        "difficulty":"Beginner","minutes":20,
        "steps":["Inspect the supplied policy and timestamps.","Repair the default and verify both contexts."],
        "hints":["A command-local UTC display is different from a saved default."],
        "sources":["https://www.lpi.org/our-certifications/exam-101-102-objectives/"],
        "explanation":"Technical source/progress fixture; complete instructional content and native review are separate gates.",
        "simplifications":"Finite virtual2026zones, C date formats and an immutable clock. No host command, files or service.",
        "initial":{"kind":"Timezone","simulation":Session::new(&d)},
        "goal":{"kind":"Timezone","definition":d},"solution":solution,"prerequisites":[]
    }]});
    crate::pack::load(&serde_json::to_string(&data).unwrap())
        .unwrap()
        .remove(0)
}
fn attempt_session(a: &mut crate::progress::Attempt) -> &mut Session {
    let crate::engine::State::Timezone { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn actual_pack_engine_source_and_completion_require_independent_work() {
    use crate::{engine, progress};
    let s = scenario();
    let engine::Goal::Timezone { definition } = &s.goal else {
        panic!()
    };
    let mut a = progress::Attempt::new(&s, 1);
    solve(&mut attempt_session(&mut a).practice, &definition.example);
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "@timezone-check").unwrap();
    assert!(engine::checks(&a.state, &s.goal).iter().any(|r| !r.0));
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    assert_eq!(a.step, 0);
    assert_eq!(
        engine::execute_for_goal(&mut a.state, &s.goal, "date +%z").unwrap(),
        "Exit status 0\n+0100\n"
    );
    engine::execute_for_goal(&mut a.state, &s.goal, "export TZ=UTC").unwrap();
    assert!(!progress::current_solved(&a, &s));
    let before = a.state.clone();
    assert!(engine::execute(&mut a.state, "@timezone-check").is_err());
    assert_eq!(a.state, before);
}
#[test]
fn engine_refuses_host_forms_and_mismatched_sources_without_mutation() {
    use crate::{engine, progress};
    let s = scenario();
    let mut a = progress::Attempt::new(&s, 1);
    for raw in [
        "sh script.sh",
        "cat /etc/passwd",
        "date --set=now",
        "export PATH=/usr/bin",
        "@timezone-check extra",
        "date; id",
    ] {
        let before = a.state.clone();
        assert!(
            engine::execute_for_goal(&mut a.state, &s.goal, raw).is_err(),
            "{raw}"
        );
        assert_eq!(a.state, before);
    }
    let mut bad = s.clone();
    let engine::Goal::Timezone { definition } = &mut bad.goal else {
        panic!()
    };
    definition.initial.id = "foreign-source".into();
    assert!(source(&bad).is_err());
    let mut bad = s.clone();
    let engine::State::Timezone { simulation } = &mut bad.initial else {
        panic!()
    };
    simulation.view = crate::script_teaching::View::Example;
    assert!(source(&bad).is_err());
    let mut bad = a;
    bad.scenario_version += 1;
    assert!(context(&bad, &s).is_err());
}
#[test]
fn real_disk_resume_preserves_all417_old_attempt_values_and_separate_unfinished_drafts() {
    use crate::progress;
    let s = scenario();
    let mut catalog = crate::pack::accepted417().unwrap();
    assert_eq!(catalog.len(), 417);
    assert_eq!(crate::course::accepted417(&catalog).unwrap().len(), 160);
    let mut p = progress::Progress {
        schema_version: 1,
        ..Default::default()
    };
    for old in &catalog {
        let mut a = progress::Attempt::new(old, 1);
        crate::workspace::prepare(&mut a);
        if crate::command_location_workspace::has_source(old) {
            crate::command_location_workspace::prepare(&mut a, old, true).unwrap();
        }
        if crate::accounts_template_workspace::has_source(old) {
            crate::accounts_template_workspace::prepare(&mut a, old, true).unwrap();
        }
        p.attempts.insert(old.id.clone(), a);
    }
    let preserved = serde_json::to_value(&p.attempts).unwrap();
    let mut a = progress::Attempt::new(&s, 1);
    attempt_session(&mut a).assignment.input = "date --date=@unfinished".into();
    attempt_session(&mut a).practice.input = "export TZ='unfinished".into();
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    catalog.push(s.clone());
    let dir = std::env::temp_dir().join(format!(
        "timezone-binding-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    assert!(
        dir.starts_with(
            std::env::current_dir()
                .unwrap()
                .join(".build-artifacts/tmp")
        )
    );
    std::fs::create_dir(&dir).unwrap();
    let file = dir.join("progress.json");
    progress::save_with_catalog(&file, &p, &catalog).unwrap();
    let original = std::fs::read(&file).unwrap();
    let mut restored = progress::load(&file, &catalog).unwrap();
    assert!(restored.favorites.contains(&s.id));
    let current = restored.attempts.remove(&s.id).unwrap();
    assert_eq!(serde_json::to_value(restored.attempts).unwrap(), preserved);
    assert_eq!(
        serde_json::to_value(&current).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
    assert!(!progress::current_solved(&current, &s));
    assert!(progress::save(&file, &p).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), original);
    let mut bad: progress::Progress =
        serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    let a = bad.attempts.get_mut(&s.id).unwrap();
    attempt_session(a).assignment.checked = true;
    attempt_session(a).assignment.attempted = true;
    assert!(!progress::current_solved(a, &s));
    assert!(progress::save_with_catalog(&file, &bad, &catalog).is_err());
    assert_eq!(std::fs::read(&file).unwrap(), original);
}
