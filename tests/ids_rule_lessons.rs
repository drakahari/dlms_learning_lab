use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    ids_rule_source as binding,
    ids_rule_workspace::Action,
    pack, progress,
};
use std::sync::OnceLock;
fn catalog() -> &'static Vec<pack::Scenario> {
    static LABS: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    LABS.get_or_init(|| pack::current().unwrap())
}
fn labs() -> Vec<&'static pack::Scenario> {
    catalog()
        .iter()
        .filter(|s| matches!(s.initial, State::IdsRule { .. }))
        .collect()
}
fn definition(s: &pack::Scenario) -> &dlms_learning_lab::ids_rule_workspace::Definition {
    binding::source(s).unwrap()
}
fn act(state: &mut State, s: &pack::Scenario, a: &Action, practice: bool) {
    let input = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    );
    engine::execute_for_goal(state, &s.goal, &input).unwrap();
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{} {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn registered_catalog_preserves_exact_505_scenarios_and_203_units() {
    let before = pack::accepted505().unwrap();
    let all = catalog();
    assert_eq!((before.len(), all.len()), (505, 517));
    assert_eq!(
        serde_json::to_value(&all[..505]).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let old = course::accepted505(&before).unwrap();
    let units = course::current(all).unwrap();
    assert_eq!((old.len(), units.len()), (203, 209));
    let retained: Vec<_> = units
        .iter()
        .filter(|u| u.id != "linux-ids-rule-evidence" && u.id != "ccna-poe-power-plans" && u.id != "linux-kernel-build-staging" && u.id != "linux-dkms-lifecycle" && u.id != "linux-initramfs-recovery" && u.id != "linux-pxe-recovery")
        .collect();
    assert_eq!(
        serde_json::to_value(retained).unwrap(),
        serde_json::to_value(old).unwrap()
    );
    let position = units
        .iter()
        .position(|u| u.id == "linux-ids-rule-evidence")
        .unwrap();
    let unit = &units[position];
    assert_eq!(
        unit.labs,
        [
            "lpic2-ids-header-direction",
            "lpic2-ids-content-specificity"
        ]
    );
    for prerequisite in &unit.prerequisites {
        assert!(units.iter().position(|u| u.id == *prerequisite).unwrap() < position);
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [115, 154, 168, 80]
    );
    let authored: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/ids-rule-lessons-v1.json")).unwrap();
    assert_eq!(
        serde_json::to_value(&all[505..507]).unwrap(),
        serde_json::to_value(authored.scenarios).unwrap()
    );
    assert_eq!(
        include_str!("../scenarios/ids-rule-lessons-v1.json"),
        include_str!("../scenarios/authoring/ids-rule-v1.json")
    );
}
#[test]
fn authored_solutions_and_separate_worked_examples_reach_all_real_outcomes() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = definition(s);
        let mut state = s.initial.clone();
        let State::IdsRule { simulation } = &state else {
            panic!()
        };
        let assignment = simulation.assignment.clone();
        assert!(!engine::solved(&state, &s.goal));
        for a in &d.worked {
            act(&mut state, s, a, true);
        }
        let State::IdsRule { simulation } = &state else {
            panic!()
        };
        assert_eq!(simulation.assignment, assignment);
        assert!(
            simulation
                .example
                .checks(&d.example)
                .iter()
                .all(|c| c.passed)
        );
        assert!(!engine::solved(&state, &s.goal));
        solve(s);
    }
}
#[test]
fn alternate_supported_rules_pass_and_broken_drafts_clear_success() {
    for s in labs() {
        let alternative = if s.id.ends_with("direction") {
            "alert tcp 203.0.113.25/24 any -> 192.0.2.40/32 8088 (sid:1000501;content:\"UPLOAD\";)"
        } else {
            "alert tcp 10.24.40.12/20 any -> 192.0.2.60 8443 (sid:1000502;content:\"action=reset\";content:\"admin\",nocase;)"
        };
        let mut state = s.initial.clone();
        for a in [
            Action::Edit {
                text: alternative.into(),
            },
            Action::Save,
            Action::Replay,
            Action::Check,
        ] {
            act(&mut state, s, &a, false);
        }
        assert!(engine::solved(&state, &s.goal));
        act(
            &mut state,
            s,
            &Action::Edit {
                text: "unfinished rule".into(),
            },
            false,
        );
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, s, &Action::Save, false);
        act(&mut state, s, &Action::Check, false);
        assert!(!engine::solved(&state, &s.goal));
        let mut attempt = progress::Attempt::new(s, 1);
        attempt.state = state;
        assert!(!progress::current_solved(&attempt, s));
        attempt.state = s.initial.clone();
        binding::context(&attempt, s).unwrap();
        assert!(!progress::current_solved(&attempt, s));
    }
}
#[test]
fn plausible_wrong_repairs_fail_with_record_specific_feedback() {
    for s in labs() {
        let d = definition(s);
        let canonical: serde_json::Value =
            serde_json::from_str(s.solution[0].strip_prefix(binding::PREFIX).unwrap()).unwrap();
        let body = canonical["action"]["text"].as_str().unwrap();
        let wrong = if s.id.ends_with("direction") {
            body.replace(" -> ", " <> ")
        } else {
            body.replace("content:\"action=reset\";", "")
        };
        let rows = d.assignment.checks(&wrong);
        assert!(
            rows.iter()
                .any(|c| !c.passed && c.detail.contains("should not alert"))
        );
        assert!(!d.assignment.solved(&wrong));
        let quiet = "alert tcp any any -> any any (sid:1;content:\"absent-marker\";)";
        assert!(!d.assignment.solved(quiet));
        let hardcoded = body.replacen("any ->", "41000 ->", 1);
        assert!(!d.assignment.solved(&hardcoded));
        let partial = if s.id.ends_with("direction") {
            body.replace("203.0.113.0/24", "203.0.113.10")
        } else {
            body.replace("10.24.32.0/20", "10.24.32.0/24")
        };
        if s.id.ends_with("specificity") {
            let wrong_case = body.replace("content:\"ADMIN\",nocase;", "content:\"ADMIN\";");
            assert!(!d.assignment.solved(&wrong_case));
        }
        let rows = d.assignment.checks(&partial);
        assert!(rows.iter().any(|c| c.passed));
        assert!(rows.iter().any(|c| !c.passed));
    }
}
#[test]
fn source_changes_and_arbitrary_actions_refuse_without_mutation() {
    for s in labs() {
        let mut changed = s.clone();
        let Goal::IdsRule { definition } = &mut changed.goal else {
            panic!()
        };
        definition.worked = vec![Action::Check];
        assert!(binding::source(&changed).is_err());
        let mut state = s.initial.clone();
        let before = serde_json::to_value(&state).unwrap();
        for raw in [
            "snort -c /etc/snort/snort.lua",
            "ids-rule-workspace {\"practice\":false,\"action\":{\"kind\":\"ExecuteOnHost\",\"command\":\"id\"}}",
        ] {
            assert!(engine::execute_for_goal(&mut state, &s.goal, raw).is_err());
            assert_eq!(serde_json::to_value(&state).unwrap(), before);
        }
        let mut attempt = progress::Attempt::new(s, 1);
        attempt.state = solve(s);
        let mut changed = s.clone();
        let Goal::IdsRule { definition } = &mut changed.goal else {
            panic!()
        };
        definition.assignment.cases[0].records[0].source_port = 12345;
        assert!(!progress::current_solved(&attempt, &changed));
        assert!(binding::context(&attempt, &changed).is_err());
    }
}
#[test]
fn introductions_practice_and_assignments_have_individually_authored_content() {
    let all: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(all.len(), 160);
    for s in labs() {
        let e = all.iter().find(|e| e["id"] == s.id).unwrap();
        assert_eq!(e["requirements"].as_array().unwrap().len(), 4);
        for r in e["requirements"].as_array().unwrap() {
            assert!(s.objective.contains(r.as_str().unwrap()));
        }
        assert!(e["concepts"].as_array().unwrap().len() >= 4);
        assert!(e["concepts"].as_array().unwrap().iter().any(|c| {
            let text = c["text"].as_str().unwrap();
            text.contains("Snort")
                && text.contains("Greenbone/OpenVAS")
                && text.contains("neither tool")
        }));
        assert_eq!(e["practice"].as_array().unwrap().len(), 3);
        for p in e["practice"].as_array().unwrap() {
            assert!(!p["action"].as_str().unwrap().is_empty());
            assert!(!p["observe"].as_str().unwrap().is_empty());
        }
        assert_eq!(e["worked"].as_array().unwrap().len(), 3);
        assert!(s.guide.iter().all(|g| g.commands.is_empty()));
        assert_ne!(
            definition(s).assignment.initial_rule,
            definition(s).example.initial_rule
        );
        assert!(s.simplifications.contains("No packet capture"));
    }
}

