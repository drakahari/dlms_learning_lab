//! Private backend fixture; not bundled/countable curriculum or native UI acceptance.
#[path = "../examples/support/automation_digest.rs"]
mod digest;
#[path = "../src/ui/one_off_boundary.rs"]
mod one_off_boundary;
#[path = "support/one_off_registration.rs"]
mod one_off_registration;
use dlms_learning_lab::{
    accounts,
    accounts_teaching::{self, Learning},
    backup,
    engine::{self, Goal, State},
    pack::{self, Scenario},
    progress::{self, Attempt, Progress},
    script_teaching::View,
    shell, workspace,
};
fn scenario() -> Scenario {
    pack::load(include_str!("fixtures/accounts-workspace-preview.json"))
        .unwrap()
        .remove(0)
}
fn catalog() -> Vec<Scenario> {
    let mut all = pack::bundled().unwrap();
    all.push(scenario());
    all
}
fn run(s: &mut State, g: &Goal, commands: &[&str]) {
    for c in commands {
        engine::execute_for_goal(s, g, c).unwrap_or_else(|e| panic!("{c}: {e}"));
    }
}
fn reference(s: &Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    state
}
fn simulation(s: &State) -> &accounts::State {
    let State::Accounts { simulation } = s else {
        panic!()
    };
    simulation
}
fn prepared(s: &Scenario) -> Attempt {
    let mut a = Attempt::new(s, 1);
    workspace::prepare(&mut a);
    accounts_teaching::prepare(&mut a, s.accounts_teaching.as_ref().unwrap(), true);
    a
}
fn learn(a: &mut Attempt) -> &mut Learning {
    a.workspace
        .as_mut()
        .unwrap()
        .accounts_learning
        .as_mut()
        .unwrap()
}

