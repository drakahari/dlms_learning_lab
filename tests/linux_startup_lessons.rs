use dlms_learning_lab::{
    course,
    engine::{self, Goal, State},
    pack,
    progress::{self, Attempt, Progress},
    unit_manager::{self as core, Action},
    unit_manager_teaching::Learning,
    unit_manager_terminal as terminal,
    unit_manager_workspace::{self as ws, Draft, EditAction, Requirement},
    workspace,
};
const IDS: [&str; 3] = [
    "lpic1-startup-current-future",
    "lpic1-startup-default-target",
    "lpic1-startup-install-links",
];
fn catalog() -> Vec<pack::Scenario> {
    pack::load_catalog(&[
        include_str!("../scenarios/starter-v1.json"),
        include_str!("../scenarios/linux-files-v1.json"),
        include_str!("../scenarios/linux-startup-lessons-v1.json"),
    ])
    .unwrap()
}
fn labs() -> Vec<pack::Scenario> {
    catalog()
        .into_iter()
        .filter(|s| IDS.contains(&s.id.as_str()))
        .collect()
}
fn def(s: &pack::Scenario) -> &ws::Definition {
    let Goal::UnitManager { definition } = &s.goal else {
        panic!()
    };
    definition
}
fn sim(s: &State) -> &core::State {
    let State::UnitManager { simulation } = s else {
        panic!()
    };
    simulation
}
fn solved(s: &pack::Scenario, state: &State) -> bool {
    ws::solved(def(s), sim(state), &Draft::new(sim(state)).unwrap())
}
fn reference(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for line in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
    }
    assert!(solved(s, &state), "{}", s.id);
    state
}
fn apply(s: &pack::Scenario, state: &mut State, action: Action) {
    let State::UnitManager { simulation } = state else {
        panic!()
    };
    ws::apply(simulation, &def(s).initial, &EditAction::Manager { action }).unwrap();
}
fn failure(s: &pack::Scenario, state: &State) {
    assert!(!solved(s, state));
    assert!(
        ws::checks(def(s), sim(state), &Draft::new(sim(state)).unwrap())
            .iter()
            .any(|(ok, msg)| !ok && !msg.is_empty())
    );
}
#[test]
fn three_distinct_assignments_teach_before_testing_and_preserve_235_objects() {
    let old = pack::accepted235().unwrap();
    let current = pack::accepted238().unwrap();
    assert_eq!(old.len(), 235);
    assert_eq!(current.len(), 238);
    for a in &old {
        let b = current.iter().find(|b| a.id == b.id).unwrap();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    let old_units = course::accepted235(&old).unwrap();
    let units = course::accepted238(&current).unwrap();
    assert_eq!((old_units.len(), units.len()), (63, 64));
    for a in old_units {
        let b = units.iter().find(|b| a.id == b.id).unwrap();
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    let u = units
        .iter()
        .find(|u| u.id == "linux-startup-policy")
        .unwrap();
    assert_eq!(u.labs, IDS);
    for s in labs() {
        let l = s.unit_manager_teaching.as_ref().unwrap();
        assert!(l.concepts.len() >= 3);
        l.validate().unwrap();
        assert!(l.distinct_from(sim(&s.initial)));
        assert!(!s.objective.contains("unit-manager {"));
        assert!(s.simplifications.contains("No firmware"));
        failure(&s, &s.initial);
        reference(&s);
    }
}
#[test]
fn supplied_references_and_equivalent_supported_routes_pass() {
    for s in labs() {
        let mut state = s.initial.clone();
        if s.id.ends_with("current-future") {
            for cmd in [
                "systemctl disable kiln-inspection.service",
                "systemctl enable kiln-app.service",
                "systemctl start kiln-inspection.service",
                "systemctl stop kiln-app.service",
            ] {
                let State::UnitManager { simulation } = &mut state else {
                    panic!()
                };
                terminal::execute(def(&s), simulation, cmd).unwrap();
            }
        } else if s.id.ends_with("default-target") {
            for cmd in [
                "systemctl set-default harbor.target",
                "systemctl disable harbor-inspection.service",
            ] {
                let State::UnitManager { simulation } = &mut state else {
                    panic!()
                };
                terminal::execute(def(&s), simulation, cmd).unwrap();
            }
        } else {
            for line in &s.solution {
                engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
            }
            // Preserve intent, not exact text: harmless comment is accepted in editable app source.
            let mut d = Draft::new(sim(&state)).unwrap();
            d.documents
                .get_mut("press-app.service")
                .unwrap()
                .push_str("# deliberate enrollment repair\n");
            let State::UnitManager { simulation } = &mut state else {
                panic!()
            };
            ws::apply(simulation, &def(&s).initial, &EditAction::Save { draft: d }).unwrap();
            for cmd in [
                "systemctl daemon-reload",
                "systemctl stop press-app.service",
                "systemctl start press-app.service",
            ] {
                terminal::execute(def(&s), simulation, cmd).unwrap();
            }
        }
        assert!(solved(&s, &state), "{}", s.id);
    }
}
#[test]
fn incomplete_reference_sequences_do_not_earn_credit_and_current_success_can_break() {
    for s in labs() {
        for end in 0..s.solution.len() {
            let mut state = s.initial.clone();
            for line in &s.solution[..end] {
                engine::execute_for_goal(&mut state, &s.goal, line).unwrap();
            } // final AdvancePrograms is redundant after a terminal Start, but required by internal action refs
            failure(&s, &state);
        }
        let mut state = reference(&s);
        let target = if s.id.ends_with("current-future") {
            "kiln-app.service"
        } else if s.id.ends_with("default-target") {
            "harbor-app.service"
        } else {
            "press-app.service"
        };
        apply(
            &s,
            &mut state,
            Action::Stop {
                unit: target.into(),
            },
        );
        // The first task wants app stopped: break enrollment instead.
        if s.id.ends_with("current-future") {
            apply(
                &s,
                &mut state,
                Action::Disable {
                    unit: target.into(),
                },
            );
        }
        failure(&s, &state);
    }
}
#[test]
fn each_current_requirement_has_a_targeted_failing_state() {
    for s in labs() {
        let passed = reference(&s);
        for (n, r) in def(&s).requirements.iter().enumerate() {
            let mut wrong = passed.clone();
            match r {
                Requirement::SavedConfigurationValid {} => {
                    let u = sim(&wrong).documents().keys().next().unwrap().clone();
                    apply(
                        &s,
                        &mut wrong,
                        Action::SaveUnit {
                            unit: u,
                            text: "[broken\n".into(),
                        },
                    );
                }
                Requirement::Runtime { unit, phase, .. } => {
                    let action = if matches!(phase, ws::Phase::Inactive) {
                        Action::Start { unit: unit.clone() }
                    } else {
                        Action::Stop { unit: unit.clone() }
                    };
                    apply(&s, &mut wrong, action);
                    apply(&s, &mut wrong, Action::AdvancePrograms {});
                }
                Requirement::ExecutionCurrent { unit } | Requirement::LoadedCurrent { unit } => {
                    let text = format!("{}# changed after pass\n", sim(&wrong).documents()[unit]);
                    apply(
                        &s,
                        &mut wrong,
                        Action::SaveUnit {
                            unit: unit.clone(),
                            text,
                        },
                    );
                }
                Requirement::InstallLink {
                    target,
                    unit,
                    present,
                } => {
                    if !*present {
                        // Enable follows saved WantedBy. Construct the forbidden
                        // old-target link explicitly, rather than enabling an
                        // unrelated target and calling that negative evidence.
                        let text = sim(&wrong).documents()[unit]
                            .lines()
                            .map(|line| {
                                if line.starts_with("WantedBy=") {
                                    format!("WantedBy={target}")
                                } else {
                                    line.into()
                                }
                            })
                            .collect::<Vec<String>>()
                            .join("\n")
                            + "\n";
                        apply(
                            &s,
                            &mut wrong,
                            Action::SaveUnit {
                                unit: unit.clone(),
                                text,
                            },
                        );
                    }
                    apply(
                        &s,
                        &mut wrong,
                        if *present {
                            Action::Disable { unit: unit.clone() }
                        } else {
                            Action::Enable { unit: unit.clone() }
                        },
                    );
                }
                Requirement::DefaultTarget { target } => {
                    let other = sim(&wrong)
                        .inventory()
                        .units
                        .iter()
                        .find(|u| u.ends_with(".target") && *u != target)
                        .unwrap()
                        .clone();
                    apply(&s, &mut wrong, Action::SetDefault { target: other });
                }
                _ => panic!("unexpected requirement; add targeted negative evidence"),
            }
            failure(&s, &wrong);
            let checks = ws::checks(def(&s), sim(&wrong), &Draft::new(sim(&wrong)).unwrap());
            assert!(!checks[n + 1].0, "{} {r:?}", s.id);
        }
    }
}
#[test]
fn practice_reference_experiment_retry_and_assignment_drafts_stay_separate() {
    for s in labs() {
        let l = s.unit_manager_teaching.as_ref().unwrap();
        let before = s.initial.clone();
        let mut p = Learning::new(l).unwrap();
        for a in &l.solution {
            p.execute(l, a).unwrap();
        }
        assert!(ws::solved(&l.definition, &p.state, &p.draft));
        for a in &l.try_actions {
            p.execute(l, a).unwrap();
        }
        assert!(!ws::solved(&l.definition, &p.state, &p.draft));
        for a in &l.repair {
            p.execute(l, a).unwrap();
        }
        assert!(ws::solved(&l.definition, &p.state, &p.draft));
        let json = serde_json::to_string(&p).unwrap();
        let restored = Learning::from_json(&json, l).unwrap();
        assert_eq!(restored.state, p.state);
        assert_eq!(s.initial, before);
        failure(&s, &before);
    }
}
#[test]
fn progress_roundtrip_keeps_dirty_drafts_practice_current_grade_and_selected_reset() {
    let all = catalog();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        dlms_learning_lab::unit_manager_teaching::prepare(
            &mut a,
            s.unit_manager_teaching.as_ref().unwrap(),
            true,
        )
        .unwrap();
        a.state = reference(&s);
        let synced = Draft::new(sim(&a.state)).unwrap();
        a.workspace.as_mut().unwrap().unit_manager_draft = Some(synced);
        a.workspace.as_mut().unwrap().checked = true;
        assert!(progress::current_solved(&a, &s));
        let d = a
            .workspace
            .as_mut()
            .unwrap()
            .unit_manager_draft
            .as_mut()
            .unwrap();
        let u = d.selected_unit.clone();
        d.documents
            .get_mut(&u)
            .unwrap()
            .push_str("# pending learner work\n");
        assert!(!progress::current_solved(&a, &s));
        p.favorites.insert(s.id.clone());
        p.attempts.insert(s.id.clone(), a);
    }
    let text = serde_json::to_vec(&p).unwrap();
    let q = progress::decode(&text, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&q).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("progress.json");
    progress::save_with_catalog(&path, &p, &all).unwrap();
    let mut restored = progress::load(&path, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&restored).unwrap(),
        serde_json::to_value(&p).unwrap()
    );
    let untouched = serde_json::to_value(&restored.attempts[IDS[1]]).unwrap();
    let selected = all.iter().find(|s| s.id == IDS[0]).unwrap();
    restored
        .attempts
        .insert(selected.id.clone(), Attempt::new(selected, 2));
    assert_eq!(
        serde_json::to_value(&restored.attempts[IDS[1]]).unwrap(),
        untouched
    );
    assert!(restored.favorites.contains(IDS[0]));
    assert!(!progress::current_solved(
        &restored.attempts[IDS[0]],
        selected
    ));
    progress::save_with_catalog(&path, &restored, &all).unwrap();
    assert_eq!(
        serde_json::to_value(progress::load(&path, &all).unwrap()).unwrap(),
        serde_json::to_value(restored).unwrap()
    );
}
#[test]
fn unsupported_host_commands_and_foreign_contexts_fail_without_mutating_state() {
    for s in labs() {
        let mut state = reference(&s);
        for cmd in [
            "systemctl reboot",
            "systemctl isolate default.target",
            "systemctl mask anything.service",
            "sh -c whoami",
            "cat /etc/passwd",
        ] {
            let old = state.clone();
            let State::UnitManager { simulation } = &mut state else {
                panic!()
            };
            assert!(terminal::execute(def(&s), simulation, cmd).is_err());
            assert_eq!(state, old);
        }
        let other = labs().into_iter().find(|x| x.id != s.id).unwrap();
        let mut foreign = sim(&other.initial).clone();
        let old = foreign.clone();
        assert!(terminal::execute(def(&s), &mut foreign, "systemctl get-default").is_err());
        assert_eq!(foreign, old);
    }
}

