//! Prospective authored-content gates. No normal registration or runtime claim.
use dlms_learning_lab::{
    engine::{self, State},
    one_off_workspace as work, pack,
    progress::{self, Attempt},
    script_teaching::View,
};
use serde_json::Value;
#[path = "support/ntp_registration.rs"]
mod ntp_registration;
fn lessons() -> Vec<pack::Scenario> {
    let p: pack::Pack =
        serde_json::from_str(include_str!("../scenarios/linux-one-off-v1.json")).unwrap();
    p.scenarios
}
fn reviews() -> Vec<Value> {
    serde_json::from_str(include_str!("fixtures/one-off-lessons-review-v1.json")).unwrap()
}
fn execute(s: &pack::Scenario, actions: &Value, start: &State) -> State {
    let mut state = start.clone();
    for action in actions.as_array().unwrap() {
        engine::execute_for_goal(&mut state, &s.goal, &format!("one-off {action}")).unwrap();
    }
    state
}
fn rows(s: &pack::Scenario, state: &State) -> Vec<bool> {
    engine::checks(state, &s.goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
fn expected(v: &Value) -> Vec<bool> {
    serde_json::from_value(v.clone()).unwrap()
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 17);
    work::budget::prepare(&mut a, s).unwrap();
    a
}
fn action(a: &mut Attempt, s: &pack::Scenario, v: &Value, practice: bool) {
    let v: work::Action = serde_json::from_value(v.clone()).unwrap();
    if let work::Action::Run { end } = v {
        let mut run = work::budget::PendingAttempt::begin(a, s, end, practice).unwrap();
        while !run.done() {
            run.tick(32);
        }
        run.finish(a, s).unwrap();
    } else {
        work::budget::apply_attempt(a, s, &v, practice).unwrap();
    }
}
fn actual_reference(a: &mut Attempt, s: &pack::Scenario, actions: &Value) {
    for v in actions.as_array().unwrap() {
        match v["kind"].as_str().unwrap() {
            "Write" => {
                action(
                    a,
                    s,
                    &serde_json::json!({"kind":"Edit","path":v["path"],"text":v["text"]}),
                    false,
                );
                action(
                    a,
                    s,
                    &serde_json::json!({"kind":"Save","path":v["path"]}),
                    false,
                );
            }
            "Command" => {
                action(
                    a,
                    s,
                    &serde_json::json!({"kind":"Actor","actor":v["actor"]}),
                    false,
                );
                action(
                    a,
                    s,
                    &serde_json::json!({"kind":"Environment","values":v["environment"]}),
                    false,
                );
                action(
                    a,
                    s,
                    &serde_json::json!({"kind":"Command","text":v["text"]}),
                    false,
                );
            }
            "Run" => action(a, s, v, false),
            _ => panic!("Unreviewed authoring action"),
        }
    }
}
#[test]
fn four_distinct_registered_tasks_have_real_prerequisites_and_separate_teaching() {
    let old = pack::bundled().unwrap();
    assert_eq!(ntp_registration::before_scenarios(&old).len(), 186);
    assert_eq!(
        ntp_registration::ere_registration::before_scenarios(&old).len(),
        190
    );
    assert_eq!(old.len(), 193);
    assert_eq!(
        serde_json::to_value(&old[176..180]).unwrap(),
        serde_json::to_value(lessons()).unwrap()
    );
    assert_eq!(
        old[171..176]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "ccna-physical-media-span",
            "ccna-physical-duplex-diagnosis",
            "ccna-physical-counter-baseline",
            "ccna-physical-uplink-handover",
            "ccna-private-office-plan"
        ]
    );
    let labs = lessons();
    assert_eq!(labs.len(), 4);
    let ids: std::collections::BTreeSet<_> = old
        .iter()
        .chain(labs.iter())
        .map(|s| s.id.as_str())
        .collect();
    for s in &labs {
        assert!(!old[..176].iter().any(|x| x.id == s.id));
        assert_eq!(s.track, "LPIC-1");
        assert_eq!(s.unit, "linux-one-off-jobs");
        assert!(pack::DIFFICULTIES.contains(&s.difficulty.as_str()));
        assert!(s.prerequisites.iter().all(|id| ids.contains(id.as_str())));
        let engine::Goal::OneOff { definition } = &s.goal else {
            panic!()
        };
        definition.validate().unwrap();
        let l = s.one_off_teaching.as_ref().unwrap();
        l.validate().unwrap();
        assert!(l.separate_from(definition));
        assert!(l.concepts.len() >= 3);
        assert!(!engine::solved(&s.initial, &s.goal));
    }
    assert_ne!(
        serde_json::to_value(&labs[0].goal).unwrap(),
        serde_json::to_value(&labs[1].goal).unwrap()
    );
}
#[test]
fn complete_loader_reaches_solutions_in_independently_bounded_packs_and_keeps_old_objects() {
    let old = pack::bundled().unwrap();
    let mut owned: Vec<String> = old[..176]
        .chunks(16)
        .map(|c| {
            serde_json::to_string(&pack::Pack {
                schema_version: 1,
                scenarios: c.to_vec(),
            })
            .unwrap()
        })
        .collect();
    owned.push(include_str!("../scenarios/linux-one-off-v1.json").into());
    assert!(owned.iter().all(|s| s.len() < pack::MAX_PACK_BYTES));
    let refs: Vec<&str> = owned.iter().map(String::as_str).collect();
    let all = pack::load_catalog(&refs).unwrap();
    assert_eq!(all.len(), 180);
    assert_eq!(
        serde_json::to_value(&all[..176]).unwrap(),
        serde_json::to_value(&old[..176]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&all[176..]).unwrap(),
        serde_json::to_value(lessons()).unwrap()
    );
}
#[test]
fn references_and_two_legitimate_alternatives_pass_complete_current_and_fresh_behavior() {
    for (s, r) in lessons().iter().zip(reviews()) {
        assert_eq!(s.id, r["id"]);
        let good = execute(s, &r["reference"], &s.initial);
        let rs = rows(s, &good);
        assert!(rs.iter().all(|b| *b), "{} {rs:?}", s.id);
        assert_eq!(r["alternatives"].as_array().unwrap().len(), 2);
        for alt in r["alternatives"].as_array().unwrap() {
            let result = execute(s, alt, &s.initial);
            assert!(engine::solved(&result, &s.goal), "{} alternate", s.id);
        }
        assert_eq!(
            rows(s, &s.initial),
            expected(&r["initial_rows"]),
            "{} initial",
            s.id
        );
    }
}
#[test]
fn literal_partial_and_first_sample_hardcoding_vectors_diagnose_distinct_mistakes() {
    for (s, r) in lessons().iter().zip(reviews()) {
        for name in ["partial", "hardcoded"] {
            let result = execute(s, &r[name]["actions"], &s.initial);
            let rs = rows(s, &result);
            assert_eq!(rs, expected(&r[name]["rows"]), "{} {name}", s.id);
            assert!(rs.iter().any(|b| !*b));
            assert!(
                engine::checks(&result, &s.goal)
                    .iter()
                    .any(|(ok, text)| !*ok && !text.is_empty())
            );
        }
    }
}
#[test]
fn actual_allowed_damage_invalidates_success_and_fresh_retry_reproduces_it() {
    for (s, r) in lessons().iter().zip(reviews()) {
        let good = execute(s, &r["reference"], &s.initial);
        assert!(engine::solved(&good, &s.goal));
        let damaged = execute(s, &r["damage"]["actions"], &good);
        assert_eq!(
            rows(s, &damaged),
            expected(&r["damage"]["rows"]),
            "{}",
            s.id
        );
        assert!(!engine::solved(&damaged, &s.goal));
        let retry = execute(s, &r["repair_from_initial"], &s.initial);
        assert_eq!(retry, good);
        assert!(engine::solved(&retry, &s.goal));
    }
}
#[test]
fn real_example_actions_and_unsaved_change_never_edit_assignment_or_award_credit() {
    for s in lessons() {
        let mut a = attempt(&s);
        let state = serde_json::to_value(&a.state).unwrap();
        let session = serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_session).unwrap();
        let l = s.one_off_teaching.as_ref().unwrap();
        work::budget::set_view(&mut a, &s, View::Example, 0).unwrap();
        for row in &l.example_actions {
            action(
                &mut a,
                &s,
                &serde_json::to_value(&row.action).unwrap(),
                true,
            );
        }
        assert_eq!(serde_json::to_value(&a.state).unwrap(), state);
        assert_eq!(
            serde_json::to_value(&a.workspace.as_ref().unwrap().one_off_session).unwrap(),
            session
        );
        assert!(!progress::current_solved(&a, &s));
        let practice = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap();
        assert!(
            work::checks_with_session(&l.definition, &practice.assignment, &practice.session)
                .iter()
                .all(|r| r.0)
        );
        for v in &l.try_actions {
            action(&mut a, &s, &serde_json::to_value(v).unwrap(), true);
        }
        let practice = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap();
        assert_eq!(
            work::checks_with_session(&l.definition, &practice.assignment, &practice.session)
                .iter()
                .map(|r| r.0)
                .collect::<Vec<_>>(),
            l.try_rows
        );
        work::budget::set_view(&mut a, &s, View::Assignment, 0).unwrap();
        work::budget::set_view(&mut a, &s, View::Introduction, 0).unwrap();
        for v in &l.repair {
            action(&mut a, &s, &serde_json::to_value(v).unwrap(), true);
        }
        let practice = a
            .workspace
            .as_ref()
            .unwrap()
            .one_off_learning
            .as_ref()
            .unwrap();
        assert!(
            work::checks_with_session(&l.definition, &practice.assignment, &practice.session)
                .iter()
                .all(|r| r.0)
        );
        assert_eq!(serde_json::to_value(&a.state).unwrap(), state);
        assert!(!progress::current_solved(&a, &s));
    }
}
#[test]
fn all_four_real_drafts_resume_and_selected_reset_preserves_others_preferences_and_actual_receipts()
{
    use std::collections::{BTreeMap, BTreeSet};
    let labs = lessons();
    let rev = reviews();
    let mut attempts = BTreeMap::new();
    for (s, r) in labs.iter().zip(&rev) {
        let mut a = attempt(s);
        actual_reference(&mut a, s, &r["reference"]);
        assert!(progress::current_solved(&a, s), "{}", s.id);
        let l = s.one_off_teaching.as_ref().unwrap();
        action(
            &mut a,
            s,
            &serde_json::json!({"kind":"Edit","path":l.definition.editable[0],"text":"# retained practice draft\n"}),
            true,
        );
        attempts.insert(s.id.clone(), a);
    }
    let p = progress::Progress {
        schema_version: 1,
        clock: 88,
        favorites: BTreeSet::from([labs[0].id.clone(), labs[3].id.clone()]),
        attempts,
        migration_receipts: None,
    };
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("one-off-content-progress.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::decode(&std::fs::read(&path).unwrap(), &labs).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let receipts = serde_json::to_value(&restored.migration_receipts).unwrap();
    assert!(restored.migration_receipts.is_some());
    let before = serde_json::to_value(&restored).unwrap();
    let id = &labs[0].id;
    let visited = restored.attempts[id].visited;
    work::budget::reset_attempt(restored.attempts.get_mut(id).unwrap(), &labs[0]).unwrap();
    assert_eq!(restored.clock, 88);
    assert_eq!(restored.attempts[id].visited, visited);
    assert_eq!(
        serde_json::to_value(&restored.migration_receipts).unwrap(),
        receipts
    );
    assert_eq!(
        serde_json::to_value(&restored.favorites).unwrap(),
        before["favorites"]
    );
    for s in &labs[1..] {
        assert_eq!(
            serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
            before["attempts"][&s.id]
        );
        assert!(progress::current_solved(&restored.attempts[&s.id], s));
    }
    assert!(!progress::current_solved(&restored.attempts[id], &labs[0]));
    assert_eq!(restored.attempts[id].state, labs[0].initial);
}
#[test]
fn owner_refusal_unsupported_commands_and_bad_time_retain_entire_current_work() {
    for s in lessons() {
        let mut a = attempt(&s);
        for text in [
            "at -f /work/no-such-script now + 5 minutes",
            "at -f /etc/passwd now + 5 minutes",
            "at -f /work/no-such-script teatime",
        ] {
            let before = serde_json::to_value(&a).unwrap();
            assert!(
                work::budget::apply_attempt(
                    &mut a,
                    &s,
                    &work::Action::Command { text: text.into() },
                    false
                )
                .is_err()
            );
            assert_eq!(serde_json::to_value(&a).unwrap(), before);
        }
    }
    let s = &lessons()[2];
    let mut a = attempt(s);
    action(
        &mut a,
        s,
        &serde_json::json!({"kind":"Actor","actor":"nia"}),
        false,
    );
    let before = serde_json::to_value(&a).unwrap();
    assert!(
        work::budget::apply_attempt(
            &mut a,
            s,
            &work::Action::Command {
                text: "atrm 2".into()
            },
            false
        )
        .is_err()
    );
    assert_eq!(serde_json::to_value(&a).unwrap(), before);
}

#[test]
fn unfinished_run_stop_retains_all_work_and_queued_capture_is_not_rewritten_by_later_edits() {
    for (s, r) in lessons().iter().zip(reviews()) {
        if s.id == "lpic1-at-owner-queue" {
            continue;
        }
        let mut a = attempt(s);
        let mut prefix = r["reference"].as_array().unwrap().clone();
        let last = prefix.pop().unwrap();
        assert_eq!(last["kind"], "Run");
        actual_reference(&mut a, s, &Value::Array(prefix));
        let before = serde_json::to_value(&a).unwrap();
        let end = serde_json::from_value(last["end"].clone()).unwrap();
        let mut pending = work::budget::PendingAttempt::begin(&a, s, end, false).unwrap();
        pending.tick(1);
        pending.cancel();
        assert!(pending.finish(&mut a, s).is_err());
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        assert!(!progress::current_solved(&a, s));
        action(&mut a, s, &last, false);
        assert!(progress::current_solved(&a, s));
    }
    let s = &lessons()[1];
    let r = &reviews()[1];
    let mut missing_removal = r["reference"].as_array().unwrap().clone();
    missing_removal.retain(|a| a["text"] != "atrm 1");
    let state = execute(s, &Value::Array(missing_removal), &s.initial);
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|(ok, text)| !*ok && text.contains("queue snapshot"))
    );
}
