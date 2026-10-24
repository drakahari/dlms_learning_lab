//! Unbundled context/persistence integration; no native, host service or curriculum claim.
use dlms_learning_lab::{
    backup,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell,
    unit_manager::{self as core, Action},
    unit_manager_teaching,
    unit_manager_workspace::{self as model, Draft, EditAction},
    workspace,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, AtomicUsize},
};

fn scenario() -> Scenario {
    pack::load(include_str!("fixtures/unit-manager-pack-preview.json"))
        .unwrap()
        .remove(0)
}
fn catalog() -> Vec<Scenario> {
    let mut labs = pack::bundled().unwrap();
    labs.push(scenario());
    labs
}
fn simulation(a: &Attempt) -> &core::State {
    let State::UnitManager { simulation } = &a.state else {
        panic!()
    };
    simulation
}
fn command(action: &EditAction) -> String {
    format!("unit-manager {}", serde_json::to_string(action).unwrap())
}
fn manager(action: Action) -> String {
    command(&EditAction::Manager { action })
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    unit_manager_teaching::prepare(&mut a, s.unit_manager_teaching.as_ref().unwrap(), true)
        .unwrap();
    a
}
fn sync(a: &mut Attempt) {
    let draft = Draft::new(simulation(a)).unwrap();
    a.workspace.as_mut().unwrap().unit_manager_draft = Some(draft);
}
fn solved(s: &Scenario) -> Attempt {
    let mut a = prepared(s);
    for command in &s.solution {
        engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
    }
    sync(&mut a);
    a.workspace.as_mut().unwrap().checked = true;
    assert!(progress::current_solved(&a, s));
    a
}
fn profile(s: &Scenario, a: Attempt) -> Progress {
    Progress {
        schema_version: 1,
        clock: 91,
        favorites: BTreeSet::from([s.id.clone()]),
        attempts: BTreeMap::from([(s.id.clone(), a)]),
        ..Default::default()
    }
}
fn learning(a: &mut Attempt) -> &mut unit_manager_teaching::Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .unit_manager_learning
        .as_mut()
        .unwrap()
}

