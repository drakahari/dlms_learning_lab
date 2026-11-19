use dlms_learning_lab::{
    engine::{self, State},
    kernel_source as binding,
    kernel_workspace::{Action, Kind},
    pack,
    progress::{self, Attempt, Progress},
};
fn sources() -> Vec<pack::Scenario> {
    serde_json::from_str::<pack::Pack>(include_str!("../scenarios/kernel-build-lessons-v1.json"))
        .unwrap()
        .scenarios
}
fn act(state: &mut State, s: &pack::Scenario, a: &Action, practice: bool) {
    engine::execute_for_goal(
        state,
        &s.goal,
        &format!(
            "{}{}",
            binding::PREFIX,
            serde_json::json!({"practice":practice,"action":a})
        ),
    )
    .unwrap();
}
fn solve(s: &pack::Scenario) -> State {
    let mut state = s.initial.clone();
    for input in &s.solution {
        engine::execute_for_goal(&mut state, &s.goal, input).unwrap();
    }
    assert!(
        engine::solved(&state, &s.goal),
        "{} {:?}",
        s.id,
        engine::checks(&state, &s.goal)
    );
    state
}
#[test]
fn both_authored_tasks_and_separate_worked_examples_reach_actual_outcomes() {
    let all = sources();
    assert_eq!(all.len(), 2);
    for s in all {
        let d = binding::source(&s).unwrap();
        let mut state = s.initial.clone();
        let State::KernelBuild { simulation } = &state else {
            panic!()
        };
        let original = simulation.assignment.clone();
        for a in &d.example.reference {
            act(&mut state, &s, a, true);
        }
        let State::KernelBuild { simulation } = &state else {
            panic!()
        };
        assert!(d.example.solved(&simulation.example.work));
        assert_eq!(simulation.assignment, original);
        assert!(!engine::solved(&state, &s.goal));
        solve(&s);
        assert!(
            s.guide
                .iter()
                .flat_map(|g| &g.commands)
                .all(|c| !c.command.starts_with(binding::PREFIX))
        );
    }
}
#[test]
fn supported_builtin_driver_alternatives_and_different_entry_names_are_accepted() {
    for s in sources() {
        let d = binding::source(&s).unwrap();
        let t = &d.assignment;
        let mut state = s.initial.clone();
        let c = "CONFIG_MODULES=n\nCONFIG_BLK_DEV_INITRD=y\nCONFIG_BLK_DEV_NVME=y\nCONFIG_EXT4_FS=y\nCONFIG_LOCALVERSION=\"-alternative\"\n";
        for a in [
            Action::Source {
                index: t.wanted_source,
            },
            Action::Config { text: c.into() },
            Action::SaveConfig,
            Action::Run {
                input: "make bzImage".into(),
            },
        ] {
            act(&mut state, &s, &a, false);
        }
        if t.kind == Kind::Boot {
            let release = format!("{}-alternative", t.sources[t.wanted_source].release);
            act(&mut state, &s, &Action::StageImage, false);
            act(
                &mut state,
                &s,
                &Action::Initramfs {
                    image_path: format!("{}/boot/vmlinuz-{release}", t.stage_root),
                },
                false,
            );
            act(
                &mut state,
                &s,
                &Action::EntryDraft {
                    filename: "my-kernel.conf".into(),
                    text: format!(
                        "title Another supported choice\nlinux /vmlinuz-{release}\ninitrd /initramfs-{release}.img\noptions root=UUID={} ro\n",
                        t.root.as_ref().unwrap().uuid
                    ),
                },
                false,
            );
            act(&mut state, &s, &Action::SaveEntry, false);
        }
        act(&mut state, &s, &Action::Check, false);
        assert!(
            engine::solved(&state, &s.goal),
            "{:?}",
            engine::checks(&state, &s.goal)
        );
    }
}
#[test]
fn wrong_partial_and_broken_success_have_specific_failures() {
    for s in sources() {
        let mut state = s.initial.clone();
        act(&mut state, &s, &Action::Check, false);
        assert!(!engine::solved(&state, &s.goal));
        let checks = engine::checks(&state, &s.goal);
        assert!(checks.iter().any(|c| c.0) && checks.iter().any(|c| !c.0));
        let mut state = solve(&s);
        act(
            &mut state,
            &s,
            &Action::Config {
                text: "# unfinished\n".into(),
            },
            false,
        );
        assert!(!engine::solved(&state, &s.goal));
        act(&mut state, &s, &Action::Check, false);
        assert!(
            engine::checks(&state, &s.goal)
                .iter()
                .any(|c| !c.0 && c.1.contains("configuration"))
        );
    }
    let s = sources().remove(1);
    let d = binding::source(&s).unwrap();
    let mut state = s.initial.clone();
    let old = &d.assignment.initial_store.entries[0].source;
    act(
        &mut state,
        &s,
        &Action::EntryDraft {
            filename: "new-title.conf".into(),
            text: old.replace("Supplied kernel candidate", "New kernel!"),
        },
        false,
    );
    act(&mut state, &s, &Action::SaveEntry, false);
    act(&mut state, &s, &Action::Check, false);
    assert!(!engine::solved(&state, &s.goal));
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|c| c.0 && c.1.starts_with("Candidate boot trial"))
    );
    assert!(
        engine::checks(&state, &s.goal)
            .iter()
            .any(|c| !c.0 && c.1.starts_with("Staged target image"))
    );
}
#[test]
fn assignment_and_example_drafts_roundtrip_reset_and_never_invent_mastery() {
    let all = sources();
    let mut p = Progress {
        schema_version: 1,
        ..Progress::default()
    };
    for s in &all {
        let mut a = Attempt::new(s, 1);
        a.challenge = true;
        p.favorites.insert(s.id.clone());
        act(
            &mut a.state,
            s,
            &Action::CommandDraft {
                text: "make mod".into(),
            },
            true,
        );
        act(
            &mut a.state,
            s,
            &Action::Config {
                text: "# assignment still editing\n".into(),
            },
            false,
        );
        p.attempts.insert(s.id.clone(), a);
    }
    let raw = serde_json::to_vec(&p).unwrap();
    let mut restored = progress::decode(&raw, &all).unwrap();
    assert_eq!(
        serde_json::to_value(&p).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    for s in &all {
        let a = restored.attempts.get_mut(&s.id).unwrap();
        assert!(!progress::current_solved(a, s));
        let State::KernelBuild { simulation } = &a.state else {
            panic!()
        };
        let example = simulation.example.clone();
        act(&mut a.state, s, &Action::Reset, false);
        let State::KernelBuild { simulation } = &a.state else {
            panic!()
        };
        assert_eq!(simulation.example, example);
        assert_eq!(
            simulation.assignment.work.config_draft,
            binding::source(s).unwrap().assignment.initial_config
        );
    }
}

#[test]
fn registered_curriculum_preserves_every_prior_lab_unit_and_prerequisite_order() {
    let old = pack::accepted509().unwrap();
    let current = pack::accepted511().unwrap();
    assert_eq!((old.len(), current.len()), (509, 511));
    assert_eq!(
        serde_json::to_value(&old).unwrap(),
        serde_json::to_value(&current[..509]).unwrap()
    );
    assert_eq!(
        serde_json::to_value(sources()).unwrap(),
        serde_json::to_value(&current[509..]).unwrap()
    );
    let before = dlms_learning_lab::course::accepted509(&old).unwrap();
    let units = dlms_learning_lab::course::accepted511(&current).unwrap();
    assert_eq!((before.len(), units.len()), (205, 206));
    let retained = units
        .iter()
        .filter(|u| u.id != "linux-kernel-build-staging")
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(before).unwrap(),
        serde_json::to_value(retained).unwrap()
    );
    let at = units
        .iter()
        .position(|u| u.id == "linux-kernel-build-staging")
        .unwrap();
    for p in &units[at].prerequisites {
        assert!(units.iter().position(|u| &u.id == p).unwrap() < at);
    }
    let mut counts = std::collections::BTreeMap::new();
    for s in current {
        *counts.entry(s.track).or_insert(0) += 1;
    }
    assert_eq!(
        (
            counts["CCNA"],
            counts["LPIC-1"],
            counts["LPIC-2"],
            counts["Bash/Shell"]
        ),
        (115, 154, 162, 80)
    );
}
