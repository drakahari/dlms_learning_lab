//! Bundled original reverse-zone assignments. No host DNS or commands.
use dlms_learning_lab::{
    backup, course,
    dns::{self, Data, QueryType, Record, Server, Transfer, Zone},
    dns_teaching::{self, Learning},
    dns_workspace::{Draft, Requirement, Snapshot},
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    workspace,
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    net::IpAddr,
    sync::atomic::{AtomicBool, AtomicUsize},
};

const IDS: [&str; 3] = [
    "lpic2-dns-reverse-investigation",
    "lpic2-dns-reverse-ipv6",
    "lpic2-dns-reverse-handover",
];
const PACK: &str = include_str!("../scenarios/dns-reverse-v1.json");
fn catalog() -> Vec<Scenario> {
    pack::bundled().unwrap()
}
fn labs() -> Vec<Scenario> {
    catalog()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn review(id: &str) -> Value {
    serde_json::from_str::<Vec<Value>>(include_str!("fixtures/dns-reverse-content-review.json"))
        .unwrap()
        .into_iter()
        .find(|r| r["id"] == id)
        .unwrap()
}
fn actions(v: &Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap()
}
fn server(state: &State) -> &Server {
    let State::AuthoritativeDns { server } = state else {
        panic!()
    };
    server
}
fn copy_mut(s: &mut Server, copy: Snapshot) -> &mut Zone {
    match copy {
        Snapshot::Configured => &mut s.configured,
        Snapshot::Primary => &mut s.primary,
        Snapshot::Secondary => &mut s.secondary,
    }
}
fn requirements(lab: &Scenario) -> &[Requirement] {
    let Goal::AuthoritativeDns { requirements } = &lab.goal else {
        panic!()
    };
    requirements
}
fn run(lab: &Scenario, state: &mut State, commands: &[String]) {
    for command in commands {
        engine::execute_for_goal(state, &lab.goal, command).unwrap();
    }
}
fn solved(lab: &Scenario) -> Server {
    let mut state = lab.initial.clone();
    run(lab, &mut state, &lab.solution);
    assert!(
        engine::solved(&state, &lab.goal),
        "{} {:?}",
        lab.id,
        engine::checks(&state, &lab.goal)
    );
    server(&state).clone()
}
fn bools(lab: &Scenario, state: &State) -> Vec<bool> {
    engine::checks(state, &lab.goal)
        .into_iter()
        .map(|r| r.0)
        .collect()
}
fn publish(s: &mut Server, secondary: bool) {
    s.configured.serial = s.primary.serial.wrapping_add(1);
    s.reload_primary().unwrap();
    if secondary {
        s.update_secondary().unwrap();
    }
}
fn ptr(owner: &str, target: &str) -> Record {
    Record {
        owner: owner.into(),
        ttl: 300,
        data: Data::Ptr {
            target: target.into(),
        },
    }
}
fn assert_questions(s: &Server, questions: &Value, secondary: bool) {
    for q in questions.as_array().unwrap() {
        let expected: BTreeSet<String> = q["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t.as_str().unwrap().to_ascii_lowercase())
            .collect();
        for copy in [Snapshot::Primary, Snapshot::Secondary]
            .into_iter()
            .take(if secondary { 2 } else { 1 })
        {
            let a = copy
                .zone(s)
                .query(q["owner"].as_str().unwrap(), QueryType::Ptr)
                .unwrap();
            assert!(a.authoritative && a.code == dns::Code::Noerror);
            let actual: BTreeSet<_> = a
                .records
                .iter()
                .filter_map(|r| {
                    if let Data::Ptr { target } = &r.data {
                        Some(target.to_ascii_lowercase())
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
#[test]
fn three_distinct_tasks_have_real_prerequisites_bounded_goals_and_reachable_references() {
    let all = catalog();
    let tasks = labs();
    assert_eq!(tasks.len(), 3);
    assert_eq!(all.len(), 133);
    let units = course::bundled(&all).unwrap();
    assert_eq!(units.len(), 33);
    let unit = units.iter().find(|u| u.id == "dns-reverse").unwrap();
    assert_eq!(unit.prerequisites, ["dns-forward"]);
    assert_eq!(unit.labs, IDS);
    assert!(
        tasks
            .iter()
            .all(|s| !s.simplifications.contains("Proposal only"))
    );
    assert!(
        tasks
            .iter()
            .all(|s| !serde_json::to_string(s).unwrap().contains(".test.."))
    );
    for (i, lab) in tasks.iter().enumerate() {
        assert_eq!(
            lab.prerequisites,
            [if i == 0 {
                "lpic2-dns-handover"
            } else {
                IDS[i - 1]
            }]
        );
        assert!(
            lab.prerequisites
                .iter()
                .all(|id| all.iter().any(|s| s.id == *id))
        );
        assert!(!engine::solved(&lab.initial, &lab.goal));
        let lesson = lab.dns_teaching.as_ref().unwrap();
        lesson.validate().unwrap();
        assert_ne!(
            server(&lab.initial).configured.origin,
            lesson.fixture.configured.origin
        );
        assert!(requirements(lab).len() <= 20);
        assert!(lab.guide.iter().all(|step| step.commands.is_empty()));
        for text in [&lab.objective, &lab.explanation, &lab.hints.join(" ")] {
            assert!(!text.contains("zone {"));
        }
        let s = solved(lab);
        let expected: Zone =
            serde_json::from_value(review(&lab.id)["expected_zone"].clone()).unwrap();
        assert!(s.configured.equivalent(&expected).unwrap());
        assert_questions(&s, &review(&lab.id)["questions"], i == 2);
    }
    assert_eq!(
        requirements(&tasks[2])
            .iter()
            .filter(|r| matches!(r, Requirement::NoAnswer { .. }))
            .count(),
        2
    );
    assert!(
        requirements(&tasks[2])
            .iter()
            .any(|r| matches!(r, Requirement::Published { secondary: true }))
    );
}
#[test]
fn full_question_tables_derive_from_explicit_expanded_digits_not_the_engine_helper() {
    for lab in labs() {
        let r = review(&lab.id);
        for context in [&r, &r["example"]] {
            let mut questions: Vec<_> = context["questions"].as_array().unwrap().iter().collect();
            questions.push(&context["healthy_question"]);
            if let Some(retired) = context.get("retired") {
                questions.push(retired);
            }
            for q in questions {
                let address: IpAddr = q["address"].as_str().unwrap().parse().unwrap();
                let expected = match address {
                    IpAddr::V4(a) => {
                        assert!(q.get("expanded").is_none());
                        let octets: Vec<_> = q["address"].as_str().unwrap().split('.').collect();
                        assert_eq!(octets.len(), 4);
                        assert_eq!(a.to_string(), q["address"].as_str().unwrap());
                        format!(
                            "{}.in-addr.arpa.",
                            octets.into_iter().rev().collect::<Vec<_>>().join(".")
                        )
                    }
                    IpAddr::V6(a) => {
                        let expanded = q["expanded"].as_str().unwrap();
                        assert_eq!(expanded.len(), 32);
                        assert_eq!(format!("{:032x}", u128::from(a)), expanded);
                        format!(
                            "{}.ip6.arpa.",
                            expanded
                                .chars()
                                .rev()
                                .map(|c| c.to_string())
                                .collect::<Vec<_>>()
                                .join(".")
                        )
                    }
                };
                assert_eq!(expected, q["owner"].as_str().unwrap());
            }
        }
    }
}
#[test]
fn two_meaningful_alternatives_per_task_accept_normalization_and_staged_repairs() {
    for (i, lab) in labs().iter().enumerate() {
        let r = review(&lab.id);
        assert_eq!(r["alternates"].as_array().unwrap().len(), 2);
        for alt in r["alternates"].as_array().unwrap() {
            let mut state = lab.initial.clone();
            run(lab, &mut state, &actions(alt));
            assert!(
                engine::solved(&state, &lab.goal),
                "{} {:?}",
                lab.id,
                engine::checks(&state, &lab.goal)
            );
            assert_questions(server(&state), &r["questions"], i == 2);
        }
    }
}
#[test]
fn wrong_partial_and_break_again_cases_fail_their_specific_current_rows() {
    for lab in labs() {
        let r = review(&lab.id);
        for wrong in r["wrong"].as_array().unwrap() {
            let mut state = lab.initial.clone();
            run(&lab, &mut state, &actions(&wrong["actions"]));
            assert!(!engine::solved(&state, &lab.goal));
            let rows = engine::checks(&state, &lab.goal);
            for i in wrong["failed_rows"].as_array().unwrap() {
                let (pass, feedback) = &rows[i.as_u64().unwrap() as usize];
                assert!(!pass, "{} {} intended row", lab.id, wrong["label"]);
                assert!(!feedback.is_empty() && feedback.len() <= 512);
            }
        }
        for partial in r["partial"].as_array().unwrap() {
            let mut state = lab.initial.clone();
            run(&lab, &mut state, &actions(&partial["actions"]));
            let expected: Vec<bool> =
                serde_json::from_value(partial["expected_rows"].clone()).unwrap();
            assert_eq!(
                bools(&lab, &state),
                expected,
                "{} {}",
                lab.id,
                partial["label"]
            );
            assert!(expected.contains(&true) && expected.contains(&false));
        }
        let mut state = State::AuthoritativeDns {
            server: solved(&lab),
        };
        run(&lab, &mut state, &actions(&r["break_again"]["actions"]));
        for i in r["break_again"]["failed_rows"].as_array().unwrap() {
            assert!(!bools(&lab, &state)[i.as_u64().unwrap() as usize]);
        }
    }
}
#[test]
fn every_meaningful_outcome_is_independently_broken_after_a_passing_state() {
    for lab in labs() {
        let good = solved(&lab);
        for (index, req) in requirements(&lab).iter().enumerate() {
            let mut s = good.clone();
            match req {
                Requirement::Records {
                    copy,
                    owner,
                    record_type,
                    ..
                }
                | Requirement::Resolves {
                    copy,
                    name: owner,
                    record_type,
                    ..
                } => {
                    let z = copy_mut(&mut s, *copy);
                    for row in &mut z.records {
                        if row.owner.eq_ignore_ascii_case(owner)
                            && row.data.query_type() == *record_type
                        {
                            row.data = if *record_type == QueryType::Ns {
                                Data::Ns {
                                    target: "different-ns.example.test.".into(),
                                }
                            } else {
                                Data::Ptr {
                                    target: "different-host.example.test.".into(),
                                }
                            };
                        }
                    }
                }
                Requirement::NoAnswer { copy, name, .. } => copy_mut(&mut s, *copy)
                    .records
                    .push(ptr(name, "retired-again.example.test.")),
                Requirement::Published { .. } => s.configured.serial += 1,
                _ => panic!("unexpected reverse outcome"),
            }
            assert!(s.validate().is_ok());
            assert!(!req.check(&s).0, "{} row{index}", lab.id);
            assert!(!engine::solved(
                &State::AuthoritativeDns { server: s },
                &lab.goal
            ));
        }
    }
}
#[test]
fn handover_rejects_equal_serial_staleness_and_checks_each_old_question_without_alias_shortcuts() {
    let lab = labs().remove(2);
    let mut state = lab.initial.clone();
    run(&lab, &mut state, &lab.solution[..2]);
    let mut s = server(&state).clone();
    let rows = requirements(&lab);
    assert!(rows[6].check(&s).0 && !rows[7].check(&s).0 && !rows[8].check(&s).0);
    s.primary.serial = s.secondary.serial;
    assert_eq!(s.update_secondary().unwrap(), Transfer::UnchangedEqual);
    assert!(!rows[7].check(&s).0 && !rows[8].check(&s).0);
    s.configured.serial = s.secondary.serial + 9;
    s.reload_primary().unwrap();
    assert_eq!(s.update_secondary().unwrap(), Transfer::Updated);
    assert!(rows.iter().all(|r| r.check(&s).0));
    // No requested PTR is also valid when another type exists at the retired name.
    let old = review(&lab.id)["retired"]["owner"]
        .as_str()
        .unwrap()
        .to_owned();
    s.configured.records.push(Record {
        owner: old,
        ttl: 60,
        data: Data::A {
            address: "192.0.2.36".parse().unwrap(),
        },
    });
    publish(&mut s, true);
    assert!(rows.iter().all(|r| r.check(&s).0));
}
#[test]
fn separate_examples_fill_only_then_save_publish_experiment_and_repair_are_runnable() {
    for (i, lab) in labs().iter().enumerate() {
        let lesson = lab.dns_teaching.as_ref().unwrap();
        let r = review(&lab.id);
        let assignment = serde_json::to_vec(&lab.initial).unwrap();
        let mut l = Learning::new(lesson);
        let before = l.server.clone();
        l.fill_example(lesson).unwrap();
        assert_eq!(l.server, before);
        assert!(!l.checked);
        assert!(!l.draft.matches(&l.server));
        let expected: Zone = serde_json::from_value(r["example"]["expected_zone"].clone()).unwrap();
        assert!(
            l.draft
                .parse(&l.server.configured.origin)
                .unwrap()
                .equivalent(&expected)
                .unwrap()
        );
        l.draft.save(&mut l.server).unwrap();
        l.server.reload_primary().unwrap();
        if i == 2 {
            l.server.update_secondary().unwrap();
        }
        l.checked = true;
        assert!(l.checks(lesson).iter().all(|r| r.0));
        assert_questions(&l.server, &r["example"]["questions"], i == 2);
        for action in actions(&r["example"]["experiment"]) {
            dlms_learning_lab::dns_workspace::adapter(&mut l.server, &action).unwrap();
        }
        l.draft = Draft::new(&l.server);
        let checks = l.checks(lesson);
        for row in r["example"]["failed_rows"].as_array().unwrap() {
            assert!(!checks[row.as_u64().unwrap() as usize].0);
        }
        for action in actions(&r["example"]["repair"]) {
            dlms_learning_lab::dns_workspace::adapter(&mut l.server, &action).unwrap();
        }
        l.draft = Draft::new(&l.server);
        assert!(l.checks(lesson).iter().all(|r| r.0));
        assert_eq!(serde_json::to_vec(&lab.initial).unwrap(), assignment);
    }
}
#[test]
fn all_original116_specs_and_unit_order_remain_exact_with_typed_historical_goals() {
    let prior: Value =
        serde_json::from_str(include_str!("fixtures/dns-reverse-original116.json")).unwrap();
    let all = pack::bundled().unwrap();
    assert_eq!(prior["original_scenarios"].as_array().unwrap().len(), 116);
    // Preserve the full old catalog order too: older native probes and in-process
    // contextual selections must not be redirected to a different activity.
    let prior_ids: Vec<_> = prior["original_scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        all.iter()
            .take(116)
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        prior_ids
    );
    assert_eq!(
        all.iter()
            .skip(116)
            .take(3)
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        IDS
    );
    for old in prior["original_scenarios"].as_array().unwrap() {
        let id = old["id"].as_str().unwrap();
        let s = all.iter().find(|s| s.id == id).unwrap();
        assert_eq!(s.version as u64, old["version"].as_u64().unwrap());
        let initial: State = serde_json::from_value(old["initial"].clone()).unwrap();
        assert_eq!(s.initial, initial);
        let goal: Goal = serde_json::from_value(old["goal"].clone()).unwrap();
        assert_eq!(
            serde_json::to_value(&s.goal).unwrap(),
            serde_json::to_value(goal).unwrap()
        );
        assert_eq!(serde_json::to_value(&s.solution).unwrap(), old["solution"]);
        assert_eq!(s.unit, old["unit"].as_str().unwrap());
    }
    let units = course::bundled(&all).unwrap();
    let previous = prior["units"]["units"].as_array().unwrap();
    let old_ids: BTreeSet<_> = previous.iter().map(|u| u["id"].as_str().unwrap()).collect();
    let current: Vec<_> = units
        .iter()
        .filter(|u| old_ids.contains(u.id.as_str()))
        .map(|u| serde_json::to_value(u).unwrap())
        .collect();
    assert_eq!(&current, previous);
}
#[test]
fn unfinished_assignment_and_example_drafts_roundtrip_without_changing_real6_7_histories_or_old_contexts()
 {
    let mut all = pack::bundled().unwrap();
    for lab in labs() {
        if !all.iter().any(|s| s.id == lab.id) {
            all.push(lab);
        }
    }
    let baseline: Value =
        serde_json::from_str(include_str!("fixtures/dns-reverse-original116.json")).unwrap();
    let original_ids: BTreeSet<_> = baseline["original_scenarios"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_owned())
        .collect();
    let mut p = progress::decode(
        include_bytes!("fixtures/shell-v7-synthetic-progress-original.json"),
        &all,
    )
    .unwrap();
    for s in pack::bundled()
        .unwrap()
        .into_iter()
        .filter(|s| original_ids.contains(&s.id))
    {
        p.attempts
            .entry(s.id.clone())
            .or_insert_with(|| Attempt::new(&s, 1));
    }
    assert_eq!(p.attempts.len(), 116);
    let old = serde_json::to_value(&p).unwrap();
    assert!(
        old["attempts"]
            .as_object()
            .unwrap()
            .values()
            .any(|a| a["workspace"]["script_history"]["engine_revision"] == 6)
    );
    assert!(old["attempts"].as_object().unwrap().values().any(|a| {
        a["state"]["run"]["engine_revision"] == 7
            || a["workspace"]["script_history_more"]
                .as_array()
                .is_some_and(|v| v.iter().any(|r| r["engine_revision"] == 7))
    }));
    for lab in labs() {
        let mut a = Attempt::new(&lab, 2);
        workspace::prepare(&mut a);
        dns_teaching::prepare(&mut a, lab.dns_teaching.as_ref().unwrap(), true);
        run(&lab, &mut a.state, &lab.solution);
        let w = a.workspace.as_mut().unwrap();
        w.dns_draft = Some(Draft::new(server(&a.state)));
        assert!(progress::current_solved(&a, &lab));
        let w = a.workspace.as_mut().unwrap();
        let d = w.dns_draft.as_mut().unwrap();
        d.text.push_str("\nunfinished assignment");
        d.serial = "bad serial".into();
        d.query_name = "unfinished lookup /".into();
        d.query_type = QueryType::Ptr;
        let example = w.dns_learning.as_mut().unwrap();
        example.draft.text.push_str("\nindependent example draft");
        example.draft.query_type = QueryType::Ptr;
        assert!(!progress::current_solved(&a, &lab));
        p.attempts.insert(lab.id.clone(), a);
    }
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&bytes, &all).unwrap();
    assert_eq!(serde_json::to_vec(&restored).unwrap(), bytes);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress.json");
    progress::save(&path, &restored).unwrap();
    let loaded = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for (id, expected) in old["attempts"].as_object().unwrap() {
        assert_eq!(
            serde_json::to_value(&loaded.attempts[id]).unwrap(),
            *expected
        );
    }
    let untouched = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    let first = labs().remove(0);
    let favorites = restored.favorites.clone();
    restored
        .attempts
        .insert(first.id.clone(), Attempt::new(&first, 3));
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[1]]).unwrap(),
        untouched
    );
    assert_eq!(restored.favorites, favorites);
    for (id, expected) in old["attempts"].as_object().unwrap() {
        assert_eq!(
            serde_json::to_value(&restored.attempts[id]).unwrap(),
            *expected
        );
    }
}
#[test]
fn new_only_backup_roundtrip_keeps_both_drafts_and_rejects_foreign_practice_context() {
    let all = catalog();
    let tasks = labs();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for lab in &tasks {
        let mut a = Attempt::new(lab, 1);
        workspace::prepare(&mut a);
        dns_teaching::prepare(&mut a, lab.dns_teaching.as_ref().unwrap(), true);
        let w = a.workspace.as_mut().unwrap();
        w.dns_draft
            .as_mut()
            .unwrap()
            .text
            .push_str("\nunfinished assignment");
        w.dns_learning
            .as_mut()
            .unwrap()
            .draft
            .text
            .push_str("\nunfinished practice");
        p.attempts.insert(lab.id.clone(), a);
    }
    let expected = serde_json::to_value(&p).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let export = dir.path().join("export.json");
    backup::export(&export, &p, &all).unwrap();
    let candidate =
        backup::inspect(&export, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    assert_eq!(candidate.rechecked, 0);
    assert_eq!(serde_json::to_value(&candidate.progress).unwrap(), expected);
    let target = dir.path().join("progress.json");
    backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&target, &all).unwrap()).unwrap(),
        expected
    );
    let mut bad = expected.clone();
    bad["attempts"][IDS[0]]["workspace"]["dns_learning"] =
        expected["attempts"][IDS[1]]["workspace"]["dns_learning"].clone();
    assert!(progress::decode(&serde_json::to_vec(&bad).unwrap(), &all).is_err());
    let prior_bytes = std::fs::read(&target).unwrap();
    let mut invalid: Progress = serde_json::from_value(expected).unwrap();
    let State::AuthoritativeDns { server } = &mut invalid.attempts.get_mut(IDS[0]).unwrap().state
    else {
        panic!()
    };
    server.primary.origin = "foreign.test.".into();
    assert!(progress::save(&target, &invalid).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), prior_bytes);
}
#[test]
fn malformed_editor_work_is_atomic_and_negative_pack_outcomes_cannot_escape_authority() {
    for lab in labs() {
        let s = solved(&lab);
        let mut d = Draft::new(&s);
        let before = s.clone();
        d.text.push_str("\n$INCLUDE /host/private");
        let mut candidate = s.clone();
        assert!(d.save(&mut candidate).is_err());
        assert_eq!(candidate, before);
        assert!(d.text.contains("$INCLUDE"));
    }
    let mut proposal: Value = serde_json::from_str(PACK).unwrap();
    proposal["scenarios"][2]["goal"]["requirements"][6]["copy"] = "Configured".into();
    assert!(
        pack::load_catalog(&[
            include_str!("../scenarios/starter-v1.json"),
            include_str!("../scenarios/dns-forward-v1.json"),
            &serde_json::to_string(&proposal).unwrap()
        ])
        .is_err()
    );
}