#[test]
fn catalog_bound_progress_retains_both_drafts_and_refuses_unknown_sources_safely() {
    let all = catalog();
    let mut p = progress::Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = progress::Attempt::new(s, 1);
        a.state = solve(s);
        act(
            &mut a.state,
            s,
            &Action::Edit {
                text: "# unfinished assignment\nalert ".into(),
            },
            false,
        );
        act(
            &mut a.state,
            s,
            &Action::Edit {
                text: "# separate unfinished example\nalert tcp ".into(),
            },
            true,
        );
        act(&mut a.state, s, &Action::Select { case: 1 }, true);
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let temp = tempfile::tempdir().unwrap();
    assert!(temp.path().starts_with(std::env::var_os("TMPDIR").unwrap()));
    let path = temp.path().join("ids-progress.json");
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    progress::save_with_catalog(&path, &p, all).unwrap();
    let before = std::fs::read(&path).unwrap();
    let refused = progress::save_with_catalog(&path, &p, &[]).unwrap_err();
    assert!(refused.contains("IDS-rule"), "{refused}");
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let restored = progress::load(&path, all).unwrap();
    assert_eq!(restored.favorites, p.favorites);
    for s in labs() {
        let a = &restored.attempts[&s.id];
        binding::context(a, s).unwrap();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(&p.attempts[&s.id]).unwrap()
        );
        assert!(!progress::current_solved(a, s));
    }
}
