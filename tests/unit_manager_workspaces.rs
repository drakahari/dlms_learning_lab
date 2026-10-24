//! Unbundled pure adapter tests; no host services, commands or account effects.
use dlms_learning_lab::{
    unit_manager::{self as core, Action, State},
    unit_manager_teaching::{Learning, Lesson},
    unit_manager_workspace::{
        self as workspace, Definition, Draft, EditAction, Phase, Requirement,
    },
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Fixture {
    definition: Definition,
    lesson: Lesson,
    solution: Vec<EditAction>,
    alternate: Vec<EditAction>,
    names: Value,
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/unit-manager-workspace-preview.json")).unwrap()
}
fn run(def: &Definition, actions: &[EditAction]) -> State {
    let mut s = def.initial.clone();
    for a in actions {
        workspace::apply(&mut s, &def.initial, a).unwrap();
    }
    s
}
fn manager(s: &mut State, a: Action) {
    s.apply(&a).unwrap();
}
fn pass(def: &Definition, s: &State) -> bool {
    workspace::solved(def, s, &Draft::new(s).unwrap())
}
fn name<'a>(f: &'a Fixture, key: &str) -> &'a str {
    f.names[key].as_str().unwrap()
}

#[test]
fn unbundled_reference_and_before_after_alternative_meet_current_and_fresh_outcomes() {
    let f = fixture();
    f.definition.validate().unwrap();
    f.lesson.validate().unwrap();
    assert!(f.lesson.distinct_from(&f.definition.initial));
    assert!(!pass(&f.definition, &f.definition.initial));
    let a = run(&f.definition, &f.solution);
    let b = run(&f.definition, &f.alternate);
    assert_ne!(a.documents(), b.documents());
    assert!(pass(&f.definition, &a) && pass(&f.definition, &b));
    let before = serde_json::to_vec(&a).unwrap();
    let (fresh, report, _, ordering) = workspace::inspect_fresh(&a).unwrap();
    assert!(report.requested_succeeded().unwrap());
    assert!(ordering.precedes(name(&f, "prep"), name(&f, "app")));
    assert!(fresh.result(name(&f, "app")).unwrap().is_some());
    assert_eq!(serde_json::to_vec(&a).unwrap(), before);
    assert!(
        !dlms_learning_lab::pack::bundled()
            .unwrap()
            .iter()
            .any(|s| s.id == "unit-manager-workspace-preview")
    );
}
#[test]
fn raw_save_is_atomic_and_keeps_malformed_unicode_without_reloading_or_running() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    let loaded = s.loaded_sources().clone();
    let runtime = s.runtime().clone();
    let mut draft = Draft::new(&s).unwrap();
    draft.documents.insert(
        name(&f, "app").into(),
        "[Unit]\nDescription=éß\nNotSupported=yes\n".into(),
    );
    draft.apply(&mut s).unwrap();
    assert_eq!(s.documents(), &draft.documents);
    assert_eq!(s.loaded_sources(), &loaded);
    assert_eq!(s.runtime(), &runtime);
    assert!(!pass(&f.definition, &s));
    let encoded = serde_json::to_string(&draft).unwrap();
    let round: Draft = serde_json::from_str(&encoded).unwrap();
    assert_eq!(round, draft);
    let saved = serde_json::to_vec(&s).unwrap();
    let mut bad = draft.clone();
    bad.documents.remove(name(&f, "app"));
    assert!(bad.apply(&mut s).is_err());
    assert_eq!(serde_json::to_vec(&s).unwrap(), saved);
    let mut bad = draft;
    bad.configuration.insert(
        name(&f, "app_config").into(),
        "x".repeat(core::MAX_CONFIG_BYTES + 1),
    );
    assert!(bad.apply(&mut s).is_err());
    assert_eq!(serde_json::to_vec(&s).unwrap(), saved);
}
#[test]
fn pending_edit_save_reload_and_restart_are_separate_success_boundaries() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    let mut draft = Draft::new(&s).unwrap();
    draft
        .documents
        .get_mut(name(&f, "app"))
        .unwrap()
        .push_str("# harmless supported comment\n");
    assert!(pass(&f.definition, &s));
    assert!(!workspace::solved(&f.definition, &s, &draft));
    draft.apply(&mut s).unwrap();
    assert!(!pass(&f.definition, &s));
    manager(&mut s, Action::DaemonReload {});
    assert!(!pass(&f.definition, &s));
    manager(
        &mut s,
        Action::Stop {
            unit: name(&f, "app").into(),
        },
    );
    manager(
        &mut s,
        Action::Start {
            unit: name(&f, "app").into(),
        },
    );
    assert!(!pass(&f.definition, &s)); // simple start was only Dispatched
    manager(&mut s, Action::AdvancePrograms {});
    assert!(pass(&f.definition, &s));
    // Identical initially loaded/saved text is coherence, with no action-history requirement.
    let initial = &f.definition.initial;
    assert!(!initial.needs_daemon_reload(name(&f, "app")).unwrap());
}
#[test]
fn unchanged_running_output_cannot_hide_changed_application_inputs() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    manager(
        &mut s,
        Action::SaveConfig {
            path: name(&f, "app_config").into(),
            text: "PORT=8089\n".into(),
        },
    );
    assert_eq!(s.result(name(&f, "app")).unwrap().unwrap().port, Some(8088));
    assert!(!pass(&f.definition, &s));
    manager(
        &mut s,
        Action::SaveConfig {
            path: name(&f, "app_config").into(),
            text: "PORT=8088\n".into(),
        },
    );
    assert!(pass(&f.definition, &s)); // Returning to equal consumed values needs no invented history.
}
#[test]
fn running_now_does_not_replace_enabled_default_or_fresh_positive_workloads() {
    let f = fixture();
    let solved = run(&f.definition, &f.solution);
    let mut disabled = solved.clone();
    manager(
        &mut disabled,
        Action::Disable {
            unit: name(&f, "app").into(),
        },
    );
    assert_eq!(
        disabled.result(name(&f, "app")).unwrap().unwrap().port,
        Some(8088)
    );
    assert!(!pass(&f.definition, &disabled));
    let mut wrong = solved.clone();
    manager(
        &mut wrong,
        Action::SetDefault {
            target: name(&f, "spare").into(),
        },
    );
    assert!(!pass(&f.definition, &wrong));
    let (fresh, _, _, _) = workspace::inspect_fresh(&wrong).unwrap();
    assert!(fresh.artifacts().is_empty());
    assert!(solved.artifacts().contains_key(name(&f, "artifact")));
}
#[test]
fn explicit_preservation_catches_collateral_documents_inputs_and_boot_links() {
    let f = fixture();
    let solved = run(&f.definition, &f.solution);
    let mut wrong = solved.clone();
    let mut draft = Draft::new(&wrong).unwrap();
    draft
        .documents
        .get_mut(name(&f, "target"))
        .unwrap()
        .push_str("# unrelated edit\n");
    draft.apply(&mut wrong).unwrap();
    manager(&mut wrong, Action::DaemonReload {});
    assert!(!pass(&f.definition, &wrong));
    assert!(
        workspace::checks(&f.definition, &wrong, &Draft::new(&wrong).unwrap())
            .iter()
            .any(|(p, m)| !p && m.contains("document unchanged"))
    );
    let mut wrong = solved.clone();
    manager(
        &mut wrong,
        Action::Enable {
            unit: name(&f, "prep").into(),
        },
    );
    assert!(!pass(&f.definition, &wrong));
    assert!(wrong.result(name(&f, "app")).unwrap().is_some());
    let mut wrong = solved;
    manager(
        &mut wrong,
        Action::SaveConfig {
            path: "/etc/work/observer.conf".into(),
            text: "PORT=6061\n".into(),
        },
    );
    assert!(!pass(&f.definition, &wrong));
}
#[test]
fn fresh_order_inspector_rejects_full_graph_path_through_unstarted_intermediary() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    let mut draft = Draft::new(&s).unwrap();
    let app = name(&f, "app");
    let prep = name(&f, "prep");
    let observer = name(&f, "observer");
    let spare = name(&f, "spare");
    let t = draft.documents.get_mut(app).unwrap();
    *t = t.replace(&format!("After={prep}"), &format!("After={observer}"));
    draft
        .documents
        .get_mut(spare)
        .unwrap()
        .push_str(&format!("After={prep}\nBefore={app}\n"));
    draft.apply(&mut s).unwrap();
    manager(&mut s, Action::DaemonReload {});
    assert!(s.ordered_before(prep, app).unwrap());
    let (fresh, _, _, order) = workspace::inspect_fresh(&s).unwrap();
    assert!(!order.contains(spare));
    assert!(!order.precedes(prep, app));
    assert_eq!(fresh.result(app).unwrap().unwrap().port, Some(8088)); // lucky layer order cannot earn ordering credit
    assert!(
        workspace::checks(&f.definition, &s, &Draft::new(&s).unwrap())
            .iter()
            .any(|(p, m)| !p && m.contains("actual transaction ordering path"))
    );
}
#[test]
fn old_artifact_does_not_hide_a_current_failed_producer_even_after_input_is_restored() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    manager(
        &mut s,
        Action::Stop {
            unit: name(&f, "prep").into(),
        },
    );
    manager(
        &mut s,
        Action::SaveConfig {
            path: name(&f, "prepare_config").into(),
            text: "WRONG=ready\n".into(),
        },
    );
    manager(
        &mut s,
        Action::Start {
            unit: name(&f, "prep").into(),
        },
    );
    assert!(s.artifacts().contains_key(name(&f, "artifact")));
    manager(
        &mut s,
        Action::SaveConfig {
            path: name(&f, "prepare_config").into(),
            text: "VALUE=ready\n".into(),
        },
    );
    let checks = workspace::checks(&f.definition, &s, &Draft::new(&s).unwrap());
    assert!(
        checks
            .iter()
            .any(|(p, m)| !p && m.starts_with(name(&f, "artifact")))
    );
}
#[test]
fn fill_only_examples_and_practice_edits_do_not_touch_assignment_or_earn_credit() {
    let f = fixture();
    let assignment = run(&f.definition, &f.solution);
    let before = serde_json::to_vec(&assignment).unwrap();
    let mut l = Learning::new(&f.lesson).unwrap();
    let original = l.state.clone();
    l.fill_worked(&f.lesson).unwrap();
    assert_eq!(l.state, original);
    assert!(l.transcript.is_empty() && !l.checked);
    assert!(!l.draft.matches(&l.state));
    assert!(l.checks(&f.lesson).is_empty());
    l.save_draft(&f.lesson).unwrap();
    assert!(
        l.state
            .needs_daemon_reload("sample-application.service")
            .unwrap()
    );
    for a in f.lesson.solution.iter().skip(1) {
        l.execute(&f.lesson, a).unwrap();
    }
    l.checked = true;
    assert!(l.checks(&f.lesson).iter().all(|c| c.0));
    for a in &f.lesson.try_actions {
        l.execute(&f.lesson, a).unwrap();
    }
    assert!(l.checks(&f.lesson).iter().any(|c| !c.0));
    for a in &f.lesson.repair {
        l.execute(&f.lesson, a).unwrap();
    }
    assert!(l.checks(&f.lesson).iter().all(|c| c.0));
    l.draft
        .configuration
        .get_mut("/etc/sample/application.conf")
        .unwrap()
        .push_str("# practice draft\n");
    let encoded = serde_json::to_string(&l).unwrap();
    let resumed: Learning = serde_json::from_str(&encoded).unwrap();
    assert!(resumed.valid_for(&f.lesson));
    assert_eq!(resumed.draft, l.draft);
    let reset = Learning::new(&f.lesson).unwrap();
    assert_eq!(reset.state, f.lesson.definition.initial);
    assert_eq!(serde_json::to_vec(&assignment).unwrap(), before);
    assert!(pass(&f.definition, &assignment));
}
#[test]
fn strict_maps_foreign_context_and_bounded_outcomes_reject_without_touching_originals() {
    let f = fixture();
    let mut s = run(&f.definition, &f.solution);
    let before = s.clone();
    let mut wrong = Draft::new(&s).unwrap();
    wrong
        .documents
        .insert("foreign.service".into(), String::new());
    assert!(wrong.apply(&mut s).is_err());
    assert_eq!(s, before);
    let text = serde_json::to_string(&Draft::new(&s).unwrap()).unwrap();
    let duplicate = text.replacen("\"version\":1", "\"version\":1,\"version\":1", 1);
    assert!(serde_json::from_str::<Draft>(&duplicate).is_err());
    let duplicate_slot = text.replacen(
        "\"documents\":{",
        "\"documents\":{\"work-application.service\":\"\",",
        1,
    );
    assert!(serde_json::from_str::<Draft>(&duplicate_slot).is_err());
    let mut encoded = serde_json::to_value(&f.definition).unwrap();
    encoded["requirements"] = Value::Array(vec![json!({"kind":"SavedConfigurationValid"}); 49]);
    assert!(serde_json::from_value::<Definition>(encoded).is_err());
    let mut raw = serde_json::to_value(&s).unwrap();
    raw["inventory"]["fixture"] = json!("foreign-fixture");
    let foreign: State = serde_json::from_value(raw).unwrap();
    assert!(!pass(&f.definition, &foreign));
    let mut l = Learning::new(&f.lesson).unwrap();
    l.state = foreign;
    assert!(!l.valid_for(&f.lesson));
    let mut invalid = f.definition.clone();
    invalid.requirements.push(Requirement::Runtime {
        unit: name(&f, "app").into(),
        phase: Phase::Inactive,
        port: None,
        value: None,
    });
    assert!(invalid.validate().is_err());
    let mut invalid = f.definition;
    invalid.fresh.requirements.clear();
    assert!(invalid.validate().is_err());
}

