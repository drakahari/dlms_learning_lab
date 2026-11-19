//! Source/save/draft integration. All accounts and homes remain simulated.
#[path = "support/accounts_template_fixture.rs"]
mod fixture;
use dlms_learning_lab::{
    accounts_template_workspace as work,
    engine::{self, State},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn fresh(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    work::prepare(&mut a, s, true).unwrap();
    a
}
fn edit(a: &mut Attempt, body: &str, mode: &str) {
    let d = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_template_drafts
        .as_mut()
        .unwrap();
    d.selected = fixture::ASSIGNMENT_PATH.into();
    let row = d.leaves.get_mut(fixture::ASSIGNMENT_PATH).unwrap();
    row.contents = body.into();
    row.mode = mode.into();
}
fn save(a: &mut Attempt, s: &pack::Scenario) -> Result<(), String> {
    let initial = work::source(s)?.unwrap().initial;
    let State::Accounts { simulation } = &mut a.state else {
        panic!()
    };
    a.workspace
        .as_mut()
        .unwrap()
        .accounts_template_drafts
        .as_mut()
        .unwrap()
        .save_selected(initial, simulation)
}
fn finish(a: &mut Attempt, s: &pack::Scenario) {
    edit(a, fixture::ASSIGNMENT_TEXT, "640");
    save(a, s).unwrap();
    for cmd in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
    }
    assert!(progress::current_solved(a, s));
}
fn clone_p(p: &Progress) -> Progress {
    serde_json::from_slice(&serde_json::to_vec(p).unwrap()).unwrap()
}
#[test]
fn native_reference_runs_only_in_pack_gate_and_new_attempt_has_original_unsolved_values() {
    let s = fixture::scenario();
    let mut bare = Attempt::new(&s, 0);
    work::prepare(&mut bare, &s, true).unwrap();
    assert_eq!(bare.workspace.as_ref().unwrap().version, 1);
    work::context(&bare, &s).unwrap();
    let mut a = Attempt::new(&s, 1);
    let state = serde_json::to_value(&a.state).unwrap();
    workspace::prepare(&mut a);
    work::prepare(&mut a, &s, true).unwrap();
    assert_eq!(serde_json::to_value(&a.state).unwrap(), state);
    work::context(&a, &s).unwrap();
    assert!(!progress::current_solved(&a, &s));
    let row = &a
        .workspace
        .as_ref()
        .unwrap()
        .accounts_template_drafts
        .as_ref()
        .unwrap()
        .leaves[fixture::ASSIGNMENT_PATH];
    assert_ne!(row.contents, fixture::ASSIGNMENT_TEXT);
    for command in &s.solution {
        assert!(dlms_learning_lab::accounts::parse(command).is_ok());
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    assert!(
        !engine::solved(&a.state, &s.goal),
        "real account commands cannot replace the required native template repair"
    );
}
#[test]
fn prepare_refusal_is_atomic_even_for_new_ipv4_or_invalid_workspace_metadata() {
    let s = fixture::scenario();
    for kind in 0..4 {
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        match kind {
            0 => {
                a.ipv4 = Some(dlms_learning_lab::ipv4::Lesson::from_legacy(
                    std::net::Ipv4Addr::new(192, 0, 2, 77),
                    27,
                    0,
                ))
            }
            1 => a.workspace.as_mut().unwrap().version = 9,
            2 => a.scenario_version += 1,
            _ => a.workspace.as_mut().unwrap().service_port = Some("foreign service draft".into()),
        };
        let before = serde_json::to_value(&a).unwrap();
        assert!(work::prepare(&mut a, &s, true).is_err());
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            before,
            "never publish fields before checking the complete candidate"
        );
    }
}
#[test]
fn missing_resumed_or_one_sided_metadata_is_not_backfilled_or_solved() {
    let s = fixture::scenario();
    for kind in 0..3 {
        let mut a = fresh(&s);
        match kind {
            0 => {
                let w = a.workspace.as_mut().unwrap();
                w.accounts_template_drafts = None;
                w.accounts_learning = None;
            }
            1 => a.workspace.as_mut().unwrap().accounts_template_drafts = None,
            _ => a.workspace.as_mut().unwrap().accounts_learning = None,
        };
        let before = serde_json::to_value(&a).unwrap();
        assert!(work::prepare(&mut a, &s, false).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn real_native_save_then_useradd_is_partial_until_all_outcomes_and_pending_edits_clear_credit() {
    let s = fixture::scenario();
    let mut a = fresh(&s);
    edit(&mut a, fixture::ASSIGNMENT_TEXT, "640");
    assert!(!work::ready(&a, &s));
    save(&mut a, &s).unwrap();
    assert!(work::ready(&a, &s));
    let rows = engine::checks(&a.state, &s.goal);
    assert!(rows.iter().any(|r| r.0) && rows.iter().any(|r| !r.0));
    assert!(!progress::current_solved(&a, &s));
    for cmd in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, cmd).unwrap();
    }
    assert!(progress::current_solved(&a, &s));
    let applied = serde_json::to_value(&a.state).unwrap();
    edit(&mut a, "Pending different default\n", "08");
    assert!(engine::solved(&a.state, &s.goal));
    assert!(!progress::current_solved(&a, &s));
    let draft = serde_json::to_value(&a.workspace).unwrap();
    assert!(save(&mut a, &s).is_err());
    assert_eq!(serde_json::to_value(&a.state).unwrap(), applied);
    assert_eq!(serde_json::to_value(&a.workspace).unwrap(), draft);
    edit(&mut a, fixture::ASSIGNMENT_TEXT, "0640");
    save(&mut a, &s).unwrap();
    assert!(progress::current_solved(&a, &s));
    edit(&mut a, fixture::ASSIGNMENT_TEXT, "640");
    assert!(
        progress::current_solved(&a, &s),
        "equivalent octal spelling is not a broken configuration"
    );
}
#[test]
fn invalid_unsaved_work_roundtrips_separately_from_applied_state_and_other_progress() {
    let s = fixture::scenario();
    let old = pack::load(include_str!("../scenarios/accounts-lifecycle-v1.json"))
        .unwrap()
        .remove(0);
    let catalog = vec![s.clone(), old.clone()];
    let mut p = Progress {
        schema_version: 1,
        clock: 3,
        ..Default::default()
    };
    p.favorites.insert(s.id.clone());
    p.attempts.insert(old.id.clone(), Attempt::new(&old, 1));
    p.attempts.insert(s.id.clone(), fresh(&s));
    let untouched = serde_json::to_value(&p.attempts[&old.id]).unwrap();
    let a = p.attempts.get_mut(&s.id).unwrap();
    finish(a, &s);
    edit(a, "Unsaved\0new default\n", "08");
    let learning = a
        .workspace
        .as_mut()
        .unwrap()
        .accounts_learning
        .as_mut()
        .unwrap();
    let d = learning.template_drafts.as_mut().unwrap();
    d.selected = fixture::EXAMPLE_PATH.into();
    d.leaves.get_mut(fixture::EXAMPLE_PATH).unwrap().contents =
        "Separate unfinished gallery draft\n".into();
    let before = serde_json::to_value(&p).unwrap();
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let mut restored = progress::decode(&std::fs::read(&path).unwrap(), &catalog).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), before);
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    let a = restored.attempts.get_mut(&s.id).unwrap();
    let saved = serde_json::to_value(&a.workspace).unwrap();
    work::prepare(a, &s, false).unwrap();
    assert_eq!(serde_json::to_value(&a.workspace).unwrap(), saved);
    edit(a, fixture::ASSIGNMENT_TEXT, "640");
    save(a, &s).unwrap();
    assert!(progress::current_solved(a, &s));
    restored.attempts.insert(s.id.clone(), fresh(&s));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&old.id]).unwrap(),
        untouched
    );
    assert_eq!(restored.favorites, p.favorites);
}
#[test]
fn catalog_save_rejects_erasure_forged_rights_wrong_practice_family_and_version_before_bytes_change()
 {
    let s = fixture::scenario();
    let catalog = vec![s.clone()];
    let mut p = Progress {
        schema_version: 1,
        clock: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), fresh(&s));
    finish(p.attempts.get_mut(&s.id).unwrap(), &s);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &catalog).unwrap();
    let original = std::fs::read(&path).unwrap();
    for kind in 0..9 {
        let mut bad = clone_p(&p);
        let a = bad.attempts.get_mut(&s.id).unwrap();
        match kind {
            0 => a.workspace.as_mut().unwrap().accounts_template_drafts = None,
            1 => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .accounts_learning
                    .as_mut()
                    .unwrap()
                    .template_drafts = None
            }
            2 => {
                let State::Accounts { simulation } = &mut a.state else {
                    panic!()
                };
                let mut policy = simulation.policy().clone();
                policy.editable_skeleton_leaves.clear();
                *simulation = dlms_learning_lab::accounts::State::new(
                    policy,
                    simulation.configuration().clone(),
                )
                .unwrap();
            }
            3 => {
                let State::Accounts { simulation } = &a.state else {
                    panic!()
                };
                a.workspace
                    .as_mut()
                    .unwrap()
                    .accounts_learning
                    .as_mut()
                    .unwrap()
                    .template_drafts
                    .as_mut()
                    .unwrap()
                    .source = simulation.clone();
            }
            4 => a.scenario_version += 1,
            5 => {
                a.workspace
                    .as_mut()
                    .unwrap()
                    .accounts_template_drafts
                    .as_mut()
                    .unwrap()
                    .selected = "/etc/foreign".into();
            }
            6 => {
                let State::Accounts { simulation } = &mut a.state else {
                    panic!()
                };
                let mut policy = simulation.policy().clone();
                policy
                    .editable_skeleton_leaves
                    .get_mut(fixture::ASSIGNMENT_PATH)
                    .unwrap()
                    .allow_remove = true;
                *simulation = dlms_learning_lab::accounts::State::new(
                    policy,
                    simulation.configuration().clone(),
                )
                .unwrap();
            }
            7 => {
                a.state = pack::load(include_str!("../scenarios/starter-v1.json"))
                    .unwrap()
                    .remove(0)
                    .initial;
            }
            _ => {
                let State::Accounts { simulation } = &mut a.state else {
                    panic!()
                };
                let mut policy = simulation.policy().clone();
                policy.editable_skeleton_leaves.clear();
                *simulation = dlms_learning_lab::accounts::State::new(
                    policy,
                    simulation.configuration().clone(),
                )
                .unwrap();
                let w = a.workspace.as_mut().unwrap();
                w.accounts_template_drafts = None;
                w.accounts_learning = None;
            }
        };
        assert!(work::context(a, &s).is_err(), "kind{kind}");
        assert!(!progress::current_solved(a, &s));
        assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &catalog).is_err());
        assert!(progress::save_with_catalog(&path, &bad, &catalog).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), original);
    }
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), original);
}
#[test]
fn source_native_annotations_are_required_and_unlisted_reference_edits_are_refused() {
    for kind in 0..3 {
        let mut s = fixture::value();
        match kind {
            0 => s["accounts_teaching"]["template_teaching"] = serde_json::Value::Null,
            1 => {
                s["accounts_teaching"]["template_teaching"]["assignment"][0]["path"] =
                    serde_json::json!("/etc/foreign")
            }
            _ => {
                s["accounts_teaching"]["template_teaching"]["example"][0]["path"] =
                    serde_json::json!("/etc/foreign")
            }
        };
        assert!(
            pack::load(
                &serde_json::to_string(&serde_json::json!({"schema_version":1,"scenarios":[s]}))
                    .unwrap()
            )
            .is_err()
        );
    }
}