#[test]
fn malformed_new_packs_refuse_unreachable_examples_foreign_origins_and_missing_current_inputs() {
    let raw = include_str!("../scenarios/linux-startup-lessons-v1.json");
    for n in 0..5 {
        let mut v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let s = &mut v["scenarios"][0];
        match n {
            0 => {
                s["goal"]["definition"]["requirements"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|r| r["kind"] != "ExecutionCurrent");
            }
            1 => s["unit_manager_teaching"]["repair"] = serde_json::json!([]),
            2 => {
                s["unit_manager_teaching"]["solution"] =
                    serde_json::json!([{"kind":"Manager","action":{"kind":"DaemonReload"}}])
            }
            3 => {
                s["initial"]["simulation"]["inventory"]["fixture"] =
                    serde_json::json!("foreign-origin")
            }
            _ => {
                s["unit_manager_teaching"]["definition"]["initial"]["documents"]["sketchplan-app.service"] =
                    serde_json::json!("[Service]\nExecStart=/bin/sh\n")
            }
        }
        assert!(
            pack::load_catalog(&[
                include_str!("../scenarios/starter-v1.json"),
                include_str!("../scenarios/linux-files-v1.json"),
                &v.to_string()
            ])
            .is_err(),
            "invalid variant{n}"
        );
    }
}