#[test]
fn direct_oversized_teaching_draft_or_action_is_rejected_before_any_state_change() {
    let f = fixture();
    let mut learning = Learning::new(&f.lesson).unwrap();
    let before = learning.state.clone();
    let selected = learning.draft.selected_unit.clone();
    let edited = learning.draft.selected_unit.clone();
    let too_long = "x".repeat(core::MAX_UNIT_BYTES + 1);
    learning.draft.documents.insert(edited.clone(), too_long);
    assert!(!learning.valid_for(&f.lesson));
    assert!(learning.save_draft(&f.lesson).is_err());
    assert!(learning.fill_worked(&f.lesson).is_err());
    assert_eq!(learning.state, before);
    assert_eq!(
        learning.draft.documents[&edited].len(),
        core::MAX_UNIT_BYTES + 1
    );
    assert_eq!(learning.draft.selected_unit, selected);
    assert!(learning.transcript.is_empty());

    let mut healthy = Learning::new(&f.lesson).unwrap();
    let original = healthy.state.clone();
    let action = EditAction::Manager {
        action: Action::SaveUnit {
            unit: healthy.draft.selected_unit.clone(),
            text: "x".repeat(core::MAX_UNIT_BYTES + 1),
        },
    };
    assert!(healthy.execute(&f.lesson, &action).is_err());
    assert_eq!(healthy.state, original);
    assert!(healthy.transcript.is_empty());
    let mut bad_action = healthy.draft.clone();
    bad_action
        .documents
        .insert("foreign.service".into(), String::new());
    assert!(
        healthy
            .execute(&f.lesson, &EditAction::Save { draft: bad_action })
            .is_err()
    );
    assert_eq!(healthy.state, original);
}