#[test]
fn private_fixture_is_unbundled_and_reference_alternates_reach_current_whole_outcomes() {
    let s = scenario();
    let all = pack::bundled().unwrap();
    assert_eq!(one_off_registration::original_catalog(&all).len(), 176);
    assert!(!all.iter().any(|x| x.id == s.id));
    const {
        assert!(
            shell::EVALUATOR_REVISION >= 7,
            "requires the supported command-substitution generation; current records use their actual evaluator"
        );
    };
    assert!(!engine::solved(&s.initial, &s.goal));
    let good = reference(&s);
    assert!(engine::solved(&good, &s.goal));
    let mut alt = s.initial.clone();
    run(
        &mut alt,
        &s.goal,
        &[
            "getent passwd mira",
            "usermod -g review mira",
            "groupadd --gid 2200 reports",
            "usermod -G reports,audit,reports mira",
            "id mira",
        ],
    );
    assert_eq!(good, alt);
    let a = Attempt {
        state: alt,
        ..Attempt::new(&s, 1)
    };
    assert!(progress::current_solved(&a, &s));
    assert_eq!(a.step, 0);
    assert!(engine::help(&s.initial).contains("Virtual administrator"));
}
#[test]
fn raw_account_mutations_require_goal_context_but_queries_remain_available() {
    let s = scenario();
    let mut state = s.initial.clone();
    let original = state.clone();
    assert!(
        engine::execute(&mut state, "id mira")
            .unwrap()
            .contains("1101(mira)")
    );
    assert!(
        engine::execute(&mut state, "help")
            .unwrap()
            .contains("no host accounts")
    );
    assert!(
        engine::execute(&mut state, "groupadd -g 2200 reports")
            .unwrap_err()
            .contains("goal context")
    );
    assert_eq!(state, original);
    engine::execute_for_goal(&mut state, &s.goal, "groupadd -g 2200 reports").unwrap();
    assert_ne!(state, original);
}
#[test]
fn context_wrapper_prevents_identity_reassignment_before_publication_or_save() {
    let s = scenario();
    let mut state = s.initial.clone();
    run(&mut state, &s.goal, &["userdel -r mira"]);
    let before = state.clone();
    let error = engine::execute_for_goal(
        &mut state,
        &s.goal,
        "useradd -N -u 1300 -g staff -d /home/mira -s /bin/bash -m mira",
    )
    .unwrap_err();
    assert!(error.contains("identity"));
    assert_eq!(state, before);
    assert!(simulation(&state).valid_against(simulation(&s.initial)));
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(
        s.id.clone(),
        Attempt {
            state,
            ..Attempt::new(&s, 1)
        },
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    assert!(progress::load(&path, &catalog()).is_ok());
}
#[test]
fn wrong_initial_context_and_invalid_current_state_are_rejected_without_mutation() {
    let s = scenario();
    let mut state = s.initial.clone();
    let mut wrong_goal = s.goal.clone();
    if let Goal::Accounts { initial, .. } = &mut wrong_goal {
        *initial = s.accounts_teaching.as_ref().unwrap().fixture.clone();
    }
    let before = state.clone();
    assert!(engine::execute_for_goal(&mut state, &wrong_goal, "usermod -g review mira").is_err());
    assert_eq!(state, before);
    let mut value = serde_json::to_value(&state).unwrap();
    value["simulation"]["configuration"]["users"]["root"]["comment"] = "altered".into();
    let mut state: State = serde_json::from_value(value).unwrap();
    let before = state.clone();
    assert!(engine::execute_for_goal(&mut state, &s.goal, "id mira").is_err());
    assert_eq!(state, before);
}
#[test]
fn wrong_partial_readonly_and_unrelated_damage_cannot_pass_current_goals() {
    let s = scenario();
    for commands in [
        vec!["id mira", "getent group review"],
        vec!["groupadd -g 2200 reports"],
        vec!["groupadd -g 2200 reports", "usermod -G audit,reports mira"],
        vec![
            "groupadd -g 2200 reports",
            "usermod -G audit,reports bryn",
            "usermod -g review bryn",
        ],
    ] {
        let mut state = s.initial.clone();
        run(&mut state, &s.goal, &commands);
        assert!(!engine::solved(&state, &s.goal));
    }
    let mut partial = s.initial.clone();
    run(&mut partial, &s.goal, &["groupadd -g 2200 reports"]);
    let checks = engine::checks(&partial, &s.goal);
    assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
    let mut good = reference(&s);
    run(&mut good, &s.goal, &["chown bryn:staff /archive/keep.txt"]);
    assert!(!engine::solved(&good, &s.goal));
    let mut a = Attempt {
        state: reference(&s),
        ..Attempt::new(&s, 1)
    };
    assert!(progress::current_solved(&a, &s));
    engine::execute_for_goal(&mut a.state, &s.goal, "usermod -G audit mira").unwrap();
    assert!(!progress::current_solved(&a, &s));
}
#[test]
fn example_annotations_defaults_try_change_and_repair_use_only_its_immutable_context() {
    let s = scenario();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    lesson.validate().unwrap();
    assert!(lesson.separate_from(simulation(&s.initial)));
    let mut a = prepared(&s);
    let assignment = a.state.clone();
    for (index, command) in lesson.example.iter().enumerate() {
        let l = learn(&mut a);
        l.example_command = index;
        l.command = command.clone();
        l.execute(lesson).unwrap();
    }
    assert!(learn(&mut a).checks(lesson).iter().all(|c| c.0));
    assert_eq!(a.state, assignment);
    assert!(!progress::current_solved(&a, &s));
    for command in &lesson.try_actions {
        let l = learn(&mut a);
        l.command = command.clone();
        l.execute(lesson).unwrap();
    }
    assert!(
        accounts::checks(
            &lesson.fixture,
            &learn(&mut a).simulation,
            &lesson.try_outcomes
        )
        .iter()
        .all(|c| c.0)
    );
    assert!(learn(&mut a).checks(lesson).iter().any(|c| !c.0));
    for command in &lesson.repair {
        let l = learn(&mut a);
        l.command = command.clone();
        l.execute(lesson).unwrap();
    }
    assert!(learn(&mut a).checks(lesson).iter().all(|c| c.0));
    assert_eq!(a.state, assignment);
}
#[test]
fn fill_example_changes_only_command_input_and_never_executes_or_awards_credit() {
    let s = scenario();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    let mut a = prepared(&s);
    let l = learn(&mut a);
    l.example_command = 1;
    l.command = "unsaved draft".into();
    l.checked = true;
    let mut before = serde_json::to_value(&*l).unwrap();
    l.fill_example(lesson).unwrap();
    before["command"] = lesson.example[1].clone().into();
    assert_eq!(serde_json::to_value(&*l).unwrap(), before);
    assert!(!progress::current_solved(&a, &s));
    assert_eq!(a.state, s.initial);
}
#[test]
fn malformed_unsaved_command_and_separate_simulation_resume_exactly() {
    let s = scenario();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    let mut a = prepared(&s);
    learn(&mut a).view = View::Example;
    learn(&mut a).command = "usermod -G 'unfinished".into();
    let before = learn(&mut a).simulation.clone();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let mut restored = progress::load(&path, &catalog()).unwrap();
    let a = restored.attempts.get_mut(&s.id).unwrap();
    assert_eq!(learn(a).command, "usermod -G 'unfinished");
    assert!(learn(a).execute(lesson).is_err());
    assert_eq!(learn(a).simulation, before);
    assert!(!learn(a).transcript.is_empty());
    assert!(!progress::current_solved(a, &s));
}
#[test]
fn protected_assignment_and_example_contexts_cannot_be_changed_by_imported_profile() {
    let s = scenario();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), prepared(&s));
    for location in ["assignment", "example", "policy", "identity"] {
        let mut v = serde_json::to_value(&p).unwrap();
        let a = &mut v["attempts"][&s.id];
        match location {
            "assignment" => {
                a["state"]["simulation"]["configuration"]["users"]["root"]["comment"] =
                    "altered".into()
            }
            "example" => {
                a["workspace"]["accounts_learning"]["simulation"]["configuration"]["users"]["root"]
                    ["comment"] = "altered".into()
            }
            "policy" => {
                a["workspace"]["accounts_learning"]["simulation"]["policy"]["fixture"] =
                    "other-example".into()
            }
            _ => a["state"]["simulation"]["configuration"]["users"]["mira"]["uid"] = 9999.into(),
        }
        assert!(
            progress::decode(&serde_json::to_vec(&v).unwrap(), &catalog()).is_err(),
            "{location}"
        );
    }
}
#[test]
fn wrong_activity_metadata_cannot_replace_existing_profile_bytes() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let old = &all[0];
    p.attempts.insert(old.id.clone(), Attempt::new(old, 1));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let before = std::fs::read(&path).unwrap();
    let a = p.attempts.get_mut(&old.id).unwrap();
    workspace::prepare(a);
    a.workspace.as_mut().unwrap().accounts_learning =
        Some(Learning::new(s.accounts_teaching.as_ref().unwrap()));
    assert!(progress::save(&path, &p).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &all).is_err());
}
#[test]
fn assignment_overlapping_example_and_cross_lesson_selection_are_rejected() {
    let s = scenario();
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    let mut a = prepared(&s);
    learn(&mut a).simulation = simulation(&s.initial).clone();
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    assert!(progress::save(&path, &p).is_err());
    assert!(!path.exists());
    let mut a = prepared(&s);
    learn(&mut a).example_command = 7;
    let mut p = Progress {
        schema_version: 1,
        ..Default::default()
    };
    p.attempts.insert(s.id.clone(), a);
    assert!(progress::decode(&serde_json::to_vec(&p).unwrap(), &catalog()).is_err());
}
#[test]
fn all_bundled_attempts_current_records_and_selected_reset_survive_exact_roundtrip() {
    let all = catalog();
    let s = all.last().unwrap();
    let mut p = Progress {
        schema_version: 1,
        clock: 91,
        ..Default::default()
    };
    for (index, old) in all.iter().take(90).enumerate() {
        let mut a = Attempt::new(old, index as u64 + 1);
        if let Goal::Script { cases } = &old.goal {
            for command in &old.solution {
                engine::execute_for_goal(&mut a.state, &old.goal, command).unwrap();
            }
            let State::Script { source, run } = &mut a.state else {
                panic!()
            };
            *run = Some(shell::Record {
                engine_revision: shell::EVALUATOR_REVISION,
                source: source.clone(),
                results: cases.iter().map(|c| shell::run(source, c)).collect(),
            });
        }
        p.attempts.insert(old.id.clone(), a);
    }
    p.favorites.extend(all.iter().take(3).map(|s| s.id.clone()));
    let old = serde_json::to_value(&p.attempts).unwrap();
    let mut a = prepared(s);
    a.state = reference(s);
    learn(&mut a).command = "id tess".into();
    learn(&mut a).view = View::Example;
    p.attempts.insert(s.id.clone(), a);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("synthetic.json");
    progress::save(&path, &p).unwrap();
    let original = std::fs::read(&path).unwrap();
    let mut loaded = progress::load(&path, &all).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        serde_json::to_value(&p.attempts).unwrap()
    );
    assert_eq!(loaded.favorites, p.favorites);
    assert_eq!(loaded.clock, 91);
    loaded.attempts.insert(s.id.clone(), prepared(s));
    assert_eq!(loaded.attempts[&s.id].state, s.initial);
    assert!(!progress::current_solved(&loaded.attempts[&s.id], s));
    loaded.attempts.remove(&s.id);
    assert_eq!(serde_json::to_value(&loaded.attempts).unwrap(), old);
}
#[test]
fn old_kinds_keep_exact_execute_behavior_through_the_new_wrapper() {
    let all = pack::bundled().unwrap();
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.initial, State::DebianPackages { .. }))
            .count(),
        4
    );
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.initial, State::SpanningTree { .. }))
            .count(),
        4
    );
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.initial, State::RpmPackages { .. }))
            .count(),
        4
    );
    assert_eq!(
        all.iter()
            .filter(|s| matches!(s.initial, State::Nfs { .. }))
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic2-nfs-reference",
            "lpic2-nfs-publication",
            "lpic2-nfs-handover",
            "lpic2-nfs-capstone",
        ]
    );
    let ether = all
        .iter()
        .filter(|s| matches!(s.initial, State::EtherChannel { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        ether.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        [
            "ccna-lacp-initiation",
            "ccna-lacp-member-service",
            "ccna-lacp-resilient-handover"
        ]
    );
    for source in ether {
        assert!(matches!(source.goal, engine::Goal::EtherChannel { .. }));
        let mut raw = source.initial.clone();
        let before = raw.clone();
        assert!(engine::execute(&mut raw, &source.solution[0]).is_err());
        assert_eq!(
            raw, before,
            "Goal-less Ether action must retain all supplied state"
        );
        let mut contextual = source.initial.clone();
        for command in &source.solution {
            engine::execute_for_goal(&mut contextual, &source.goal, command).unwrap();
        }
        assert!(
            engine::checks(&contextual, &source.goal)
                .iter()
                .all(|r| r.0)
        );
    }
    assert_eq!(
        all[160..164]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic1-cron-weekday-report",
            "lpic1-cron-environment-output",
            "lpic1-cron-access-retirement",
            "lpic1-cron-calendar-handover",
        ]
    );
    assert!(
        all[160..164]
            .iter()
            .all(|s| matches!(s.initial, State::Cron { .. })
                && matches!(s.goal, engine::Goal::Cron { .. }))
    );
    assert_eq!(
        all[164..168]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "lpic2-backup-selective-recovery",
            "lpic2-backup-incremental-chain",
            "lpic2-backup-snapshot-branch",
            "lpic2-backup-recovery-handover",
        ]
    );
    for source in &all[164..168] {
        assert!(
            matches!(source.initial, State::FileBackup { .. })
                && matches!(source.goal, Goal::FileBackup { .. })
        );
        assert!(source.file_backup_teaching.is_some());
        let mut raw = source.initial.clone();
        let before = raw.clone();
        assert!(engine::execute(&mut raw, &source.solution[0]).is_err());
        assert_eq!(
            raw, before,
            "Goal-less backup operations preserve complete supplied state"
        );
        let mut contextual = source.initial.clone();
        for command in &source.solution {
            engine::execute_for_goal(&mut contextual, &source.goal, command).unwrap();
        }
        assert!(
            engine::checks(&contextual, &source.goal)
                .iter()
                .all(|r| r.0),
            "{}",
            source.id
        );
    }
    let physical = all
        .iter()
        .filter(|s| matches!(s.initial, State::PhysicalLink { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        physical.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        [
            "ccna-physical-media-span",
            "ccna-physical-duplex-diagnosis",
            "ccna-physical-counter-baseline",
            "ccna-physical-uplink-handover"
        ]
    );
    for source in physical {
        let mut state = source.initial.clone();
        let before = serde_json::to_value(&state).unwrap();
        assert!(engine::execute(&mut state, &source.solution[0]).is_err());
        assert_eq!(serde_json::to_value(&state).unwrap(), before);
        for input in &source.solution {
            engine::execute_for_goal(&mut state, &source.goal, input).unwrap();
        }
        assert!(engine::solved(&state, &source.goal));
    }
    let one_off = all
        .iter()
        .filter(|s| matches!(s.initial, State::OneOff { .. }))
        .collect::<Vec<_>>();
    assert_eq!(
        one_off.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        one_off_registration::IDS
    );
    for source in one_off {
        let mut raw = source.initial.clone();
        let before = raw.clone();
        assert!(engine::execute(&mut raw, &source.solution[0]).is_err());
        assert_eq!(raw, before);
        let mut contextual = source.initial.clone();
        for command in &source.solution {
            engine::execute_for_goal(&mut contextual, &source.goal, command).unwrap();
        }
        assert!(engine::solved(&contextual, &source.goal));
        let mut own = Attempt::new(source, 1);
        dlms_learning_lab::one_off_workspace::budget::prepare(&mut own, source).unwrap();
        assert!(dlms_learning_lab::one_off_workspace::context(&own, source).is_ok());
        assert!(own.workspace.as_ref().unwrap().command.is_empty());
    }
    let lvm = all
        .iter()
        .filter(|s| matches!(s.initial, State::Lvm { .. }))
        .collect::<Vec<_>>();
    assert_eq!(lvm.len(), 4);
    for source in lvm {
        let mut raw = source.initial.clone();
        let before = raw.clone();
        assert!(engine::execute(&mut raw, &source.solution[0]).is_err());
        assert_eq!(raw, before);
        let mut contextual = source.initial.clone();
        for command in &source.solution {
            engine::execute_for_goal(&mut contextual, &source.goal, command).unwrap();
        }
        assert!(engine::solved(&contextual, &source.goal));
        let mut own = Attempt::new(source, 1);
        workspace::prepare(&mut own);
        dlms_learning_lab::lvm_workspace::context(&own, source).unwrap();
        assert!(own.workspace.as_ref().unwrap().command.is_empty());
    }
    let ntp = all
        .iter()
        .filter(|s| matches!(s.initial, State::Ntp { .. }))
        .collect::<Vec<_>>();
    assert_eq!(ntp.len(), 4);
    for source in ntp {
        let mut raw = source.initial.clone();
        let before = raw.clone();
        assert!(engine::execute(&mut raw, &source.solution[0]).is_err());
        assert_eq!(raw, before);
        let mut contextual = source.initial.clone();
        for command in &source.solution {
            engine::execute_for_goal(&mut contextual, &source.goal, command).unwrap();
        }
        assert!(engine::solved(&contextual, &source.goal));
        let mut own = Attempt::new(source, 1);
        workspace::prepare(&mut own);
        dlms_learning_lab::ntp_workspace::context(&own, source).unwrap();
        assert!(own.workspace.as_ref().unwrap().command.is_empty());
    }
    let old = all
        .into_iter()
        // Compare the legacy context-free kinds. Packages and spanning tree,
        // like Apache, Basic NAT, NFS, accounts, OSPF, multi-unit services and DHCP, require pinned goals
        // and have dedicated context-rejection and goal-aware regression tests.
        .filter(|s| {
            !matches!(
                s.initial,
                State::Accounts { .. }
                    | State::Ospf { .. }
                    | State::UnitManager { .. }
                    | State::DhcpV4 { .. }
                    | State::DebianPackages { .. }
                    | State::RpmPackages { .. }
                    | State::SpanningTree { .. }
                    | State::ApacheStatic { .. }
                    | State::BasicNat { .. }
                    | State::Nfs { .. }
                    | State::EtherChannel { .. }
                    | State::Cron { .. }
                    | State::FileBackup { .. }
                    | State::PhysicalLink { .. }
                    | State::OneOff { .. }
                    | State::Lvm { .. }
                    | State::Ntp { .. }
            )
        })
        .collect::<Vec<_>>();
    let numeric_ids = [
        "shell-numeric-retry",
        "shell-numeric-intake",
        "shell-numeric-publication",
    ];
    assert_eq!(
        old.iter()
            .filter(|s| !numeric_ids.contains(&s.id.as_str())
                && s.id != "ccna-private-office-plan"
                && !["shell-loop-intake", "shell-loop-fallback"].contains(&s.id.as_str())
                && ![
                    "shell-ere-event-view",
                    "shell-ere-inventory-review",
                    "shell-ere-incident-report"
                ]
                .contains(&s.id.as_str()))
            .count(),
        119,
        "Every original119 context-free assignment remains in the loop"
    );
    assert_eq!(
        old.iter()
            .filter(|s| numeric_ids.contains(&s.id.as_str()))
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        numeric_ids
    );
    assert!(
        old.iter()
            .filter(|s| numeric_ids.contains(&s.id.as_str()))
            .all(|s| matches!(s.initial, State::Script { .. })
                && matches!(s.goal, engine::Goal::Script { .. })
                && s.teaching.is_some())
    );
    assert_eq!(
        old.iter()
            .filter(|s| ![
                "shell-ere-event-view",
                "shell-ere-inventory-review",
                "shell-ere-incident-report"
            ]
            .contains(&s.id.as_str()))
            .count(),
        125,
        "all original123 plus two loop assignments remain in the real wrapper loop"
    );
    assert_eq!(
        old.len(),
        128,
        "three ERE scripts also use the same context-free wrapper loop"
    );
    assert_eq!(
        old[125..].iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        [
            "shell-ere-event-view",
            "shell-ere-inventory-review",
            "shell-ere-incident-report"
        ]
    );
    assert!(old[125..].iter().all(
        |s| matches!(s.initial, State::Script { .. }) && matches!(s.goal, Goal::Script { .. })
    ));
    assert_eq!(
        old[113..116]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-command-options",
            "shell-command-dispatch",
            "shell-command-plan"
        ]
    );
    assert!(old[113..116].iter().all(|s| {
        matches!(&s.initial, State::Script { .. }) && matches!(&s.goal, engine::Goal::Script { .. })
    }));
    assert_eq!(
        old[116..119]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        [
            "shell-optional-policy",
            "shell-optional-report",
            "shell-optional-handover"
        ]
    );
    assert!(
        old[116..119]
            .iter()
            .all(|s| matches!(s.initial, State::Script { .. })
                && matches!(s.goal, engine::Goal::Script { .. }))
    );
    let office = old
        .iter()
        .filter(|s| s.id == "ccna-private-office-plan")
        .collect::<Vec<_>>();
    assert_eq!(office.len(), 1);
    assert!(matches!(office[0].initial, State::Ipv4Routing { .. }));
    assert_eq!(
        old[122].id, "ccna-private-office-plan",
        "All previous122 entries stay ahead of the one new existing-kind project"
    );
    assert_eq!(
        old[..123].len(),
        123,
        "All original123 context-free entries retained"
    );
    assert_eq!(
        old[123..125]
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>(),
        ["shell-loop-intake", "shell-loop-fallback"]
    );
    assert!(
        old[123..]
            .iter()
            .all(|s| matches!(s.initial, State::Script { .. })
                && matches!(s.goal, Goal::Script { .. })
                && s.teaching.is_some())
    );
    for s in old {
        let mut raw = s.initial.clone();
        let mut contextual = s.initial.clone();
        for command in &s.solution {
            let left = engine::execute(&mut raw, command);
            let right = engine::execute_for_goal(&mut contextual, &s.goal, command);
            assert_eq!(left, right, "{} / {command}", s.id);
            assert_eq!(raw, contextual, "{} / {command}", s.id);
        }
    }
}
#[test]
fn packs_reject_unpinned_goals_missing_teaching_wrong_examples_and_unreachable_solutions() {
    let original: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/accounts-workspace-preview.json")).unwrap();
    for mutation in [
        "no-teaching",
        "wrong-goal",
        "bad-example",
        "read-only",
        "bad-try",
        "no-annotation",
    ] {
        let mut v = original.clone();
        let s = &mut v["scenarios"][0];
        match mutation {
            "no-teaching" => s["accounts_teaching"] = serde_json::Value::Null,
            "wrong-goal" => s["goal"]["initial"]["policy"]["fixture"] = "different".into(),
            "bad-example" => s["accounts_teaching"]["fixture"] = s["initial"]["simulation"].clone(),
            "read-only" => s["solution"] = serde_json::json!(["id mira"]),
            "bad-try" => s["accounts_teaching"]["try_actions"] = serde_json::json!(["id tess"]),
            _ => s["accounts_teaching"]["example_commands"] = serde_json::json!([]),
        }
        assert!(pack::load(&v.to_string()).is_err(), "{mutation}");
    }
}
#[test]
fn practice_transcripts_and_goal_collections_have_explicit_decode_bounds() {
    let s = scenario();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    let mut l = Learning::new(lesson);
    for _ in 0..45 {
        l.command = "id tess".into();
        l.execute(lesson).unwrap();
    }
    assert_eq!(l.transcript.len(), accounts_teaching::MAX_TRANSCRIPT_ROWS);
    assert!(l.valid_for(lesson));
    let mut v = serde_json::to_value(&l).unwrap();
    v["command"] = "x".repeat(accounts_teaching::MAX_COMMAND_BYTES + 1).into();
    assert!(serde_json::from_value::<Learning>(v).is_err());
    let mut v = serde_json::to_value(&l).unwrap();
    v["transcript"] =
        serde_json::json!(["x".repeat(accounts_teaching::MAX_TRANSCRIPT_ROW_BYTES + 1)]);
    assert!(serde_json::from_value::<Learning>(v).is_err());
    let mut v = serde_json::to_value(&s.goal).unwrap();
    let requirements = v["requirements"].as_array_mut().unwrap();
    let first = requirements[0].clone();
    requirements.resize(accounts::MAX_REQUIREMENTS + 1, first);
    assert!(serde_json::from_value::<Goal>(v).is_err());
}

