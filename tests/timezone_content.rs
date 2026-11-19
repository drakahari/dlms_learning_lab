//! Actual authored proposals; integration/native acceptance remains separate.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack, progress, timezone_workspace as work,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic1-timezone-system-display",
    "lpic1-timezone-process-override",
];
fn authored() -> Vec<pack::Scenario> {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v23.json")).unwrap();
        assert_eq!(
            raw.scenarios[raw.scenarios.len() - 2..]
                .iter()
                .map(|s| s.id.as_str())
                .collect::<Vec<_>>(),
            IDS
        );
        let mut all = pack::accepted417().unwrap();
        assert_eq!(all.len(), 417);
        all.extend(
            raw.scenarios
                .into_iter()
                .filter(|s| IDS.contains(&s.id.as_str())),
        );
        let texts = all
            .chunks(8)
            .map(|rows| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":rows}))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let refs = texts.iter().map(String::as_str).collect::<Vec<_>>();
        pack::load_catalog(&refs)
            .unwrap()
            .into_iter()
            .filter(|s| IDS.contains(&s.id.as_str()))
            .collect()
    })
    .clone()
}
fn definition(s: &pack::Scenario) -> &work::Definition {
    let Goal::Timezone { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut progress::Attempt) -> &mut work::Session {
    let State::Timezone { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn execute(a: &mut progress::Attempt, s: &pack::Scenario, text: &str) {
    engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
}
#[test]
fn both_real_taught_methods_and_different_examples_reach_actual_policy_outcomes() {
    for s in authored() {
        let d = definition(&s);
        d.validate().unwrap();
        let mut a = progress::Attempt::new(&s, 1);
        for c in &d.example.reference {
            let draft = &mut session(&mut a).practice;
            draft.input = c.text.clone();
            draft.run(&d.example).unwrap().unwrap();
        }
        assert!(
            session(&mut a)
                .practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(|r| r.correct)
        );
        assert_eq!(session(&mut a).assignment, work::Draft::new(&d.initial));
        assert!(!progress::current_solved(&a, &s));
        for c in &s.solution {
            execute(&mut a, &s, c)
        }
        assert!(progress::current_solved(&a, &s));
        assert_eq!(a.step, 0);
        assert_eq!(
            session(&mut a)
                .assignment
                .world(&d.initial)
                .unwrap()
                .clock(),
            d.initial.initial.clock
        );
    }
}
#[test]
fn alternate_supported_literal_forms_and_nonritual_repairs_pass() {
    let labs = authored();
    for (s, raw) in [
        (&labs[0], "timedatectl set-timezone 'Europe/Berlin'"),
        (&labs[1], "unset 'TZ'"),
    ] {
        let mut a = progress::Attempt::new(s, 1);
        execute(&mut a, s, raw);
        execute(&mut a, s, "@timezone-check");
        assert!(progress::current_solved(&a, s));
        // Investigation commands are useful, not grading rituals.
        assert_eq!(session(&mut a).assignment.history.len(), 1);
    }
}
#[test]
fn plausible_process_only_empty_pinned_and_printed_success_shortcuts_fail_specifically() {
    let labs = authored();
    let s = &labs[0];
    let d = definition(s);
    let mut a = progress::Attempt::new(s, 1);
    execute(&mut a, s, "export TZ=Europe/Berlin");
    execute(&mut a, s, "@timezone-check");
    assert!(!progress::current_solved(&a, s));
    let rows = session(&mut a).assignment.checks(&d.initial).unwrap();
    assert!(
        rows.iter()
            .any(|r| r.correct && r.label.starts_with("Current process"))
    );
    assert!(
        rows.iter()
            .any(|r| !r.correct && r.label == "Configured system zone")
    );
    assert!(
        rows.iter()
            .filter(|r| !r.correct)
            .all(|r| !r.feedback.is_empty())
    );
    let s = &labs[1];
    let d = definition(s);
    for raw in [
        "export TZ=",
        "export TZ=Europe/Berlin",
        "date -u '+%F %T %z %Z'",
        "date '+all requirements pass'",
    ] {
        let mut a = progress::Attempt::new(s, 1);
        execute(&mut a, s, raw);
        execute(&mut a, s, "@timezone-check");
        assert!(!progress::current_solved(&a, s), "{raw}");
        let rows = session(&mut a).assignment.checks(&d.initial).unwrap();
        assert!(
            rows.iter()
                .any(|r| !r.correct && r.label.starts_with("After the supplied zone handover")),
            "{raw}"
        );
    }
}
#[test]
fn after_success_state_changes_invalidate_but_harmless_views_do_not_break_correct_work() {
    for s in authored() {
        let mut a = progress::Attempt::new(&s, 1);
        for c in &s.solution {
            execute(&mut a, &s, c)
        }
        assert!(progress::current_solved(&a, &s));
        execute(&mut a, &s, "date -u +%s");
        assert!(progress::current_solved(&a, &s));
        execute(&mut a, &s, "export TZ=UTC");
        assert!(!progress::current_solved(&a, &s));
        assert!(!session(&mut a).assignment.checked && !session(&mut a).assignment.attempted);
        execute(&mut a, &s, "@timezone-check");
        assert!(!progress::current_solved(&a, &s));
        let before = a.state.clone();
        assert!(engine::execute_for_goal(&mut a.state, &s.goal, "sh repair.sh").is_err());
        assert_eq!(a.state, before);
    }
}
#[test]
fn retained417_values_160_units_and_proposed_prerequisites_are_not_rewritten() {
    let old: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v22.json")).unwrap();
    let new: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v23.json")).unwrap();
    assert_eq!((old.scenarios.len(), new.scenarios.len()), (44, 46));
    assert_eq!(
        serde_json::to_value(&new.scenarios[..44]).unwrap(),
        serde_json::to_value(&old.scenarios).unwrap()
    );
    let old = pack::accepted417().unwrap();
    let old_units = course::accepted417(&old).unwrap();
    assert_eq!((old.len(), old_units.len()), (417, 160));
    let mut all = old.clone();
    all.extend(authored());
    assert_eq!(all.len(), 419);
    let raw = serde_json::json!({"version":1,"units":[{
        "id":"linux-timezone-handovers","track":"LPIC-1","title":"Time zones and process handovers",
        "summary":"Repair regional defaults and process overrides.","prerequisites":["linux-localization-reports"],"labs":IDS
    }]});
    let proposed = course::load(&serde_json::to_string(&raw).unwrap(), &all);
    // A standalone index cannot satisfy external prerequisites; combine the
    // actual160 predecessor values before checking the complete graph.
    assert!(proposed.is_err());
    let mut value = serde_json::to_value(&old_units).unwrap();
    value.as_array_mut().unwrap().push(raw["units"][0].clone());
    let units = course::load(
        &serde_json::to_string(&serde_json::json!({"version":1,"units":value})).unwrap(),
        &all,
    )
    .unwrap();
    assert_eq!(units.len(), 161);
    for prior in &old_units {
        let now = units.iter().find(|u| u.id == prior.id).unwrap();
        assert_eq!(
            serde_json::to_value(now).unwrap(),
            serde_json::to_value(prior).unwrap()
        );
    }
    // Explicit integration admits both validated proposals; native teaching review remains separate.
    let current = pack::current().unwrap();
    assert_eq!(current.len(), 424);
    let actual_units = course::current(&current).unwrap();
    assert_eq!(actual_units.len(), 163);
    assert!(IDS.iter().all(|id| current.iter().any(|s| s.id == *id)));
}