#[test]
fn direct_ten_thousand_action_lessons_cannot_enter_any_replay_or_clone_api() {
    let f = fixture();
    let mut huge = f.lesson.clone();
    huge.solution = (0..10_000)
        .map(|_| EditAction::Manager {
            action: Action::AdvancePrograms {},
        })
        .collect();
    assert!(huge.validate_bounds().is_err());
    assert!(huge.validate().is_err());
    assert!(Learning::new(&huge).is_err());
    let mut learning = Learning::new(&f.lesson).unwrap();
    let original = learning.state.clone();
    let draft = learning.draft.clone();
    assert!(!learning.valid_for(&huge));
    assert!(learning.fill_worked(&huge).is_err());
    assert!(learning.save_draft(&huge).is_err());
    assert!(
        learning
            .execute(
                &huge,
                &EditAction::Manager {
                    action: Action::AdvancePrograms {}
                }
            )
            .is_err()
    );
    assert_eq!(learning.state, original);
    assert_eq!(learning.draft, draft);
    assert!(learning.transcript.is_empty());

    let mut boundary = f.lesson.clone();
    while boundary.solution.len() < workspace::MAX_ACTIONS {
        boundary.solution.push(EditAction::Manager {
            action: Action::AdvancePrograms {},
        });
    }
    assert!(boundary.validate_bounds().is_ok());
    boundary.solution.push(EditAction::Manager {
        action: Action::AdvancePrograms {},
    });
    assert!(boundary.validate_bounds().is_err());
    let mut bad_argument = f.lesson;
    bad_argument.solution[0] = EditAction::Manager {
        action: Action::SaveConfig {
            path: "/etc/sample/application.conf".into(),
            text: "x".repeat(core::MAX_CONFIG_BYTES + 1),
        },
    };
    assert!(bad_argument.validate_bounds().is_err());
    assert!(Learning::new(&bad_argument).is_err());
}

