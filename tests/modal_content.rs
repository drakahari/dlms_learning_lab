use dlms_learning_lab::{
    engine,
    modal_editing::Key,
    modal_source as binding,
    modal_workspace::{Event, Requirement},
    pack,
    progress::{self, Attempt, Progress},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/modal-editing-lessons-v1.json")).unwrap()
}
fn session(state: &mut engine::State) -> &mut dlms_learning_lab::modal_workspace::Session {
    let engine::State::ModalEditing { simulation } = state else {
        panic!()
    };
    simulation
}
fn input(
    a: &mut dlms_learning_lab::modal_workspace::Attempt,
    f: &dlms_learning_lab::modal_workspace::Fixture,
    text: &str,
) {
    for c in text.chars() {
        a.apply(
            f,
            &Event::Key {
                key: match c {
                    '\x1b' => Key::Escape,
                    '\r' => Key::Enter,
                    _ => Key::Character(c),
                },
            },
        )
        .unwrap();
    }
}
#[test]
fn all_three_taught_assignments_and_distinct_examples_work_without_cross_credit() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let d = binding::source(&s).unwrap();
        assert_ne!(d.initial, d.example);
        let mut state = s.initial.clone();
        assert!(!engine::solved(&state, &s.goal));
        session(&mut state).show_worked(d).unwrap();
        session(&mut state)
            .apply(d, true, &Event::Check {})
            .unwrap();
        assert!(session(&mut state).practice.solved(&d.example));
        assert!(!engine::solved(&state, &s.goal));
        for e in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, e).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        assert!(session(&mut state).assignment.solved(&d.initial));
    }
}
#[test]
fn equivalent_supported_edits_pass_including_record_order_whitespace_and_draft_undo() {
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let a = &mut session(&mut state).assignment;
        if a.editor().modified() {
            input(a, &d.initial, "u");
        }
        for r in &d.initial.requirements {
            let text = match r {
                Requirement::Settings { path, values } => {
                    let old = &d.initial.files[path].text;
                    let mut rows = vec!["# repaired by an alternative edit".to_owned()];
                    for line in old.lines().rev() {
                        let (k, v) = line.split_once('=').unwrap();
                        rows.push(format!(
                            "{k} = {}",
                            values.get(k).map(String::as_str).unwrap_or(v)
                        ));
                    }
                    Some((path, rows.join("\n")))
                }
                Requirement::Records { path, counts } => {
                    let mut rows = std::collections::BTreeMap::<String, usize>::new();
                    for line in d.initial.files[path].text.lines() {
                        *rows.entry(line.into()).or_default() += 1;
                    }
                    for (k, n) in counts {
                        rows.insert(k.clone(), *n);
                    }
                    Some((
                        path,
                        rows.into_iter()
                            .rev()
                            .flat_map(|(line, n)| std::iter::repeat_n(line, n))
                            .collect::<Vec<_>>()
                            .join("\n"),
                    ))
                }
                Requirement::Unchanged { .. } => None,
            };
            if let Some((path, text)) = text {
                a.apply(&d.initial, &Event::Open { path: path.clone() })
                    .unwrap();
                let lines = a.editor().contents().lines().count();
                input(a, &d.initial, "gg");
                for _ in 0..lines {
                    input(a, &d.initial, "dd");
                }
                input(a, &d.initial, "i");
                a.apply(&d.initial, &Event::Insert { text }).unwrap();
                input(a, &d.initial, "\x1b:w\r");
            }
        }
        a.apply(&d.initial, &Event::Check {}).unwrap();
        assert!(engine::solved(&state, &s.goal), "{}", s.id);
    }
}
#[test]
fn partial_wrong_unsaved_and_broken_again_states_do_not_keep_success() {
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        for input in s.solution.iter().take(s.solution.len() / 2) {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        session(&mut state)
            .apply(d, false, &Event::Check {})
            .unwrap();
        assert!(!engine::solved(&state, &s.goal));
        state = s.initial.clone();
        for input in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &s.goal));
        let a = &mut session(&mut state).assignment;
        input(a, &d.initial, "gg");
        assert!(a.solved(&d.initial));
        input(a, &d.initial, "i");
        a.apply(&d.initial, &Event::Insert { text: "BAD".into() })
            .unwrap();
        assert!(!a.checked);
        a.apply(&d.initial, &Event::Check {}).unwrap();
        assert!(!engine::solved(&state, &s.goal));
        input(&mut session(&mut state).assignment, &d.initial, "\x1b:w\r");
        session(&mut state)
            .apply(d, false, &Event::Check {})
            .unwrap();
        assert!(!engine::solved(&state, &s.goal));
    }
}
#[test]
fn actual_progress_io_preserves_partial_mode_history_favorites_and_completed_outcomes() {
    for s in labs() {
        let d = binding::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        binding::context(&a, &s).unwrap();
        let sim = session(&mut a.state);
        sim.show_worked(d).unwrap();
        sim.apply(
            d,
            false,
            &Event::Key {
                key: Key::Character('i'),
            },
        )
        .unwrap();
        sim.apply(
            d,
            false,
            &Event::Insert {
                text: "unfinished ".into(),
            },
        )
        .unwrap();
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
        let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&p).unwrap()
        );
        assert!(!progress::current_solved(&loaded.attempts[&s.id], &s));
        let mut done = Attempt::new(&s, 2);
        workspace::prepare(&mut done);
        for e in &s.solution {
            engine::execute_for_goal(&mut done.state, &s.goal, e).unwrap();
        }
        assert!(progress::current_solved(&done, &s));
        p.attempts.insert(s.id.clone(), done);
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
        let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert!(progress::current_solved(&loaded.attempts[&s.id], &s));
        assert!(loaded.favorites.contains(&s.id));
        let mut reset = loaded;
        session(&mut reset.attempts.get_mut(&s.id).unwrap().state)
            .reset_assignment(d)
            .unwrap();
        assert!(!progress::current_solved(&reset.attempts[&s.id], &s));
        assert!(reset.favorites.contains(&s.id));
    }
}
#[test]
fn malformed_authoring_paths_task_bindings_and_mismatched_progress_refuse() {
    for s in labs() {
        let mut state = s.initial.clone();
        let original = serde_json::to_value(&state).unwrap();
        for bad in [
            "!whoami",
            "modal-lab {\"kind\":\"Open\",\"path\":\"/etc/passwd\"}",
            "modal-lab {\"kind\":\"Open\",\"path\":\"/lab/../escape\"}",
            "modal-lab {\"kind\":\"Key\",\"key\":{\"kind\":\"Shell\",\"value\":\"anything\"}}",
        ] {
            assert!(engine::execute_for_goal(&mut state, &s.goal, bad).is_err());
            assert_eq!(serde_json::to_value(&state).unwrap(), original);
        }
        let mut attempt = Attempt::new(&s, 1);
        workspace::prepare(&mut attempt);
        attempt.scenario_version += 1;
        assert!(binding::context(&attempt, &s).is_err());
        assert!(!progress::current_solved(&attempt, &s));
    }
    let mut pack: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/modal-editing-lessons-v1.json")).unwrap();
    pack["scenarios"][0]["goal"]["definition"]["id"] = serde_json::json!("wrong-source");
    assert!(pack::load(&pack.to_string()).is_err());
}

#[test]
fn normal_catalog_keeps_every_prior_scenario_and_unit_exact_and_registers_new_progression() {
    let old = pack::accepted445().unwrap();
    let current = pack::current().unwrap();
    assert_eq!(old.len(), 445);
    assert_eq!(current.len(), 452);
    for s in &old {
        let found = current.iter().find(|n| n.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(found).unwrap(),
            "{}",
            s.id
        );
    }
    let old_units = dlms_learning_lab::course::accepted445(&old).unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!(old_units.len(), 176);
    assert_eq!(units.len(), 179);
    for u in &old_units {
        let found = units.iter().find(|n| n.id == u.id).unwrap();
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(found).unwrap(),
            "{}",
            u.id
        );
    }
    let base = units
        .iter()
        .position(|u| u.id == "linux-file-foundations")
        .unwrap();
    let next = units
        .iter()
        .position(|u| u.id == "modal-file-editing")
        .unwrap();
    assert_eq!(next, base + 1);
    assert_eq!(
        units[next].labs,
        labs().iter().map(|s| s.id.clone()).collect::<Vec<_>>()
    );
    assert_eq!(current.iter().filter(|s| s.track == "LPIC-1").count(), 151);
}
