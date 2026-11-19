//! Authored reachability/outcome proof before catalog admission; no real RPM.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    rpm_packages_teaching as teaching,
    rpm_packages_terminal::Outcome,
    workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic1-rpm-metadata-preserve", "lpic1-rpm-mixed-recovery"];
fn labs() -> &'static Vec<pack::Scenario> {
    static S: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    S.get_or_init(|| {
        let before = pack::accepted305().unwrap();
        let mut texts = before
            .chunks(8)
            .map(|s| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":s}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        texts.push(include_str!("../scenarios/rpm-metadata-lessons-v1.json").into());
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn definition(s: &pack::Scenario) -> &dlms_learning_lab::rpm_packages_workspace::Definition {
    let Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn run(s: &pack::Scenario, state: &mut State, c: &str) {
    engine::execute_for_goal(state, &s.goal, c).unwrap();
}
#[test]
fn metadata_authored_tasks_and_separate_taught_experiments_repair_real_current_state() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = definition(s);
        assert!(d.allow_metadata_restore);
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for c in &s.solution {
            run(s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert!(l.separate_from_definition(d));
        assert!(l.definition.allow_metadata_restore);
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, d, l, true).unwrap();
        let initial = a.state.clone();
        let example = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
        for c in &l.example_commands {
            example.insert(l, &c.command).unwrap();
            assert!(matches!(
                example.run(l).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        for c in &l.try_change.commands {
            example.insert(l, &c.command).unwrap();
            assert!(matches!(
                example.run(l).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert_eq!(
            example.checks(l).iter().map(|r| r.0).collect::<Vec<_>>(),
            l.try_change.expected
        );
        for c in &l.repair {
            example.insert(l, &c.command).unwrap();
            assert!(matches!(
                example.run(l).unwrap(),
                Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        assert_eq!(a.state, initial);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn queries_wrong_scope_metadata_only_partial_and_unrequested_upgrade_never_earn_completion() {
    for s in labs() {
        let d = definition(s);
        let mut state = s.initial.clone();
        for c in [
            "rpm -q queue-agent",
            "rpm -ql queue-agent",
            "rpm --restore report-pages",
        ] {
            run(s, &mut state, c);
        }
        assert!(!engine::solved(&state, &s.goal));
        let State::RpmPackages { simulation } = &mut state else {
            panic!()
        };
        assert!(!d.checks(simulation)[3].0);
        assert!(d.checks(simulation)[3].1.contains("mode"));
        if s.id == IDS[1] {
            run(s, &mut state, "rpm --restore queue-agent");
            let State::RpmPackages { simulation } = &state else {
                panic!()
            };
            let rows = d.checks(simulation);
            assert!(rows[3].0 && rows[4].0);
            assert!(!rows[7].0);
            assert!(rows[7].1.contains("payload"));
            assert!(rows[7].1.contains("File text still differs"));
            assert!(rows[7].1.contains("restoring attributes cannot replace damaged text"));
        }
        for c in &s.solution {
            run(s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal));
        run(s, &mut state, "rpm -U /rpms/report-pages-v2.rpm");
        assert!(!engine::solved(&state, &s.goal));
        let State::RpmPackages { simulation } = &state else {
            panic!()
        };
        assert!(!d.checks(simulation)[1].0);
    }
}
#[test]
fn supported_alternate_orders_and_real_state_invalidation_preserve_text_and_neighbors() {
    for s in labs() {
        let mut state = s.initial.clone();
        let commands = if s.id == IDS[1] {
            vec![
                "rpm --reinstall /rpms/report-pages-v1.rpm",
                "rpm --restore report-pages",
                "rpm --restore queue-agent",
            ]
        } else {
            vec!["rpm --restore report-pages", "rpm --restore queue-agent"]
        };
        for c in commands {
            run(s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal));
        let State::RpmPackages { simulation } = &state else {
            panic!()
        };
        assert_eq!(
            simulation.runtime().files["/srv/teaching/queue-agent.conf"].contents,
            "style=site\n"
        );
        run(s, &mut state, "rpm -e desk-notes");
        assert!(!engine::solved(&state, &s.goal));
        run(s, &mut state, "rpm -i /rpms/desk-notes-v1.rpm");
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn actual_progress_save_read_preserves_assignment_example_drafts_favorites_and_selected_reset() {
    for s in labs() {
        let d = definition(s);
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, d, l, true).unwrap();
        for c in &s.solution {
            run(s, &mut a.state, c);
        }
        assert!(progress::current_solved(&a, s));
        let w = a.workspace.as_mut().unwrap();
        w.rpm_session.as_mut().unwrap().command = "rpm -q desk-notes".into();
        w.rpm_learning.as_mut().unwrap().terminal.command = "rpm -q gallery-panel".into();
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, labs()).unwrap();
        let mut q = progress::load(&path, labs()).unwrap();
        assert_eq!(
            serde_json::to_value(&p).unwrap(),
            serde_json::to_value(&q).unwrap()
        );
        assert!(progress::current_solved(&q.attempts[&s.id], s));
        let a = q.attempts.get_mut(&s.id).unwrap();
        *a = Attempt::new(s, 2);
        workspace::prepare(a);
        teaching::prepare(a, d, l, true).unwrap();
        assert_eq!(a.state, s.initial);
        assert!(q.favorites.contains(&s.id));
    }
}

#[test]
fn registered_recovery_preserves305_existing_labs102_units_and_real_prerequisites() {
    let old = pack::accepted305().unwrap();
    let all = pack::accepted307().unwrap();
    assert_eq!((old.len(), all.len()), (305, 307));
    assert_eq!(
        serde_json::to_value(&all[..305]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [74, 109, 47, 77]
    );
    for s in labs() {
        let registered = all.iter().find(|r| r.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(registered).unwrap(),
            serde_json::to_value(s).unwrap()
        );
        let at = all.iter().position(|r| r.id == s.id).unwrap();
        for p in &s.prerequisites {
            assert!(all.iter().position(|r| &r.id == p).unwrap() < at);
        }
    }
    let before = dlms_learning_lab::course::accepted305(&old).unwrap();
    let units = dlms_learning_lab::course::accepted307(&all).unwrap();
    assert_eq!((before.len(), units.len()), (102, 103));
    let kept = units
        .iter()
        .filter(|u| u.id != "rpm-payload-recovery")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(kept).unwrap(),
        serde_json::to_value(before).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "rpm-payload-recovery")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < at);
    }
}