#[test]
fn exact_practice_origin_is_pinned_but_legitimate_evolving_inputs_can_coincide() {
    let f = fixture();
    let mut learning = Learning::new(&f.lesson).unwrap();
    let assignment = run(&f.definition, &f.solution);
    let assignment_before = assignment.clone();
    let mut other = f.lesson.clone();
    manager(
        &mut other.definition.initial,
        Action::SaveConfig {
            path: "/etc/sample/application.conf".into(),
            text: "PORT=different-original-input\n".into(),
        },
    );
    assert!(learning.state.same_context(&other.definition.initial));
    assert!(!learning.valid_for(&other));
    let encoded = serde_json::to_string(&learning).unwrap();
    assert!(Learning::from_json(&encoded, &other).is_err());
    learning
        .draft
        .configuration
        .insert("/etc/sample/application.conf".into(), "PORT=8088\n".into());
    learning.save_draft(&f.lesson).unwrap(); // Same current number as assignment, independent state.
    assert!(learning.valid_for(&f.lesson));
    assert_eq!(
        learning.state.configuration()["/etc/sample/application.conf"],
        assignment.configuration()["/etc/work/application.conf"]
    );
    assert_eq!(assignment, assignment_before);
    assert_eq!(learning.fixture, f.lesson.definition.initial);
    assert_eq!(
        f.definition.initial.configuration()["/etc/work/application.conf"],
        "PORT=not-a-number\n"
    );
    let round = Learning::from_json(&serde_json::to_string(&learning).unwrap(), &f.lesson).unwrap();
    assert_eq!(round.draft, learning.draft);
}

