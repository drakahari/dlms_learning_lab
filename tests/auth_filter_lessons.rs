//! Authored draft behavior. Registration/native acceptance is tested separately.
use dlms_learning_lab::{
    auth_filter_source as binding,
    auth_filter_workspace::Action,
    engine::{self, Goal, State},
    pack, progress,
};
use std::sync::OnceLock;
fn labs() -> &'static Vec<pack::Scenario> {
    static BASE: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    static LABS: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    LABS.get_or_init(|| {
        let prior = BASE.get_or_init(|| pack::current().unwrap());
        let mut texts = prior
            .chunks(8)
            .map(|s| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":s}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let additions: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/auth-filter-lessons-v1.json")).unwrap();
        // This remains valid before and after normal-catalog registration; a
        // duplicate authored ID must not be inserted as a separate lab.
        if !prior.iter().any(|s| s.id == additions.scenarios[0].id) {
            texts.push(serde_json::to_string(&additions).unwrap());
        }
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>())
            .unwrap()
            .into_iter()
            .filter(|s| {
                [
                    "lpic2-auth-filter-detection",
                    "lpic2-auth-filter-exceptions",
                ]
                .contains(&s.id.as_str())
            })
            .collect()
    })
}
fn definition(s: &pack::Scenario) -> &dlms_learning_lab::auth_filter_workspace::Definition {
    let Goal::AuthFilter { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn act(state: &mut State, s: &pack::Scenario, a: &Action, practice: bool) {
    let raw = format!(
        "{}{}",
        binding::PREFIX,
        serde_json::json!({"practice":practice,"action":a})
    );
    engine::execute_for_goal(state, &s.goal, &raw).unwrap();
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
fn authored_methods_and_separate_worked_examples_reach_real_outcomes() {
    assert_eq!(labs().len(), 2);
    for s in labs() {
        let d = binding::source(s).unwrap();
        let mut state = s.initial.clone();
        let State::AuthFilter { simulation } = &state else {
            panic!()
        };
        let assignment = simulation.assignment.clone();
        assert!(!engine::solved(&state, &s.goal));
        for a in &d.worked {
            act(&mut state, s, a, true);
        }
        let State::AuthFilter { simulation } = &state else {
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
fn supported_alternatives_pass_without_exact_filter_string_matching() {
    let methods: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/auth-filter-methods.json")).unwrap();
    for s in labs() {
        let m = methods.iter().find(|m| m["id"] == s.id).unwrap();
        let d = definition(s);
        let alternate = m["alternate"].as_str().unwrap();
        assert_ne!(alternate, m["canonical"].as_str().unwrap());
        assert!(d.assignment.solved(alternate));
        let mut state = s.initial.clone();
        for a in [
            Action::Edit {
                text: alternate.into(),
            },
            Action::Save,
            Action::Run,
            Action::Check,
        ] {
            act(&mut state, s, &a, false);
        }
        assert!(engine::solved(&state, &s.goal));
    }
}
#[test]
fn partial_hardcoded_and_overbroad_approaches_fail_with_specific_feedback() {
    for s in labs() {
        let d = definition(s);
        assert!(d.assignment.cases.len() >= 3);
        let app = if s.id.ends_with("exceptions") {
            "gateway"
        } else {
            "vault"
        };
        let event = if s.id.ends_with("exceptions") {
            "denied"
        } else {
            "rejected"
        };
        let tail = if s.id.ends_with("exceptions") {
            " source=[a-z]+"
        } else {
            ""
        };
        let hardcoded = format!(
            "[Definition]\nfailregex = ^{app} {event} user=alex from <HOST> port [0-9]+{tail}$\n"
        );
        assert!(!d.assignment.solved(&hardcoded));
        let broad = format!(
            "[Definition]\nfailregex = ^{app} [a-z]+ user=[A-Za-z0-9_-]+ from <HOST> port [0-9]+{tail}$\n"
        );
        let checks = d.assignment.checks(&broad);
        assert!(
            checks
                .iter()
                .any(|c| !c.passed && c.detail.contains("non-failure"))
        );
    }
    let s = labs()
        .iter()
        .find(|s| s.id.ends_with("exceptions"))
        .unwrap();
    let methods: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("fixtures/auth-filter-methods.json")).unwrap();
    let full = methods.iter().find(|m| m["id"] == s.id).unwrap()["canonical"]
        .as_str()
        .unwrap();
    let partial = full
        .lines()
        .filter(|l| !l.starts_with("ignoreregex"))
        .collect::<Vec<_>>()
        .join("\n");
    let checks = definition(s).assignment.checks(&partial);
    assert!(checks.iter().any(|c| c.passed));
    assert!(
        checks
            .iter()
            .any(|c| !c.passed && c.detail.contains("narrow ignore rule"))
    );
    let ignore_all = format!("{partial}\nignoreregex = ^.*$\n");
    assert!(!definition(s).assignment.solved(&ignore_all));
}
#[test]
fn current_success_breaks_after_saved_or_unsaved_changes_and_reset_is_selected() {
    for s in labs() {
        let mut state = solve(s);
        let original = s.initial.clone();
        act(
            &mut state,
            s,
            &Action::Edit {
                text: "[Definition]\nfailregex = ^never <HOST>$\n".into(),
            },
            false,
        );
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, s, &Action::Save, false);
        act(&mut state, s, &Action::Check, false);
        assert!(!engine::solved(&state, &s.goal));
        assert_eq!(
            serde_json::to_value(&s.initial).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        let mut attempt = progress::Attempt::new(s, 1);
        attempt.state = state;
        assert!(!progress::current_solved(&attempt, s));
        attempt.state = s.initial.clone();
        binding::context(&attempt, s).unwrap();
        assert!(!progress::current_solved(&attempt, s));
    }
}
#[test]
fn invalid_source_task_or_internal_action_is_not_silently_accepted() {
    for s in labs() {
        let mut changed = s.clone();
        let Goal::AuthFilter { definition } = &mut changed.goal else {
            panic!()
        };
        definition.worked = vec![Action::Check];
        assert!(binding::source(&changed).is_err());
        let mut state = s.initial.clone();
        let before = serde_json::to_value(&state).unwrap();
        assert!(engine::execute_for_goal(&mut state,&s.goal,"auth-filter-workspace {\"practice\":false,\"action\":{\"kind\":\"ExecuteOnHost\",\"command\":\"id\"}}").is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
    }
}

#[test]
fn registration_preserves_all503_labs202_units_and_existing_teaching_exactly() {
    let prior = pack::accepted503().unwrap();
    let all = pack::accepted505().unwrap();
    assert_eq!((prior.len(), all.len()), (503, 505));
    assert_eq!(
        serde_json::to_value(&all[..503]).unwrap(),
        serde_json::to_value(&prior).unwrap()
    );
    let before = dlms_learning_lab::course::accepted503(&prior).unwrap();
    let current = dlms_learning_lab::course::accepted505(&all).unwrap();
    assert_eq!((before.len(), current.len()), (202, 203));
    let retained: Vec<_> = current
        .iter()
        .filter(|u| u.id != "linux-auth-log-filters")
        .collect();
    assert_eq!(
        serde_json::to_value(&retained).unwrap(),
        serde_json::to_value(&before).unwrap()
    );
    let unit = current
        .iter()
        .find(|u| u.id == "linux-auth-log-filters")
        .unwrap();
    assert_eq!(
        unit.labs,
        [
            "lpic2-auth-filter-detection",
            "lpic2-auth-filter-exceptions"
        ]
    );
    for id in &unit.prerequisites {
        assert!(
            current.iter().position(|u| u.id == *id).unwrap()
                < current.iter().position(|u| u.id == unit.id).unwrap()
        );
    }
    assert_eq!(
        ["CCNA", "LPIC-1", "LPIC-2", "Bash/Shell"]
            .map(|t| all.iter().filter(|s| s.track == t).count()),
        [113, 154, 158, 80]
    );
    let entries: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../scenarios/presentation/lpic2.json")).unwrap();
    assert_eq!(entries.len(), 160);
    let mut prefix = serde_json::to_value(&entries[..156]).unwrap();
    fn sort(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Array(a) => {
                for v in a {
                    sort(v)
                }
            }
            serde_json::Value::Object(m) => {
                let ordered: std::collections::BTreeMap<_, _> =
                    std::mem::take(m).into_iter().collect();
                for (k, mut v) in ordered {
                    sort(&mut v);
                    m.insert(k, v);
                }
            }
            _ => {}
        }
    }
    sort(&mut prefix);
    assert_eq!(
        dlms_learning_lab::developer_digest::hash(&serde_json::to_vec(&prefix).unwrap()),
        "4680e71ab4a0ddae4a081e6b8d05eb990e303c7c13af59d23318f8d0ceef0c77"
    );
    for s in labs() {
        let e = entries.iter().find(|e| e["id"] == s.id).unwrap();
        assert_eq!(e["practice"].as_array().unwrap().len(), 3);
        assert!(e["concepts"].as_array().unwrap().len() >= 4);
        assert!(e["worked"].as_array().unwrap().len() >= 4);
        assert_eq!(e["requirements"].as_array().unwrap().len(), 4);
        for screen in ["introduction", "example", "assignment"] {
            assert!(e["review"][screen].as_str().unwrap().len() > 40);
        }
    }
}
