//! Authored data, actual engine/terminal, alternate outcomes and progress; no host RPM.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    rpm_packages_teaching as teaching, rpm_packages_terminal as terminal,
    rpm_packages_workspace as work, workspace,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = ["lpic2-vendor-backport", "lpic2-vendor-patch-set"];
fn labs() -> &'static Vec<pack::Scenario> {
    static LABS: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    LABS.get_or_init(|| {
        let old = pack::accepted501().unwrap();
        let mut texts = old
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        texts.push(include_str!("../scenarios/advisory-patch-lessons-v1.json").into());
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::RpmPackages { definition } = &s.goal else {
        panic!()
    };
    definition
}
#[test]
fn fixed_headers_cannot_mask_damaged_modeled_patch_payloads() {
    for source in labs() {
        let prefix = if source.id == IDS[0] {
            "relay"
        } else {
            "dispatch"
        };
        let mut state = source.initial.clone();
        for command in &source.solution {
            run(source, &mut state, command);
        }
        assert!(engine::solved(&state, &source.goal));
        let State::RpmPackages { simulation } = &state else {
            panic!()
        };
        for field in ["contents", "mode", "owner", "group", "mtime"] {
            let mut runtime = simulation.runtime().clone();
            let file = runtime
                .files
                .get_mut(&format!("/srv/{prefix}/agent.txt"))
                .unwrap();
            match field {
                "contents" => file.contents = "Damaged simulated application payload\n".into(),
                "mode" => file.mode = 0o600,
                "owner" => file.owner = "alex".into(),
                "group" => file.group = "ops".into(),
                "mtime" => file.mtime = 1,
                _ => unreachable!(),
            }
            let mut damaged = State::RpmPackages {
                simulation: dlms_learning_lab::rpm_packages::State::new(
                    simulation.catalog().clone(),
                    runtime,
                )
                .unwrap(),
            };
            assert!(
                !engine::solved(&damaged, &source.goal),
                "{} {field}: headers alone must not prove an intact modeled patch",
                source.id
            );
            run(
                source,
                &mut damaged,
                &format!("rpm --reinstall /media/{prefix}/{prefix}-agent-1.0-4.train8.noarch.rpm"),
            );
            assert!(engine::solved(&damaged, &source.goal));
        }
    }
}
fn run(s: &pack::Scenario, state: &mut State, command: &str) {
    engine::execute_for_goal(state, &s.goal, command).unwrap();
}
#[test]
fn taught_solutions_and_wrong_release_experiments_have_actual_outcomes() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for c in &s.solution {
            run(s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert!(l.separate_from_definition(definition(s)));
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, definition(s), l, true).unwrap();
        let example = a.workspace.as_mut().unwrap().rpm_learning.as_mut().unwrap();
        for c in &l.example_commands {
            example.insert(l, &c.command).unwrap();
            assert!(matches!(
                example.run(l).unwrap(),
                terminal::Outcome::Utility { status: 0, .. }
            ));
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        for c in &l.try_change.commands {
            example.insert(l, &c.command).unwrap();
            example.run(l).unwrap();
        }
        let rows = example.checks(l);
        assert_eq!(
            rows.iter().map(|r| r.0).collect::<Vec<_>>(),
            l.try_change.expected
        );
        assert!(rows.iter().any(|r| !r.0 && r.1.contains("notes")));
        for c in &l.repair {
            example.insert(l, &c.command).unwrap();
            example.run(l).unwrap();
        }
        assert!(example.checks(l).iter().all(|r| r.0));
        assert_eq!(a.state, s.initial);
        assert!(!progress::current_solved(&a, s));
    }
}
#[test]
fn quoted_archive_paths_and_sequential_dependency_updates_are_valid() {
    for s in labs() {
        let mut state = s.initial.clone();
        let prefix = if s.id == IDS[0] { "relay" } else { "dispatch" };
        if prefix == "dispatch" {
            run(
                s,
                &mut state,
                "rpm -U /media/dispatch/dispatch-codec-3.0-4.train8.noarch.rpm",
            );
            assert!(!engine::solved(&state, &s.goal));
        }
        run(
            s,
            &mut state,
            &format!("rpm -U '/media/{prefix}/{prefix}-agent-1.0-4.train8.noarch.rpm'"),
        );
        assert!(engine::solved(&state, &s.goal));
        let State::RpmPackages { simulation } = &state else {
            panic!()
        };
        assert_eq!(
            simulation.runtime().files[&format!("/srv/{prefix}/agent.conf")].contents,
            "zone=site-east\n"
        );
        assert_eq!(
            simulation.runtime().files[&format!("/srv/{prefix}/agent.conf.rpmnew")].contents,
            "zone=vendor-v2\n"
        );
    }
}
#[test]
fn agent_only_update_refuses_atomically_and_codec_only_is_partial() {
    let s = &labs()[1];
    let mut state = s.initial.clone();
    let before = state.clone();
    assert!(
        engine::execute_for_goal(
            &mut state,
            &s.goal,
            "rpm -U /media/dispatch/dispatch-agent-1.0-4.train8.noarch.rpm"
        )
        .is_err()
    );
    assert_eq!(state, before);
    run(
        s,
        &mut state,
        "rpm -U /media/dispatch/dispatch-codec-3.0-2.train8.noarch.rpm",
    );
    let State::RpmPackages { simulation } = &state else {
        panic!()
    };
    let rows = definition(s).checks(simulation);
    assert!(!rows[0].0);
    assert!(rows[1].0);
    assert!(rows.last().unwrap().0);
    assert!(!engine::solved(&state, &s.goal));
}
#[test]
fn wrong_release_and_removed_neighbor_invalidate_real_success() {
    for s in labs() {
        let prefix = if s.id == IDS[0] { "relay" } else { "dispatch" };
        let mut wrong = s.initial.clone();
        if prefix == "dispatch" {
            run(
                s,
                &mut wrong,
                "rpm -U /media/dispatch/dispatch-codec-3.0-2.train8.noarch.rpm",
            );
        }
        run(
            s,
            &mut wrong,
            &format!("rpm -U /media/{prefix}/{prefix}-agent-1.0-5.train9.noarch.rpm"),
        );
        assert!(!engine::solved(&wrong, &s.goal));
        let State::RpmPackages { simulation } = &wrong else {
            panic!()
        };
        assert!(
            definition(s).checks(simulation)[0]
                .1
                .contains("another release")
        );
        // A selected reset returns to the supplied state; downgrades stay unsupported.
        let mut state = s.initial.clone();
        for c in &s.solution {
            run(s, &mut state, c);
        }
        assert!(engine::solved(&state, &s.goal));
        run(s, &mut state, &format!("rpm -e {prefix}-notes"));
        assert!(!engine::solved(&state, &s.goal));
        run(
            s,
            &mut state,
            &format!("rpm -i /media/{prefix}/{prefix}-notes-1.0-1.train8.noarch.rpm"),
        );
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn queries_and_unsupported_escape_spellings_never_change_or_complete_work() {
    for s in labs() {
        let d = definition(s);
        let mut state = d.initial.clone();
        for c in [
            "rpm --all",
            "rpm -U /etc/passwd",
            "rpm -U ../../file.rpm",
            "sh -c echo",
            "rpm -q site; touch /tmp/x",
            "rpm -U https://example.invalid/a.rpm",
        ] {
            let before = state.clone();
            assert!(matches!(
                terminal::execute(d, &mut state, c),
                terminal::Outcome::SimulatorUnsupported { .. }
            ));
            assert_eq!(before, state);
        }
        assert!(!d.checks(&state).iter().all(|r| r.0));
    }
}
#[test]
fn actual_save_resume_keeps_distinct_drafts_favorite_and_selected_reset() {
    for s in labs() {
        let d = definition(s);
        let l = s.rpm_packages_teaching.as_ref().unwrap();
        let mut a = Attempt::new(s, 1);
        workspace::prepare(&mut a);
        teaching::prepare(&mut a, d, l, true).unwrap();
        for c in &s.solution {
            run(s, &mut a.state, c);
        }
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_session
            .as_mut()
            .unwrap()
            .command = "rpm -q assignment-draft".into();
        a.workspace
            .as_mut()
            .unwrap()
            .rpm_learning
            .as_mut()
            .unwrap()
            .terminal
            .command = "rpm -q example-draft".into();
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
        assert_eq!(a.state, s.initial);
        assert!(q.favorites.contains(&s.id));
        assert!(!progress::current_solved(a, s));
    }
}

#[test]
fn registered_patch_unit_preserves_all501_labs_and201_units_exactly() {
    let old = pack::accepted501().unwrap();
    let all = pack::ccna_pilot().unwrap();
    assert_eq!((old.len(), all.len()), (501, 505));
    assert_eq!(
        serde_json::to_value(&all[..501]).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let old_units = dlms_learning_lab::course::accepted501(&old).unwrap();
    let units = dlms_learning_lab::course::ccna_pilot(&all).unwrap();
    assert_eq!((old_units.len(), units.len()), (201, 203));
    let retained = units
        .iter()
        .filter(|u| !["linux-advisory-patching", "linux-auth-log-filters"].contains(&u.id.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(old_units).unwrap()
    );
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [113, 154, 158, 80]
    );
    for source in labs() {
        let selected = all.iter().find(|s| s.id == source.id).unwrap();
        assert_eq!(
            serde_json::to_value(selected).unwrap(),
            serde_json::to_value(source).unwrap()
        );
        let at = all.iter().position(|s| s.id == source.id).unwrap();
        for prerequisite in &source.prerequisites {
            assert!(all.iter().position(|s| &s.id == prerequisite).unwrap() < at);
        }
    }
}
