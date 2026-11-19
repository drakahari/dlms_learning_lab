//! Authored methods and negative outcomes; registration/native acceptance are separate.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    maintenance_notice_workspace as work, pack,
    progress::{self, Attempt, Progress},
};
const IDS: [&str; 2] = [
    "lpic2-maintenance-login-notices",
    "lpic2-maintenance-restart-handover",
];
fn labs() -> Vec<pack::Scenario> {
    static AUTHORED: std::sync::OnceLock<Vec<pack::Scenario>> = std::sync::OnceLock::new();
    AUTHORED
        .get_or_init(|| {
            let pending: pack::Pack =
                serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v18.json"))
                    .unwrap();
            let baseline = pack::accepted404().unwrap();
            let mut map: std::collections::BTreeMap<_, _> = baseline
                .into_iter()
                .chain(pending.scenarios)
                .map(|s| (s.id.clone(), s))
                .collect();
            let mut needed = std::collections::BTreeSet::new();
            let mut queue = IDS.iter().map(|id| id.to_string()).collect::<Vec<_>>();
            while let Some(id) = queue.pop() {
                if needed.insert(id.clone()) {
                    queue.extend(map[&id].prerequisites.clone());
                }
            }
            let texts = needed
                .iter()
                .map(|id| {
                    serde_json::to_string(&pack::Pack {
                        schema_version: 1,
                        scenarios: vec![map.remove(id).unwrap()],
                    })
                    .unwrap()
                })
                .collect::<Vec<_>>();
            let refs = texts.iter().map(String::as_str).collect::<Vec<_>>();
            let loaded = pack::load_catalog(&refs).unwrap();
            IDS.iter()
                .map(|id| loaded.iter().find(|s| s.id == *id).unwrap().clone())
                .collect()
        })
        .clone()
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::MaintenanceNotice { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn state(a: &mut Attempt) -> &mut work::Session {
    let State::MaintenanceNotice { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn solve(a: &mut Attempt, s: &pack::Scenario) {
    for op in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, op).unwrap();
    }
    assert!(progress::current_solved(a, s));
}
#[test]
fn both_distinct_methods_reach_every_case_and_keep_worked_actions_ungraded() {
    let labs = labs();
    assert_eq!(labs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    assert!(!definition(&labs[0]).initial.requires_restart);
    assert!(definition(&labs[1]).initial.requires_restart);
    for s in labs {
        let d = definition(&s);
        let mut a = Attempt::new(&s, 1);
        assert!(!progress::current_solved(&a, &s));
        assert_ne!(d.initial.required, d.example.required);
        assert_ne!(d.initial.facts, d.example.facts);
        for action in &d.example.reference {
            state(&mut a)
                .practice
                .apply_action(&d.example, action.clone())
                .unwrap();
        }
        state(&mut a).practice.check(&d.example).unwrap();
        assert!(state(&mut a).practice.solved(&d.example));
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(
            state(&mut a).assignment,
            work::Draft::new(&d.initial).unwrap()
        );
        solve(&mut a, &s);
        assert!(engine::checks(&a.state, &s.goal).iter().all(|r| r.0));
    }
}
#[test]
fn actual_editor_and_terminal_method_works_without_internal_reference_strings() {
    for s in labs() {
        let t = &definition(&s).initial;
        let mut a = Attempt::new(&s, 1);
        let draft = &mut state(&mut a).assignment;
        if t.requires_restart {
            draft.run(t, "shutdown -c").unwrap();
            draft.run(t, "shutdown -r +30").unwrap();
        }
        draft.edit(t.required.render()).unwrap();
        draft.save(t).unwrap();
        draft.selected = work::Editor::Stdin;
        draft.edit(t.required.render()).unwrap();
        draft.command = "wall".into();
        assert!(draft.run_and_record(t).unwrap().is_empty());
        draft.check(t).unwrap();
        assert!(progress::current_solved(&a, &s));
    }
}
#[test]
fn equivalent_notice_order_case_and_delay_after_a_clock_advance_pass_by_outcome() {
    for s in labs() {
        let t = &definition(&s).initial;
        let mut a = Attempt::new(&s, 1);
        let d = &mut state(&mut a).assignment;
        if t.requires_restart {
            d.run(t, "shutdown -c 'The inherited time is cancelled'")
                .unwrap();
            d.advance(t, 5).unwrap();
            d.run(t, "shutdown -r +0025 'Approved window'").unwrap();
        }
        let alt = format!(
            " CONTACT : {}\nENDS: {:02}:{:02}\nSERVICE: {}\nstarts: {:02}:{:02}\n",
            t.required.contact.to_ascii_uppercase(),
            t.required.ends / 60,
            t.required.ends % 60,
            t.required.service.to_ascii_uppercase(),
            t.required.starts / 60,
            t.required.starts % 60
        );
        d.selected = work::Editor::Stdin;
        d.edit(alt.clone()).unwrap();
        d.run(t, "wall").unwrap();
        d.selected = work::Editor::Motd;
        d.edit(alt).unwrap();
        d.save(t).unwrap();
        d.check(t).unwrap();
        assert!(progress::current_solved(&a, &s));
    }
}
#[test]
fn copied_worked_bulletins_partial_repairs_and_newer_wrong_announcements_do_not_pass() {
    for s in labs() {
        let def = definition(&s);
        let t = &def.initial;
        let mut a = Attempt::new(&s, 1);
        let d = &mut state(&mut a).assignment;
        d.edit(def.example.required.render()).unwrap();
        d.save(t).unwrap();
        d.selected = work::Editor::Stdin;
        d.edit(def.example.required.render()).unwrap();
        d.run(t, "wall").unwrap();
        d.check(t).unwrap();
        assert!(!d.solved(t));
        assert!(
            d.checks(t).unwrap()[0]
                .feedback
                .contains("different service")
        );
        // Save the right file while live recipients still have the wrong example.
        d.selected = work::Editor::Motd;
        d.edit(t.required.render()).unwrap();
        d.save(t).unwrap();
        d.check(t).unwrap();
        let checks = d.checks(t).unwrap();
        assert!(checks[0].correct);
        assert!(
            checks
                .iter()
                .any(|c| c.label.starts_with("Current session") && !c.correct)
        );
        assert!(!d.solved(t));
        let mut a = Attempt::new(&s, 1);
        solve(&mut a, &s);
        let d = &mut state(&mut a).assignment;
        d.selected = work::Editor::Stdin;
        d.edit("The bulletin is no longer valid".into()).unwrap();
        assert!(!d.checked);
        d.run(t, "wall").unwrap();
        d.check(t).unwrap();
        assert!(!d.solved(t));
        assert!(
            d.checks(t).unwrap()[0].correct,
            "Correct saved-file result must remain clear"
        );
    }
}
#[test]
fn stale_schedule_cancellation_alone_unsaved_edits_and_expired_notice_are_diagnosed() {
    let labs = labs();
    let s = &labs[1];
    let t = &definition(s).initial;
    let mut a = Attempt::new(s, 1);
    state(&mut a).assignment.check(t).unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        state(&mut a)
            .assignment
            .checks(t)
            .unwrap()
            .iter()
            .any(|c| c.feedback.contains("pending restart does not match"))
    );
    state(&mut a).assignment.run(t, "shutdown -c").unwrap();
    state(&mut a).assignment.check(t).unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        state(&mut a)
            .assignment
            .checks(t)
            .unwrap()
            .iter()
            .any(|c| c.feedback.contains("only part"))
    );
    let mut a = Attempt::new(s, 1);
    solve(&mut a, s);
    state(&mut a).assignment.selected = work::Editor::Motd;
    state(&mut a)
        .assignment
        .edit(t.required.render().replace("Starts:", "Missing:"))
        .unwrap();
    state(&mut a).assignment.check(t).unwrap();
    assert!(!progress::current_solved(&a, s));
    assert!(
        state(&mut a)
            .assignment
            .checks(t)
            .unwrap()
            .iter()
            .any(|c| c.label == "Saved files and sent message"
                && !c.correct
                && c.feedback.contains("Save changed login files"))
    );
    let first = &labs[0];
    let mut a = Attempt::new(first, 1);
    solve(&mut a, first);
    state(&mut a)
        .assignment
        .advance(&definition(first).initial, 30)
        .unwrap();
    state(&mut a)
        .assignment
        .check(&definition(first).initial)
        .unwrap();
    assert!(!progress::current_solved(&a, first));
}
#[test]
fn drafts_favorites_and_peer_attempt_survive_resume_and_only_selected_reset() {
    let labs = labs();
    let mut p = Progress {
        schema_version: 1,
        clock: 5,
        ..Default::default()
    };
    for s in &labs {
        let mut a = Attempt::new(s, 1);
        solve(&mut a, s);
        state(&mut a).practice.selected = work::Editor::Stdin;
        state(&mut a)
            .practice
            .edit("unfinished worked input".into())
            .unwrap();
        state(&mut a).practice.command = "who".into();
        p.attempts.insert(s.id.clone(), a);
        p.favorites.insert(s.id.clone());
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("authored-notices.json");
    progress::save_with_catalog(&path, &p, &labs).unwrap();
    let mut restored = progress::load(&path, &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let peer = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    let practice = state(restored.attempts.get_mut(IDS[0]).unwrap())
        .practice
        .clone();
    state(restored.attempts.get_mut(IDS[0]).unwrap())
        .reset_assignment(definition(&labs[0]))
        .unwrap();
    progress::save_with_catalog(&path, &restored, &labs).unwrap();
    let mut read = progress::load(&path, &labs).unwrap();
    assert!(!progress::current_solved(&read.attempts[IDS[0]], &labs[0]));
    assert_eq!(serde_json::to_value(&read.attempts[IDS[1]]).unwrap(), peer);
    assert_eq!(
        state(read.attempts.get_mut(IDS[0]).unwrap()).practice,
        practice
    );
    assert!(read.favorites.contains(IDS[0]));
}
#[test]
fn unreachable_sources_hidden_solved_starters_and_unsupported_actions_refuse() {
    for s in labs() {
        let mut bad = s.clone();
        let Goal::MaintenanceNotice { definition } = &mut bad.goal else {
            panic!()
        };
        definition.initial.reference.pop();
        assert!(work::source(&bad).is_err());
        let mut a = Attempt::new(&s, 1);
        let before = a.state.clone();
        for input in [
            "wall /etc/passwd",
            "shutdown now",
            "cat ../../personal",
            "notice-action {\"kind\":\"apply\",\"action\":{\"kind\":\"execute_host\"}}",
        ] {
            assert!(engine::execute_for_goal(&mut a.state, &s.goal, input).is_err());
            assert_eq!(a.state, before);
        }
    }
}
#[test]
fn old_thirty_one_scenario_values_and_reachable_prerequisites_are_preserved() {
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v17.json")).unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v18.json")).unwrap();
    let prior = old["scenarios"].as_array().unwrap();
    let all = new["scenarios"].as_array().unwrap();
    assert_eq!(prior.len(), 31);
    assert_eq!(all.len(), 33);
    assert_eq!(&all[..31], prior.as_slice());
    let mut available = pack::ccna_pilot().unwrap();
    let new_labs = labs();
    for s in &new_labs {
        for prereq in &s.prerequisites {
            assert!(
                available.iter().any(|p| &p.id == prereq),
                "Missing taught prerequisite {prereq}"
            );
        }
        available.push(s.clone());
    }
    for s in new_labs {
        assert_eq!(definition(&s).initial.facts.logins.len(), 3);
        assert_eq!(s.unit, "linux-maintenance-notices");
        assert!(s.sources.iter().any(|u| u.contains("lpi.org")));
        assert!(s.simplifications.contains("not Linux syntax"));
    }
}

#[test]
fn registered_unit_retains_all404_previous_scenarios_and155_units_without_fabricated_credit() {
    let old = pack::accepted404().unwrap();
    let current = pack::accepted406().unwrap();
    assert_eq!((old.len(), current.len()), (404, 406));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..404]).unwrap()
    );
    let prior = dlms_learning_lab::course::accepted404(&old).unwrap();
    let units = dlms_learning_lab::course::accepted406(&current).unwrap();
    assert_eq!((prior.len(), units.len()), (155, 156));
    let kept = units
        .iter()
        .filter(|u| u.id != "linux-maintenance-notices")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(&prior).unwrap(),
        serde_json::to_value(kept).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-maintenance-notices")
        .unwrap();
    assert_eq!(units[at - 1].id, "linux-capacity-planning");
    assert_eq!(units[at].labs, IDS.map(str::to_string));
    for (track, count) in [
        ("CCNA", 93),
        ("LPIC-1", 133),
        ("LPIC-2", 100),
        ("Bash/Shell", 80),
    ] {
        assert_eq!(current.iter().filter(|s| s.track == track).count(), count);
    }
    for s in &current[404..] {
        let a = Attempt::new(s, 1);
        assert!(!progress::current_solved(&a, s));
        let initial = &definition(s).initial;
        assert_eq!(
            state(&mut a.clone()).assignment,
            work::Draft::new(initial).unwrap()
        );
    }
}
