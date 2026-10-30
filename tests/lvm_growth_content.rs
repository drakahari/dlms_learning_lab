//! Prospective full-model/codec/content gates against normal186. No host LVM.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    lvm_workspace::{self as work, Session},
    pack::{self, Pack, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
#[path = "support/lvm_original_catalog.rs"]
mod original;
#[path = "support/lvm_registration.rs"]
mod registration;
use serde_json::{Value, json};
use std::collections::BTreeSet;
const DATA: &str = include_str!("../scenarios/lvm-growth-v1.json");
const REVIEW: &str = include_str!("fixtures/lvm-growth-review.json");
const IDS: [&str; 4] = [
    "lpic2-lvm-free-space",
    "lpic2-lvm-add-pv",
    "lpic2-lvm-filesystem-gap",
    "lpic2-lvm-handover",
];
fn v<T: serde::Serialize + ?Sized>(x: &T) -> Value {
    serde_json::to_value(x).unwrap()
}
fn authored() -> Vec<Scenario> {
    serde_json::from_str::<Pack>(DATA).unwrap().scenarios
}
fn catalog() -> Vec<Scenario> {
    let mut all = original::load();
    assert!(IDS.iter().all(|id| !all.iter().any(|s| &s.id == id)));
    all.extend(authored());
    let texts = all
        .chunks(16)
        .map(|s| json!({"schema_version":1,"scenarios":s}).to_string())
        .collect::<Vec<_>>();
    let expected =
        pack::load_catalog(&texts.iter().map(String::as_str).collect::<Vec<_>>()).unwrap();
    let current190 = pack::bundled().unwrap();
    assert_eq!(current190.len(), 190);
    assert_eq!(
        v(&expected),
        v(registration::ntp_registration::before_scenarios(
            &current190
        ))
    );
    expected
}
fn definition(s: &Scenario) -> &work::Definition {
    let Goal::Lvm { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn session(s: &State) -> &Session {
    let State::Lvm { simulation } = s else {
        panic!()
    };
    simulation
}
fn session_mut(s: &mut State) -> &mut Session {
    let State::Lvm { simulation } = s else {
        panic!()
    };
    simulation
}
fn rows(state: &State, s: &Scenario) -> Vec<bool> {
    engine::checks(state, &s.goal).iter().map(|r| r.0).collect()
}
fn commands(state: &mut State, s: &Scenario, cs: &Value) {
    for c in cs.as_array().unwrap() {
        engine::execute_for_goal(state, &s.goal, c.as_str().unwrap()).unwrap();
    }
}
fn bits(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn profile(s: &Scenario, state: State) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = Attempt::new(s, 1);
    a.state = state;
    workspace::prepare(&mut a);
    p.attempts.insert(s.id.clone(), a);
    p
}
#[test]
fn four_substantive_tasks_have_closed_contexts_separate_practice_and_real_prerequisites() {
    let old = original::load();
    let all = catalog();
    assert_eq!(v(&all[..old.len()]), v(&old));
    let labs = &all[old.len()..];
    assert_eq!(labs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), IDS);
    let unit: Value = serde_json::from_str(REVIEW).unwrap();
    let raw48 = registration::before_units(include_bytes!("../scenarios/units-v1.json")).unwrap();
    let mut units =
        dlms_learning_lab::course::load(std::str::from_utf8(&raw48).unwrap(), &old).unwrap();
    units.push(serde_json::from_value(unit["unit_proposal"].clone()).unwrap());
    let units =
        dlms_learning_lab::course::load(&json!({"version":1,"units":units}).to_string(), &all)
            .unwrap();
    assert_eq!(units.last().unwrap().labs, IDS);
    for s in labs {
        let d = definition(s);
        d.validate().unwrap();
        work::source(s).unwrap();
        assert!(s.terminal_teaching.is_none());
        assert!(!engine::solved(&s.initial, &s.goal));
        assert!(
            d.initial
                .devices
                .keys()
                .all(|p| !d.lesson.fixture.devices.contains_key(p))
        );
        assert!(
            s.prerequisites
                .iter()
                .all(|id| all.iter().any(|p| &p.id == id))
        );
    }
    assert_ne!(definition(&labs[0]).initial, definition(&labs[1]).initial);
    assert_eq!(
        definition(&labs[2])
            .initial
            .lv_bytes("service/reports")
            .unwrap(),
        24 * dlms_learning_lab::lvm::MIB
    );
    assert_eq!(definition(&labs[3]).targets.len(), 2);
}
#[test]
fn all_references_and_two_distinct_legitimate_alternatives_solve_current_state() {
    let review: Value = serde_json::from_str(REVIEW).unwrap();
    let all = catalog();
    for row in review["labs"].as_array().unwrap() {
        let s = all
            .iter()
            .find(|s| Some(s.id.as_str()) == row["id"].as_str())
            .unwrap();
        let alts = row["alternatives"].as_array().unwrap();
        assert_eq!(alts.len(), 2);
        let programs = std::iter::once(v(&s.solution))
            .chain(alts.iter().cloned())
            .collect::<Vec<_>>();
        assert_eq!(
            programs
                .iter()
                .map(Value::to_string)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
        for program in programs {
            let mut state = s.initial.clone();
            commands(&mut state, s, &program);
            assert!(
                engine::solved(&state, &s.goal),
                "{}: {:?}",
                s.id,
                engine::checks(&state, &s.goal)
            );
            assert_eq!(session(&state).example, definition(s).lesson.fixture);
            assert!(session(&state).example_transcript.is_empty());
        }
    }
}
#[test]
fn partial_wrong_and_report_only_work_has_exact_nonvacuous_failure_rows() {
    let review: Value = serde_json::from_str(REVIEW).unwrap();
    for row in review["labs"].as_array().unwrap() {
        let s = authored()
            .into_iter()
            .find(|s| Some(s.id.as_str()) == row["id"].as_str())
            .unwrap();
        assert_eq!(rows(&s.initial, &s), bits(&row["initial_rows"]));
        for key in ["partial", "report_only"] {
            let mut state = s.initial.clone();
            commands(&mut state, &s, &row[key]["commands"]);
            let actual = rows(&state, &s);
            assert_eq!(actual, bits(&row[key]["rows"]));
            assert!(actual.contains(&false) && actual.contains(&true));
        }
        let mut state = s.initial.clone();
        for c in row["wrong"]["commands"].as_array().unwrap() {
            let before = v(&state);
            assert!(engine::execute_for_goal(&mut state, &s.goal, c.as_str().unwrap()).is_err());
            assert_eq!(v(&state), before);
        }
        assert_eq!(rows(&state, &s), bits(&row["wrong"]["rows"]));
    }
}
#[test]
fn worked_insert_is_inert_and_practice_experiment_repair_never_installs_assignment_work() {
    for s in authored() {
        let d = definition(&s);
        let mut a = session(&s.initial).clone();
        let initial = a.current.clone();
        assert!(!a.example_checked);
        assert!(a.example_transcript.is_empty());
        assert!(d.practice_checks(&a).iter().any(|r| !r.0));
        for (i, c) in d.lesson.worked.iter().enumerate() {
            let before = a.example.clone();
            let transcript = a.example_transcript.clone();
            a.insert(d, i).unwrap();
            assert_eq!(a.example, before);
            assert_eq!(a.example_transcript, transcript);
            assert_eq!(a.example_command, c.text);
            assert_eq!(a.current, initial);
            a.execute(d, &c.text, true).unwrap();
        }
        assert!(d.practice_checks(&a).iter().all(|r| r.0));
        assert_eq!(a.current, initial);
        let before = a.example.clone();
        a.execute(d, &d.lesson.experiment, true).unwrap();
        assert_ne!(a.example, before);
        assert!(d.practice_checks(&a).iter().any(|r| !r.0));
        a.execute(d, &d.lesson.repair, true).unwrap();
        assert!(d.practice_checks(&a).iter().all(|r| r.0));
        assert_eq!(a.current, initial);
        assert!(!d.checks(&a).iter().all(|r| r.0));
        a.reset_example(d).unwrap();
        assert_eq!(a.example, d.lesson.fixture);
        assert_eq!(a.current, initial);
        assert!(a.example_transcript.is_empty());
    }
}
#[test]
fn capacity_and_data_damage_invalidate_current_truth_and_explicit_repair_or_reset_recovers() {
    for s in authored() {
        let d = definition(&s);
        let mut state = s.initial.clone();
        commands(&mut state, &s, &v(&s.solution));
        assert!(engine::solved(&state, &s.goal));
        let id = &d.targets[0].volume;
        let (vg, lv) = id.split_once('/').unwrap();
        let initial_blocks = d.initial.groups[vg].volumes[lv]
            .filesystem
            .as_ref()
            .unwrap()
            .blocks;
        session_mut(&mut state)
            .current
            .groups
            .get_mut(vg)
            .unwrap()
            .volumes
            .get_mut(lv)
            .unwrap()
            .filesystem
            .as_mut()
            .unwrap()
            .blocks = initial_blocks;
        assert!(!engine::solved(&state, &s.goal));
        engine::execute_for_goal(&mut state, &s.goal, &format!("resize2fs /dev/{id}")).unwrap();
        assert!(engine::solved(&state, &s.goal));
        let fs = session_mut(&mut state)
            .current
            .groups
            .get_mut(vg)
            .unwrap()
            .volumes
            .get_mut(lv)
            .unwrap()
            .filesystem
            .as_mut()
            .unwrap();
        fs.files.values_mut().next().unwrap().contents = "damaged data".into();
        assert!(!engine::solved(&state, &s.goal));
        let mut reset = s.initial.clone();
        commands(&mut reset, &s, &v(&s.solution));
        assert!(engine::solved(&reset, &s.goal));
    }
}
#[test]
fn context_foreign_kind_changed_source_overlong_packets_and_invalid_aliases_fail_atomically() {
    for s in authored() {
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        work::context(&a, &s).unwrap();
        commands(&mut a.state, &s, &v(&s.solution));
        assert!(progress::current_solved(&a, &s));
        let mut wrong = s.clone();
        let current_fs = session(&a.state)
            .current
            .filesystem_bytes(&definition(&s).targets[0].volume)
            .unwrap();
        let Goal::Lvm {
            definition: changed_def,
        } = &mut wrong.goal
        else {
            panic!()
        };
        changed_def.targets[0].minimum_filesystem = current_fs + 4096;
        changed_def.targets[0].minimum_lv =
            changed_def.targets[0].minimum_lv.max(current_fs + 4096);
        assert!(!progress::current_solved(&a, &wrong));
        let mut state = s.initial.clone();
        for text in [
            "resize2fs data/reports".to_string(),
            "pvs; ignored".into(),
            "x".repeat(2049),
        ] {
            let before = v(&state);
            assert!(engine::execute_for_goal(&mut state, &s.goal, &text).is_err());
            assert_eq!(v(&state), before);
        }
        let before = v(&a);
        a.workspace.as_mut().unwrap().command = "foreign terminal metadata".into();
        let foreign = v(&a);
        assert!(work::context(&a, &s).is_err());
        workspace::prepare(&mut a);
        assert_eq!(v(&a), foreign);
        assert_ne!(v(&a), before);
        let d = definition(&s);
        let mut current = session(&s.initial).clone();
        let before = v(&current);
        assert!(current.set_command(d, "é".repeat(2048), false).is_err());
        assert_eq!(v(&current), before);
        assert!(current.insert(d, 12).is_err());
        assert_eq!(v(&current), before);
    }
}
#[test]
fn save_reopen_and_selected_reset_preserve_other_attempts_favorites_raw_files_and_receipts() {
    let all = catalog();
    let s = all.iter().find(|s| s.id == IDS[0]).unwrap();
    let peer = all.iter().find(|s| s.id == "lpic2-storage").unwrap();
    let mut state = s.initial.clone();
    commands(&mut state, s, &v(&s.solution));
    let mut p = profile(s, state);
    p.attempts.insert(peer.id.clone(), Attempt::new(peer, 2));
    p.favorites.insert(s.id.clone());
    let before = v(&p);
    let receipts = v(&p.migration_receipts);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    assert_eq!(v(&p), before);
    assert_eq!(v(&p.migration_receipts), receipts);
    let raw = std::fs::read(&path).unwrap();
    let mut restored = progress::decode(&raw, &all).unwrap();
    assert!(progress::current_solved(&restored.attempts[&s.id], s));
    let keep = v(&restored.attempts[&peer.id]);
    let receipt = v(&restored.migration_receipts);
    let visited = restored.attempts[&s.id].visited;
    let mut fresh = Attempt::new(s, visited);
    workspace::prepare(&mut fresh);
    restored.attempts.insert(s.id.clone(), fresh);
    assert!(!progress::current_solved(&restored.attempts[&s.id], s));
    assert_eq!(v(&restored.attempts[&peer.id]), keep);
    assert!(restored.favorites.contains(&s.id));
    assert_eq!(v(&restored.migration_receipts), receipt);
    assert_eq!(std::fs::read(&path).unwrap(), raw);
}
#[test]
fn malformed_source_and_progress_contexts_refuse_without_fabricated_history_or_mastery() {
    let all = catalog();
    let s = all.iter().find(|s| s.id == IDS[0]).unwrap();
    let p = profile(s, s.initial.clone());
    let raw = serde_json::to_vec(&p).unwrap();
    assert!(progress::decode(&raw, &all).is_ok());
    for field in ["initial", "example_initial"] {
        let mut value = v(&p);
        let devices = value["attempts"][&s.id]["state"]["simulation"][field]["devices"]
            .as_object_mut()
            .unwrap();
        let extra = devices
            .values()
            .find(|v| v["role"] == "PhysicalVolume")
            .unwrap()
            .clone();
        devices.insert("/dev/foreign-added".into(), extra);
        assert!(progress::decode(&serde_json::to_vec(&value).unwrap(), &all).is_err());
    }
    let mut changed = s.clone();
    changed.terminal_teaching = all.iter().find_map(|s| s.terminal_teaching.clone());
    assert!(changed.terminal_teaching.is_some());
    assert!(work::source(&changed).is_err());
    let mut value = v(&p);
    value["attempts"][&s.id]["state"]["simulation"]["transcript"] =
        json!([{"input":"pvs","output":"fake display only"}]);
    let loaded = progress::decode(&serde_json::to_vec(&value).unwrap(), &all).unwrap();
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
}
#[test]
fn streaming_session_collections_and_outer_raw_admission_remain_bounded() {
    let s = authored().remove(0);
    let mut raw = v(session(&s.initial));
    raw["selected_example"] = json!(12);
    assert!(
        serde_json::from_value::<Session>(raw)
            .unwrap()
            .validate()
            .is_err()
    );
    let row = "{\"input\":\"pvs\",\"output\":\"display\"}";
    let original = serde_json::to_string(session(&s.initial)).unwrap();
    let replacement = format!(
        "\"transcript\":[{},POISON_MUST_NOT_BE_READ]",
        std::iter::repeat_n(row, 32).collect::<Vec<_>>().join(",")
    );
    let raw = original.replacen("\"transcript\":[]", &replacement, 1);
    let e = serde_json::from_str::<Session>(&raw).unwrap_err();
    assert!(e.to_string().contains("item limit"));
    assert!(pack::load(&" ".repeat(pack::MAX_PACK_BYTES + 1)).is_err());
    let all = catalog();
    assert!(progress::decode(&vec![b' '; 16_000_001], &all).is_err());
}
