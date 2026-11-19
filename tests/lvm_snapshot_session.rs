use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{
        self as work, Session,
        snapshot_flow::{self as flow, Operation},
    },
    pack,
    progress::{Attempt, Progress},
    script_teaching::View,
};
#[path = "support/lvm_snapshot_flow_fixture.rs"]
mod fixture;
use fixture::{command, definition};
use std::sync::OnceLock;
fn source() -> pack::Scenario {
    static BASE: OnceLock<pack::Scenario> = OnceLock::new();
    let mut s = BASE
        .get_or_init(|| {
            pack::bundled()
                .unwrap()
                .into_iter()
                .find(|s| s.id == "lpic2-lvm-free-space")
                .unwrap()
        })
        .clone();
    let d = definition();
    s.id = "test-only-snapshot-binding".into();
    s.initial = State::Lvm {
        simulation: Box::new(Session::new(&d).unwrap()),
    };
    s.solution = d
        .snapshot_task
        .as_ref()
        .unwrap()
        .reference
        .iter()
        .map(|op| flow::encode(op).unwrap())
        .collect();
    s.goal = Goal::Lvm {
        definition: Box::new(d),
    };
    s
}
fn d(s: &pack::Scenario) -> &work::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(a: &mut Attempt) -> &mut Session {
    let State::Lvm { simulation } = &mut a.state else {
        panic!()
    };
    simulation
}
#[test]
fn explicit_profile_requires_complete_separate_typed_source_and_teaching() {
    let def = definition();
    def.validate().unwrap();
    let mut bad = def.clone();
    bad.snapshot_task = None;
    assert!(bad.shape().is_err());
    let mut bad = def.clone();
    bad.lesson.snapshot_practice = None;
    assert!(bad.shape().is_err());
    let mut bad = def.clone();
    bad.lesson.fixture = def.initial.clone();
    assert!(bad.shape().is_err());
    let mut bad = def.clone();
    bad.lesson.worked.push(work::Command {
        text: "lvs".into(),
        explanation: "Inspect".into(),
    });
    assert!(bad.shape().is_err());
    let mut bad = def.clone();
    bad.lesson.snapshot_practice.as_mut().unwrap().worked[0]
        .explanation
        .clear();
    assert!(bad.shape().is_err());
    let mut bad = def.clone();
    bad.lesson
        .snapshot_practice
        .as_mut()
        .unwrap()
        .worked
        .swap(0, 1);
    assert!(bad.validate().is_err());
    let mut bad = def.clone();
    bad.snapshot_task.as_mut().unwrap().reference.swap(0, 1);
    assert!(bad.validate().is_err());
}
#[test]
fn practice_command_insertion_is_inert_helpers_are_explicit_and_assignment_stays_unsolved() {
    let def = definition();
    let mut s = Session::new(&def).unwrap();
    let before = s.clone();
    s.insert(&def, 0).unwrap();
    assert_eq!(s.example, before.example);
    assert_eq!(s.current, before.current);
    assert!(!s.example_command.is_empty());
    let saved = s.clone();
    assert!(s.insert(&def, 1).is_err());
    assert_eq!(s, saved);
    let text = s.example_command.clone();
    s.execute(&def, &text, true).unwrap();
    let p = def.lesson.snapshot_practice.as_ref().unwrap();
    s.apply_snapshot(&def, &p.worked[1].operation, true)
        .unwrap();
    assert!(def.practice_checks(&s).iter().all(|r| r.0));
    assert!(def.checks(&s).iter().any(|r| !r.0));
    assert_eq!(s.current, def.initial);
    s.example_checked = true;
    s.checked = true;
    s.apply_snapshot(&def, &p.experiment, true).unwrap();
    assert!(!s.example_checked && s.checked);
    assert!(def.practice_checks(&s).iter().any(|r| !r.0));
    for op in &p.repair {
        s.apply_snapshot(&def, op, true).unwrap();
    }
    assert!(def.practice_checks(&s).iter().all(|r| r.0));
    s.reset_example(&def).unwrap();
    assert_eq!(s.example, def.lesson.fixture);
    assert_eq!(s.current, def.initial);
    assert!(s.example_transcript.is_empty());
}
#[test]
fn source_bound_current_grading_save_resume_invalidation_and_selected_reset_remain_honest() {
    let source = source();
    work::source(&source).unwrap();
    let def = d(&source);
    let mut a = Attempt::new(&source, 0);
    work::context(&a, &source).unwrap();
    session(&mut a)
        .set_command(def, "lvs".into(), false)
        .unwrap();
    session(&mut a).set_view(def, View::Assignment).unwrap();
    for text in &source.solution {
        engine::execute_for_goal(&mut a.state, &source.goal, text).unwrap();
    }
    session(&mut a).checked = true;
    assert!(engine::solved(&a.state, &source.goal));
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(source.id.clone(), a.clone());
    p.favorites.insert(source.id.clone());
    let bytes = serde_json::to_vec(&p).unwrap();
    let p = dlms_learning_lab::progress::decode(&bytes, std::slice::from_ref(&source)).unwrap();
    let mut restored = p.attempts[&source.id].clone();
    assert!(p.favorites.contains(&source.id));
    work::context(&restored, &source).unwrap();
    assert!(engine::solved(&restored.state, &source.goal));
    assert_eq!(session(&mut restored).command, "lvs");
    assert_eq!(session(&mut restored).view, View::Assignment);
    session(&mut restored)
        .apply_snapshot(
            def,
            &command("lvcreate -s -L 1M -n extra -c 128 work/report"),
            false,
        )
        .unwrap();
    assert!(!session(&mut restored).checked);
    assert!(!engine::solved(&restored.state, &source.goal));
    assert!(session(&mut restored).transcript.len() > session(&mut a).transcript.len());
    session(&mut restored)
        .apply_snapshot(def, &command("lvremove -y work/extra"), false)
        .unwrap();
    assert!(engine::solved(&restored.state, &source.goal));
    let reset = Attempt::new(&source, 1);
    assert!(!engine::solved(&reset.state, &source.goal));
    assert!(reset.transcript.is_empty());
    assert_eq!(
        serde_json::to_value(&a.state).unwrap(),
        serde_json::to_value(&p.attempts[&source.id].state).unwrap()
    );
}
#[test]
fn forged_checks_foreign_source_and_nested_helper_packets_never_award_or_mutate() {
    let source = source();
    let def = d(&source);
    let mut a = Attempt::new(&source, 0);
    session(&mut a).checked = true;
    assert!(!engine::solved(&a.state, &source.goal));
    for op in [
        Operation::ApplyUpdate { index: 9 },
        Operation::CopyRecord {
            origin: "sample/report".into(),
            snapshot: "saved".into(),
            record: "row0".into(),
        },
        command("sh -c lvs"),
    ] {
        let saved = a.clone();
        assert!(session(&mut a).apply_snapshot(def, &op, false).is_err());
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&saved).unwrap()
        );
    }
    let saved = a.clone();
    let text = format!(
        "{}{{\"kind\":\"apply_update\",\"index\":0,\"host\":\"sh\"}}",
        flow::PREFIX
    );
    assert!(session(&mut a).execute(def, &text, false).is_err());
    assert_eq!(
        serde_json::to_value(&a).unwrap(),
        serde_json::to_value(&saved).unwrap()
    );
    let mut wrong = source.clone();
    let Goal::Lvm { definition } = &mut wrong.goal else {
        panic!()
    };
    definition.initial = definition.lesson.fixture.clone();
    assert!(work::context(&a, &wrong).is_err());
}
#[test]
fn old_definitions_wire_session_and_untyped_goal_guard_are_preserved() {
    let sources = pack::accepted323().unwrap();
    for source in sources
        .iter()
        .filter(|s| matches!(s.goal, Goal::Lvm { .. }))
    {
        let def = d(source);
        assert!(def.snapshot_task.is_none());
        assert!(def.lesson.snapshot_practice.is_none());
        let value = serde_json::to_value(def).unwrap();
        assert!(value.get("snapshot_task").is_none());
        assert!(value["lesson"].get("snapshot_practice").is_none());
        let mut attempt = Attempt::new(source, 0);
        work::context(&attempt, source).unwrap();
        for text in &source.solution {
            engine::execute_for_goal(&mut attempt.state, &source.goal, text).unwrap();
        }
        assert!(engine::solved(&attempt.state, &source.goal));
    }
    let def = definition();
    assert!(!work::same_problem(&def.initial, &def.initial));
}
#[test]
fn bounded_transcripts_roundtrip_and_malformed_resume_fail_without_silently_repairing() {
    let def = definition();
    let mut s = Session::new(&def).unwrap();
    for _ in 0..40 {
        s.execute(&def, "lvs", false).unwrap();
    }
    assert_eq!(s.transcript.len(), 32);
    let value = serde_json::to_value(&s).unwrap();
    let restored: Session = serde_json::from_value(value.clone()).unwrap();
    def.bound(&restored).unwrap();
    assert_eq!(restored, s);
    let mut value = value;
    value["current"]["snapshots"] = serde_json::Value::Null;
    let wrong: Session = serde_json::from_value(value).unwrap();
    assert!(def.bound(&wrong).is_err());
    let before = s.clone();
    assert!(s.select_example(&def, 16).is_err());
    assert_eq!(s, before);
    s.select_example(&def, 1).unwrap();
    assert_eq!(s.selected_example, 1);
}
