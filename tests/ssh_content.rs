use dlms_learning_lab::{
    engine, pack,
    progress::{self, Attempt, Progress},
    ssh_workspace::{self as work, Document, Event},
    workspace,
};
fn labs() -> Vec<pack::Scenario> {
    pack::load(include_str!("../scenarios/ssh-access-lessons-v1.json")).unwrap()
}
#[test]
fn main_catalog_registers_new_lessons_without_altering_any_existing_scenario_or_unit() {
    let old = pack::accepted442().unwrap();
    let current = pack::current().unwrap();
    assert_eq!(old.len(), 442);
    assert_eq!(current.len(), 448);
    for s in &old {
        let retained = current.iter().find(|next| next.id == s.id).unwrap();
        assert_eq!(
            serde_json::to_value(s).unwrap(),
            serde_json::to_value(retained).unwrap(),
            "{}",
            s.id
        );
    }
    let old_units = dlms_learning_lab::course::accepted442(&old).unwrap();
    let units = dlms_learning_lab::course::current(&current).unwrap();
    assert_eq!(old_units.len(), 174);
    assert_eq!(units.len(), 177);
    for u in old_units {
        let retained = units.iter().find(|next| next.id == u.id).unwrap();
        assert_eq!(
            serde_json::to_value(u).unwrap(),
            serde_json::to_value(retained).unwrap()
        );
    }
    for s in labs() {
        assert!(current.iter().any(|next| next.id == s.id));
    }
}
#[test]
fn three_distinct_drafts_validate_and_the_taught_repairs_satisfy_every_real_requirement() {
    let labs = labs();
    assert_eq!(labs.len(), 3);
    for s in labs {
        let d = work::source(&s).unwrap();
        assert!(!engine::solved(&s.initial, &s.goal));
        let mut state = s.initial.clone();
        for e in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, e).unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
        let engine::State::SshAccess { simulation } = state else {
            panic!()
        };
        assert!(simulation.assignment.solved(&d.initial));
        assert!(!simulation.practice.checked);
    }
}
#[test]
fn assignment_drafts_resume_through_actual_progress_io_and_example_work_never_awards_credit() {
    for s in labs() {
        let d = work::source(&s).unwrap();
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        let engine::State::SshAccess { simulation } = &mut a.state else {
            panic!()
        };
        simulation.fill_example(d).unwrap();
        let doc = d.initial.documents[0].clone();
        let e = match doc {
            Document::KnownHosts => Event::Text {
                document: doc,
                text: "unfinished known hosts".into(),
            },
            Document::KeyFile { .. } => Event::Text {
                document: doc,
                text: "unfinished key file".into(),
            },
            _ => panic!(),
        };
        simulation.apply(d, &e, false).unwrap();
        assert!(!engine::solved(&a.state, &s.goal));
        let mut p = Progress {
            schema_version: 1,
            clock: 1,
            ..Default::default()
        };
        p.attempts.insert(s.id.clone(), a);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("progress.json");
        progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).unwrap();
        let loaded = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert_eq!(
            serde_json::to_value(loaded).unwrap(),
            serde_json::to_value(p).unwrap()
        );
    }
}
#[test]
fn saved_only_or_unsafe_shortcuts_and_wrong_host_identity_cannot_pass() {
    for s in labs() {
        let mut state = s.initial.clone();
        for action in &s.solution {
            if !action.contains("ApplyPolicy") {
                engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
            }
        }
        if s.id == "lpic2-ssh-policy-handover" {
            assert!(!engine::solved(&state, &s.goal));
        }
        let mut state = s.initial.clone();
        for action in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, action).unwrap();
        }
        let d = work::source(&s).unwrap();
        let engine::State::SshAccess { simulation } = &mut state else {
            panic!()
        };
        if d.initial.documents.contains(&Document::KnownHosts) {
            simulation
                .apply(
                    d,
                    &Event::Text {
                        document: Document::KnownHosts,
                        text: String::new(),
                    },
                    false,
                )
                .unwrap();
        } else {
            let p = d.initial.scope.editable_text[0].clone();
            simulation
                .apply(
                    d,
                    &Event::Metadata {
                        path: p,
                        owner: "root".into(),
                        group: "root".into(),
                        mode: "000".into(),
                    },
                    false,
                )
                .unwrap();
        }
        simulation.apply(d, &Event::Check {}, false).unwrap();
        assert!(!engine::solved(&state, &s.goal));
    }
}

#[test]
fn completed_current_credit_survives_actual_storage_and_unsaved_edits_clear_it() {
    for s in labs() {
        let mut a = Attempt::new(&s, 1);
        workspace::prepare(&mut a);
        for action in &s.solution {
            engine::execute_for_goal(&mut a.state, &s.goal, action).unwrap();
        }
        assert!(progress::current_solved(&a, &s));
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
        let mut read = progress::load(&path, std::slice::from_ref(&s)).unwrap();
        assert!(progress::current_solved(&read.attempts[&s.id], &s));
        assert!(read.favorites.contains(&s.id));
        let a = read.attempts.get_mut(&s.id).unwrap();
        let engine::State::SshAccess { simulation } = &mut a.state else {
            panic!()
        };
        let d = work::source(&s).unwrap();
        let doc = d.initial.documents[0].clone();
        simulation
            .apply(
                d,
                &Event::Text {
                    document: doc,
                    text: "unfinished change".into(),
                },
                false,
            )
            .unwrap();
        assert!(!progress::current_solved(a, &s));
    }
}

