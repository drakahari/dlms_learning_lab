//! Prospective independent practice checks; no app/progress/UI integration yet.
use dlms_learning_lab::{
    nfs as core, nfs_teaching as teaching, nfs_workspace as work, script_teaching::View,
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
struct Case {
    definition: work::Definition,
    lesson: teaching::Lesson,
}
fn cases() -> Vec<Case> {
    let v: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/nfs-workbench-v1.json")).unwrap();
    serde_json::from_value(v["cases"].clone()).unwrap()
}
fn exact<T: serde::Serialize>(a: &T, b: &T) {
    assert!(
        serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap(),
        "full practice value changed"
    );
}
fn worked(l: &teaching::Lesson) -> teaching::Learning {
    let mut state = teaching::Learning::new(l).unwrap();
    for step in &l.example_actions {
        state.perform(l, &step.action).unwrap();
    }
    assert!(state.checks(l).iter().all(|r| r.0));
    state
}
#[test]
fn all_four_distinct_examples_change_really_and_repair_every_declared_row() {
    for c in cases() {
        assert!(c.lesson.separate_from(&c.definition.initial));
        c.lesson.validate().unwrap();
        let mut p = worked(&c.lesson);
        let before = p.simulation.clone();
        for a in &c.lesson.try_change.actions {
            p.perform(&c.lesson, a).unwrap();
        }
        assert_ne!(p.simulation, before);
        assert_eq!(
            p.checks(&c.lesson).iter().map(|r| r.0).collect::<Vec<_>>(),
            c.lesson.try_change.expected
        );
        for a in &c.lesson.repair {
            p.perform(&c.lesson, a).unwrap();
        }
        assert!(p.checks(&c.lesson).iter().all(|r| r.0));
        assert!(
            !c.definition
                .checks(&c.definition.initial)
                .iter()
                .all(|r| r.0)
        );
    }
}
#[test]
fn intro_fill_only_preserves_saved_applied_live_assignment_and_explicit_action_is_separate() {
    let c = cases().remove(0);
    let assignment = c.definition.initial.clone();
    let mut p = teaching::Learning::new(&c.lesson).unwrap();
    assert_eq!(p.view, View::Introduction);
    let saved = p.simulation.runtime().servers[0].saved.clone();
    let applied = p.simulation.runtime().servers[0].applied.clone();
    let mounts = p.simulation.runtime().mounts.clone();
    p.fill_selected(&c.lesson).unwrap();
    assert_eq!(p.simulation.runtime().servers[0].saved, saved);
    assert_eq!(p.simulation.runtime().servers[0].applied, applied);
    assert_eq!(p.simulation.runtime().mounts, mounts);
    assert!(p.session.entries.is_empty());
    assert_ne!(p.simulation.runtime().servers[0].draft, saved);
    exact(&assignment, &c.definition.initial);
    p.selected_example = 1;
    let before = p.simulation.clone();
    p.fill_selected(&c.lesson).unwrap();
    assert_eq!(p.simulation, before, "filling Save selection does not Save");
    p.perform(&c.lesson, &c.lesson.example_actions[1].action)
        .unwrap();
    assert_ne!(p.simulation, before);
    assert_eq!(p.simulation.runtime().servers[0].applied, applied);
}
#[test]
fn separate_context_pin_and_selected_example_admission_reject_foreign_metadata() {
    let c = cases().remove(0);
    assert!(!c.lesson.separate_from(&c.lesson.definition.initial));
    let mut p = worked(&c.lesson);
    let before = p.clone();
    let foreign = cases().remove(1).lesson;
    assert!(!p.valid_for(&foreign));
    assert!(
        p.perform(&foreign, &foreign.example_actions[0].action)
            .is_err()
    );
    exact(&p, &before);
    p.selected_example = c.lesson.example_actions.len();
    assert!(!p.valid_for(&c.lesson));
    assert!(p.fill_selected(&c.lesson).is_err());
    let mut v = serde_json::to_value(&before).unwrap();
    v["simulation"] = serde_json::to_value(c.definition.initial).unwrap();
    assert!(serde_json::from_value::<teaching::Learning>(v).is_err());
    let mut v = serde_json::to_value(&before).unwrap();
    v["selected_example"] = json!(16);
    assert!(serde_json::from_value::<teaching::Learning>(v).is_err());
}
#[test]
fn full_evolving_practice_two_raw_requests_roundtrip_and_reset_are_independent() {
    for c in cases() {
        let assignment = c.definition.initial.clone();
        let mut p = worked(&c.lesson);
        p.view = View::Assignment;
        p.checked = true;
        p.session.relative = "../unfinished query".into();
        p.session.write_text = "unsent replacement\n".into();
        let bytes = serde_json::to_vec(&p).unwrap();
        let restored = teaching::Learning::from_json(&bytes).unwrap();
        exact(&restored, &p);
        assert!(restored.valid_for(&c.lesson));
        assert!(restored.checks(&c.lesson).iter().all(|r| r.0));
        let mut selected = restored;
        selected.reset(&c.lesson).unwrap();
        assert_eq!(selected.view, View::Example);
        assert_eq!(selected.simulation, c.lesson.definition.initial);
        assert!(selected.session.entries.is_empty());
        assert_eq!(selected.session.relative, "");
        exact(&assignment, &c.definition.initial);
    }
}
#[test]
fn readonly_observation_keeps_checked_truth_actual_mutation_invalidates_and_history_is_not_credit()
{
    let c = cases().remove(0);
    let mut p = worked(&c.lesson);
    p.checked = true;
    let spec = match &c.lesson.definition.requirements[0] {
        work::Requirement::CurrentMount {
            client,
            specification,
        } => (client.clone(), specification.clone()),
        _ => panic!(),
    };
    let profile = p.simulation.inventory().profiles[0].id.clone();
    p.perform(
        &c.lesson,
        &core::Action::ReadExisting {
            client: spec.0,
            mountpoint: spec.1.mountpoint,
            profile,
            relative: "notice.txt".into(),
        },
    )
    .unwrap();
    assert!(p.checked);
    for a in &c.lesson.try_change.actions {
        p.perform(&c.lesson, a).unwrap();
    }
    assert!(!p.checked);
    assert!(!p.checks(&c.lesson).iter().all(|r| r.0));
    p.checked = true;
    assert!(!p.checks(&c.lesson).iter().all(|r| r.0));
}
#[test]
fn oversized_actions_in_each_authored_location_and_wrong_numeric_context_fail_before_new_practice()
{
    let c = cases().remove(0);
    for position in 0..3 {
        let mut l = c.lesson.clone();
        let bad = core::Action::SetExportsDraft {
            server: l.definition.initial.inventory().servers[0].id.clone(),
            text: "x".repeat(core::MAX_DOCUMENT + 1),
        };
        match position {
            0 => l.example_actions[0].action = bad,
            1 => l.try_change.actions[0] = bad,
            _ => l.repair[0] = bad,
        };
        assert!(!l.bounded());
        assert!(teaching::Learning::new(&l).is_err());
    }
    let mut l = c.lesson.clone();
    if let core::Action::SetExportsDraft { server, .. } = &mut l.example_actions[0].action {
        *server = "unknown".into();
    }
    assert!(!l.bounded());
}
#[test]
fn direct_and_decoded_prose_nul_unknown_fields_raw_caps_and_escaped_total_match() {
    let c = cases().remove(0);
    let mut l = c.lesson.clone();
    l.concepts[0].text = "bad\0prose".into();
    assert!(!l.bounded());
    assert!(teaching::Lesson::from_json(&serde_json::to_vec(&l).unwrap()).is_err());
    let mut v = serde_json::to_value(&c.lesson).unwrap();
    v["future"] = json!(true);
    assert!(serde_json::from_value::<teaching::Lesson>(v).is_err());
    assert!(teaching::Lesson::from_json(&vec![b' '; teaching::MAX_LESSON + 1]).is_err());
    assert!(teaching::Learning::from_json(&vec![b' '; teaching::MAX_LEARNING + 1]).is_err());
    let mut l = c.lesson.clone();
    let action = core::Action::SetExportsDraft {
        server: l.definition.initial.inventory().servers[0].id.clone(),
        text: ("\t".repeat(511) + "\n").repeat(16),
    };
    l.example_actions = (0..16)
        .map(|n| teaching::ActionStep {
            title: format!("Source {n}"),
            explanation: "bounded source authoring".into(),
            action: action.clone(),
        })
        .collect();
    l.try_change.actions = vec![action.clone(); 16];
    l.repair = vec![action; 16];
    assert!(
        !l.bounded(),
        "legal scalar/action sizes cannot bypass whole Lesson escaped cap"
    );
}
#[test]
fn learning_component_closure_stays_below_fixed_quota_without_reducing_any_global_cap() {
    let closure = [
        work::MAX_DEFINITION,
        core::MAX_STATE,
        work::MAX_SESSION,
        4096,
    ]
    .into_iter()
    .sum::<usize>();
    assert!(closure <= teaching::MAX_LEARNING);
    assert_eq!(teaching::MAX_LEARNING, 1024 * 1024);
    assert_eq!(work::MAX_ATTEMPT, 2 * 1024 * 1024);
    for c in cases() {
        let p = worked(&c.lesson);
        assert!(serde_json::to_vec(&p).unwrap().len() <= teaching::MAX_LEARNING);
        let mut bad = p.clone();
        bad.session.entries.push(work::Entry {
            operation: "Read".into(),
            detail: "\0".into(),
        });
        assert!(!bad.valid());
        assert!(teaching::Learning::from_json(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
}

#[test]
fn fill_reconciles_disjoint_client_selections_but_preserves_unrelated_raw_inputs() {
    let c = cases().remove(0);
    let mut l = c.lesson;
    let mut inventory = l.definition.initial.inventory().clone();
    let old = inventory.clients[0].id.clone();
    let new = inventory.clients[1].id.clone();
    let old_mp = inventory.clients[0].mountpoints[1].clone();
    let new_mp = "/mnt/gallery0/new-reference".to_owned();
    let writer = inventory
        .profiles
        .iter()
        .find(|p| p.id.ends_with("writer"))
        .unwrap()
        .id
        .clone();
    let helper = inventory
        .profiles
        .iter()
        .find(|p| p.id.ends_with("helper"))
        .unwrap()
        .id
        .clone();
    inventory.clients[0].profiles = vec![writer.clone()];
    inventory.clients[1].profiles = vec![helper.clone()];
    inventory.clients[1].mountpoints = vec![new_mp.clone()];
    let new_ip = inventory.clients[1].address;
    l.definition.initial =
        core::State::new(inventory, l.definition.initial.runtime().clone()).unwrap();
    if let core::Action::SetExportsDraft { text, .. } = &mut l.example_actions[0].action {
        let (first, rest) = text.split_once('\n').unwrap();
        *text = format!("{first} {new_ip}(ro)\n{rest}");
    }
    let root = l.definition.initial.inventory().roots[1].path.clone();
    let server = l.definition.initial.inventory().servers[0].id.clone();
    let index = l.example_actions.len();
    for action in [
        core::Action::SaveFstabDraft {
            client: new.clone(),
        },
        core::Action::MountExplicit {
            client: new.clone(),
            specification: core::MountSpec {
                server,
                root,
                mountpoint: new_mp.clone(),
                access: core::Access::Ro,
            },
        },
        core::Action::ReadExisting {
            client: new.clone(),
            mountpoint: new_mp.clone(),
            profile: helper.clone(),
            relative: "notice.txt".into(),
        },
    ] {
        l.example_actions.push(teaching::ActionStep {
            title: "Additional selected-client action".into(),
            explanation: "Use only the supplied new client's own profile and target".into(),
            action,
        });
    }
    l.validate().unwrap();
    let mut p = teaching::Learning::new(&l).unwrap();
    p.session.client = old.clone();
    p.session.mountpoint = old_mp.clone();
    p.session.profile = writer.clone();
    p.session.relative = "../unfinished request".into();
    p.session.write_text = "unsent replacement".into();
    assert!(p.valid_for(&l));
    p.selected_example = index;
    let state = p.simulation.clone();
    let entries = p.session.entries.clone();
    p.fill_selected(&l).unwrap();
    assert_eq!(p.session.client, new);
    assert_eq!(p.session.profile, "");
    assert_eq!(p.session.mountpoint, "");
    assert_eq!(p.session.relative, "../unfinished request");
    assert_eq!(p.session.write_text, "unsent replacement");
    assert_eq!(p.session.entries, entries);
    assert_eq!(p.simulation, state);
    p.session.profile = helper.clone();
    p.session.mountpoint = new_mp.clone();
    p.fill_selected(&l).unwrap();
    assert_eq!(p.session.profile, helper);
    assert_eq!(
        p.session.mountpoint, new_mp,
        "still-valid dependent selections remain exact"
    );
    p.session.client = old;
    p.session.profile = writer;
    p.session.mountpoint = old_mp;
    p.selected_example = index + 2;
    p.fill_selected(&l).unwrap();
    assert_eq!(p.session.client, new);
    assert_eq!(p.session.profile, helper, "action-owned profile wins");
    assert_eq!(p.session.mountpoint, new_mp);
    assert_eq!(p.session.relative, "notice.txt");
    assert_eq!(p.session.write_text, "unsent replacement");
    assert_eq!(p.simulation, state);
    exact(&c.definition.initial, &c.definition.initial.clone());
}
fn large_valid_write_lesson() -> teaching::Lesson {
    fn replace(v: &mut serde_json::Value, old: &str, new: &str) {
        match v {
            serde_json::Value::String(s) if s == old => *s = new.into(),
            serde_json::Value::Array(a) => {
                for x in a {
                    replace(x, old, new)
                }
            }
            serde_json::Value::Object(o) => {
                for x in o.values_mut() {
                    replace(x, old, new)
                }
            }
            _ => {}
        }
    }
    let c = cases().remove(1);
    let mut v = serde_json::to_value(c.lesson).unwrap();
    replace(&mut v, "gallery1 published result\n", &"\u{1}".repeat(4096));
    serde_json::from_value(v).unwrap()
}
#[test]
fn escaped_annotated_fill_has_identical_positive_control_and_preserves_full_value_on_capacity_refusal()
 {
    let l = large_valid_write_lesson();
    l.validate().unwrap();
    let index = l
        .example_actions
        .iter()
        .position(|s| matches!(s.action, core::Action::WriteExisting { .. }))
        .unwrap();
    let mut low = teaching::Learning::new(&l).unwrap();
    low.selected_example = index;
    let state = low.simulation.clone();
    low.fill_selected(&l).unwrap();
    assert_eq!(low.session.write_text, "\u{1}".repeat(4096));
    assert_eq!(low.simulation, state);
    assert!(
        low.session.entries.is_empty(),
        "Fill does not run a write or append output"
    );
    let mut full = teaching::Learning::new(&l).unwrap();
    full.selected_example = index;
    full.session.entries = (0..16)
        .map(|n| work::Entry {
            operation: format!("Imported display {n}"),
            detail: "x".repeat(3900),
        })
        .collect();
    assert!(full.valid_for(&l));
    let before = full.clone();
    let error = full.fill_selected(&l).unwrap_err();
    assert!(error.contains("aggregate"));
    exact(&full, &before);
    assert!(full.valid_for(&l));
}
