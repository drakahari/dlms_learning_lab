//! Actual unregistered authored unit; no placeholder or synthetic catalog credit.
use dlms_learning_lab::{
    engine::{self, State},
    ftp_identity_source as binding,
    ftp_identity_workspace::{Action, Event},
    pack,
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/authoring/ftp-identity-v1.json"))
        .unwrap()
        .scenarios
}
fn closure(new: &[pack::Scenario]) -> Vec<pack::Scenario> {
    let old = pack::current().unwrap();
    let mut all = new.to_vec();
    let mut at = 0;
    while at < all.len() {
        for id in all[at].prerequisites.clone() {
            if !all.iter().any(|s| s.id == id) {
                all.push(
                    old.iter()
                        .find(|s| s.id == id)
                        .expect("real existing prerequisite")
                        .clone(),
                );
            }
        }
        at += 1;
    }
    all
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for c in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, c).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{}: {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
fn act(state: &mut State, s: &pack::Scenario, a: Action) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!("{}{}", binding::PREFIX, serde_json::to_string(&a).unwrap()),
    )
    .unwrap();
}
#[test]
fn all_three_real_lessons_load_with_real_prerequisites_and_taught_methods_work() {
    let sources = sources();
    assert_eq!(sources.len(), 3);
    assert!(
        pack::load(include_str!("../scenarios/authoring/ftp-identity-v1.json"))
            .unwrap_err()
            .contains("invalid prerequisites")
    );
    pack::load(
        &serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: closure(&sources),
        })
        .unwrap(),
    )
    .unwrap();
    for s in sources {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::FtpIdentity { simulation } = &mut state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        for a in &d.worked {
            simulation.action(d, a, true).unwrap();
        }
        assert_eq!(simulation.assignment, original);
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
    }
}
#[test]
fn partial_repairs_and_permissions_shortcuts_cannot_complete_authored_assignments() {
    for s in sources() {
        let mut partial = s.initial.clone();
        act(&mut partial, &s, Action::Check);
        assert!(!engine::solved(&partial, &s.goal));
        for c in s.solution.iter().take(2) {
            engine::execute_for_goal(&mut partial, &s.goal, c).unwrap();
        }
        act(&mut partial, &s, Action::Check);
        assert!(!engine::solved(&partial, &s.goal));
        let mut state = solve(&s);
        act(
            &mut state,
            &s,
            Action::Edit {
                text: "local_enable=NO\nanonymous_enable=NO\n".into(),
            },
        );
        act(&mut state, &s, Action::Save);
        act(
            &mut state,
            &s,
            Action::Event {
                event: Event::Admin {
                    input: "systemctl restart vsftpd".into(),
                },
            },
        );
        act(&mut state, &s, Action::Check);
        assert!(!engine::solved(&state, &s.goal));
        assert!(engine::checks(&state, &s.goal)
            .iter()
            .any(|(p, m)| !p && m.contains("Local account login is disabled")));
    }
}
#[test]
fn authored_upload_tasks_accept_equivalent_private_masks_and_chroot_handover_accepts_inverse_policy(
) {
    for s in sources().into_iter().skip(1) {
        let mut state = solve(&s);
        let State::FtpIdentity { simulation } = &state else {
            panic!()
        };
        let config = simulation
            .assignment
            .world
            .saved
            .replace("local_umask=077", "local_umask=0177");
        act(&mut state, &s, Action::Edit { text: config });
        act(&mut state, &s, Action::Save);
        act(
            &mut state,
            &s,
            Action::Event {
                event: Event::Admin {
                    input: "systemctl restart vsftpd".into(),
                },
            },
        );
        act(&mut state, &s, Action::Check);
        assert!(engine::solved(&state, &s.goal));
        if s.id.ends_with("handover") {
            let State::FtpIdentity { simulation } = &state else {
                panic!()
            };
            let config = simulation
                .assignment
                .world
                .saved
                .replace("chroot_local_user=YES", "chroot_local_user=NO");
            act(&mut state, &s, Action::Edit { text: config });
            act(&mut state, &s, Action::Save);
            act(
                &mut state,
                &s,
                Action::Event {
                    event: Event::SaveFile {
                        path: "/etc/vsftpd.chroot_list".into(),
                        text: "mira\n".into(),
                    },
                },
            );
            act(
                &mut state,
                &s,
                Action::Event {
                    event: Event::Admin {
                        input: "systemctl restart vsftpd".into(),
                    },
                },
            );
            act(&mut state, &s, Action::Check);
            assert!(engine::solved(&state, &s.goal));
        }
    }
}
#[test]
fn invalid_pack_reference_and_unsupported_guide_diagnostics_preserve_the_contract() {
    let original = sources();
    let mut bad = original.clone();
    bad[0].solution = vec![format!(
        "{}{}",
        binding::PREFIX,
        serde_json::to_string(&Action::Check).unwrap()
    )];
    assert!(pack::load(
        &serde_json::to_string(&pack::Pack {
            schema_version: 1,
            scenarios: closure(&bad)
        })
        .unwrap()
    )
    .is_err());
    let mut s = original[0].clone();
    s.guide[0].commands.push(pack::CommandExample {
        command: "curl https://real.invalid/".into(),
        explanation: "Unsupported, must fail before content acceptance.".into(),
        simulator_only: false,
    });
    assert!(binding::source(&s)
        .unwrap_err()
        .contains("Unsupported or unreachable"));
}

#[test]
fn registration_preserves_every_494_prior_lab_198_units_and_exact_aggregation_values() {
    let before = pack::accepted494().unwrap();
    let now = pack::accepted497().unwrap();
    assert_eq!((before.len(), now.len()), (494, 497));
    for old in &before {
        let new = now.iter().find(|s| s.id == old.id).unwrap();
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(new).unwrap()
        );
    }
    let old_units = dlms_learning_lab::course::accepted494(&before).unwrap();
    let new_units = dlms_learning_lab::course::accepted497(&now).unwrap();
    assert_eq!((old_units.len(), new_units.len()), (198, 199));
    for old in &old_units {
        assert_eq!(
            serde_json::to_value(old).unwrap(),
            serde_json::to_value(new_units.iter().find(|u| u.id == old.id).unwrap()).unwrap()
        );
    }
    let unit = new_units
        .iter()
        .position(|u| u.id == "linux-ftp-identity")
        .unwrap();
    for id in ["samba-standalone-shares", "linux-unit-manager-foundations"] {
        assert!(new_units.iter().position(|u| u.id == id).unwrap() < unit);
    }
    let old: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v59.json")).unwrap();
    let new: serde_json::Value =
        serde_json::from_str(include_str!("../scenarios/boot-entry-lessons-v60.json")).unwrap();
    assert_eq!(old["scenarios"].as_array().unwrap().len(), 121);
    assert_eq!(new["scenarios"].as_array().unwrap().len(), 124);
    assert_eq!(
        old["scenarios"].as_array().unwrap().as_slice(),
        &new["scenarios"].as_array().unwrap()[..121]
    );
    let ids = now
        .iter()
        .filter(|s| s.unit == "linux-ftp-identity")
        .map(|s| s.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "lpic2-ftp-local-jail",
            "lpic2-ftp-upload-rights",
            "lpic2-ftp-chroot-handover"
        ]
    );
}
