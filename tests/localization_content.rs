//! Real authored proposals; source registration/native acceptance are separate.
use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    localization_workspace::{self as work, Action},
    pack, progress,
};
use std::sync::OnceLock;
const IDS: [&str; 2] = [
    "lpic1-locale-report-handover",
    "lpic1-encoding-import-repair",
];
fn authored() -> Vec<pack::Scenario> {
    static DATA: OnceLock<Vec<pack::Scenario>> = OnceLock::new();
    DATA.get_or_init(|| {
        let raw: pack::Pack =
            serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v22.json")).unwrap();
        let mut all = pack::accepted415().unwrap();
        assert_eq!(all.len(), 415);
        all.extend(
            raw.scenarios
                .into_iter()
                .filter(|s| IDS.contains(&s.id.as_str())),
        );
        let texts = all
            .chunks(8)
            .map(|labs| {
                serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":labs}))
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
    let Goal::Localization { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut progress::Attempt) -> &mut work::Session {
    let State::Localization { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
fn action(a: &mut progress::Attempt, s: &pack::Scenario, step: Action) {
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &format!("{}{}", work::PREFIX, serde_json::to_string(&step).unwrap()),
    )
    .unwrap();
}
fn checked(a: &mut progress::Attempt, s: &pack::Scenario) {
    engine::execute_for_goal(&mut a.state, &s.goal, "@locale-check").unwrap();
}
#[test]
fn each_actual_taught_method_and_different_example_reaches_behavior_without_ritual_or_example_credit()
 {
    for s in authored() {
        let d = definition(&s);
        d.validate().unwrap();
        let mut a = progress::Attempt::new(&s, 1);
        for step in &d.example.reference {
            session(&mut a)
                .practice
                .perform(&d.example, step.clone())
                .unwrap()
                .unwrap();
        }
        assert!(
            session(&mut a)
                .practice
                .check(&d.example)
                .unwrap()
                .iter()
                .all(|r| r.correct)
        );
        assert!(!progress::current_solved(&a, &s));
        assert_eq!(session(&mut a).assignment, work::Draft::new(&d.initial));
        for raw in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
        assert_eq!(a.step, 0);
    }
}
#[test]
fn equivalent_effective_categories_and_normal_iconv_flag_variations_pass() {
    let labs = authored();
    let s = &labs[0];
    let mut a = progress::Attempt::new(s, 1);
    action(
        &mut a,
        s,
        Action::SaveLocale {
            text: "LANG=C.UTF-8\nLC_MESSAGES=de_DE.UTF-8\n".into(),
        },
    );
    action(&mut a, s, Action::NewSession {});
    checked(&mut a, s);
    assert!(progress::current_solved(&a, s));
    let s = &labs[1];
    let mut a = progress::Attempt::new(s, 1);
    for raw in [
        "iconv /work/roster-latin1.txt -tUTF8 -fLATIN1 -o/work/roster-utf8.txt",
        "iconv --from-code=US-ASCII --to-code UTF-8 --output=/work/counts-utf8.txt /work/counts-ascii.txt",
        "iconv -fUTF8 -tUTF8 -o/work/notes-copy.txt /work/notes-utf8.txt",
    ] {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    checked(&mut a, s);
    assert!(progress::current_solved(&a, s));
}
#[test]
fn partial_saved_only_work_blank_outputs_wrong_characters_and_rebroken_success_fail_specifically() {
    let labs = authored();
    let s = &labs[0];
    let mut a = progress::Attempt::new(s, 1);
    action(
        &mut a,
        s,
        Action::SaveLocale {
            text: "LANG=de_DE.UTF-8\nLC_NUMERIC=C\n".into(),
        },
    );
    checked(&mut a, s);
    let rows = session(&mut a)
        .assignment
        .checks(&definition(s).initial)
        .unwrap();
    assert_eq!(rows.iter().filter(|r| r.correct).count(), 2);
    assert!(!progress::current_solved(&a, s));
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    engine::execute_for_goal(&mut a.state, &s.goal, "export LC_ALL=C").unwrap();
    assert!(!progress::current_solved(&a, s));
    let s = &labs[1];
    let mut a = progress::Attempt::new(s, 1);
    checked(&mut a, s);
    let rows = session(&mut a)
        .assignment
        .checks(&definition(s).initial)
        .unwrap();
    assert!(
        rows.iter()
            .all(|r| !r.correct && r.feedback.contains("No output was saved"))
    );
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    assert!(progress::current_solved(&a, s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "iconv -f LATIN1 -t UTF8 -o /work/notes-copy.txt /work/notes-utf8.txt",
    )
    .unwrap();
    checked(&mut a, s);
    let rows = session(&mut a)
        .assignment
        .checks(&definition(s).initial)
        .unwrap();
    assert_eq!(rows.iter().filter(|r| !r.correct).count(), 1);
    assert!(rows[2].feedback.contains("valid UTF-8"));
    assert!(!progress::current_solved(&a, s));
}
#[test]
fn negative_codec_status_writes_a_real_prefix_and_retry_repairs_without_changing_the_original() {
    let s = authored().remove(1);
    let mut a = progress::Attempt::new(&s, 1);
    for raw in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, raw).unwrap();
    }
    let output = engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "iconv -f LATIN1 -t ASCII -o /work/roster-utf8.txt /work/roster-latin1.txt",
    )
    .unwrap();
    assert!(output.starts_with("Exit status 1\n"));
    let world = session(&mut a)
        .assignment
        .world(&definition(&s).initial)
        .unwrap();
    assert_eq!(world.files().read("roster-utf8.txt").unwrap(), b"Caf");
    assert_eq!(
        world.files().read("roster-latin1.txt").unwrap(),
        b"Caf\xe9;ni\xf1o\n"
    );
    assert!(!progress::current_solved(&a, &s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        "iconv -f LATIN1 -t UTF8 -o /work/roster-utf8.txt /work/roster-latin1.txt",
    )
    .unwrap();
    checked(&mut a, &s);
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn old415_scenario_values_and159_units_are_retained_and_proposed_prerequisites_are_reachable() {
    let before: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v21.json")).unwrap();
    let next: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v22.json")).unwrap();
    assert_eq!(before["scenarios"].as_array().unwrap().len(), 42);
    assert_eq!(
        &next["scenarios"].as_array().unwrap()[..42],
        before["scenarios"].as_array().unwrap()
    );
    let old = pack::accepted415().unwrap();
    let units = course::accepted415(&old).unwrap();
    assert_eq!((old.len(), units.len()), (415, 159));
    let current = pack::current().unwrap();
    assert_eq!(current.len(), 424);
    for old in &old {
        let actual = current.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(actual).unwrap()
        );
    }
    let mut all = old;
    all.extend(authored());
    let mut index = serde_json::json!({"version":1,"units":units});
    let mut extra: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/localization-units-v1.json")).unwrap();
    let target = index["units"].as_array_mut().unwrap();
    let after = target
        .iter()
        .position(|u| u["id"] == "linux-cron-foundations")
        .unwrap();
    target.insert(after + 1, extra["units"].as_array_mut().unwrap().remove(0));
    let proposed = course::load(&serde_json::to_string(&index).unwrap(), &all).unwrap();
    assert_eq!(proposed.len(), 160);
    assert_eq!(proposed[after + 1].id, "linux-localization-reports");
    for original in units {
        assert_eq!(
            serde_json::to_value(proposed.iter().find(|u| u.id == original.id).unwrap()).unwrap(),
            serde_json::to_value(original).unwrap()
        );
    }
}