#[test]
fn new_names_and_numeric_ids_may_overlap_the_other_workspace_and_still_save_and_decode() {
    let all = catalog();
    let s = all.last().unwrap();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    let dir = tempfile::tempdir().unwrap();
    // Each command starts in its own original workspace. Coincidences across
    // independent simulations must not become a new global identity namespace.
    for (i, (assignment, command)) in [
        (true, "groupadd -g 3000 scratch"), // example primary-group GID
        (true, "groupadd -g 2101 user-id-overlap"), // example user UID
        (true, "groupadd -g 3200 sample-staff"), // example group name
        (
            true,
            "useradd -N -u 2101 -g staff -d /home/tess -s /bin/bash -M tess",
        ),
        (false, "groupadd -g 1000 scratch"), // assignment primary-group GID
        (false, "groupadd -g 1101 user-id-overlap"), // assignment user UID
        (false, "groupadd -g 3400 staff"),   // assignment group name
        (
            false,
            "useradd -N -u 1101 -g sample-staff -d /home/mira -s /bin/bash -M mira",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut a = prepared(s);
        if assignment {
            engine::execute_for_goal(&mut a.state, &s.goal, command).unwrap();
            assert_eq!(learn(&mut a).simulation, lesson.fixture);
        } else {
            learn(&mut a).command = command.into();
            learn(&mut a).execute(lesson).unwrap();
            assert_eq!(a.state, s.initial);
        }
        assert!(simulation(&a.state).valid_against(simulation(&s.initial)));
        assert!(learn(&mut a).valid_for(lesson));
        let mut p = Progress {
            schema_version: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let path = dir.path().join(format!("synthetic-overlap-{i}.json"));
        progress::save(&path, &p).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let loaded = progress::load(&path, &all).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert_eq!(
            serde_json::to_value(&loaded.attempts).unwrap(),
            serde_json::to_value(&p.attempts).unwrap()
        );
        progress::save(&path, &loaded).unwrap();
        assert!(progress::load(&path, &all).is_ok());
    }
}

#[test]
fn accounts_export_inspection_and_confirmed_restore_preserve_both_drafts_and_current_states() {
    use std::sync::atomic::{AtomicBool, AtomicUsize};
    let all = catalog();
    let s = all.last().unwrap();
    let lesson = s.accounts_teaching.as_ref().unwrap();
    let mut a = prepared(s);
    a.state = reference(s);
    a.workspace.as_mut().unwrap().command = "usermod -G 'unfinished assignment draft".into();
    for command in &lesson.example {
        learn(&mut a).command = command.clone();
        learn(&mut a).execute(lesson).unwrap();
    }
    for command in &lesson.try_actions {
        learn(&mut a).command = command.clone();
        learn(&mut a).execute(lesson).unwrap();
    }
    learn(&mut a).view = View::Example;
    learn(&mut a).checked = true;
    learn(&mut a).command = "usermod -aG sample-audit 'unfinished example draft".into();
    assert!(progress::current_solved(&a, s));
    assert!(learn(&mut a).checks(lesson).iter().any(|c| !c.0));
    let mut exported = Progress {
        schema_version: 1,
        clock: 19,
        ..Default::default()
    };
    exported.favorites.insert(s.id.clone());
    exported.attempts.insert(s.id.clone(), a);
    let expected_attempts = serde_json::to_value(&exported.attempts).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("synthetic-accounts-backup.json");
    let target = dir.path().join("synthetic-current-progress.json");
    let _lock = progress::lock_profile(&target).unwrap();
    let mut prior = Progress {
        schema_version: 1,
        ..Default::default()
    };
    prior.attempts.insert(s.id.clone(), prepared(s));
    progress::save(&target, &prior).unwrap();
    let prior_bytes = std::fs::read(&target).unwrap();
    backup::export(&input, &exported, &all).unwrap();
    let input_bytes = std::fs::read(&input).unwrap();
    let candidate =
        backup::inspect(&input, &all, &AtomicBool::new(false), &AtomicUsize::new(0)).unwrap();
    // Account credit is recomputed from current typed configuration, not a saved
    // script result. The existing rechecked counter counts script results only.
    assert_eq!(candidate.rechecked, 0);
    assert!(candidate.summary(&all).contains("1 currently complete"));
    assert_eq!(
        serde_json::to_value(&candidate.progress.attempts).unwrap(),
        expected_attempts
    );
    assert_eq!(std::fs::read(&target).unwrap(), prior_bytes); // inspection is read-only
    assert_eq!(std::fs::read(&input).unwrap(), input_bytes);
    let restored = backup::restore(&target, &candidate, &all).unwrap();
    assert_eq!(
        std::fs::read(restored.safety_copy.unwrap()).unwrap(),
        prior_bytes
    );
    let mut loaded = progress::load(&target, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded.attempts).unwrap(),
        expected_attempts
    );
    assert_eq!(loaded.clock, 19);
    assert_eq!(loaded.favorites, exported.favorites);
    let a = loaded.attempts.get_mut(&s.id).unwrap();
    assert_eq!(
        a.workspace.as_ref().unwrap().command,
        "usermod -G 'unfinished assignment draft"
    );
    assert_eq!(
        learn(a).command,
        "usermod -aG sample-audit 'unfinished example draft"
    );
    assert!(progress::current_solved(a, s));
    assert!(learn(a).checks(lesson).iter().any(|c| !c.0));
    assert_eq!(std::fs::read(&input).unwrap(), input_bytes);
}