#[test]
fn private_pack_reaches_current_fresh_outcomes_and_retains_typed_alternate_behavior() {
    let s = scenario();
    let labs = pack::bundled().unwrap();
    assert!(labs.len() >= 98);
    assert!(!labs.iter().any(|v| v.id == s.id));
    assert_eq!(shell::EVALUATOR_REVISION, 7);
    assert!(!engine::solved(&s.initial, &s.goal));
    let reference = solved(&s);
    assert_eq!(reference.step, 0);
    let source: Value =
        serde_json::from_str(include_str!("fixtures/unit-manager-workspace-preview.json")).unwrap();
    let alternatives: Vec<EditAction> =
        serde_json::from_value(source["alternate"].clone()).unwrap();
    let mut a = prepared(&s);
    for action in &alternatives {
        engine::execute_for_goal(&mut a.state, &s.goal, &command(action)).unwrap();
    }
    sync(&mut a);
    assert_ne!(
        simulation(&a).documents(),
        simulation(&reference).documents()
    );
    assert!(progress::current_solved(&a, &s));
}
#[test]
fn mutations_require_matching_goal_and_internal_data_is_bounded_not_a_systemctl_cli() {
    let s = scenario();
    let mut a = prepared(&s);
    let original = a.state.clone();
    for text in [
        s.solution[0].as_str(),
        "systemctl start work-application.service",
        "sh -c 'touch /home/owner/file'",
        "unit-manager {}",
    ] {
        assert!(engine::execute(&mut a.state, text).is_err());
        assert_eq!(a.state, original);
    }
    let other = pack::bundled().unwrap().remove(0).goal;
    assert!(engine::execute_for_goal(&mut a.state, &other, &s.solution[0]).is_err());
    assert_eq!(a.state, original);
    let mut foreign = serde_json::to_value(&s.goal).unwrap();
    foreign["definition"]["initial"]["inventory"]["fixture"] = json!("different-inventory");
    let goal: Goal = serde_json::from_value(foreign).unwrap();
    assert!(engine::execute_for_goal(&mut a.state, &goal, &s.solution[0]).is_err());
    assert_eq!(a.state, original);
    for text in ["unit-manager {\"kind\":\"Manager\",\"action\":{\"kind\":\"Start\",\"unit\":\"work.target\",\"unknown\":true}}".into()," ".repeat(model::MAX_ACTION_BYTES+1)]{assert!(engine::execute_for_goal(&mut a.state,&s.goal,&text).is_err());assert_eq!(a.state,original);}
    assert!(
        engine::execute(&mut a.state, "help")
            .unwrap()
            .contains("No host")
    );
    assert!(
        engine::execute(&mut a.state, "show")
            .unwrap()
            .contains("Virtual unit manager")
    );
}
#[test]
fn raw_pending_edits_clear_current_credit_without_forcing_walkthrough_or_destroying_work() {
    let s = scenario();
    let mut a = solved(&s);
    let old = simulation(&a).clone();
    let draft = a
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_draft
        .as_mut()
        .unwrap();
    draft
        .documents
        .get_mut("work-application.service")
        .unwrap()
        .push_str("\n[unsupported]\n");
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(simulation(&a), &old);
    assert!(a.workspace.as_ref().unwrap().checked);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unfinished.json");
    let p = profile(&s, a);
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let labs = catalog();
    let mut p = progress::load(&path, &labs).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    let a = p.attempts.get_mut(&s.id).unwrap();
    assert!(!progress::current_solved(a, &s));
    assert!(
        a.workspace
            .as_ref()
            .unwrap()
            .unit_manager_draft
            .as_ref()
            .unwrap()
            .documents["work-application.service"]
            .contains("unsupported")
    );
    sync(a);
    assert!(progress::current_solved(a, &s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &manager(Action::Stop {
            unit: "work-application.service".into(),
        }),
    )
    .unwrap();
    sync(a);
    assert!(!progress::current_solved(a, &s));
    engine::execute_for_goal(
        &mut a.state,
        &s.goal,
        &manager(Action::Start {
            unit: "work-application.service".into(),
        }),
    )
    .unwrap();
    sync(a);
    assert!(!progress::current_solved(a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, &manager(Action::AdvancePrograms {})).unwrap();
    sync(a);
    assert!(progress::current_solved(a, &s));
    assert_eq!(a.step, 0);
}
#[test]
fn pack_rejects_wrong_pairing_changed_original_foreign_example_or_unreachable_policy() {
    let original: Value =
        serde_json::from_str(include_str!("fixtures/unit-manager-pack-preview.json")).unwrap();
    assert!(pack::load(&original.to_string()).is_ok());
    for attack in [
        "goal",
        "original",
        "example",
        "missing",
        "unreachable",
        "unknown",
    ] {
        let mut raw = original.clone();
        let s = &mut raw["scenarios"][0];
        match attack {
            "goal" => s["goal"] = serde_json::to_value(&pack::bundled().unwrap()[0].goal).unwrap(),
            "original" => {
                s["goal"]["definition"]["initial"]["configuration"]["/etc/work/application.conf"] =
                    json!("PORT=another-original\n")
            }
            "example" => {
                s["unit_manager_teaching"]["definition"]["initial"] =
                    s["initial"]["simulation"].clone()
            }
            "missing" => {
                s.as_object_mut().unwrap().remove("unit_manager_teaching");
            }
            "unreachable" => s["solution"] = json!([manager(Action::AdvancePrograms {})]),
            _ => s["initial"]["simulation"]["cached_success"] = json!(true),
        }
        assert!(pack::load(&raw.to_string()).is_err(), "{attack}");
    }
}
#[test]
fn an_independently_reachable_example_cannot_reuse_the_assignment_origin() {
    let mut raw: Value =
        serde_json::from_str(include_str!("fixtures/unit-manager-pack-preview.json")).unwrap();
    let s = &mut raw["scenarios"][0];
    let actions = s["solution"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            serde_json::from_str::<Value>(
                v.as_str().unwrap().strip_prefix("unit-manager ").unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let definition = s["goal"]["definition"].clone();
    s["unit_manager_teaching"]["definition"] = definition;
    s["unit_manager_teaching"]["solution"] = Value::Array(actions.clone());
    s["unit_manager_teaching"]["example_actions"]=json!(actions.iter().map(|a|json!({"title":"Typed example action","explanation":"Perform the supplied virtual action; no host operation.","action":a})).collect::<Vec<_>>());
    s["unit_manager_teaching"]["try_actions"] =
        json!([{"kind":"Manager","action":{"kind":"Stop","unit":"work-application.service"}}]);
    s["unit_manager_teaching"]["try_outcomes"] = json!([{"kind":"Runtime","unit":"work-application.service","phase":"Inactive","port":null,"value":null}]);
    s["unit_manager_teaching"]["repair"] = json!([{"kind":"Manager","action":{"kind":"Start","unit":"work-application.service"}},{"kind":"Manager","action":{"kind":"AdvancePrograms"}}]);
    let lesson: unit_manager_teaching::Lesson =
        serde_json::from_value(s["unit_manager_teaching"].clone()).unwrap();
    lesson.validate().unwrap();
    assert!(pack::load(&raw.to_string()).is_err());
}
#[test]
fn exact_origins_inventory_draft_and_kind_pins_reject_imports_without_rewriting_source() {
    let s = scenario();
    let all = catalog();
    let p = profile(&s, solved(&s));
    let original = serde_json::to_value(&p).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("original.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    for attack in [
        "descriptor",
        "inventory",
        "draft",
        "origin",
        "cross-kind",
        "schema",
        "unknown",
    ] {
        let mut raw = original.clone();
        let a = &mut raw["attempts"][&s.id];
        match attack {
            "descriptor" => {
                a["state"]["simulation"]["inventory"]["programs"]["/sim/bin/not-supplied"] = json!({"kind":"PortDaemon","config":"/etc/work/observer.conf","key":"PORT","prerequisite":null})
            }
            "inventory" => {
                a["state"]["simulation"]["inventory"]["fixture"] = json!("foreign-fixture")
            }
            "draft" => {
                a["workspace"]["unit_manager_draft"]["selected_unit"] = json!("foreign.service")
            }
            "origin" => {
                a["workspace"]["unit_manager_learning"]["fixture"]["configuration"]["/etc/sample/application.conf"] =
                    json!("PORT=foreign-original\n")
            }
            "cross-kind" => a["state"] = serde_json::to_value(&all[0].initial).unwrap(),
            "schema" => a["state"]["simulation"]["schema_version"] = json!(99),
            _ => a["workspace"]["unit_manager_draft"]["invented_field"] = json!(true),
        }
        if attack == "descriptor" || attack == "inventory" {
            let parsed: State = serde_json::from_value(a["state"].clone()).unwrap();
            assert!(pack::validate_state(&parsed).is_ok());
        }
        if attack == "origin" {
            let parsed: unit_manager_teaching::Learning =
                serde_json::from_value(a["workspace"]["unit_manager_learning"].clone()).unwrap();
            assert!(parsed.valid());
            assert!(!parsed.valid_for(s.unit_manager_teaching.as_ref().unwrap()));
        }
        let encoded = serde_json::to_vec(&raw).unwrap();
        assert!(progress::decode(&encoded, &all).is_err(), "{attack}");
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        let forged = dir.path().join(format!("forged-{attack}.json"));
        std::fs::write(&forged, &encoded).unwrap();
        assert!(
            backup::inspect(&forged, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).is_err()
        );
        assert_eq!(std::fs::read(&forged).unwrap(), encoded);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
}
#[test]
fn save_side_wrong_kind_and_invalid_direct_draft_retain_previous_profile_exactly() {
    let s = scenario();
    let mut p = profile(&s, prepared(&s));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    p.attempts
        .get_mut(&s.id)
        .unwrap()
        .workspace
        .as_mut()
        .unwrap()
        .unit_manager_draft
        .as_mut()
        .unwrap()
        .version = 99;
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    p.attempts.insert(s.id.clone(), prepared(&s));
    let old = pack::bundled()
        .unwrap()
        .into_iter()
        .find(|v| matches!(v.initial, State::Service { .. }))
        .unwrap();
    let mut wrong = Attempt::new(&old, 2);
    workspace::prepare(&mut wrong);
    wrong.workspace.as_mut().unwrap().unit_manager_draft = p.attempts[&s.id]
        .workspace
        .as_ref()
        .unwrap()
        .unit_manager_draft
        .clone();
    p.attempts.insert(old.id.clone(), wrong);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
}
#[test]
fn practice_fill_apply_check_and_reset_are_isolated_from_assignment_and_mode_flags() {
    let s = scenario();
    let mut a = prepared(&s);
    let assignment = a.state.clone();
    let assignment_draft = a.workspace.as_ref().unwrap().unit_manager_draft.clone();
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    learning(&mut a).fill_worked(lesson).unwrap();
    assert_eq!(a.state, assignment);
    assert_eq!(
        a.workspace.as_ref().unwrap().unit_manager_draft,
        assignment_draft
    );
    assert!(!progress::current_solved(&a, &s));
    learning(&mut a).save_draft(lesson).unwrap();
    for action in lesson.solution.iter().skip(1) {
        learning(&mut a).execute(lesson, action).unwrap();
    }
    learning(&mut a).checked = true;
    assert!(learning(&mut a).checks(lesson).iter().all(|c| c.0));
    assert!(!progress::current_solved(&a, &s));
    a.challenge = true;
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(a.state, assignment);
    assert_eq!(
        a.workspace.as_ref().unwrap().unit_manager_draft,
        assignment_draft
    );
    let saved = serde_json::to_value(learning(&mut a)).unwrap();
    unit_manager_teaching::prepare(&mut a, lesson, false).unwrap();
    assert_eq!(serde_json::to_value(learning(&mut a)).unwrap(), saved);
    a.workspace.as_mut().unwrap().unit_manager_learning =
        Some(unit_manager_teaching::Learning::new(lesson).unwrap());
    assert_eq!(a.state, assignment);
}

#[derive(Deserialize)]
struct Prior {
    id: String,
    version: u32,
}
#[derive(Deserialize)]
struct OldRecord {
    id: String,
    scenario_version: u32,
    cases: Vec<shell::Case>,
    record: shell::Record,
}
fn populate_prior98(all: &[Scenario]) -> Progress {
    let ids: Vec<Prior> =
        serde_json::from_str(include_str!("fixtures/unit-manager-prior-98-ids.json")).unwrap();
    let expected = ids
        .iter()
        .map(|v| (v.id.clone(), v.version))
        .collect::<BTreeMap<_, _>>();
    let current = pack::bundled().unwrap();
    assert_eq!(expected.len(), 98);
    assert_eq!(
        current
            .iter()
            .filter(|s| expected.contains_key(&s.id))
            .map(|s| (s.id.clone(), s.version))
            .collect::<BTreeMap<_, _>>(),
        expected
    );
    let mut p = Progress {
        schema_version: 1,
        clock: 991,
        ..Default::default()
    };
    for (i, s) in current
        .iter()
        .filter(|s| expected.contains_key(&s.id))
        .enumerate()
    {
        let mut a = Attempt::new(s, i as u64 + 1);
        if let Goal::Script { cases } = &s.goal {
            for text in &s.solution {
                engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
            }
            let State::Script { source, run } = &mut a.state else {
                panic!()
            };
            *run = Some(shell::Record {
                engine_revision: 7,
                source: source.clone(),
                results: cases.iter().map(|c| shell::run(source, c)).collect(),
            });
            assert!(progress::current_solved(&a, s));
        } else if matches!(s.initial, State::Service { .. }) {
            for text in &s.solution {
                engine::execute_for_goal(&mut a.state, &s.goal, text).unwrap();
            }
            assert!(progress::current_solved(&a, s));
        }
        workspace::prepare(&mut a);
        if let Some(lesson) = &s.service_teaching {
            dlms_learning_lab::service_teaching::prepare(&mut a, lesson, true);
        }
        a.workspace.as_mut().unwrap().command = format!("Retained {} draft ü", s.id);
        a.transcript
            .push("Retained synthetic prior investigation".into());
        a.challenge = i % 2 == 0;
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let old: Vec<OldRecord> =
        serde_json::from_str(include_str!("fixtures/shell-v6-authored-records.json")).unwrap();
    assert_eq!(old.len(), 33);
    assert_eq!(
        old.iter().map(|r| r.record.results.len()).sum::<usize>(),
        182
    );
    for record in old {
        let s = all.iter().find(|s| s.id == record.id).unwrap();
        assert_eq!(s.version, record.scenario_version);
        let Goal::Script { cases } = &s.goal else {
            panic!()
        };
        assert_eq!(
            serde_json::to_value(cases).unwrap(),
            serde_json::to_value(record.cases).unwrap()
        );
        assert_eq!(record.record.engine_revision, 6);
        assert!(record.record.valid());
        let a = p.attempts.get_mut(&record.id).unwrap();
        a.workspace.as_mut().unwrap().script_history = Some(record.record);
        assert!(progress::legacy_script_credit(a, s));
        assert!(progress::current_solved(a, s));
    }
    p
}
#[test]
fn exact98_current7_actual6_histories_and_new_drafts_survive_profile_backup_restore_selected_reset()
{
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = populate_prior98(&all);
    let old = serde_json::to_value(&p.attempts).unwrap();
    let mut a = solved(s);
    let lesson = s.unit_manager_teaching.as_ref().unwrap();
    learning(&mut a).fill_worked(lesson).unwrap();
    learning(&mut a).view = View::Example;
    *learning(&mut a)
        .draft
        .configuration
        .get_mut("/etc/sample/application.conf")
        .unwrap() = "\0".repeat(core::MAX_CONFIG_BYTES);
    *a.workspace
        .as_mut()
        .unwrap()
        .unit_manager_draft
        .as_mut()
        .unwrap()
        .documents
        .get_mut("work-application.service")
        .unwrap() = "\0".repeat(core::MAX_UNIT_BYTES);
    assert!(!progress::current_solved(&a, s));
    p.favorites.insert(s.id.clone());
    p.attempts.insert(s.id.clone(), a);
    let expected = serde_json::to_value(&p.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &p).unwrap();
    let exact = std::fs::read(&path).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), exact);
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), expected);
    progress::save(&path, &loaded).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), exact);
    let export = dir.path().join("portable.json");
    backup::export(&export, &loaded, &all).unwrap();
    let exported = std::fs::read(&export).unwrap();
    assert!(
        serde_json::from_slice::<Value>(&exported)
            .unwrap()
            .get("migration_receipts")
            .is_none()
    );
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        expected
    );
    assert_eq!(std::fs::read(&export).unwrap(), exported);
    let target = dir.path().join("restored.json");
    let _lock = progress::lock_profile(&target).unwrap();
    progress::save(&target, &profile(s, prepared(s))).unwrap();
    let before = std::fs::read(&target).unwrap();
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        before
    );
    let mut final_profile = progress::load(&target, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&final_profile.attempts).unwrap(),
        expected
    );
    final_profile.attempts.insert(s.id.clone(), prepared(s));
    progress::save(&target, &final_profile).unwrap();
    let final_profile = progress::load(&target, &all).unwrap();
    for (id, attempt) in old.as_object().unwrap() {
        assert_eq!(
            serde_json::to_value(&final_profile.attempts[id]).unwrap(),
            *attempt,
            "{id}"
        );
    }
    assert!(final_profile.favorites.contains(&s.id));
    assert!(!progress::current_solved(&final_profile.attempts[&s.id], s));
    assert_eq!(final_profile.clock, 991);
}
#[test]
fn original_single_service_reference_keeps_exact_legacy_execution_behavior() {
    for s in pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| matches!(s.initial, State::Service { .. }))
    {
        let mut raw = s.initial.clone();
        let mut contextual = s.initial.clone();
        for text in &s.solution {
            assert_eq!(
                engine::execute(&mut raw, text),
                engine::execute_for_goal(&mut contextual, &s.goal, text)
            );
            assert_eq!(raw, contextual);
        }
        assert!(engine::solved(&raw, &s.goal));
    }
}
