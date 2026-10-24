//! Unbundled integration fixture; not additional authored curriculum.
use dlms_learning_lab::{
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    script_teaching::View,
    storage,
    storage_teaching::{self, Learning},
    storage_workspace::{self as model, Draft, Requirement},
    workspace,
};

fn scenario() -> pack::Scenario {
    pack::load(include_str!("fixtures/storage-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn storage(s: &State) -> &storage::State {
    let State::PersistentMounts { storage } = s else {
        panic!()
    };
    storage
}
fn storage_mut(s: &mut State) -> &mut storage::State {
    let State::PersistentMounts { storage } = s else {
        panic!()
    };
    storage
}
fn attempt(s: &pack::Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    storage_teaching::prepare(&mut a, s.storage_teaching.as_ref().unwrap(), true);
    a
}
fn solved(s: &pack::Scenario) -> Attempt {
    let mut a = attempt(s);
    for command in &s.solution {
        engine::execute(&mut a.state, command).unwrap();
    }
    a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
    a
}
fn records(a: Attempt, s: &pack::Scenario) -> Progress {
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    p.favorites.insert(s.id.clone());
    p
}

#[test]
fn fixture_is_reachable_but_unbundled_and_teaching_is_required() {
    let s = scenario();
    assert!(!engine::solved(&s.initial, &s.goal));
    assert!(progress::current_solved(&solved(&s), &s));
    let all = pack::bundled().unwrap();
    assert_eq!(all.len(), 102);
    assert!(all.iter().all(|s| s.id != "test-storage-workspace"));
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/storage-workspace-preview.json")).unwrap();
    value["scenarios"][0]
        .as_object_mut()
        .unwrap()
        .remove("storage_teaching");
    assert!(pack::load(&value.to_string()).is_err());
}

#[test]
fn configuration_diagnosis_does_not_repeat_one_source_error_for_dependent_outcomes() {
    let s = scenario();
    let state = storage(&s.initial);
    assert!(
        model::checks(state, requirements(&s))
            .iter()
            .filter(|c| !c.0)
            .count()
            > 1
    );
    let issues = model::issues(state, requirements(&s));
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("No exact source match"));
}

#[test]
fn save_live_state_pending_drafts_and_current_invalidation_are_independent() {
    let s = scenario();
    let mut a = attempt(&s);
    let first = model::Action::Save {
        text: storage(&solved(&s).state).configuration().into(),
    };
    first.apply(storage_mut(&mut a.state)).unwrap();
    assert!(!engine::solved(&a.state, &s.goal));
    let checks = engine::checks(&a.state, &s.goal);
    assert!(checks.iter().any(|c| c.0));
    assert!(checks.iter().any(|c| !c.0));
    model::command(storage_mut(&mut a.state), "mount /work").unwrap();
    assert!(engine::solved(&a.state, &s.goal));
    assert!(
        !progress::current_solved(&a, &s),
        "old unsaved draft must not inherit applied-state success"
    );
    a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
    assert!(progress::current_solved(&a, &s));
    a.workspace
        .as_mut()
        .unwrap()
        .storage_draft
        .as_mut()
        .unwrap()
        .text
        .push_str("\n# pending edit");
    assert!(!progress::current_solved(&a, &s));
    a.workspace.as_mut().unwrap().storage_draft = Some(Draft::new(storage(&a.state)));
    model::command(storage_mut(&mut a.state), "umount /reports").unwrap();
    assert!(!progress::current_solved(&a, &s));
}

#[test]
fn alternatives_pass_but_current_device_paths_wrong_policy_or_missing_peers_fail() {
    let s = scenario();
    let a = solved(&s);
    let text = storage(&a.state).configuration();
    let alternate = format!(
        "# Different supported spelling and order\n{}",
        text.lines().rev().collect::<Vec<_>>().join("\n")
    )
    .replace(" rw ", " defaults,rw,auto ")
    .replace(" ro ", " rw,ro,defaults ");
    let mut candidate = storage(&a.state).clone();
    candidate.save_configuration(alternate).unwrap();
    assert!(
        model::checks(&candidate, requirements(&s))
            .iter()
            .all(|c| c.0)
    );
    for bad in [
        text.replace("UUID=11111111-1111-4111-8111-111111111111", "/dev/vdb1"),
        text.replace("/work ext4 rw", "/work ext4 ro"),
        text.replace("/work ext4 rw", "/work ext4 rw,noauto"),
        text.lines().take(1).collect::<Vec<_>>().join("\n"),
        text.replace(" 0 2", " 1 0"),
    ] {
        let mut candidate = storage(&a.state).clone();
        candidate.save_configuration(bad).unwrap();
        assert!(
            model::checks(&candidate, requirements(&s))
                .iter()
                .any(|c| !c.0)
        );
    }
}
fn requirements(s: &pack::Scenario) -> &[Requirement] {
    let Goal::PersistentMounts { requirements, .. } = &s.goal else {
        panic!()
    };
    requirements
}

#[test]
fn malformed_goal_tables_and_absence_only_shortcuts_are_rejected() {
    let s = scenario();
    let inv = storage(&s.initial).inventory();
    let req = requirements(&s);
    assert!(model::valid_requirements(req, inv));
    assert!(!model::valid_requirements(&[], inv));
    for index in 0..4 {
        let mut bad = req.to_vec();
        bad.remove(index);
        assert!(!model::valid_requirements(&bad, inv));
    }
    let mut bad = req.to_vec();
    bad.push(req[0].clone());
    assert!(!model::valid_requirements(&bad, inv));
    for field in ["expected", "devices"] {
        let mut value = serde_json::to_value(req).unwrap();
        value[4][field] = serde_json::json!({});
        let bad: Vec<Requirement> = serde_json::from_value(value).unwrap();
        assert!(!model::valid_requirements(&bad, inv));
    }
    let mut bad = req.to_vec();
    for r in &mut bad {
        if let Requirement::FreshSession { expected, .. } = r {
            for value in expected.values_mut() {
                *value = None;
            }
        }
    }
    assert!(!model::valid_requirements(&bad, inv));
    let mut bad = req.to_vec();
    for r in &mut bad {
        if let Requirement::Mounted { target, .. } = r {
            *r = Requirement::Unmounted {
                target: target.clone(),
            };
        }
    }
    assert!(!model::valid_requirements(&bad, inv));
    let mut bad = req.to_vec();
    bad.retain(|r| !matches!(r,Requirement::FreshSession{devices,..} if devices!=inv.devices()));
    assert!(!model::valid_requirements(&bad, inv));
}

#[test]
fn duplicate_fresh_goal_keys_are_rejected_instead_of_silently_overwritten() {
    let original = include_str!("fixtures/storage-workspace-preview.json");
    assert!(pack::load(original).is_ok());
    let (header, fresh) = original.split_at(original.find("\"kind\": \"FreshSession\"").unwrap());
    let duplicate = header.to_owned()
        + &fresh.replacen(
            "\"/dev/vdb1\": \"work\"",
            "\"/dev/vdb1\": \"invalid-ignored\", \"/dev/vdb1\": \"work\"",
            1,
        );
    assert!(pack::load(&duplicate).is_err());
    let duplicate = original.replacen("\"expected\": {", "\"expected\": {\"/work\": null,", 1);
    assert!(pack::load(&duplicate).is_err());
}

#[test]
fn two_manual_sources_can_swap_without_a_parse_or_duplicate_error_but_must_fail_identity() {
    use std::collections::BTreeMap;
    use storage::{Filesystem, Inventory, Mount, Volume};
    let uuids = [
        "11111111-1111-4111-8111-111111111111",
        "22222222-2222-4222-8222-222222222222",
        "33333333-3333-4333-8333-333333333333",
    ];
    let names = ["manual-one", "manual-two", "online"];
    let points = ["/one", "/two", "/online"];
    let devices: BTreeMap<_, _> = [
        ("/dev/vdb1", "manual-one"),
        ("/dev/vdc1", "manual-two"),
        ("/dev/vdd1", "online"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect();
    let volumes = names
        .iter()
        .zip(uuids)
        .map(|(id, uuid)| {
            (
                id.to_string(),
                Volume {
                    uuid: uuid.into(),
                    filesystem: Filesystem::Ext4,
                },
            )
        })
        .collect();
    let inv = Inventory::new(
        volumes,
        devices.clone(),
        points.iter().map(|p| p.to_string()).collect(),
    )
    .unwrap();
    let live = points
        .iter()
        .zip(names)
        .map(|(p, id)| {
            (
                p.to_string(),
                Mount {
                    volume: id.into(),
                    read_only: false,
                },
            )
        })
        .collect();
    let config = points
        .iter()
        .zip(uuids)
        .enumerate()
        .map(|(i, (p, id))| {
            format!(
                "UUID={id} {p} ext4 rw,{} 0 2\n",
                if i < 2 { "noauto" } else { "auto" }
            )
        })
        .collect::<String>();
    let mut state = storage::State::new(inv, config, live).unwrap();
    let mut req = Vec::new();
    for (i, (p, id)) in points.iter().zip(names).enumerate() {
        req.push(Requirement::Configured {
            target: p.to_string(),
            volume: id.into(),
            read_only: false,
            automatic: i == 2,
            dump: 0,
            pass: 2,
        });
        req.push(Requirement::Mounted {
            target: p.to_string(),
            volume: id.into(),
            read_only: false,
        });
    }
    let mut swapped = devices.clone();
    swapped.insert("/dev/vdb1".into(), "manual-two".into());
    swapped.insert("/dev/vdc1".into(), "manual-one".into());
    req.push(Requirement::FreshSession {
        devices: swapped.clone(),
        expected: BTreeMap::from([
            ("/one".into(), None),
            ("/two".into(), None),
            (
                "/online".into(),
                Some(Mount {
                    volume: "online".into(),
                    read_only: false,
                }),
            ),
        ]),
    });
    assert!(model::valid_requirements(&req, state.inventory()));
    assert!(model::checks(&state, &req).iter().all(|c| c.0));
    let bad = state
        .configuration()
        .replace(&format!("UUID={}", uuids[0]), "/dev/vdb1")
        .replace(&format!("UUID={}", uuids[1]), "/dev/vdc1");
    state.save_configuration(bad).unwrap();
    let fresh = state.fresh_session(swapped).unwrap();
    assert!(
        fresh.state.validate_configuration().is_empty(),
        "the identity guard must work even without a duplicate/type/parse failure"
    );
    assert_eq!(fresh.state.live().len(), 1);
    assert!(
        model::checks(&state, &req)
            .iter()
            .any(|c| !c.0 && c.1.contains("Fresh-session saved identity"))
    );
}

#[test]
fn manual_entries_must_keep_saved_identity_under_changed_names_even_when_skipped() {
    let s = scenario();
    let mut req = requirements(&s).to_vec();
    for r in &mut req {
        match r {
            Requirement::Configured {
                target, automatic, ..
            } if target == "/reports" => *automatic = false,
            Requirement::FreshSession { expected, .. } => {
                expected.insert("/reports".into(), None);
            }
            _ => {}
        }
    }
    let mut good = storage(&solved(&s).state).clone();
    let text = good
        .configuration()
        .replace("/reports ext4 ro", "/reports ext4 ro,noauto");
    good.save_configuration(text).unwrap();
    assert!(model::valid_requirements(&req, good.inventory()));
    assert!(model::checks(&good, &req).iter().all(|c| c.0));
    let text = good
        .configuration()
        .replace("UUID=22222222-2222-4222-8222-222222222222", "/dev/vdc1");
    good.save_configuration(text).unwrap();
    let fail = model::checks(&good, &req)
        .into_iter()
        .filter(|c| !c.0)
        .collect::<Vec<_>>();
    assert!(fail.iter().any(|c| c.1.contains("Fresh-session saved")));
}

#[test]
fn terminal_is_literal_and_pack_actions_are_not_learner_commands() {
    let s = scenario();
    let mut state = storage(&s.initial).clone();
    for input in [
        "mount -a",
        "mount -o remount,rw /reports",
        "sudo mount /work",
        "cat /etc/passwd",
        "mount /work; id",
        "$(mount /work)",
        &s.solution[0],
        "blkid /dev/../../etc",
    ] {
        let before = state.clone();
        assert!(model::command(&mut state, input).is_err(), "{input}");
        assert_eq!(state, before);
    }
    let text = model::command(&mut state, "blkid /dev/vdb1").unwrap();
    assert!(text.contains("11111111-1111-4111-8111-111111111111"));
    assert!(
        model::command(&mut state, "cat /etc/fstab")
            .unwrap()
            .contains("99999999")
    );
    assert!(
        model::command(&mut state, "help")
            .unwrap()
            .contains("simulator actions")
    );
}

#[test]
fn example_fill_and_run_are_isolated_and_worked_values_do_not_run_automatically() {
    let s = scenario();
    let lesson = s.storage_teaching.as_ref().unwrap();
    let mut a = attempt(&s);
    let original = a.state.clone();
    let mut l = Learning::new(lesson);
    l.view = View::Example;
    let before = l.state.clone();
    l.fill_example(lesson).unwrap();
    assert_eq!(l.state, before);
    assert!(!l.checked);
    assert!(!l.draft.matches(&l.state));
    assert!(l.draft.save(&mut l.state));
    assert!(
        !model::checks(&l.state, &lesson.outcomes)
            .iter()
            .all(|c| c.0)
    );
    l.draft.command = "mount /practice-work".into();
    l.draft.run_command(&mut l.state);
    l.checked = true;
    assert!(l.checks(lesson).iter().all(|c| c.0));
    a.workspace.as_mut().unwrap().storage_learning = Some(l);
    assert_eq!(a.state, original);
    assert!(!progress::current_solved(&a, &s));
}

#[test]
fn declared_practice_change_reveals_a_persistence_mistake_and_is_repairable() {
    let s = scenario();
    let lesson = s.storage_teaching.as_ref().unwrap();
    let mut state = lesson.fixture.clone();
    for action in &lesson.example_solution {
        model::adapter(&mut state, action).unwrap();
    }
    assert!(model::checks(&state, &lesson.outcomes).iter().all(|c| c.0));
    let live = state.live().clone();
    for action in &lesson.try_actions {
        model::adapter(&mut state, action).unwrap();
    }
    assert_eq!(*state.live(), live);
    let failures = model::checks(&state, &lesson.outcomes)
        .into_iter()
        .filter(|c| !c.0)
        .collect::<Vec<_>>();
    assert!(failures.iter().any(|c| c.1.contains("Fresh-session")));
    for action in &lesson.example_solution {
        model::adapter(&mut state, action).unwrap();
    }
    assert!(model::checks(&state, &lesson.outcomes).iter().all(|c| c.0));
}

#[test]
fn progress_roundtrip_keeps_invalid_drafts_separate_examples_and_all_previous_attempts() {
    let s = scenario();
    let all = pack::bundled().unwrap();
    let mut catalog = all.clone();
    catalog.push(s.clone());
    let mut a = solved(&s);
    let w = a.workspace.as_mut().unwrap();
    w.storage_draft.as_mut().unwrap().text =
        "# Invalid unsaved Unicode text\n".to_owned() + &"字".repeat(16_000);
    w.storage_learning.as_mut().unwrap().draft.command = "blkid".into();
    let mut p = records(a, &s);
    for old in &all {
        p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    }
    let encoded = serde_json::to_vec(&p).unwrap();
    let restored = progress::decode(&encoded, &catalog).unwrap();
    assert_eq!(restored.attempts.len(), all.len() + 1);
    assert!(restored.favorites.contains(&s.id));
    assert!(!progress::current_solved(&restored.attempts[&s.id], &s));
    assert_eq!(
        serde_json::to_value(&restored.attempts[&s.id]).unwrap(),
        serde_json::to_value(&p.attempts[&s.id]).unwrap()
    );
}

#[test]
fn coordinated_inventory_or_example_rewrites_are_rejected_before_restore() {
    let s = scenario();
    let p = records(solved(&s), &s);
    let original = serde_json::to_value(&p).unwrap();
    assert!(
        progress::decode(
            &serde_json::to_vec(&original).unwrap(),
            std::slice::from_ref(&s)
        )
        .is_ok()
    );
    for field in ["assignment", "example"] {
        let mut value = original.clone();
        let root = &mut value["attempts"][&s.id];
        let state = if field == "assignment" {
            &mut root["state"]["storage"]
        } else {
            &mut root["workspace"]["storage_learning"]["state"]
        };
        state["inventory"]["volumes"][if field == "assignment" {
            "work"
        } else {
            "practice-work"
        }]["uuid"] = serde_json::json!("88888888-8888-4888-8888-888888888888");
        assert!(
            progress::decode(
                &serde_json::to_vec(&value).unwrap(),
                std::slice::from_ref(&s)
            )
            .is_err()
        );
    }
}

#[test]
fn save_side_wrong_type_metadata_keeps_original_file_and_reset_is_selected_only() {
    let s = scenario();
    let all = pack::bundled().unwrap();
    let mut p = records(solved(&s), &s);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("progress-v1.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut wrong = Attempt::new(&all[0], 1);
    wrong.workspace = p.attempts[&s.id].workspace.clone();
    p.attempts.insert(all[0].id.clone(), wrong);
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    p.attempts.remove(&all[0].id);
    let old = p.attempts[&s.id].visited;
    let mut reset = Attempt::new(&s, old);
    workspace::prepare(&mut reset);
    storage_teaching::prepare(&mut reset, s.storage_teaching.as_ref().unwrap(), true);
    p.attempts.insert(s.id.clone(), reset);
    assert_eq!(p.attempts[&s.id].state, s.initial);
    assert!(p.favorites.contains(&s.id));
    assert!(!progress::current_solved(&p.attempts[&s.id], &s));
}

#[test]
fn transcript_and_invalid_unicode_draft_limits_preserve_retry_without_large_echo() {
    let s = scenario();
    let mut state = storage(&s.initial).clone();
    let before = state.clone();
    let mut d = Draft::new(&state);
    d.text = "字".repeat(16_384);
    assert!(d.valid());
    assert!(!d.save(&mut state));
    assert_eq!(state, before);
    d.append_output(&"字".repeat(100_000));
    assert!(d.output.len() <= model::MAX_OUTPUT);
    assert!(d.output.is_char_boundary(d.output.len()));
    d.command = "x".repeat(100_000);
    d.run_command(&mut state);
    assert!(d.valid());
    assert!(d.output.contains("4KiB"));
    assert_eq!(state, before);
}