fn pad_escaped(mut text: String, limit: usize) -> String {
    while text.len() < limit {
        let remaining = limit - text.len();
        if remaining == 1 {
            text.push('\n');
            continue;
        }
        let count = (remaining - 2).min(510);
        text.push('#');
        for i in 0..count {
            text.push(if i % 2 == 0 { '"' } else { '\\' });
        }
        text.push('\n');
    }
    text
}
fn maximum_bounded_lesson() -> Lesson {
    use dlms_learning_lab::unit_manager::{Inventory, Program};
    use std::collections::{BTreeMap, BTreeSet};
    let tasks = (0..8)
        .map(|i| format!("{}{}.service", "s".repeat(55), i))
        .collect::<Vec<_>>();
    let targets = (0..8)
        .map(|i| format!("{}{}.target", "t".repeat(56), i))
        .collect::<Vec<_>>();
    let paths = (0..16)
        .map(|i| format!("/etc/{}{}", "c".repeat(121), i))
        .collect::<Vec<_>>();
    let artifacts = (0..8)
        .map(|i| format!("{}{}", "a".repeat(63), i))
        .collect::<Vec<_>>();
    let value = "\"\\".repeat(128);
    let key = "V".repeat(32);
    let mut inventory = Inventory {
        schema_version: 1,
        fixture: "m".repeat(64),
        units: tasks.iter().chain(&targets).cloned().collect(),
        config_paths: paths.iter().cloned().collect(),
        programs: BTreeMap::new(),
        artifact_ids: artifacts.iter().cloned().collect(),
    };
    let mut documents = BTreeMap::new();
    let configuration = paths
        .iter()
        .map(|p| {
            (
                p.clone(),
                pad_escaped(format!("{key}={value}\n"), core::MAX_CONFIG_BYTES),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (i, unit) in tasks.iter().enumerate() {
        let exec = format!("/sim/bin/{}{}", "p".repeat(86), i);
        inventory.programs.insert(
            exec.clone(),
            Program::ValueTask {
                config: paths[i].clone(),
                key: key.clone(),
                artifact: artifacts[i].clone(),
                prerequisite: None,
            },
        );
        documents.insert(unit.clone(),pad_escaped(format!("[Unit]\nDescription={}\n[Service]\nType=oneshot\nExecStart={exec}\nRemainAfterExit=yes\n","d".repeat(160)),core::MAX_UNIT_BYTES));
    }
    for (i, unit) in targets.iter().enumerate() {
        let mut text = "[Unit]\nDescription=Maximum bounded target\n".to_owned();
        if i == 0 {
            for other in tasks.iter().chain(targets.iter().skip(1)) {
                text.push_str(&format!("Wants={other}\n"));
            }
        }
        documents.insert(unit.clone(), pad_escaped(text, core::MAX_UNIT_BYTES));
    }
    let mut worked = State::new(
        inventory,
        documents.clone(),
        configuration.clone(),
        documents,
        targets[0].clone(),
    )
    .unwrap();
    manager(
        &mut worked,
        Action::Start {
            unit: targets[0].clone(),
        },
    );
    assert_eq!(worked.runtime().len(), 16);
    assert_eq!(worked.artifacts().len(), 8);
    let fixed = Draft::new(&worked).unwrap();
    let mut raw = serde_json::to_value(&worked).unwrap();
    for v in raw["documents"].as_object_mut().unwrap().values_mut() {
        *v = Value::String("\0".repeat(core::MAX_UNIT_BYTES));
    }
    for v in raw["configuration"].as_object_mut().unwrap().values_mut() {
        *v = Value::String("\0".repeat(core::MAX_CONFIG_BYTES));
    }
    let initial: State = serde_json::from_value(raw).unwrap();
    let runtime = Requirement::Runtime {
        unit: tasks[0].clone(),
        phase: Phase::Exited,
        port: None,
        value: Some(serde_json::from_value(json!(value)).unwrap()),
    };
    let execution = Requirement::ExecutionCurrent {
        unit: tasks[0].clone(),
    };
    let definition = Definition {
        initial,
        requirements: vec![
            Requirement::SavedConfigurationValid {},
            runtime.clone(),
            execution.clone(),
        ],
        fresh: workspace::Fresh {
            requirements: vec![runtime, execution],
            started: tasks.iter().chain(&targets).cloned().collect(),
            ordering: vec![workspace::Ordering {
                before: tasks[0].clone(),
                after: targets[0].clone(),
            }],
        },
        preservation: workspace::Preservation {
            unit_documents: BTreeSet::new(),
            configuration: BTreeSet::new(),
            preserve_other_boot_links: true,
            editable_boot_links: vec![],
            default_target: true,
        },
    };
    let action = EditAction::Save { draft: fixed };
    Lesson {prerequisite_note:"Synthetic maximum-size metadata; not a bundled learner lesson.".into(),concepts:vec![dlms_learning_lab::script_teaching::Concept {title:"Separate saved and consumed data".into(),text:"Saving identical original inputs can restore coherence without inventing execution history. This fixture exists only to check bounded serialization.".into()}],example_description:"Restore all supplied raw documents and configuration without touching their loaded/current snapshots.".into(),definition,solution:vec![action.clone()],example_actions:vec![dlms_learning_lab::unit_manager_teaching::AnnotatedAction {title:"Save supplied example source".into(),explanation:"This typed save restores the exact known current source. It does not execute a command or award assignment credit.".into(),action}],try_change:"Stop only the first oneshot; its required active/exited phase becomes inactive. Start that unit to repair.".into(),try_actions:vec![EditAction::Manager {action:Action::Stop {unit:tasks[0].clone()}}],try_outcomes:vec![Requirement::Runtime {unit:tasks[0].clone(),phase:Phase::Inactive,port:None,value:None}],repair:vec![EditAction::Manager {action:Action::Start {unit:tasks[0].clone()}}]}
}
#[test]
fn maximum_escaped_two_state_learning_and_synthetic_envelope_roundtrip_with_honest_caps() {
    use dlms_learning_lab::unit_manager_teaching::{self as teaching, Message};
    let lesson = maximum_bounded_lesson();
    lesson.validate().unwrap();
    // Module-level reader cap differs from the future whole-pack2MB cap.
    // No claim is made that all maximum fields can fit one normal scenario pack.
    for encoded in [
        serde_json::to_string(&lesson).unwrap(),
        serde_json::to_string_pretty(&lesson).unwrap(),
    ] {
        assert!(encoded.len() <= teaching::MAX_LESSON_BYTES);
        Lesson::from_json(&encoded).unwrap();
        println!("maximum bounded Lesson bytes={}", encoded.len());
    }
    let mut learning = Learning::new(&lesson).unwrap();
    learning.transcript = (0..100)
        .map(|_| Message {
            failed: true,
            text: "\0".repeat(256),
        })
        .collect();
    assert!(learning.valid_for(&lesson));
    for encoded in [
        serde_json::to_string(&learning).unwrap(),
        serde_json::to_string_pretty(&learning).unwrap(),
    ] {
        assert!(encoded.len() <= teaching::MAX_LEARNING_BYTES);
        let round = Learning::from_json(&encoded, &lesson).unwrap();
        assert_eq!(round.fixture, learning.fixture);
        assert_eq!(round.state, learning.state);
        assert_eq!(round.draft, learning.draft);
        println!(
            "maximum escaped2State+draft Learning bytes={}",
            encoded.len()
        );
    }
    // A size-only surrounding envelope, explicitly NOT the app's Progress type.
    // Real profile compatibility remains deferred until optional hooks are added.
    #[derive(serde::Serialize, serde::Deserialize)]
    struct SyntheticSizeEnvelope {
        synthetic_not_app_progress: bool,
        learning: Learning,
    }
    let envelope = SyntheticSizeEnvelope {
        synthetic_not_app_progress: true,
        learning,
    };
    let encoded = serde_json::to_vec_pretty(&envelope).unwrap();
    assert!(encoded.len() <= 16 * 1024 * 1024);
    let decoded: SyntheticSizeEnvelope = serde_json::from_slice(&encoded).unwrap();
    assert!(decoded.synthetic_not_app_progress);
    assert!(decoded.learning.valid_for(&lesson));
    assert!(Learning::from_json(&" ".repeat(teaching::MAX_LEARNING_BYTES + 1), &lesson).is_err());
    assert!(Lesson::from_json(&" ".repeat(teaching::MAX_LESSON_BYTES + 1)).is_err());
}