#[test]
fn supported_alternative_records_modes_and_key_only_policy_pass_each_authored_task() {
    for s in labs() {
        let mut state = s.initial.clone();
        for input in &s.solution {
            let mut event: Event =
                serde_json::from_str(input.strip_prefix("ssh-lab ").unwrap()).unwrap();
            match &mut event {
                Event::Text {
                    document: Document::KnownHosts,
                    text,
                } => {
                    *text = text
                        .lines()
                        .rev()
                        .map(|line| format!("{line} alternate comment\n"))
                        .collect();
                }
                Event::Text {
                    document: Document::KeyFile { .. },
                    text,
                } => {
                    *text = format!("# public identities, not private credentials\n{text}\n");
                }
                Event::Text {
                    document: Document::ServerConfig,
                    text,
                } => {
                    // AuthenticationMethods can constrain authentication even when
                    // password method defaults remain enabled. Do not require one
                    // exact spelling of the independent security requirements.
                    *text = "AllowUsers rescue\nAllowUsers morgan\nAuthenticationMethods publickey\nPermitRootLogin no\nStrictModes yes\nAuthorizedKeysFile .ssh/authorized_keys\n".into();
                }
                Event::Metadata { mode, .. } => {
                    *mode = if mode == "750" { "0755" } else { "0644" }.into();
                }
                _ => {}
            }
            engine::execute_for_goal(
                &mut state,
                &s.goal,
                &format!("ssh-lab {}", serde_json::to_string(&event).unwrap()),
            )
            .unwrap();
        }
        assert!(
            engine::solved(&state, &s.goal),
            "{}: {:?}",
            s.id,
            engine::checks(&state, &s.goal)
        );
    }
}

#[test]
fn saved_wrong_keys_unsafe_modes_and_unapplied_policy_fail_for_the_actual_reason() {
    for s in labs() {
        let d = work::source(&s).unwrap();
        let mut state = s.initial.clone();
        for input in &s.solution {
            engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
        }
        let engine::State::SshAccess { simulation } = &mut state else {
            panic!()
        };
        let corrupt = if s.id == "lpic1-ssh-server-trust" {
            Event::Text {
                document: Document::KnownHosts,
                text: format!(
                    "store.lab.test {}\n{}",
                    d.initial.scope.identities["admin-key"].line(),
                    d.initial
                        .world
                        .known_hosts
                        .lines()
                        .find(|l| l.starts_with("archive."))
                        .unwrap()
                ),
            }
        } else if s.id == "lpic2-ssh-key-authorization" {
            Event::Metadata {
                path: d.initial.scope.editable_text[0].clone(),
                owner: "morgan".into(),
                group: "ops".into(),
                mode: "0664".into(),
            }
        } else {
            Event::Text {
                document: Document::ServerConfig,
                text: "StrictModes no\n".into(),
            }
        };
        let document = match &corrupt {
            Event::Text { document, .. } => document.clone(),
            Event::Metadata { path, .. } => Document::KeyFile { path: path.clone() },
            _ => unreachable!(),
        };
        simulation.apply(d, &corrupt, false).unwrap();
        simulation
            .apply(d, &Event::Save { document }, false)
            .unwrap();
        simulation.apply(d, &Event::Check {}, false).unwrap();
        let rows = simulation.assignment.checks(&d.initial);
        assert!(rows.iter().any(|r| !r.0), "{}: {rows:?}", s.id);
        assert!(
            rows.iter().any(|r| r.0),
            "partial work was lost: {}: {rows:?}",
            s.id
        );
        assert!(!simulation.assignment.solved(&d.initial));
        if s.id == "lpic2-ssh-policy-handover" {
            assert!(
                rows.iter().any(|r| !r.0 && r.1.contains("apply")),
                "{rows:?}"
            );
            simulation.apply(d, &Event::ApplyPolicy {}, false).unwrap();
            simulation.apply(d, &Event::Check {}, false).unwrap();
            assert!(!simulation.assignment.solved(&d.initial));
        }
    }
}

#[test]
fn source_bound_storage_rejects_forged_inventory_protected_files_and_cross_task_identity() {
    for s in labs() {
        for alteration in 0..3 {
            let mut a = Attempt::new(&s, 1);
            workspace::prepare(&mut a);
            let engine::State::SshAccess { simulation } = &mut a.state else {
                panic!()
            };
            match alteration {
                0 => simulation.id = "another-task".into(),
                1 => {
                    simulation
                        .assignment
                        .world
                        .tree
                        .files
                        .insert("/etc/authorized_keys".into(), "outside scope".into());
                }
                _ => simulation.assignment.world.tree.home = "/home/someone-else".into(),
            }
            assert!(work::context(&a, &s).is_err());
            assert!(!progress::current_solved(&a, &s));
            let mut p = Progress {
                schema_version: 1,
                clock: 1,
                ..Default::default()
            };
            p.attempts.insert(s.id.clone(), a);
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("progress.json");
            assert!(progress::save_with_catalog(&path, &p, std::slice::from_ref(&s)).is_err());
            assert!(
                !path.exists(),
                "invalid data must not replace a saved profile"
            );
        }
    }
}
